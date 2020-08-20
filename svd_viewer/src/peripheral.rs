use crate::{register::RegisterElement, svd::Peripheral};
use yew::{html, prelude::*, Component, ComponentLink, Html, ShouldRender};
use yewtil::NeqAssign;

pub struct PeripheralCard {
    link: ComponentLink<PeripheralCard>,
    props: Props,
    collapsed: bool,
}

pub enum Msg {
    Collapse,
    Watch((u32, Option<Callback<u32>>)),
}

#[derive(Properties, Clone, PartialEq)]
pub struct Props {
    pub peripheral: Peripheral,
    pub watch: Callback<(u32, Option<Callback<u32>>)>,
}

impl Component for PeripheralCard {
    type Message = Msg;
    type Properties = Props;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        PeripheralCard {
            link,
            props,
            collapsed: true,
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::Collapse => self.collapsed = !self.collapsed,
            Msg::Watch(address) => self.props.watch.emit(address),
        }
        true
    }

    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        self.props.neq_assign(props)
    }

    fn view(&self) -> Html {
        html! {
            <div class="card mt-1">
                <div class="card-header">
                    <div class="d-flex w-100 justify-content-between align-items-center">
                        <h5 class="mb-1">{self.props.peripheral.display_name.as_ref().unwrap_or(&self.props.peripheral.name)}</h5>
                        <span>{ format!("{:#08X?}", self.props.peripheral.base_address) }</span>
                        <span>{ self.props.peripheral.description.as_deref().unwrap_or("") }</span>
                        <small>{"3 days ago"}</small>
                        <button type="button" class="btn btn-primary" onclick=self.link.callback(move |value| {
                            Msg::Collapse
                        })>{ "Show" }</button>
                    </div>
                </div>
                <div class=("card-body", "collapse" , if self.collapsed { "" } else { "show" })>
                    { for self.props.peripheral.registers.iter().map(|register| html! { <RegisterElement
                        register={register}
                        watch=self.link.callback(move |value| {
                            Msg::Watch(value)
                        })
                    /> } ) }
                </div>
            </div>
        }
    }
}
