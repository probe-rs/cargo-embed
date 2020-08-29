use std::ops::RangeInclusive;
use svd_parser::BitRange;
use svd_parser::EnumeratedValues;
use svd_parser::Usage;
use yew::{html, prelude::*, virtual_dom::VNode, Component, ComponentLink, Html, ShouldRender};
use yewtil::NeqAssign;

pub struct FieldElement {
    _link: ComponentLink<FieldElement>,
    props: Props,
}

pub enum Msg {}

#[derive(Properties, Clone, PartialEq)]
pub struct Props {
    pub name: String,
    pub value: u32,
    #[prop_or_default]
    pub bit_range: Option<BitRange>,
    #[prop_or_default]
    pub offset: u32,
    #[prop_or_default]
    pub index: Option<String>,
    #[prop_or_default]
    pub address: Option<u32>,
    #[prop_or_default]
    pub enumerated_values: Vec<EnumeratedValues>,
}

impl Component for FieldElement {
    type Message = Msg;
    type Properties = Props;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        FieldElement {
            _link: link,
            props: props,
        }
    }

    fn update(&mut self, _msg: Self::Message) -> ShouldRender {
        true
    }

    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        self.props.neq_assign(props)
    }

    fn view(&self) -> Html {
        let value = action_view(self);
        html! { <>
            <td>{ &self.props.name }</td>
            <td>{ self.props.address.map_or_else(|| "".to_string(), |v| format!("{:#08X?}", v)) }</td>
            <td>{ value }</td>
            <td> { if let Some(bit_range) = self.props.bit_range { html! { <>
                { for (bit_range.msb() + 1..31 + 1).rev().into_iter().map(|i| html! {
                    <div class="border border-light m-1" style="width: 20px; height: 20px; float: left;"></div>
                }) }
                { for (bit_range.lsb()..bit_range.msb() + 1).rev().into_iter().map(|i| html! {
                    <div class="border border-primary m-1 p-1" style="width: 20px; height: 20px; float: left; font-size: 10px;">{ i }</div>
                }) }
                { for (0..bit_range.lsb()).rev().into_iter().map(|i| html! {
                    <div class="border border-light m-1" style="width: 20px; height: 20px; float: left;"></div>
                }) }
            </> } } else {
                html! { for (0..32).rev().into_iter().map(|i| html! {
                    <div class="border border-light m-1" style="width: 20px; height: 20px; float: left;"></div>
                } ) }
            } } </td>
        </> }
    }
}

fn action_view(field: &FieldElement) -> VNode {
    if field.props.enumerated_values.len() > 1 {
        log::info!("{:?}", field.props.enumerated_values);
        if let Some(enumerated_values) = field.props.enumerated_values.iter().find(|ev| {
            ev.usage == None || ev.usage == Some(Usage::Read) || ev.usage == Some(Usage::ReadWrite)
        }) {
            html! { <select> { for enumerated_values.values.iter().map(|ev| html! { <option>
                { &ev.name }
            </option> }) } </select>}
        } else {
            html! { format!("{:#08X?}", field.props.value) }
        }
    } else {
        if let Some(enumerated_values) = field.props.enumerated_values.first() {
            match enumerated_values.usage {
                Some(Usage::Read) => {
                    // Read only field.
                    let value = field
                        .props
                        .bit_range
                        .map_or(field.props.value, |r| field.props.value >> r.msb());
                    return enumerated_values
                        .values
                        .iter()
                        .find(|ev| ev.value == Some(value))
                        .map_or_else(
                            || html! { format!("{:#08X?}", value) },
                            |ev| html! { format!("{:#08X?}", ev.name) },
                        );
                }
                Some(Usage::Write) => {
                    log::error!("Single Write only fields are not implemented yet!");
                    html! {}
                }
                Some(Usage::ReadWrite) | None => {
                    html! { <select> {
                        for enumerated_values.values.iter().map(|ev| html! { <option
                            selected=Some(field.props.value) == ev.value
                            value=field.props.value
                        >
                            { &ev.name }
                        </option> })
                    } </select> }
                }
            }
        } else {
            html! { format!("{:#08X?}", field.props.value) }
        }
    }
}

fn print_bits(value: u32, range: RangeInclusive<u32>) -> String {
    let mut s = String::new();
    for i in range {
        s += if (value >> i) & 1 == 1 { "1" } else { "0" };
    }
    s
}
