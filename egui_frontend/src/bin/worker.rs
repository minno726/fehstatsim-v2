use gloo_worker::Registrable;

use egui_frontend::SimWorker;

fn main() {
    console_error_panic_hook::set_once();

    SimWorker::registrar().register();
}
