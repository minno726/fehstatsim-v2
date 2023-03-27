use gloo_worker::Registrable;

use sycamore_frontend::worker::SimWorker;

fn main() {
    console_error_panic_hook::set_once();

    SimWorker::registrar().register();
}
