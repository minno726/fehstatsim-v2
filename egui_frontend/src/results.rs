use std::fmt::Write;

use egui::Ui;
use summon_simulator::frequency_counter::FrequencyCounter;

use crate::{banner::UiBanner, goal::GoalState};

#[derive(Debug, PartialEq)]
pub enum Data {
    Present(FrequencyCounter),
    Waiting,
    Invalidated,
}

pub struct ResultsState {
    pub data: Data,
}

impl ResultsState {
    pub fn new() -> Self {
        Self {
            data: Data::Waiting,
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

fn data_percentiles_to_string(data: &FrequencyCounter, data_label: &str) -> String {
    let sample_percentiles = [0.25f32, 0.5, 0.75, 0.9, 0.99];
    let data = percentiles(data, &sample_percentiles);
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

pub fn display_results(ui: &mut Ui, banner: &UiBanner, goal: &GoalState, results: &ResultsState) {
    let label = if goal.is_single && !goal.single.is_quantity_goal {
        "copies or fewer"
    } else {
        "orbs or less"
    };
    ui.label(match &results.data {
        Data::Present(data) => data_percentiles_to_string(data, label),
        Data::Waiting => "Not run yet".to_string(),
        Data::Invalidated => "Canceled".to_string(),
    });
}
