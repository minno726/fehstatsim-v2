use std::rc::Rc;

use enumset::EnumSet;
use instant::Duration;
use summon_simulator::{
    banner::StandardBanner,
    frequency_counter::FrequencyCounter,
    goal::{Goal, UnitCountGoal, UnitGoal},
    types::{Color, Pool},
};
use yew::prelude::*;
use yew_agent::{Bridge, Bridged};

use crate::agent::{SimWorker, SimWorkerInput};

pub struct App {
    worker: Box<dyn Bridge<SimWorker>>,
    data: Option<FrequencyCounter>,
    is_running: bool,
}

pub enum AppMsg {
    RunClicked,
    DataReceived(FrequencyCounter),
}

impl Component for App {
    type Message = AppMsg;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        let cb = {
            let link = ctx.link().clone();
            move |e| link.send_message(Self::Message::DataReceived(e))
        };

        Self {
            worker: SimWorker::bridge(Rc::new(cb)),
            data: None,
            is_running: false,
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let data_display = if let Some(data) = self.data.as_ref() {
            html! {
                <p> { data_percentiles_to_string(data) } </p>
            }
        } else {
            html! {
                <p> { "Haven't started yet" } </p>
            }
        };

        html! {
            <>
            <button onclick={ctx.link().callback(|_| AppMsg::RunClicked)}>{
                if self.is_running {
                    "Stop"
                } else {
                    "Run"
                }
            }
            </button>
            {data_display}
            </>
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            AppMsg::DataReceived(data) => {
                if self.is_running {
                    self.data = Some(data);
                    true
                } else {
                    false
                }
            }
            AppMsg::RunClicked => {
                if !self.is_running {
                    let standard_banner = StandardBanner::Standard {
                        focus: [1, 1, 1, 1],
                    }
                    .as_generic_banner(false);
                    let standard_goal = Goal::Quantity(UnitCountGoal::new(
                        vec![UnitGoal {
                            color: Color::Red,
                            copies: 1,
                            pools: EnumSet::from(Pool::Focus),
                        }],
                        false,
                    ));

                    self.worker.send(SimWorkerInput::Run {
                        banner: standard_banner,
                        goal: standard_goal,
                        target_interval: Duration::from_millis(500),
                    });
                    self.is_running = true;

                    true
                } else {
                    self.worker.send(SimWorkerInput::Stop);
                    self.is_running = false;
                    true
                }
            }
        }
    }
}

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
    use std::fmt::Write;
    let sample_percentiles = [0.25f32, 0.5, 0.75, 0.9, 0.99, 1.0];
    let num_samples = data.iter().sum::<u32>();
    let data = percentiles(data, &sample_percentiles);
    let mut output = format!("({} samples) ", num_samples);
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
