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
}

#[derive(Properties, Clone, PartialEq)]
pub struct Props {
    pub peripheral: Peripheral,
    pub watch: Callback<u32>,
    pub set: Callback<(u32, u32)>,
    pub collapsed: bool,
}

impl Component for PeripheralCard {
    type Message = Msg;
    type Properties = Props;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        PeripheralCard {
            link,
            collapsed: props.collapsed,
            props,
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::Collapse => self.collapsed = !self.collapsed,
        }
        true
    }

    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        self.props.neq_assign(props)
    }

    fn view(&self) -> Html {
        html! { <>
            <tr>
                <td>
                    <button type="button" class="m-0 p-0 btn btn-link" onclick=self.link.callback(move |_value| {
                        Msg::Collapse
                    })>
                        { if self.collapsed { html! {
                            <svg width="1em" height="1em" viewBox="0 0 16 16" class="bi bi-caret-right-fill" fill="currentColor" xmlns="http://www.w3.org/2000/svg">
                                <path d="M12.14 8.753l-5.482 4.796c-.646.566-1.658.106-1.658-.753V3.204a1 1 0 0 1 1.659-.753l5.48 4.796a1 1 0 0 1 0 1.506z"/>
                            </svg>
                        } } else { html! {
                            <svg width="1em" height="1em" viewBox="0 0 16 16" class="bi bi-caret-down-fill" fill="currentColor" xmlns="http://www.w3.org/2000/svg">
                                <path d="M7.247 11.14L2.451 5.658C1.885 5.013 2.345 4 3.204 4h9.592a1 1 0 0 1 .753 1.659l-4.796 5.48a1 1 0 0 1-1.506 0z"/>
                            </svg>
                        } } }
                    </button>
                </td>
                <td>
                    {self.props.peripheral.display_name.as_ref().unwrap_or(&self.props.peripheral.name)}
                </td>
                <td>
                    { format!("{:#08X?}", self.props.peripheral.base_address) }
                </td>
                <td>
                    { self.props.peripheral.description.as_deref().unwrap_or("") }
                </td>
                <td>
                </td>
            </tr>
            {if !self.collapsed { html! {<tr>
                <td colspan=5>
                    <table class="table">
                    { for self.props.peripheral.registers.iter().enumerate().map(|(i, register)| html! { <RegisterElement
                        key=i
                        register={register}
                        watch=&self.props.watch
                        set=&self.props.set
                    /> } ) }
                    </table>
                </td>
            </tr> }} else { html! {} }}
        </> }
    }
}
