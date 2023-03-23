use enumset::EnumSet;
use gloo_worker::Spawnable;
use instant::Duration;
use std::fmt::Write;
use summon_simulator::{
    banner::StandardBanner,
    frequency_counter::FrequencyCounter,
    goal::{Goal, UnitCountGoal, UnitGoal},
    types::{Color, Pool},
};
use sycamore::{prelude::*, rt::Event};
use sycamore_frontend::{SimWorker, SimWorkerInput};

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

struct AppState {
    banner_units: RcSignal<Vec<RcSignal<UnitListItem>>>,
}

#[derive(PartialEq, Eq, Clone)]
struct UnitListItem {
    name: RcSignal<Option<String>>,
    color: RcSignal<Color>,
    fourstar_focus: RcSignal<bool>,
}

#[component(inline_props)]
fn BannerUnitListItem<G: Html>(cx: Scope, unit: RcSignal<UnitListItem>) -> View<G> {
    let extend_name = {
        let unit = unit.clone();
        move |_| {
            let mut name = unit.get().as_ref().name.get().as_ref().clone().unwrap();
            name.push('r');
            unit.get().name.set(Some(name));
        }
    };

    view! { cx,
        p(on:click=extend_name) { ( (*unit.get().name.get()).clone().unwrap_or("Unnamed".to_string()) ) }
    }
}

#[component]
fn BannerUnitList<G: Html>(cx: Scope) -> View<G> {
    let app_state = use_context::<AppState>(cx);

    let add_unit = {
        let units = app_state.banner_units.clone();
        move |_| {
            let mut new_units = (*units.get()).clone();
            new_units.push(create_rc_signal(UnitListItem {
                name: create_rc_signal(Some("Whatever".to_string())),
                color: create_rc_signal(Color::Red),
                fourstar_focus: create_rc_signal(false),
            }));
            app_state.banner_units.set(new_units);
        }
    };

    view! { cx,
        Indexed(
            iterable=&app_state.banner_units,
            view=move |cx, item| view! { cx,
                BannerUnitListItem(unit=item)
            }
        )

        button(on:click=add_unit) { "+" }
    }
}

#[component]
fn BannerSelector<G: Html>(cx: Scope) -> View<G> {
    view! { cx,
        BannerUnitList
    }
}

#[component]
fn App<G: Html>(cx: Scope) -> View<G> {
    let banner = StandardBanner::Standard {
        focus: [1, 1, 1, 1],
    }
    .as_generic_banner(false);
    let goal = Goal::Quantity(UnitCountGoal::new(
        vec![UnitGoal {
            color: Color::Red,
            copies: 1,
            pools: EnumSet::from(Pool::Focus),
        }],
        true,
    ));
    let state = create_rc_signal(Vec::<u32>::new());
    let sample_percentiles = [0.25f32, 0.5, 0.75, 0.9, 0.99, 1.0];

    let bridge = {
        let state = state.clone();
        SimWorker::spawner()
            .callback(move |data| state.set(percentiles(&data, &sample_percentiles)))
            .spawn("./worker.js")
    };

    let app_state = AppState {
        banner_units: create_rc_signal(vec![]),
    };
    let _app_state = provide_context(cx, app_state);

    let onclick = {
        let goal = goal.clone();
        let banner = banner.clone();
        move |_| {
            bridge.send(SimWorkerInput::Run {
                banner: banner.clone(),
                goal: goal.clone(),
                target_interval: Duration::from_millis(1000),
            });
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
        BannerSelector {}
        br {}
        button(on:click=onclick) { "Run in worker!" }
        br {}
        label { (if state.get().is_empty() {
            "No data".to_string()
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
