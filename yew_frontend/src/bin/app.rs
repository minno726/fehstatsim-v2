use yew_frontend::app::App;

fn main() {
    console_error_panic_hook::set_once();

    yew::Renderer::<App>::new().render();
}
