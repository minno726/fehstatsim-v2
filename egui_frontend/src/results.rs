use std::fmt::Write;

use egui::{Ui, Widget};
use summon_simulator::frequency_counter::FrequencyCounter;

use crate::{banner::UiBanner, goal::GoalState};

#[derive(Debug, PartialEq)]
pub enum Data {
    Present(FrequencyCounter),
    Waiting,
    Invalidated,
}

#[derive(Debug, PartialEq)]
pub enum DisplayType {
    Text,
    Chart,
}

pub struct ResultsState {
    pub data: Data,
    pub typ: DisplayType,
    pub percentile_slider: u32,
}

impl ResultsState {
    pub fn new() -> Self {
        Self {
            data: Data::Invalidated,
            typ: DisplayType::Text,
            percentile_slider: 500,
        }
    }
}

fn percentiles(data: &FrequencyCounter, values: &[f32], invert: bool) -> Vec<u32> {
    let total = data.iter().sum::<u32>();
    let mut cum_total = 0;
    let mut cur_value_idx = 0;
    let mut result = Vec::new();
    let iter: &mut dyn Iterator<Item = (usize, &u32)>;
    let mut backward_iter = data.iter().enumerate().rev();
    let mut forward_iter = data.iter().enumerate();
    if invert {
        iter = &mut backward_iter;
    } else {
        iter = &mut forward_iter;
    }
    for (i, &data_point) in iter {
        cum_total += data_point;
        if cum_total as f32 > total as f32 * values[cur_value_idx] {
            result.push(i as u32);
            cur_value_idx += 1;
            if cur_value_idx >= values.len() {
                return result;
            }
        }
    }
    let filler = if invert { 0 } else { (data.len() - 1) as u32 };
    while result.len() < values.len() {
        result.push(filler);
    }
    result
}

fn data_percentiles_to_string(data: &FrequencyCounter, data_label: &str, invert: bool) -> String {
    let sample_percentiles = [0.25f32, 0.5, 0.75, 0.9, 0.99];
    let data = percentiles(data, &sample_percentiles, invert);
    let mut output = String::new();
    for i in 0..sample_percentiles.len() {
        writeln!(
            &mut output,
            "{}%: {} {}",
            (sample_percentiles[i] * 100.0).round() as u32,
            data[i],
            data_label
        )
        .unwrap();
    }
    output
}

pub fn display_results(
    ui: &mut Ui,
    banner: &UiBanner,
    goal: &GoalState,
    results: &mut ResultsState,
) {
    ui.heading("Results");
    ui.horizontal(|ui| {
        ui.selectable_value(&mut results.typ, DisplayType::Text, "Text");
        ui.selectable_value(&mut results.typ, DisplayType::Chart, "Chart");
    });
    match results.typ {
        DisplayType::Text => display_text_results(ui, banner, goal, results),
        DisplayType::Chart => display_chart_results(ui, banner, goal, results),
    }
}

fn display_text_results(
    ui: &mut Ui,
    _banner: &UiBanner,
    goal: &GoalState,
    results: &mut ResultsState,
) {
    let is_orb_goal = goal.is_single && !goal.single.is_quantity_goal;
    let label = if is_orb_goal {
        "copies or more"
    } else {
        "orbs or less"
    };
    match &results.data {
        Data::Present(data) => {
            ui.label(data_percentiles_to_string(data, label, is_orb_goal));
            ui.horizontal(|ui| {
                if ui.button("-0.1%").clicked() {
                    results.percentile_slider -= 1;
                }
                egui::Slider::new(&mut results.percentile_slider, 0..=1000)
                    .clamp_to_range(true)
                    .step_by(10.0)
                    .smart_aim(false)
                    .show_value(false)
                    .ui(ui);
                if ui.button("+0.1%").clicked() {
                    results.percentile_slider += 1;
                }
                results.percentile_slider = results.percentile_slider.clamp(1, 999);
            });
            let custom_percentile = percentiles(
                data,
                &[results.percentile_slider as f32 / 1000.0],
                is_orb_goal,
            )[0];
            ui.label(format!(
                "{}%: {} {}",
                results.percentile_slider as f32 / 10.0,
                custom_percentile,
                label
            ));
        }
        Data::Waiting => {
            ui.spinner();
        }
        Data::Invalidated => {}
    };
}

fn display_chart_results(
    ui: &mut Ui,
    _banner: &UiBanner,
    _goal: &GoalState,
    results: &mut ResultsState,
) {
    match &results.data {
        Data::Present(_) => {
            ui.label("Pretend that there's one of those \"Under Construction\" animated gifs from the early internet here.");
        }
        Data::Waiting => {
            ui.spinner();
        }
        Data::Invalidated => {}
    }
}
