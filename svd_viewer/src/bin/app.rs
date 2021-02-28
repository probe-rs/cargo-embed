use log::Level;

fn main() {
    console_log::init_with_level(Level::Debug).unwrap();
    yew::start_app::<svd_viewer::Model>();
}
