#![recursion_limit = "1024"]

mod field;
mod file;
mod peripheral;
mod register;
mod svd;
pub mod svd_loader;

use anyhow::Context;
use peripheral::PeripheralCard;
use svd_loader::SvdLoadingState;
use yew::{prelude::*, web_sys::File};

use interface::{Command, Register, Update};
use yew_services::{
    reader::{FileData, ReaderTask},
    websocket::{WebSocketStatus, WebSocketTask},
    ReaderService, WebSocketService,
};

/// This is the main component that represents our app.
/// We mount it in app.rs but also need it in the Worker, which is why it's part of this library and not just the app.
pub struct Model {
    link: ComponentLink<Model>,
    reader_tasks: Vec<ReaderTask>,
    websocket_task: Option<WebSocketTask>,
    device: SvdLoadingState,
    poll_interval: usize,
    watching_addresses: Vec<u32>,
    watch: Callback<u32>,
    set: Callback<(u32, u32)>,
    loader: Box<dyn yew::Bridge<svd_loader::Loader>>,
    halted: bool,
}

/// Represents a new websocket event in any form.
pub enum WebsocketEvent {
    SendData(Command),
    Opened,
    Lost,
}

/// The main message struct we use in our app.
pub enum Msg {
    SvdFileDataLoaded(FileData),
    SvdParsingComplete(SvdLoadingState),
    UserSelectedFiles(File),
    Search(String),
    WebSocketData(Update),
    WebsocketEvent(WebsocketEvent),
    Watch(u32),
    Set((u32, u32)),
    Halt,
    None,
}

impl Model {
    fn request_halt(&mut self) {
        self.send_request(&Command::Halt);
    }

    fn request_run(&mut self) {
        self.send_request(&Command::Run);
    }

    fn send_request(&mut self, command: &Command) {
        let data = serde_json::to_string(command).unwrap();
        self.websocket_task.as_mut().unwrap().send(Ok(data));
    }
}

impl Component for Model {
    type Message = Msg;
    type Properties = ();

    fn create(_: Self::Properties, link: ComponentLink<Self>) -> Self {
        let websocket_callback = link.callback(|data: Result<String, anyhow::Error>| {
            let data = data.unwrap();
            let update = serde_json::from_str(&data).unwrap();
            Msg::WebSocketData(update)
        });
        let notification = link.callback(|status| match status {
            WebSocketStatus::Opened => Msg::WebsocketEvent(WebsocketEvent::Opened),
            WebSocketStatus::Closed | WebSocketStatus::Error => {
                Msg::WebsocketEvent(WebsocketEvent::Lost)
            }
        });
        let watch = link.callback(move |value| Msg::Watch(value));
        let set = link.callback(move |value| Msg::Set(value));

        let device = SvdLoadingState::Loading;

        let callback = link.callback(|device| Msg::SvdParsingComplete(device));
        let mut loader = svd_loader::Loader::bridge(callback);
        loader.send(crate::file::TEST_SVD.into());

        Model {
            link,
            reader_tasks: vec![],
            websocket_task: Some(
                WebSocketService::connect_text(
                    "ws://localhost:3031/",
                    websocket_callback,
                    notification,
                )
                .unwrap(),
            ),
            device,
            poll_interval: 1000,
            watching_addresses: vec![],
            watch,
            set,
            loader,
            halted: false,
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::SvdFileDataLoaded(filedata) => {
                let xml = String::from_utf8(filedata.content)
                    .context("The SVD file appears to contain invalid UTF8 data.");
                match xml {
                    Ok(xml) => self.loader.send(xml),
                    Err(error) => {
                        self.update(Msg::SvdParsingComplete(SvdLoadingState::Failed(
                            error.to_string(),
                        )));
                    }
                }
            }
            Msg::SvdParsingComplete(device) => {
                self.device = device;
            }
            Msg::UserSelectedFiles(file) => {
                let callback = self.link.callback(Msg::SvdFileDataLoaded);
                let task = ReaderService::read_file(file, callback).unwrap();
                self.reader_tasks.push(task);
            }
            Msg::Search(term) => {
                if let SvdLoadingState::Loaded(device) = &mut self.device {
                    for peripheral in &mut device.peripherals {
                        peripheral.show = peripheral.name.starts_with(&term);

                        let mut found_register = false;
                        for register in &mut peripheral.registers {
                            register.show = if register.name.starts_with(&term) {
                                found_register = true;
                                true
                            } else {
                                false
                            };

                            let mut found_field = false;
                            if let Some(fields) = &mut register.fields {
                                for field in fields {
                                    field.1 = if register.name.starts_with(&term) {
                                        found_field = true;
                                        true
                                    } else {
                                        false
                                    };
                                }
                            }
                            register.show |= found_field;
                        }
                        peripheral.show |= found_register;
                    }
                }
            }
            Msg::WebSocketData(data) => {
                match data {
                    Update::Registers(register_updates) => {
                        if let SvdLoadingState::Loaded(device) = &mut self.device {
                            for register_update in register_updates.registers {
                                for peripheral in &mut device.peripherals {
                                    for register in &mut peripheral.registers {
                                        if register.address == register_update.address {
                                            register.value = register_update.value;
                                            return true;
                                        }
                                    }
                                }
                            }
                        }
                    }
                    Update::Halted => {
                        self.halted = true;
                        return true;
                    }
                    Update::Running => {
                        self.halted = false;
                        return true;
                    }
                }
                return false;
            }
            Msg::WebsocketEvent(event) => match event {
                WebsocketEvent::SendData(command) => {
                    log::info!("Socket send data.");
                    let data = serde_json::to_string(&command).unwrap();
                    self.websocket_task.as_mut().unwrap().send(Ok(data));
                }
                WebsocketEvent::Opened => {
                    log::info!("Socket configuring.");

                    let command = Command::Watch(vec![]);
                    let data = serde_json::to_string(&command).unwrap();
                    self.websocket_task.as_mut().unwrap().send(Ok(data));
                }
                WebsocketEvent::Lost => {
                    log::info!("Socket lost.");
                    let callback = self.link.callback(|data: Result<String, anyhow::Error>| {
                        let data = data.unwrap();
                        let update = serde_json::from_str(&data).unwrap();
                        Msg::WebSocketData(update)
                    });
                    let notification = self.link.callback(|status| match status {
                        WebSocketStatus::Opened => Msg::None,
                        WebSocketStatus::Closed | WebSocketStatus::Error => {
                            Msg::WebsocketEvent(WebsocketEvent::Lost)
                        }
                    });
                    self.websocket_task = Some(
                        WebSocketService::connect_text(
                            "ws://localhost:3031/",
                            callback,
                            notification,
                        )
                        .unwrap(),
                    );
                }
            },
            Msg::Watch(address) => {
                log::info!("WATCH {}", address);
                self.watching_addresses.push(address);

                let command = Command::Watch(self.watching_addresses.clone());
                let data = serde_json::to_string(&command).unwrap();
                self.websocket_task.as_mut().unwrap().send(Ok(data));
            }
            Msg::Set((address, value)) => {
                log::info!("WRITE {}: {}", address, value);
                self.watching_addresses.push(address);

                let command = Command::SetRegister(Register { value, address });
                let data = serde_json::to_string(&command).unwrap();
                self.websocket_task.as_mut().unwrap().send(Ok(data));
            }
            Msg::Halt => {
                if self.halted {
                    self.request_run();
                } else {
                    self.request_halt();
                }
            }
            Msg::None => return false,
        }
        true
    }

