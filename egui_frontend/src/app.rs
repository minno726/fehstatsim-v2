use egui::Vec2;

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
        write!(
            &mut output,
            "{}%: {}, ",
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
        _cc: &eframe::CreationContext<'_>,
        data_update: Rc<Cell<Option<<SimWorker as Worker>::Output>>>,
        bridge: WorkerBridge<SimWorker>,
    ) -> Self {
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
                    egui::CollapsingHeader::new("Banner")
                        .default_open(true)
                        .show(ui, |ui| {
                            if display_banner(ui, banner) {
                                *data = None;
                                *goal = GoalState::new(banner.current.clone(), goal.is_single);
                            }
                        });

                    egui::CollapsingHeader::new("Goal")
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

    fn max_size_points(&self) -> Vec2 {
        Vec2::new(800.0, f32::INFINITY)
    }
}
