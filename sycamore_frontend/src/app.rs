use crate::{
    banner::{BannerSelector, UiBanner},
    goal::{GoalSelector, GoalState, MultiGoal, SingleGoal},
    worker::{SimWorker, SimWorkerInput},
};

use gloo_worker::Spawnable;
use instant::Duration;
use std::fmt::Write;
use summon_simulator::frequency_counter::FrequencyCounter;
use sycamore::prelude::*;

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

#[component]
pub fn App<G: Html>(cx: Scope) -> View<G> {
    let results = create_rc_signal(Vec::<u32>::new());
    let is_running = create_signal(cx, false);
    let sample_percentiles = [0.25f32, 0.5, 0.75, 0.9, 0.99, 1.0];

    let banner = create_signal(
        cx,
        UiBanner {
            name: "Test".into(),
            starting_rates: create_signal(cx, (3, 3)),
            units: create_signal(cx, vec![]),
        },
    );
    let goal = create_signal(
        cx,
        GoalState {
            is_single: create_signal(cx, false),
            single: create_signal(
                cx,
                SingleGoal {
                    is_quantity_goal: create_signal(cx, true),
                    unit_count_goal: create_signal(cx, 0),
                    orb_limit: create_signal(cx, 0),
                    unit: create_signal(cx, "".into()),
                },
            ),
            multi: create_signal(
                cx,
                MultiGoal {
                    unit_count_goals: create_signal(cx, Vec::new()),
                    require_all: create_signal(cx, true),
                },
            ),
        },
    );

    let bridge = {
        let results = results.clone();
        SimWorker::spawner()
            .callback(move |result| results.set(percentiles(&result, &sample_percentiles)))
            .spawn("./worker.js")
    };

    let banner_str = {
        let banner = banner.clone();
        move || format!("{:?}", banner.get().to_generic_banner())
    };

    let onclick = {
        let banner = banner.clone();
        let goal = goal.clone();
        move |_| {
            if *is_running.get() {
                bridge.send(SimWorkerInput::Stop);
                is_running.set(false);
            } else {
                if let Some(banner) = banner.get().to_generic_banner() {
                    if let Some(goal) = goal.get().to_goal() {
                        bridge.send(SimWorkerInput::Run {
                            banner,
                            goal,
                            target_interval: Duration::from_millis(500),
                        });
                        is_running.set(true);
                    }
                }
            }
        }
    };

    let fmt_output = {
        let results = results.clone();
        move || {
            let results = results.get();
            if results.is_empty() {
                "No data".to_string()
            } else {
                let mut output = String::new();
                for i in 0..sample_percentiles.len() {
                    write!(
                        &mut output,
                        "{}%: {}, ",
                        (sample_percentiles[i] * 100.0).round() as u32,
                        results[i]
                    )
                    .unwrap();
                }
                output
            }
        }
    };

    {
        view! { cx,
            BannerSelector(banner=banner)
            p { (banner_str()) }
            GoalSelector(banner=banner, goal=goal)
            br {}
            button(on:click=onclick) {
                (if !*is_running.get() {
                    "Run in worker!"
                } else {
                    "Stop"
                })
            }
            br {}
            label { (fmt_output()) }
        }
    }
}
