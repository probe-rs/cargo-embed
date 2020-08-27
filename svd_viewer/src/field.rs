use std::ops::RangeInclusive;
use svd_parser::BitRange;
use svd_parser::EnumeratedValues;
use svd_parser::Usage;
use yew::{html, prelude::*, Component, ComponentLink, Html, ShouldRender};
use yewtil::NeqAssign;

pub struct FieldElement {
    _link: ComponentLink<FieldElement>,
    props: Props,
}

pub enum Msg {}

#[derive(Properties, Clone, PartialEq)]
pub struct Props {
    #[prop_or_default]
    pub offset: u32,
    #[prop_or_default]
    pub index: Option<String>,
    pub name: String,
    #[prop_or_default]
    pub address: Option<u32>,
    pub value: u32,
    pub bit_range: BitRange,
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
        if &self.props.name == "SENSE" {
            log::info!("{:#?}", &self.props.enumerated_values);
        }
        let value = if let Some(enumerated_values) =
            self.props.enumerated_values.iter().find(|ev| {
                ev.usage == None
                    || ev.usage == Some(Usage::Read)
                    || ev.usage == Some(Usage::ReadWrite)
            }) {
            html! { <select> { for enumerated_values.values.iter().map(|ev| html! { <option>
                { &ev.name }
            </option> }) } </select>}
        } else {
            html! { format!("{:#08X?}", self.props.value) }
        };
        html! { <>
            <td>{ &self.props.name }</td>
            <td>{ self.props.address.map_or_else(|| "".to_string(), |v| format!("{:#08X?}", v)) }</td>
            <td>{ value }</td>
            <td>
                { for (self.props.bit_range.msb() + 1..31 + 1).rev().into_iter().map(|i| html! {
                    <div class="border border-light m-1" style="width: 20px; height: 20px; float: left;"></div>
                }) }
                { for (self.props.bit_range.lsb()..self.props.bit_range.msb() + 1).rev().into_iter().map(|i| html! {
                    <div class="border border-primary m-1 p-1" style="width: 20px; height: 20px; float: left; font-size: 10px;">{ i }</div>
                }) }
                { for (0..self.props.bit_range.lsb()).rev().into_iter().map(|i| html! {
                    <div class="border border-light m-1" style="width: 20px; height: 20px; float: left;"></div>
                }) }
            </td>
        </> }
    }
}

fn print_bits(value: u32, range: RangeInclusive<u32>) -> String {
    let mut s = String::new();
    for i in range {
        s += if (value >> i) & 1 == 1 { "1" } else { "0" };
    }
    s
}
