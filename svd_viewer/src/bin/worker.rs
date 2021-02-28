use svd_viewer::svd_loader::Worker;
use yew::agent::Threaded;

fn main() {
    wasm_logger::init(wasm_logger::Config::default());
    Worker::register();
}
