use std::fmt::Write;

use dioxus::{html::summary, prelude::*};
use dioxus_frontend::worker::{SimWorker, SimWorkerInput};
use enumset::EnumSet;

use gloo_worker::Spawnable;
use instant::Duration;
use log::Level;
use summon_simulator::{
    banner::{GenericBanner, StandardBanner},
    frequency_counter::FrequencyCounter,
    goal::{BudgetGoal, Goal, UnitCountGoal, UnitGoal},
    types::{Color, Pool},
};

mod worker;

fn main() {
    wasm_logger::init(wasm_logger::Config::new(Level::Warn));
    console_error_panic_hook::set_once();

    dioxus_web::launch(app);
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

fn full_data_text(data: &FrequencyCounter) -> impl Iterator<Item = String> {
    let mut output = String::new();
    let num_samples = data.iter().sum::<u32>();
    let mut total = 0;
    let mut output = vec![];
    for (result, &frequency) in data.iter().enumerate() {
        if frequency > 0 {
            total += frequency;
            output.push(format!(
                "{}: {:.2}% ({:.2}% cum.)\n",
                result,
                frequency as f32 / num_samples as f32 * 100.0,
                total as f32 / num_samples as f32 * 100.0
            ));
        }
    }
    output.into_iter()
}

fn app(cx: Scope) -> Element {
    let is_running = use_state(cx, || false);
    let data = use_state(cx, || None::<FrequencyCounter>);
    let bridge = {
        to_owned![data];
        use_context_provider(cx, || {
            SimWorker::spawner()
                .callback(move |result| data.set(Some(result)))
                .spawn("./worker.js")
        })
    };
    let _banner = use_context_provider(cx, || None::<GenericBanner>);
    let _goal = use_context_provider(cx, || None::<Goal>);

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

    let data_display = if let Some(data) = data.get() {
        rsx!(
            p { data_percentiles_to_string(data) }
            ul {
                for s in full_data_text(data) {
                    li { s }
                }
            }
        )
    } else {
        rsx!(p { "Haven't started yet" })
    };

    let run_button = if *is_running.get() {
        rsx!( button { onclick: move |_| { 
            is_running.set(false);
            bridge.send(SimWorkerInput::Stop)
        }, "Stop"} )
    } else {
        rsx!( button { onclick: move |_| {
            is_running.set(true);
            bridge.send(SimWorkerInput::Run {
                banner: standard_banner.clone(),
                goal: standard_goal.clone(),
                target_interval: Duration::from_millis(1000),
            })
        }, "Run"} )
    };

    cx.render(rsx! (
        div {
            style: "text-align: center;",
            banner_selector {}
            run_button
            details {
                summary { "Results" }
                div {
                    style: "max-height: 40em; overflow: scroll;",
                    data_display
                }
            }
        }
    ))
}

fn banner_selector(cx: Scope) -> Element {
    let banner = use_context::<GenericBanner>(cx);
    cx.render(rsx!("todo!"))
}
