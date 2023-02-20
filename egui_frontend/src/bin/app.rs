use std::sync::mpsc;

use egui_frontend::SimWorker;
use gloo_worker::Spawnable;

fn main() {
    console_error_panic_hook::set_once();
    tracing_wasm::set_as_global_default();

    let web_options = eframe::WebOptions::default();

    wasm_bindgen_futures::spawn_local(async {
        eframe::start_web(
            "the_canvas_id",
            web_options,
            Box::new(|cc| {
                let ctx = cc.egui_ctx.clone();
                let (sender, receiver) = mpsc::channel();
                let bridge = SimWorker::spawner()
                    .callback(move |response| {
                        sender.send(response).unwrap();
                        ctx.request_repaint();
                    })
                    .spawn("./worker.js");

                Box::new(egui_frontend::App::new(cc, receiver, bridge))
            }),
        )
        .await
        .expect("failed to start eframe");
    });
}
