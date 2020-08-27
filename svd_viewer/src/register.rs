use crate::{field::FieldElement, svd::Register};
use svd_parser::svd::bitrange::BitRangeType;
use svd_parser::BitRange;
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
        html! { <>
            <tr style="border-top: 4px solid black;">
                <FieldElement
                    name=self.props.register.name.clone()
                    value=self.props.register.value
                    bit_range=BitRange { offset: 0, width: 32, range_type: BitRangeType::BitRange }
                    address=self.props.register.address
                />
                <td>
                    <button
                        class="btn btn-link"
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
                </td>
            </tr>
            { self.props.register.fields.as_ref().map_or_else(
                || html! { },
                |fields| if fields.len() > 1 {
                    html! { { for fields.iter().rev().map(|register| match register {
                        Field::Single(info) => { html! {
                            <tr><FieldElement
                                name=info.name.clone()
                                value=self.props.register.value
                                bit_range=info.bit_range
                                enumerated_values=info.enumerated_values.clone()
                            /></tr>
                        } }
                        Field::Array(info, dim) => html! { for (0..dim.dim).map(|d| {
                            html! { <tr><FieldElement
                                name=info.name.clone()
                                offset={ dim.dim_increment * d }
                                index={ dim.dim_index.as_ref().map(|i| i.get(d as usize).map(Clone::clone)).flatten() },
                                value=self.props.register.value
                                bit_range=info.bit_range
                                enumerated_values=info.enumerated_values.clone()
                            /></tr> }
                        }) },
                    } ) }
                    }
                } else {
                    html! {}
                }
            ) }
        </> }
    }
}
