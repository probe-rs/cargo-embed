use serde::{Deserialize, Serialize};
use yew::worker::{Agent, AgentLink, HandlerId, Public};

use crate::svd::Device;

/// The state of our SVD representation.
/// Tells whether it's loaded or not.
#[derive(Debug, Serialize, Deserialize)]
pub enum SvdLoadingState {
    Loading,
    Loaded(Device),
    Failed(String),
}

/// An async worker which can load a given SVD in the background.
pub struct Loader {
    link: AgentLink<Loader>,
}

impl Agent for Loader {
    type Reach = Public<Self>;
    type Message = ();
    type Input = String;
    type Output = SvdLoadingState;

    fn create(link: AgentLink<Self>) -> Self {
        Self { link }
    }

    fn update(&mut self, _: Self::Message) {}

    fn handle_input(&mut self, msg: Self::Input, who: HandlerId) {
        let device_state = Self::parse_svd(&msg);
        self.link.respond(who, device_state);
    }

    fn name_of_resource() -> &'static str {
        "worker.js"
    }
}

impl Loader {
    /// Parses a string containing valid SVD XML into a Device representation which later can be used to display data.
    /// The returned loading state tells if the SVD was parsed successfully or not.
    fn parse_svd(svd: &str) -> SvdLoadingState {
        let device_state = svd_parser::parse(svd).map(From::from);
        match device_state {
            Ok(device) => SvdLoadingState::Loaded(device),
            Err(error) => SvdLoadingState::Failed(error.to_string()),
        }
    }
}
