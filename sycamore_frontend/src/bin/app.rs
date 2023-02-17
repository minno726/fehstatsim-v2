use enumset::EnumSet;
use gloo_worker::Spawnable;
use std::fmt::Write;
use summon_simulator::{
    banner::BannerType,
    frequency_counter::FrequencyCounter,
    goal::{UnitCountGoal, UnitGoal},
    types::{Color, Pool},
};
use sycamore::prelude::*;
use sycamore_frontend::SimWorker;

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
fn App<G: Html>(cx: Scope) -> View<G> {
    let banner = BannerType::Standard {
        focus: [1, 1, 1, 1],
    }
    .as_generic_banner(false);
    let goal = UnitCountGoal::new(vec![UnitGoal {
        color: Color::Red,
        copies: 1,
        pools: EnumSet::from(Pool::Focus),
    }]);
    let state = create_rc_signal(Vec::<u32>::new());
    let sample_percentiles = [0.25f32, 0.5, 0.75, 0.9, 0.99, 1.0];

    let bridge = {
        let state = state.clone();
        SimWorker::spawner()
            .callback(move |(data,)| state.set(percentiles(&data, &sample_percentiles)))
            .spawn("./worker.js")
    };

    let onclick = {
        let goal = goal.clone();
        let banner = banner.clone();
        move |_| {
            bridge.send((banner.clone(), goal.clone(), 20000));
        }
    };

    let fmt_output = {
        let state = state.clone();
        move || {
            let data = state.get();
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
    };
    view! { cx,
        button(on:click=onclick) { "Run in worker!" }
        br {}
        label { (if state.get().is_empty() {
            "Haven't run yet".to_string()
        } else {
            fmt_output()
        }) }
    }
}

fn main() {
    console_error_panic_hook::set_once();

    sycamore::render(|cx| {
        view! { cx, App {}}
    });
}
