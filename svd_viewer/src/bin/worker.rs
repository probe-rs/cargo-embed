use svd_viewer::svd_loader::Loader;
use yew::agent::Threaded;

fn main() {
    wasm_logger::init(wasm_logger::Config::default());
    Loader::register();
}
