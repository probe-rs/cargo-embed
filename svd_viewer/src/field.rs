use std::ops::Range;
use svd_parser::BitRange;
use svd_parser::EnumeratedValues;
use svd_parser::{bitrange::BitRangeType, Usage};
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
        false
    }

    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        if self.props.neq_assign(props) {
            if extract_value(self) != 0 {
                return true;
            }
        }
        false
    }

    fn view(&self) -> Html {
        let value = action_view(self);
        html! { <>
            <td>{ &self.props.name }</td>
            <td>{ self.props.address.map_or_else(|| "".to_string(), |v| format!("0x{:08X?}", v)) }</td>
            <td>{ value }</td>
            <td> { if let Some(bit_range) = self.props.bit_range { html! { <>
                { display_bits(bit_range.msb() + 1..31 + 1, self.props.value, false) }
                { display_bits(bit_range.lsb()..bit_range.msb() + 1, self.props.value, true) }
                { display_bits(0..bit_range.lsb(), self.props.value, false) }
            </> } } else {
                display_bits(0..32, self.props.value, true)
            } } </td>
        </> }
    }
}

fn action_view(field: &FieldElement) -> VNode {
    let value = extract_value(&field);
    if field.props.enumerated_values.len() > 1 {
        if let Some(br) = field.props.bit_range {
            if br.msb() - br.lsb() == 0 {
                log::info!("{:?}", field.props.enumerated_values);
                let read = field
                    .props
                    .enumerated_values
                    .iter()
                    .find(|ev| ev.usage == Some(Usage::Read));
                let write = field
                    .props
                    .enumerated_values
                    .iter()
                    .find(|ev| ev.usage == Some(Usage::Write));
                if let (Some(read), Some(write)) = (read, write) {
                    // if value == 1 {
                    //     html! { <button> { &read.values[0].name } </button> }
                    // } else {
                    //     html! { <button> { &read.values[0].name } </button> }
                    // }

                    html! { <select>
                        { for read.values.iter().map(|ev| html! { <option>
                            { &ev.name }
                        </option> }) }
                    </select> }
                } else {
                    log::error!("The SVD file is corrupt. Not both, a read and a write behavior, were specified. Options were: {:?}, {:?}", read, write);
                    html! { format!("0x{:08X?}", value) }
                }
            } else {
                log::error!("This case is unexpected and unimplemented. Please report a bug.");
                html! { format!("0x{:08X?}", value) }
            }
        } else {
            html! { format!("0x{:08X?}", value) }
        }
    } else {
        if let Some(enumerated_values) = field.props.enumerated_values.first() {
            match enumerated_values.usage {
                Some(Usage::Read) => {
                    // Read only field.
                    let value = field.props.bit_range.map_or(value, |r| value >> r.msb());
                    return enumerated_values
                        .values
                        .iter()
                        .find(|ev| ev.value == Some(value))
                        .map_or_else(
                            || html! { format!("0x{:08X?}", value) },
                            |ev| html! { format!("0x{:08X?}", ev.name) },
                        );
                }
                Some(Usage::Write) => {
                    log::error!("Single Write only fields are not implemented yet!");
                    html! { format!("0x{:08X?}", value) }
                }
                Some(Usage::ReadWrite) | None => {
                    html! { <select> {
                        for enumerated_values.values.iter().map(|ev| html! { <option
                            selected=Some(value) == ev.value
                            value=value
                        >
                            { &ev.name }
                        </option> })
                    } </select> }
                }
            }
        } else {
            html! { format!("0x{:08X?}",  extract_value(&field)) }
        }
    }
}

fn display_bits(range: Range<u32>, value: u32, active: bool) -> VNode {
    html! { for range.rev().into_iter().map(|i| html! {
        <div
            class=(
                "bit",
                "border",
                "m-1",
                "text-center",
                if active { "border-primary" } else { "border-light" },
                if active && ((value >> i) & 1 == 1) { "bg-primary" } else { "bg-white" },
                if active && ((value >> i) & 1 == 1) { "text-white" } else { "text-primary" }
            )>
            { if active { format!("{}", i) } else { "".to_string() } }
        </div>
    }) }
}

fn extract_value(field: &FieldElement) -> u32 {
    let bit_range = field.props.bit_range.unwrap_or_else(|| BitRange {
        offset: 0,
        width: 32,
        range_type: BitRangeType::BitRange,
    });
    let left_shift = 31 - bit_range.msb();

    (field.props.value << left_shift) >> (bit_range.lsb() + left_shift)
}
