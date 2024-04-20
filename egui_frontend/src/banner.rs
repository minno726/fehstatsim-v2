use egui::{TextStyle, Ui};
use summon_simulator::{banner::GenericBanner, types::Color};

use crate::app::with_colored_dot;

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
pub struct UiUnit {
    pub name: String,
    pub color: Color,
    pub fourstar_focus: bool,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, serde::Deserialize)]

pub struct FivestarRates {
    focus: u8,
    fivestar: u8,
}

impl From<(u8, u8)> for FivestarRates {
    fn from((focus, fivestar): (u8, u8)) -> Self {
        FivestarRates { focus, fivestar }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
pub struct UiBanner {
    pub name: String,
    pub starting_rates: FivestarRates,
    pub has_focus_charges: bool,
    pub has_spark: bool,
    pub units: Vec<UiUnit>,
}

impl UiBanner {
    pub fn to_sim_banner(&self) -> Option<GenericBanner> {
        let mut focus_sizes = [0; 4];
        let mut fourstar_focus_sizes = [0; 4];
        for unit in &self.units {
            focus_sizes[unit.color as usize] += 1;
            if unit.fourstar_focus {
                fourstar_focus_sizes[unit.color as usize] += 1;
            }
        }
        let banner = GenericBanner {
            starting_rates: (self.starting_rates.focus, self.starting_rates.fivestar),
            focus_sizes,
            fourstar_focus_sizes,
            has_spark: self.has_spark,
            has_charges: self.has_focus_charges,
        };
        if banner.is_valid() {
            Some(banner)
        } else {
            None
        }
    }
}

#[derive(Clone)]
pub struct BannerState {
    pub current: UiBanner,
    pub available: Vec<UiBanner>,
}

impl BannerState {
    pub fn new() -> BannerState {
        let defaults = default_banners();
        BannerState {
            current: defaults[0].clone(),
            available: defaults,
        }
    }
}

pub fn default_banners() -> Vec<UiBanner> {
    vec![
        UiBanner {
            name: "Generic Legendary Banner".into(),
            starting_rates: (8, 0).into(),
            units: vec![
                UiUnit {
                    name: "Red 1".into(),
                    color: Color::Red,
                    fourstar_focus: false,
                },
                UiUnit {
                    name: "Red 2".into(),
                    color: Color::Red,
                    fourstar_focus: false,
                },
                UiUnit {
                    name: "Red 3".into(),
                    color: Color::Red,
                    fourstar_focus: false,
                },
                UiUnit {
                    name: "Blue 1".into(),
                    color: Color::Blue,
                    fourstar_focus: false,
                },
                UiUnit {
                    name: "Blue 2".into(),
                    color: Color::Blue,
                    fourstar_focus: false,
                },
                UiUnit {
                    name: "Blue 3".into(),
                    color: Color::Blue,
                    fourstar_focus: false,
                },
                UiUnit {
                    name: "Green 1".into(),
                    color: Color::Green,
                    fourstar_focus: false,
                },
                UiUnit {
                    name: "Green 2".into(),
                    color: Color::Green,
                    fourstar_focus: false,
                },
                UiUnit {
                    name: "Green 3".into(),
                    color: Color::Green,
                    fourstar_focus: false,
                },
                UiUnit {
                    name: "Colorless 1".into(),
                    color: Color::Colorless,
                    fourstar_focus: false,
                },
                UiUnit {
                    name: "Colorless 2".into(),
                    color: Color::Colorless,
                    fourstar_focus: false,
                },
                UiUnit {
                    name: "Colorless 3".into(),
                    color: Color::Colorless,
                    fourstar_focus: false,
                },
            ],
            has_focus_charges: false,
            has_spark: false,
        },
        UiBanner {
            name: "Generic Hero Fest".into(),
            starting_rates: (5, 3).into(),
            units: vec![
                UiUnit {
                    name: "Red".into(),
                    color: Color::Red,
                    fourstar_focus: false,
                },
                UiUnit {
                    name: "Blue".into(),
                    color: Color::Blue,
                    fourstar_focus: false,
                },
                UiUnit {
                    name: "Green".into(),
                    color: Color::Green,
                    fourstar_focus: false,
                },
                UiUnit {
                    name: "Colorless".into(),
                    color: Color::Colorless,
                    fourstar_focus: false,
                },
            ],
            has_focus_charges: false,
            has_spark: false,
        },
    ]
}

/// Says what to invalidate based on which controls changed
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InvalidationResult {
    /// Nothing changed at all
    NoChange,
    /// Only textual changes with no effect on the simulation
    Nothing,
    /// The current goal is still valid, but the results have changed
    ResultsOnly,
    /// The banner changed in a way that could make the existing goal make no sense.
    Everything,
}

impl InvalidationResult {
    pub fn changed(&mut self) {
        *self = match *self {
            Self::NoChange => Self::Nothing,
            _ => *self,
        }
    }
    pub fn invalidate_results(&mut self) {
        *self = match *self {
            Self::Everything => Self::Everything,
            _ => Self::ResultsOnly,
        };
    }

    pub fn invalidate_all(&mut self) {
        *self = Self::Everything;
    }

    pub fn combine(&mut self, other: Self) {
        use InvalidationResult::*;
        *self = match (*self, other) {
            (Everything, _) | (_, Everything) => Everything,
            (ResultsOnly, _) | (_, ResultsOnly) => ResultsOnly,
            (Nothing, _) | (_, Nothing) => Nothing,
            (NoChange, NoChange) => NoChange,
        }
    }
}

pub(crate) fn display_banner(ui: &mut Ui, state: &mut BannerState) -> InvalidationResult {
    let mut invalidation_result = InvalidationResult::NoChange;

    let mut custom_banner = state.current.clone();
    custom_banner.name = "Custom".into();
    let banner_name_before = state.current.name.clone();

    egui::ComboBox::from_label("Banner")
        .selected_text(state.current.name.clone())
        .show_ui(ui, |ui| {
            ui.selectable_value(
                &mut state.current,
                custom_banner.clone(),
                custom_banner.name,
            );
            for banner in &state.available {
                ui.selectable_value(&mut state.current, banner.clone(), banner.name.clone());
            }
        });
    // Switching to a "custom" banner isn't actually a change, since it starts out equivalent to
    // whatever the currently-selected banner is
    if state.current.name != banner_name_before && state.current.name != "Custom" {
        invalidation_result.invalidate_all();
    }

    let details_open = match (
        banner_name_before == "Custom",
        state.current.name == "Custom",
    ) {
        (true, true) | (false, false) => None,
        // Close when switching away from a custom banner
        (true, false) => Some(false),
        // Open when switching to a custom banner
        (false, true) => Some(true),
    };

    egui::CollapsingHeader::new("Details")
        .open(details_open)
        .show(ui, |ui| {
            if state.current.name != "Custom" {
                if ui.button("Edit").clicked() {
                    state.current = state.current.clone();
                    state.current.name = "Custom".into();
                }
            }

            fn rates_to_text(rates: FivestarRates) -> &'static str {
                match (rates.focus, rates.fivestar) {
                    (3, 3) => "3%/3% (Standard)",
                    (4, 2) => "4%/2% (Weekly Revival)",
                    (8, 0) => "8%/0% (Legendary/Mythic)",
                    (5, 3) => "5%/3% (Hero Fest)",
                    (6, 0) => "6%/0% (Remix/Double Special)",
                    _ => "ERROR: invalid starting rates",
                }
            }

            let is_custom_banner = state.current.name == "Custom";
            ui.add_enabled_ui(is_custom_banner, |ui| {
                let starting_rates_before = state.current.starting_rates;
                egui::ComboBox::from_label("Rate")
                    .selected_text(rates_to_text(state.current.starting_rates))
                    .show_ui(ui, |ui| {
                        for rate in [(3, 3), (4, 2), (8, 0), (5, 3), (6, 0)] {
                            ui.selectable_value(
                                &mut state.current.starting_rates,
                                rate.into(),
                                rates_to_text(rate.into()),
                            );
                        }
                    });
                if state.current.starting_rates != starting_rates_before {
                    invalidation_result.invalidate_results();
                }
            });
            if ui
                .checkbox(&mut state.current.has_focus_charges, "Focus charges?")
                .changed()
            {
                invalidation_result.invalidate_results();
            }
            if ui
                .checkbox(&mut state.current.has_spark, "Spark?")
                .changed()
            {
                invalidation_result.invalidate_results();
            }
            ui.add_enabled_ui(is_custom_banner, |ui| {
                invalidation_result.combine(display_unit_list(ui, &mut state.current.units));
                if ui.button("+ Add another unit").clicked() {
                    invalidation_result.invalidate_results();
                    state.current.units.push(UiUnit {
                        name: "New Unit".into(),
                        color: Color::Red,
                        fourstar_focus: false,
                    });
                }
            });
        });
    invalidation_result
}

fn display_unit_list(ui: &mut Ui, units: &mut Vec<UiUnit>) -> InvalidationResult {
    let mut invalidation_result = InvalidationResult::NoChange;

    let mut to_delete = Vec::new();
    for (i, unit) in units.iter_mut().enumerate() {
        ui.group(|ui| {
            ui.horizontal(|ui| {
                ui.with_layout(egui::Layout::left_to_right(egui::Align::Min), |ui| {
                    if ui.text_edit_singleline(&mut unit.name).changed() {
                        invalidation_result.changed();
                    }
                });
                if ui.button("X").clicked() {
                    invalidation_result.invalidate_all();
                    to_delete.push(i);
                }
            });
            ui.horizontal(|ui| {
                let unit_color_before = unit.color;
                egui::ComboBox::from_id_source((i, "display_unit_list"))
                    .selected_text(with_colored_dot(
                        &format!("{:?}", unit.color),
                        unit.color,
                        TextStyle::Button.resolve(&ui.ctx().style()),
                    ))
                    .width(120.0)
                    .show_ui(ui, |ui| {
                        ui.selectable_value(
                            &mut unit.color,
                            Color::Red,
                            with_colored_dot(
                                "Red",
                                Color::Red,
                                TextStyle::Button.resolve(&ui.ctx().style()),
                            ),
                        );
                        ui.selectable_value(
                            &mut unit.color,
                            Color::Blue,
                            with_colored_dot(
                                "Blue",
                                Color::Blue,
                                TextStyle::Button.resolve(&ui.ctx().style()),
                            ),
                        );
                        ui.selectable_value(
                            &mut unit.color,
                            Color::Green,
                            with_colored_dot(
                                "Green",
                                Color::Green,
                                TextStyle::Button.resolve(&ui.ctx().style()),
                            ),
                        );
                        ui.selectable_value(
                            &mut unit.color,
                            Color::Colorless,
                            with_colored_dot(
                                "Colorless",
                                Color::Colorless,
                                TextStyle::Button.resolve(&ui.ctx().style()),
                            ),
                        );
                    });
                if unit.color != unit_color_before {
                    invalidation_result.invalidate_results();
                }
                if ui.checkbox(&mut unit.fourstar_focus, "4* focus?").changed() {
                    invalidation_result.invalidate_results();
                }
            });
        });
    }
    for &i in to_delete.iter().rev() {
        units.remove(i);
    }

    invalidation_result
}
