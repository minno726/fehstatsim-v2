use sycamore::view;
use sycamore_frontend::app::App;

fn main() {
    console_error_panic_hook::set_once();

    sycamore::render(|cx| {
        view! { cx, App {}}
    });
}
