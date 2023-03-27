use yew_agent::PublicWorker;
use yew_frontend::agent::SimWorker;

fn main() {
    console_error_panic_hook::set_once();

    SimWorker::register();
}
