#![recursion_limit = "1024"]

mod field;
mod file;
mod peripheral;
mod register;
mod svd;
pub mod svd_loader;

use anyhow::Context;
use peripheral::PeripheralCard;
use yew::{
    prelude::*,
    services::{
        reader::{File, FileData, ReaderTask},
        websocket::{WebSocketStatus, WebSocketTask},
        ReaderService, WebSocketService,
    },
};

use interface::{Command, Register, Update};
use serde::{Deserialize, Serialize};
use svd::Device;

pub struct Model {
    link: ComponentLink<Model>,
    reader: ReaderService,
    reader_tasks: Vec<ReaderTask>,
    websocket_task: Option<WebSocketTask>,
    device: DeviceState,
    poll_interval: usize,
    watching_addresses: Vec<u32>,
    watch: Callback<u32>,
    set: Callback<(u32, u32)>,
    loader: Box<dyn yew::Bridge<svd_loader::Worker>>,
}

pub enum WebsocketEvent {
    SendData(Command),
    Opened,
    Lost,
}

pub enum Msg {
    Loaded(FileData),
    SvdParsed(DeviceState),
    Files(File),
    UpdatePollInterval(usize),
    WebSocketData(Update),
    WebsocketEvent(WebsocketEvent),
    Watch(u32),
    Set((u32, u32)),
    None,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum DeviceState {
    Loading,
    Loaded(Device),
    Failed(String),
}

impl Component for Model {
    type Message = Msg;
    type Properties = ();

    fn create(_: Self::Properties, mut link: ComponentLink<Self>) -> Self {
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

        let device = DeviceState::Loading;

        let callback = link.callback(|device| Msg::SvdParsed(device));
        let mut loader = svd_loader::Worker::bridge(callback);
        loader.send(crate::file::TEST_SVD.into());
        log::debug!("AFTER");

        Model {
            link,
            reader: ReaderService::new(),
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
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::Loaded(filedata) => {
                let xml = String::from_utf8(filedata.content)
                    .context("The SVD file appears to contain invalid UTF8 data.");
                match xml {
                    Ok(xml) => self.loader.send(xml),
                    Err(error) => {
                        self.update(Msg::SvdParsed(DeviceState::Failed(error.to_string())));
                    }
                }
            }
            Msg::SvdParsed(device) => {
                self.device = device;
                log::debug!("HEREHEREHERE");
            }
            Msg::Files(file) => {
                let callback = self.link.callback(Msg::Loaded);
                let task = self.reader.read_file(file, callback).unwrap();
                self.reader_tasks.push(task);
            }
            Msg::UpdatePollInterval(ms) => self.poll_interval = ms,
            Msg::WebSocketData(data) => {
                match data {
                    Update::Registers(register_updates) => {
                        if let DeviceState::Loaded(device) = &mut self.device {
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
                    let command = Command::UpdateInterval(self.poll_interval);
                    let data = serde_json::to_string(&command).unwrap();
                    self.websocket_task.as_mut().unwrap().send(Ok(data));

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
                <nav class="navbar navbar-expand-lg navbar-light bg-light">
                    <a class="navbar-brand" href="#">{ "SVD Viewer" }</a>
                    <div class="collapse navbar-collapse" id="navbar">
                        <ul class="navbar-nav mr-auto">
                            <li class="nav-item">
                                <input type="file" onchange=self.link.callback(move |value| {
                                    if let ChangeData::Files(files) = value {
                                        if let Some(file) = files.get(0) {
                                            Msg::Files(file)
                                        } else {
                                            Msg::None
                                        }
                                    } else {
                                        Msg::None
                                    }
                                })/>
                            </li>
                            <li>
                                <input
                                    class="form-control mr-sm-2"
                                    type="text"
                                    placeholder="Update Interval"
                                    aria-label="Update Interval"
                                    oninput=self.link.callback(move |value: InputData| {
                                        if let Ok(value) = value.value.parse::<usize>() {
                                            return Msg::UpdatePollInterval(value);
                                        }
                                        Msg::None
                                    })
                                    value=self.poll_interval
                                />
                            </li>
                        </ul>
                        <form class="my-2 my-lg-0 d-flex">
                            <input class="form-control mr-sm-2" type="search" placeholder="Search" aria-label="Search" />
                            <button class="btn btn-outline-success my-2 my-sm-0" type="submit">{ "Search" }</button>
                        </form>
                    </div>
                </nav>

                <div class="container-fluid main">
                    <div class="row mt-1">
                        <div class="col d-flex align-items-center justify-content-center">
                            { match &self.device {
                                DeviceState::Loaded(device) => html! { <table class="table mt-1">
                                    { for device.peripherals.iter().enumerate().map(|(i, peripheral)| html! {<PeripheralCard
                                        peripheral={peripheral}
                                        collapsed=(i!=device.peripherals.len())
                                        watch=&self.watch
                                        set=&self.set
                                    />}) }
                                </table> },
                                DeviceState::Failed(error) => html! { format!("Failed to load the SVG {}", error) },
                                DeviceState::Loading => {
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
// #[wasm_bindgen(start)]
// pub fn run_app() {
//     console_log::init_with_level(Level::Debug).unwrap();
//     yew::start_app::<Model>();
// }
