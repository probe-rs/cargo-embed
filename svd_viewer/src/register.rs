use crate::{field::FieldElement, svd::Register};
use svd_parser::Field;
use yew::{html, prelude::*, Component, ComponentLink, Html, ShouldRender};
use yewtil::NeqAssign;

pub struct RegisterElement {
    link: ComponentLink<RegisterElement>,
    props: Props,
    update: Callback<u32>,
    watching: bool,
}

pub enum Msg {
    Updated(u32),
    Watch,
}

#[derive(Properties, Clone, PartialEq)]
pub struct Props {
    pub register: Register,
    pub watch: Callback<(u32, Option<Callback<u32>>)>,
}

impl Component for RegisterElement {
    type Message = Msg;
    type Properties = Props;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        let update = link.callback(move |value| Msg::Updated(value));

        RegisterElement {
            link,
            props,
            update,
            watching: false,
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::Updated(value) => self.props.register.value = value,
            Msg::Watch => {
                self.props
                    .watch
                    .emit((self.props.register.address, Some(self.update.clone())));
                self.watching = !self.watching;
            }
        }
        true
    }

    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        self.props.neq_assign(props)
    }

    fn view(&self) -> Html {
        html! {
            <div class="container-fluid">
                <div class="row">
                    <div class="col-1">
                        <div>{ &self.props.register.name }</div>
                        <div>{ format!("@ {:#08X?}", self.props.register.address) }</div>
                        <div>{ format!("= {:#08X?}", self.props.register.value) }</div>
                    </div>
                    <div class="col-10">
                        { self.props.register.fields.as_ref().map_or_else(
                            || html! { <div class="d-flex justify-content-center align-items-center bg-warning" style="width:100%; height: 100%">{ "UNUSED" }</div> },
                            |fields| html! { <div class="d-flex" style="max-width:100%">
                                { for fields.iter().rev().map(|register| match register {
                                    Field::Single(info) => {
                                        html! { <div style=format!("width:{}%", info.bit_range.width as f32 / 32f32 * 100f32)><FieldElement info=info value=self.props.register.value /></div> }
                                    }
                                    Field::Array(info, dim) => html! { for (0..dim.dim).map(|d| {
                                        html! { <div colspan=info.bit_range.width><FieldElement
                                            info=info
                                            offset={ dim.dim_increment * d }
                                            index={ dim.dim_index.as_ref().map(|i| i.get(d as usize).map(Clone::clone)).flatten() },
                                            value=self.props.register.value
                                        /></div> }
                                    }) },
                                } ) }
                            </div> }
                        ) }
                    </div>
                    <div class="col-1">
                        <button
                            class="btn btn-outline-primary"
                            type="button"
                            onclick=self.link.callback(move |_| {
                                Msg::Watch
                            })
                        >
                        { if self.watching {
                            html! { <>
                                <span class="spinner-grow spinner-grow-sm" role="status" aria-hidden="true"></span>
                                <span class="sr-only">{ "Loading..." }</span>
                            </> }
                         } else {
                             html! { <svg width="1em" height="1em" viewBox="0 0 16 16" class="bi bi-eye-fill" fill="currentColor" xmlns="http://www.w3.org/2000/svg">
                                <path d="M10.5 8a2.5 2.5 0 1 1-5 0 2.5 2.5 0 0 1 5 0z"/>
                                <path fill-rule="evenodd" d="M0 8s3-5.5 8-5.5S16 8 16 8s-3 5.5-8 5.5S0 8 0 8zm8 3.5a3.5 3.5 0 1 0 0-7 3.5 3.5 0 0 0 0 7z"/>
                            </svg> }
                         } }
                        </button>
                    </div>
                </div>
            </div>
        }
    }
}