    fn change(&mut self, _: Self::Properties) -> ShouldRender {
        false
    }

    fn view(&self) -> Html {
        html! {
            <>
                <nav class="navbar navbar-expand-lg fixed-top navbar-light bg-light">
                    <div class="container-fluid">
                        <ul class="navbar-nav mr-auto">
                            <li class="nav-item">
                                <input type="file" onchange=self.link.callback(move |value| {
                                    if let ChangeData::Files(files) = value {
                                        if let Some(file) = files.get(0) {
                                            Msg::UserSelectedFiles(file)
                                        } else {
                                            Msg::None
                                        }
                                    } else {
                                        Msg::None
                                    }
                                })/>
                            </li>
                        </ul>

                        <ul class="navbar-nav mr-auto">
                            <li>
                                <button
                                    type="button"
                                    class="btn btn-link"
                                    onclick=self.link.callback(|_| Msg::Halt)
                                >
                                    <h1>
                                        { if self.halted { html! {
                                            <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" fill="currentColor" class="bi bi-play-circle-fill" viewBox="0 0 16 16">
                                                <path d="M16 8A8 8 0 1 1 0 8a8 8 0 0 1 16 0zM6.79 5.093A.5.5 0 0 0 6 5.5v5a.5.5 0 0 0 .79.407l3.5-2.5a.5.5 0 0 0 0-.814l-3.5-2.5z"/>
                                            </svg>
                                        }} else { html! {
                                            <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" fill="currentColor" class="bi bi-pause-circle-fill" viewBox="0 0 16 16">
                                                <path d="M16 8A8 8 0 1 1 0 8a8 8 0 0 1 16 0zM6.25 5C5.56 5 5 5.56 5 6.25v3.5a1.25 1.25 0 1 0 2.5 0v-3.5C7.5 5.56 6.94 5 6.25 5zm3.5 0c-.69 0-1.25.56-1.25 1.25v3.5a1.25 1.25 0 1 0 2.5 0v-3.5C11 5.56 10.44 5 9.75 5z"/>
                                            </svg>
                                        }}}
                                    </h1>
                                </button>
                            </li>

                        </ul>

                        <form class="my-2 my-lg-0 d-flex">
                            <input
                                class="form-control mr-sm-2"
                                type="search"
                                placeholder="Search"
                                aria-label="Search"
                                oninput=self.link.callback(move |value: InputData| {
                                    Msg::Search(value.value)
                                })
                            />
                        </form>
                    </div>
                </nav>

                <div class="container-fluid main">
                    <div class="row mt-1">
                        <div class="col d-flex align-items-center justify-content-center">
                            { match &self.device {
                                SvdLoadingState::Loaded(device) => html! { <table class="table mt-1">
                                    { for device.peripherals.iter().enumerate().map(|(i, peripheral)| if peripheral.show {
                                        html! {<PeripheralCard
                                            peripheral={ peripheral }
                                            collapsed=i != device.peripherals.len()
                                            watch=&self.watch
                                            set=&self.set
                                        />}
                                    } else {
                                        html!{}
                                    } ) }
                                </table> },
                                SvdLoadingState::Failed(error) => html! { format!("Failed to load the SVG {}", error) },
                                SvdLoadingState::Loading => {
                                    html! { <div class="d-flex flex-column align-items-center">
                                        <div class="spinner-border mb-2" role="status">
                                            <span class="visually-hidden">{ "Loading..." }</span>
                                        </div>
                                        <p>{ "Loading .." }</p>
                                    </div> }
                                }
                            }}
                        </div>
                    </div>
                </div>
            </>
        }
    }
}
