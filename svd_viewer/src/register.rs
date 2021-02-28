use crate::{field::FieldElement, svd::Register};
use svd_parser::Field;
use yew::{html, prelude::*, Component, ComponentLink, Html, ShouldRender};
use yewtil::NeqAssign;

pub struct RegisterElement {
    link: ComponentLink<RegisterElement>,
    props: Props,
    watching: bool,
    pub set: Callback<u32>,
}

pub enum Msg {
    Watch,
    None,
}

#[derive(Properties, Clone, PartialEq)]
pub struct Props {
    pub register: Register,
    pub watch: Callback<u32>,
    pub set: Callback<(u32, u32)>,
}

impl Component for RegisterElement {
    type Message = Msg;
    type Properties = Props;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        let p = props.clone();
        let set = link.callback(move |value| {
            p.set.emit((p.register.address, value));
            Msg::None
        });
        RegisterElement {
            link,
            props,
            watching: false,
            set,
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::Watch => {
                self.props.watch.emit(self.props.register.address);
                self.watching = !self.watching;
                true
            }
            Msg::None => false,
        }
    }

    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        self.props.neq_assign(props)
    }

    fn view(&self) -> Html {
        if self.watching {
            html! { <>
                <tr>
                    <FieldElement
                        name=self.props.register.name.clone()
                        value=self.props.register.value
                        address=self.props.register.address
                        set=&self.set
                    />
                    <td>
                        <button
                            class="btn btn-outline-danger btn-watch"
                            type="button"
                            onclick=self.link.callback(move |_| {
                                Msg::Watch
                            })
                        >
                            { "Unwatch" }
                        </button>
                    </td>
                </tr>

                { self.props.register.fields.as_ref().map_or_else(
                    || html! { },
                    |fields| if fields.len() > 1 {
                        html! { { for fields.iter().rev().map(|register| match &register.0 {
                            Field::Single(info) => { html! {
                                <tr><FieldElement
                                    name=info.name.clone()
                                    value=self.props.register.value
                                    bit_range=Some(info.bit_range)
                                    enumerated_values=info.enumerated_values.clone()
                                    set=&self.set
                                /></tr>
                            } }
                            Field::Array(info, dim) => html! { for (0..dim.dim).map(|d| {
                                html! { <tr><FieldElement
                                    name=info.name.clone()
                                    offset={ dim.dim_increment * d }
                                    index={ dim.dim_index.as_ref().map(|i| i.get(d as usize).map(Clone::clone)).flatten() }
                                    value=self.props.register.value
                                    bit_range=Some(info.bit_range)
                                    enumerated_values=info.enumerated_values.clone()
                                    set=&self.set
                                /></tr> }
                            }) },
                        } ) }
                        }
                    } else {
                        html! {}
                    }
                ) }
            </> }
        } else {
            html! { <tr>
                <td colspan="4">{ &self.props.register.name }</td>
                <td>
                    <button
                        class="btn btn-outline-primary btn-watch"
                        type="button"
                        onclick=self.link.callback(move |_| {
                            Msg::Watch
                        })
                    >
                        { "Watch" }
                    </button>
                </td>
            </tr> }
        }
    }
}
