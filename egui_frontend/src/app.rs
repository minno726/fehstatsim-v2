use egui::RichText;
use gloo_console::log;
use gloo_worker::{Worker, WorkerBridge};
use std::{cell::Cell, fmt::Write, rc::Rc, time::Duration};
use summon_simulator::frequency_counter::FrequencyCounter;

use crate::{
    banner::{display_banner, BannerState},
    goal::{display_goal, GoalState},
    SimWorker, SimWorkerInput,
};

fn percentiles(data: &FrequencyCounter, values: &[f32]) -> Vec<u32> {
    let total = data.iter().sum::<u32>();
    let mut cum_total = 0;
    let mut cur_value_idx = 0;
    let mut result = Vec::new();
    for (i, &data_point) in data.iter().enumerate() {
        cum_total += data_point;
        if cum_total as f32 > total as f32 * values[cur_value_idx] {
            result.push(i as u32);
            cur_value_idx += 1;
            if cur_value_idx >= values.len() {
                return result;
            }
        }
    }
    while result.len() < values.len() {
        result.push((data.len() - 1) as u32);
    }
    result
}

fn data_percentiles_to_string(data: &FrequencyCounter) -> String {
    let sample_percentiles = [0.25f32, 0.5, 0.75, 0.9, 0.99, 1.0];
    let data = percentiles(data, &sample_percentiles);
    let mut output = String::new();
    for i in 0..sample_percentiles.len() {
        writeln!(
            &mut output,
            "{}%: {}",
            (sample_percentiles[i] * 100.0).round() as u32,
            data[i]
        )
        .unwrap();
    }
    output
}

pub struct App {
    // data
    data: Option<FrequencyCounter>,
    banner: BannerState,
    goal: GoalState,

    // status
    is_running: bool,

    // communication
    data_update: Rc<Cell<Option<<SimWorker as Worker>::Output>>>,
    bridge: WorkerBridge<SimWorker>,
}

impl App {
    pub fn new(
        cc: &eframe::CreationContext<'_>,
        data_update: Rc<Cell<Option<<SimWorker as Worker>::Output>>>,
        bridge: WorkerBridge<SimWorker>,
    ) -> Self {
        App::set_text_styles(cc);
        let banner = BannerState::new();
        let goal = GoalState::new(banner.current.clone(), true);
        App {
            data: None,
            data_update,
            bridge,
            is_running: false,
            banner,
            goal,
        }
    }

    fn set_text_styles(ctx: &eframe::CreationContext<'_>) {
        use egui::FontFamily::Proportional;
        use egui::FontId;
        use egui::TextStyle::*;
        ctx.egui_ctx.style_mut(|style| {
            style.text_styles = [
                (Heading, FontId::new(30.0, Proportional)),
                (Name("Heading2".into()), FontId::new(25.0, Proportional)),
                (Name("Context".into()), FontId::new(23.0, Proportional)),
                (Body, FontId::new(18.0, Proportional)),
                (Monospace, FontId::new(18.0, Proportional)),
                (Button, FontId::new(18.0, Proportional)),
                (Small, FontId::new(14.0, Proportional)),
            ]
            .into();
        });
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let Self {
            data,
            data_update,
            bridge,
            is_running,
            banner,
            goal,
        } = self;

        if let Some(worker_response) = data_update.replace(None) {
            if *is_running {
                *data = Some(worker_response);
            }
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            egui::ScrollArea::both()
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    egui::CollapsingHeader::new(RichText::new("Banner").heading())
                        .default_open(true)
                        .show(ui, |ui| {
                            if display_banner(ui, banner) {
                                *data = None;
                                *goal = GoalState::new(banner.current.clone(), goal.is_single);
                            }
                        });

                    egui::CollapsingHeader::new(RichText::new("Goal").heading())
                        .default_open(true)
                        .show(ui, |ui| {
                            if display_goal(ui, goal) {
                                *data = None;
                            }
                        });

                    if *is_running {
                        if ui.button("Stop").clicked() {
                            log!("Stop clicked");
                            bridge.send(SimWorkerInput::Stop);
                            *is_running = false;
                        }
                    } else {
                        let button = egui::Button::new("Run");
                        if let Some(sim_banner) = banner.current.to_sim_banner() {
                            if let Some(sim_goal) = goal.to_sim_goal() {
                                if ui.add(button).clicked() {
                                    log!("Run clicked");
                                    bridge.send(SimWorkerInput::Run {
                                        banner: sim_banner,
                                        goal: sim_goal,
                                        target_interval: Duration::from_millis(500),
                                    });
                                    *is_running = true;
                                }
                            } else {
                                ui.add_enabled(false, button)
                                    .on_disabled_hover_text("Invalid goal.");
                            }
                        } else {
                            ui.add_enabled(false, button)
                                .on_disabled_hover_text("Invalid banner.");
                        }
                    }
                    ui.label(
                        data.as_ref()
                            .map(data_percentiles_to_string)
                            .unwrap_or("Not run yet".into()),
                    );
                })
        });
    }
}
