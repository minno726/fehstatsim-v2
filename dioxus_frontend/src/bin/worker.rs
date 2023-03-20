use gloo_worker::Registrable;
use log::Level;

fn main() {
    wasm_logger::init(wasm_logger::Config::new(Level::Warn));
    console_error_panic_hook::set_once();

    dioxus_frontend::worker::SimWorker::registrar().register();
}
