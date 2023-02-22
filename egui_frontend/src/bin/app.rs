use std::{cell::Cell, rc::Rc};

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
                let data_update = Rc::new(Cell::new(None));
                let sender = data_update.clone();
                let bridge = SimWorker::spawner()
                    .callback(move |response| {
                        sender.set(Some(response));
                        ctx.request_repaint();
                    })
                    .spawn("./worker.js");

                Box::new(egui_frontend::App::new(cc, data_update, bridge))
            }),
        )
        .await
        .expect("failed to start eframe");
    });
}
