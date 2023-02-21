use rand::prelude::Distribution;

use crate::types::{Color, Pool};

pub struct WeightedIndexColor {
    thresholds: [u32; 3],
}

impl WeightedIndexColor {
    pub fn new(values: impl IntoIterator<Item = impl Into<f32>>) -> Self {
        let values = values.into_iter().map(|i| i.into()).collect::<Vec<_>>();
        let sum = values.iter().sum::<f32>();
        debug_assert!(values.len() == 4);
        Self {
            thresholds: [
                ((values[0] / sum) * u32::MAX as f32) as u32,
                (((values[0] + values[1]) / sum) * u32::MAX as f32) as u32,
                (((values[0] + values[1] + values[2]) / sum) * u32::MAX as f32) as u32,
            ],
        }
    }
}

impl Distribution<Color> for WeightedIndexColor {
    fn sample<R: rand::Rng + ?Sized>(&self, rng: &mut R) -> Color {
        let choice = rng.gen::<u32>();
        if choice < self.thresholds[1] {
            if choice < self.thresholds[0] {
                Color::Red
            } else {
                Color::Blue
            }
        } else if choice < self.thresholds[2] {
            Color::Green
        } else {
            Color::Colorless
        }
    }
}

pub struct WeightedIndexPool {
    thresholds: [u32; 4],
}

impl WeightedIndexPool {
    pub fn new(values: impl IntoIterator<Item = impl Into<f32>>) -> Self {
        let values = values.into_iter().map(|i| i.into()).collect::<Vec<_>>();
        let sum = values.iter().sum::<f32>();
        debug_assert!(values.len() == 5);
        Self {
            thresholds: [
                ((values[0] / sum) * u32::MAX as f32) as u32,
                (((values[0] + values[1]) / sum) * u32::MAX as f32) as u32,
                (((values[0] + values[1] + values[2]) / sum) * u32::MAX as f32) as u32,
                (((values[0] + values[1] + values[2] + values[3]) / sum) * u32::MAX as f32) as u32,
            ],
        }
    }
}

impl Distribution<Pool> for WeightedIndexPool {
    fn sample<R: rand::Rng + ?Sized>(&self, rng: &mut R) -> Pool {
        let choice = rng.gen::<u32>();
        if choice > self.thresholds[3] {
            Pool::Common
        } else if choice < self.thresholds[1] {
            if choice < self.thresholds[0] {
                Pool::Focus
            } else {
                Pool::Fivestar
            }
        } else if choice < self.thresholds[2] {
            Pool::FourstarFocus
        } else {
            Pool::FourstarSpecial
        }
    }
}
