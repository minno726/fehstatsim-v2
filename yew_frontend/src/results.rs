use summon_simulator::frequency_counter::FrequencyCounter;
use yew::prelude::*;

#[derive(PartialEq, Properties)]
pub struct ResultsProps {
    pub data: Option<FrequencyCounter>,
}

#[function_component]
pub fn Results(props: &ResultsProps) -> Html {
    let ResultsProps { data } = props;
    html! {
        <p>{ if let Some(data) = data {
            data_percentiles_to_string(data)
        } else {
            "Haven't started yet.".to_string()
        }}</p>
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
