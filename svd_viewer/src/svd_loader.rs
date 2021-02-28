use yew::{
    services::reader::FileData,
    worker::{Agent, AgentLink, HandlerId, Public},
};

use crate::DeviceState;

pub enum Msg {
    Loaded(FileData),
}

pub struct Worker {
    link: AgentLink<Worker>,
}

impl Agent for Worker {
    type Reach = Public<Self>;
    type Message = Msg;
    type Input = String;
    type Output = DeviceState;

    fn create(link: AgentLink<Self>) -> Self {
        Self { link }
    }

    fn update(&mut self, msg: Self::Message) {
        match msg {
            Msg::Loaded(_file_data) => {}
        }
    }

    fn handle_input(&mut self, msg: Self::Input, who: HandlerId) {
        let device_state = Self::parse_svd(&msg);
        self.link.respond(who, device_state);
    }

    fn name_of_resource() -> &'static str {
        "worker.js"
    }
}

impl Worker {
    fn parse_svd(svd: &str) -> DeviceState {
        log::debug!("Start parsing ..");
        let device_state = svd_parser::parse(svd).map(From::from);
        let device_state = match device_state {
            Ok(device) => DeviceState::Loaded(device),
            Err(error) => DeviceState::Failed(error.to_string()),
        };
        log::debug!("Done parsing ..");
        device_state
    }
}
