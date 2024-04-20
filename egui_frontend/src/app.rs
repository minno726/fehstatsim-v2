use eframe::web_sys;
use egui::{text::LayoutJob, Color32, FontId, RichText, TextFormat};
use gloo_console::log;
use gloo_net::http::Request;
use gloo_worker::{Worker, WorkerBridge};
use instant::Instant;
use std::{cell::Cell, rc::Rc, time::Duration};
use summon_simulator::types::Color;
use wasm_bindgen_futures::spawn_local;

use crate::{
    banner::{display_banner, BannerState, InvalidationResult, UiBanner},
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

struct Status {
    is_running: bool,
    time_started: Option<Instant>,
    last_data_received: Option<Instant>,
}

impl Status {
    fn sim_started(&mut self) {
        self.is_running = true;
        self.time_started = Some(Instant::now());
        self.last_data_received = None;
    }

    fn sim_ended(&mut self) {
        self.is_running = false;
    }
}

pub struct App {
    // data
    banner: BannerState,
    goal: GoalState,
    results: ResultsState,
    current_banner_list: Rc<Cell<Option<Vec<UiBanner>>>>,

    // status
    status: Status,

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
        let current_banner_list = Rc::new(Cell::new(None));

        // Fetch the current list from what could generously be referred to as an "API"
        {
            let ctx = cc.egui_ctx.clone();
            let current_banner_list = current_banner_list.clone();
            spawn_local(async move {
                let response = Request::get("https://s3.us-east-1.amazonaws.com/public-files.fullyconcentrated.net/current_banners.json").cache(web_sys::RequestCache::NoCache).send().await;
                if let Ok(response) = response {
                    let contents = response.json::<Vec<UiBanner>>().await;
                    if let Ok(contents) = contents {
                        current_banner_list.set(Some(contents));
                        ctx.request_repaint();
                    } else {
                        log!(
                            "Error parsing current banner list: {}",
                            contents.unwrap_err().to_string()
                        );
                    }
                } else {
                    log!(
                        "Error fetching current banner list: {}",
                        response.unwrap_err().to_string()
                    );
                }
            });
        }

        App {
            data_update,
            bridge,
            status: Status {
                is_running: false,
                time_started: None,
                last_data_received: None,
            },
            banner,
            goal,
            results,
            current_banner_list,
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
            status,
            banner,
            goal,
            results,
            current_banner_list,
            ..
        } = self;

        if let Some(worker_response) = data_update.replace(None) {
            if status.is_running && results.data != Data::Invalidated {
                results.data = Data::Present(worker_response);
                status.last_data_received = Some(Instant::now());
            }
        }

        if let Some(current_banner_list) = current_banner_list.replace(None) {
            banner.available.extend(current_banner_list);
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            egui::ScrollArea::both()
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    egui::CollapsingHeader::new("WORK IN PROGRESS")
                        .show(ui, |ui| {
                            ui.horizontal_wrapped(|ui| {
                                ui.spacing_mut().item_spacing.x = 0.0;
                                ui.label("This version of the simulator is incomplete. It has some useful additions, but is also missing some important features that the old one has. You can access the old summon simulator at ");
                                ui.hyperlink("https://fehstatsim-v1.fullyconcentrated.net/");
                                ui.label("\nIf you have any comments or suggestions, contact me on reddit at ");
                                ui.hyperlink_to("/u/minno", "https://www.reddit.com/message/compose?to=minno&subject=new%20fehstatsim%20suggestions");
                            });
                        });

                    egui::CollapsingHeader::new(RichText::new("Banner").heading())
                        .default_open(true)
                        .show(ui, |ui| {
                            match display_banner(ui, banner){
                                InvalidationResult::NoChange => {},
                                InvalidationResult::Nothing => {goal.set_banner(banner.current.clone())},
                                InvalidationResult::ResultsOnly => {bridge.send(SimWorkerInput::Stop);
                                    results.data = Data::Invalidated;
                                    goal.set_banner(banner.current.clone())},
                                InvalidationResult::Everything => {
                                    bridge.send(SimWorkerInput::Stop);
                                    results.data = Data::Invalidated;
                                    *goal = GoalState::new(banner.current.clone(), goal.is_single);
                                },
                            }
                        });

                    egui::CollapsingHeader::new(RichText::new("Goal").heading())
                        .default_open(true)
                        .show(ui, |ui| {
                            if display_goal(ui, goal) {
                                bridge.send(SimWorkerInput::Stop);
                                results.data = Data::Invalidated;
                                status.sim_ended();
                            }
                        });

                    ui.horizontal(|ui| {
                        if status.is_running {
                            ui.horizontal(|ui| {
                                if ui.button("Stop").clicked() {
                                    log!("Stop clicked");
                                    bridge.send(SimWorkerInput::Stop);
                                    status.sim_ended();
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
                                        status.sim_started();
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
                            let elapsed = status
                                .last_data_received?
                                .checked_duration_since(status.time_started?)?;
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
