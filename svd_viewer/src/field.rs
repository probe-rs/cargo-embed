use std::ops::RangeInclusive;
use svd_parser::FieldInfo;
use yew::{html, prelude::*, Component, ComponentLink, Html, ShouldRender};
use yewtil::NeqAssign;

pub struct FieldElement {
    _link: ComponentLink<FieldElement>,
    props: Props,
}

pub enum Msg {}

#[derive(Properties, Clone, PartialEq)]
pub struct Props {
    pub info: FieldInfo,
    #[prop_or_default]
    pub offset: u32,
    #[prop_or_default]
    pub index: Option<String>,
    pub value: u32,
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
        html! {
            <div>
                <div class="bg-secondary text-white p-1">
                { if self.props.info.bit_range.msb() != self.props.info.bit_range.lsb() {
                    html! {  <div class="d-flex justify-content-between">
                        <div>{ &self.props.info.bit_range.msb() }</div>
                        <div>{ &self.props.info.bit_range.lsb() }</div>
                    </div> }
                } else {
                    html! {  <div class="d-flex justify-content-center">
                        <div>{ &self.props.info.bit_range.lsb() }</div>
                    </div> }
                }}
                </div>
                <div class="p-1">
                    // <div>{ &self.info.name }</div>
                    <div class="text-center">{
                        print_bits(self.props.value, self.props.info.bit_range.lsb()..=self.props.info.bit_range.msb())
                    }</div>
                </div>
            </div>
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
