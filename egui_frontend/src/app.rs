use egui::{text::LayoutJob, Color32, FontId, RichText, TextFormat};
use gloo_console::log;
use gloo_worker::{Worker, WorkerBridge};
use std::{cell::Cell, fmt::Write, rc::Rc, time::Duration};
use summon_simulator::{frequency_counter::FrequencyCounter, types::Color};

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

pub(crate) fn with_colored_dot(text: &str, color: Color, font: FontId) -> LayoutJob {
    let mut job = LayoutJob::default();
    job.append(
        "⏺",
        0.0,
        TextFormat {
            color: match color {
                Color::Red => Color32::from_rgb(180, 58, 75),
                Color::Blue => Color32::from_rgb(54, 96, 198),
                Color::Green => Color32::from_rgb(79, 171, 62),
                Color::Colorless => Color32::from_rgb(87, 102, 109),
            },
            font_id: font.clone(),
            ..Default::default()
        },
    );
    job.append(
        text,
        4.0,
        TextFormat {
            font_id: font,
            color: Color32::PLACEHOLDER,
            ..Default::default()
        },
    );
    job
}

#[derive(Debug, PartialEq)]
pub enum Data {
    Present(FrequencyCounter),
    Waiting,
    Invalidated,
}

pub struct App {
    // data
    data: Data,
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
            data: Data::Waiting,
            data_update,
            bridge,
            is_running: false,
            banner,
            goal,
        }
    }

    fn set_text_styles(ctx: &eframe::CreationContext<'_>) {
        use egui::FontFamily::Proportional;
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
            if *is_running && *data != Data::Invalidated {
                *data = Data::Present(worker_response);
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
                                bridge.send(SimWorkerInput::Stop);
                                *data = Data::Invalidated;
                                *goal = GoalState::new(banner.current.clone(), goal.is_single);
                            }
                        });

                    egui::CollapsingHeader::new(RichText::new("Goal").heading())
                        .default_open(true)
                        .show(ui, |ui| {
                            if display_goal(ui, goal) {
                                bridge.send(SimWorkerInput::Stop);
                                *data = Data::Invalidated;
                                *is_running = false;
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
                                    if *data == Data::Invalidated {
                                        *data = Data::Waiting;
                                    }
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
                    ui.label(match data {
                        Data::Present(data) => data_percentiles_to_string(&data),
                        Data::Waiting => "Not run yet".to_string(),
                        Data::Invalidated => "Canceled".to_string(),
                    });
                })
        });
    }
}
