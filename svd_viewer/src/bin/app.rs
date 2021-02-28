use log::Level;

pub fn main() {
    console_log::init_with_level(Level::Debug).unwrap();
    yew::start_app::<svd_viewer::Model>();
}
