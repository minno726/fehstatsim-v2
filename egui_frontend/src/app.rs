use egui::Vec2;
use enumset::EnumSet;
use gloo_console::log;
use gloo_worker::{Worker, WorkerBridge};
use std::{fmt::Write, sync::mpsc, time::Duration};
use summon_simulator::{
    banner::BannerType,
    frequency_counter::FrequencyCounter,
    goal::{Goal, UnitCountGoal, UnitGoal},
    types::{Color, Pool},
};

use crate::{SimWorker, SimWorkerInput};

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
    data: Option<FrequencyCounter>,
    channel: mpsc::Receiver<<SimWorker as Worker>::Output>,
    bridge: WorkerBridge<SimWorker>,
    is_running: bool,
}

impl App {
    pub fn new(
        _cc: &eframe::CreationContext<'_>,
        channel: mpsc::Receiver<<SimWorker as Worker>::Output>,
        bridge: WorkerBridge<SimWorker>,
    ) -> Self {
        App {
            data: None,
            channel,
            bridge,
            is_running: false,
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let Self {
            data,
            channel,
            bridge,
            is_running,
        } = self;

        let banner = BannerType::Standard {
            focus: [1, 1, 1, 1],
        }
        .as_generic_banner(false);
        let goal = Goal::Quantity(UnitCountGoal::new(vec![UnitGoal {
            color: Color::Red,
            copies: 1,
            pools: EnumSet::from(Pool::Focus),
        }]));

        if let Ok(worker_response) = channel.try_recv() {
            *data = Some(worker_response);
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            egui::ScrollArea::both()
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    if *is_running {
                        if ui.button("Stop").clicked() {
                            log!("Stop clicked");
                            bridge.send(SimWorkerInput::Stop);
                            *is_running = false;
                        }
                    } else {
                        if ui.button("Run").clicked() {
                            log!("Run clicked");
                            bridge.send(SimWorkerInput::Run {
                                banner,
                                goal,
                                target_interval: Duration::from_millis(1000),
                            });
                            *is_running = true;
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
