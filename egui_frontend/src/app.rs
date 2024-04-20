use egui::{text::LayoutJob, Color32, FontId, RichText, TextFormat};
use gloo_console::log;
use gloo_worker::{Worker, WorkerBridge};
use instant::Instant;
use std::{cell::Cell, rc::Rc, time::Duration};
use summon_simulator::types::Color;

use crate::{
    banner::{display_banner, BannerState},
    goal::{display_goal, GoalState},
    results::{display_results, Data, ResultsState},
    SimWorker, SimWorkerInput,
};

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

pub struct App {
    // data
    banner: BannerState,
    goal: GoalState,
    results: ResultsState,

    // status
    is_running: bool,
    time_started: Option<Instant>,
    last_data_received: Option<Instant>,

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
        let results = ResultsState::new();
        App {
            data_update,
            bridge,
            is_running: false,
            time_started: None,
            last_data_received: None,
            banner,
            goal,
            results,
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
            data_update,
            bridge,
            is_running,
            time_started,
            last_data_received,
            banner,
            goal,
            results,
        } = self;

        if let Some(worker_response) = data_update.replace(None) {
            if *is_running && results.data != Data::Invalidated {
                results.data = Data::Present(worker_response);
                *last_data_received = Some(Instant::now());
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
                                results.data = Data::Invalidated;
                                *goal = GoalState::new(banner.current.clone(), goal.is_single);
                            }
                        });

                    egui::CollapsingHeader::new(RichText::new("Goal").heading())
                        .default_open(true)
                        .show(ui, |ui| {
                            if display_goal(ui, goal) {
                                bridge.send(SimWorkerInput::Stop);
                                results.data = Data::Invalidated;
                                *is_running = false;
                            }
                        });

                    ui.horizontal(|ui| {
                        if *is_running {
                            ui.horizontal(|ui| {
                                if ui.button("Stop").clicked() {
                                    log!("Stop clicked");
                                    bridge.send(SimWorkerInput::Stop);
                                    *is_running = false;
                                }
                            });
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
                                        *time_started = Some(Instant::now());
                                        *last_data_received = None;
                                        if results.data == Data::Invalidated {
                                            results.data = Data::Waiting;
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
                        if let Some((elapsed, num_samples)) = (|| {
                            let elapsed = last_data_received
                                .unwrap_or_else(|| Instant::now())
                                .checked_duration_since((*time_started)?)?;
                            let num_samples = results.data.data()?.iter().sum::<u32>();
                            Some((elapsed, num_samples))
                        })() {
                            let mut rate = num_samples as f32 / elapsed.as_secs_f32();
                            if rate > 10000.0 {
                                rate /= 1000.0;
                                ui.small(format!("{num_samples} samples ({rate:.0}K/s)"));
                            } else {
                                ui.small(format!("{num_samples} samples ({rate:.0}/s)"));
                            }
                        }
                    });
                    display_results(ui, &banner.current, goal, results);
                })
        });
    }
}
