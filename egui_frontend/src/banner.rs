use egui::{TextStyle, Ui};
use summon_simulator::{banner::GenericBanner, types::Color};

use crate::app::with_colored_dot;

#[derive(Clone, PartialEq, Eq)]
pub struct UiUnit {
    pub name: String,
    pub color: Color,
    pub fourstar_focus: bool,
}
#[derive(Clone, PartialEq, Eq)]
pub struct UiBanner {
    pub name: String,
    pub starting_rates: (u8, u8),
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
            starting_rates: self.starting_rates,
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
    available: Vec<UiBanner>,
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
            name: "Focus: Weekly Revival 50".into(),
            starting_rates: (4, 2),
            units: vec![
                UiUnit {
                    name: "Edelgard".into(),
                    color: Color::Green,
                    fourstar_focus: false,
                },
                UiUnit {
                    name: "Hubert".into(),
                    color: Color::Red,
                    fourstar_focus: false,
                },
                UiUnit {
                    name: "Petra".into(),
                    color: Color::Blue,
                    fourstar_focus: false,
                },
            ],
            has_focus_charges: true,
            has_spark: false,
        },
        UiBanner {
            name: "Hero Fest".into(),
            starting_rates: (5, 3),
            units: vec![
                UiUnit {
                    name: "Eitri".into(),
                    color: Color::Green,
                    fourstar_focus: false,
                },
                UiUnit {
                    name: "Ascended Idunn".into(),
                    color: Color::Blue,
                    fourstar_focus: false,
                },
                UiUnit {
                    name: "Winter Lysithea".into(),
                    color: Color::Colorless,
                    fourstar_focus: false,
                },
                UiUnit {
                    name: "Ascended Mareeta".into(),
                    color: Color::Red,
                    fourstar_focus: false,
                },
            ],
            has_focus_charges: false,
            has_spark: false,
        },
        UiBanner {
            name: "Focus: Double Mythic Heroes".into(),
            starting_rates: (8, 0),
            units: vec![
                UiUnit {
                    name: "Fomortiis".into(),
                    color: Color::Colorless,
                    fourstar_focus: false,
                },
                UiUnit {
                    name: "Gotoh".into(),
                    color: Color::Colorless,
                    fourstar_focus: false,
                },
                UiUnit {
                    name: "Arval".into(),
                    color: Color::Colorless,
                    fourstar_focus: false,
                },
                UiUnit {
                    name: "Legendary Veronica".into(),
                    color: Color::Red,
                    fourstar_focus: false,
                },
                UiUnit {
                    name: "Reginn".into(),
                    color: Color::Red,
                    fourstar_focus: false,
                },
                UiUnit {
                    name: "Rearmed Líf".into(),
                    color: Color::Red,
                    fourstar_focus: false,
                },
                UiUnit {
                    name: "Ullr".into(),
                    color: Color::Blue,
                    fourstar_focus: false,
                },
                UiUnit {
                    name: "Legendary Dimitri".into(),
                    color: Color::Blue,
                    fourstar_focus: false,
                },
                UiUnit {
                    name: "Monica".into(),
                    color: Color::Blue,
                    fourstar_focus: false,
                },
                UiUnit {
                    name: "Legendary Edelgard".into(),
                    color: Color::Green,
                    fourstar_focus: false,
                },
                UiUnit {
                    name: "Freyja".into(),
                    color: Color::Green,
                    fourstar_focus: false,
                },
                UiUnit {
                    name: "Ascended Hilda".into(),
                    color: Color::Green,
                    fourstar_focus: false,
                },
            ],
            has_focus_charges: false,
            has_spark: false,
        },
        UiBanner {
            name: "Legendary & Mythic Hero Remix".into(),
            starting_rates: (6, 0),
            units: vec![
                UiUnit {
                    name: "Legendary Leif".into(),
                    color: Color::Colorless,
                    fourstar_focus: false,
                },
                UiUnit {
                    name: "Altina".into(),
                    color: Color::Red,
                    fourstar_focus: false,
                },
                UiUnit {
                    name: "Ascended Florina".into(),
                    color: Color::Colorless,
                    fourstar_focus: false,
                },
                UiUnit {
                    name: "Fjorm".into(),
                    color: Color::Blue,
                    fourstar_focus: false,
                },
                UiUnit {
                    name: "Legendary Tiki".into(),
                    color: Color::Blue,
                    fourstar_focus: false,
                },
                UiUnit {
                    name: "Legendary Lyn".into(),
                    color: Color::Green,
                    fourstar_focus: false,
                },
                UiUnit {
                    name: "Gunnthrá".into(),
                    color: Color::Green,
                    fourstar_focus: false,
                },
                UiUnit {
                    name: "Legendary Marth".into(),
                    color: Color::Red,
                    fourstar_focus: false,
                },
            ],
            has_focus_charges: false,
            has_spark: false,
        },
        UiBanner {
            name: "Focus: Double Special Heroes".into(),
            starting_rates: (6, 0),
            units: vec![
                UiUnit {
                    name: "Flame Lyn".into(),
                    color: Color::Blue,
                    fourstar_focus: false,
                },
                UiUnit {
                    name: "Flame Tana".into(),
                    color: Color::Green,
                    fourstar_focus: false,
                },
                UiUnit {
                    name: "Bridal Cecilia".into(),
                    color: Color::Red,
                    fourstar_focus: true,
                },
                UiUnit {
                    name: "Summer Lyon".into(),
                    color: Color::Red,
                    fourstar_focus: true,
                },
                UiUnit {
                    name: "Thief Nina".into(),
                    color: Color::Colorless,
                    fourstar_focus: false,
                },
                UiUnit {
                    name: "Thief Cath".into(),
                    color: Color::Blue,
                    fourstar_focus: false,
                },
                UiUnit {
                    name: "Summer Dimitri".into(),
                    color: Color::Blue,
                    fourstar_focus: false,
                },
                UiUnit {
                    name: "Summer Nifl".into(),
                    color: Color::Colorless,
                    fourstar_focus: false,
                },
            ],
            has_focus_charges: true,
            has_spark: true,
        },
        UiBanner {
            name: "An Unusually Long Name For A Banner So I Can Check That It Fits".into(),
            starting_rates: (3, 3),
            has_focus_charges: false,
            has_spark: true,
            units: vec![
                UiUnit {
                    name: "A Unit With An Unusually Long Name".into(),
                    color: Color::Red,
                    fourstar_focus: false,
                },
                UiUnit {
                    name: "Another Unit With An Unusually Long Name".into(),
                    color: Color::Blue,
                    fourstar_focus: false,
                },
                UiUnit {
                    name: "Short".into(),
                    color: Color::Green,
                    fourstar_focus: false,
                },
            ],
        },
    ]
}

pub(crate) fn display_banner(ui: &mut Ui, state: &mut BannerState) -> bool {
    let mut banner_changed = false;

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
        banner_changed = true;
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
            fn rates_to_text(rates: (u8, u8)) -> &'static str {
                match rates {
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
                                rate,
                                rates_to_text(rate),
                            );
                        }
                    });
                if state.current.starting_rates != starting_rates_before {
                    banner_changed = true;
                }
            });
            if ui
                .checkbox(&mut state.current.has_focus_charges, "Focus charges?")
                .changed()
            {
                banner_changed = true;
            }
            if ui
                .checkbox(&mut state.current.has_spark, "Spark?")
                .changed()
            {
                banner_changed = true;
            }
            ui.add_enabled_ui(is_custom_banner, |ui| {
                if display_unit_list(ui, &mut state.current.units) {
                    banner_changed = true;
                }
                if ui.button("+ Add another unit").clicked() {
                    banner_changed = true;
                    state.current.units.push(UiUnit {
                        name: "New Unit".into(),
                        color: Color::Red,
                        fourstar_focus: false,
                    });
                }
            });
        });
    banner_changed
}

fn display_unit_list(ui: &mut Ui, units: &mut Vec<UiUnit>) -> bool {
    let mut units_changed = false;

    let mut to_delete = Vec::new();
    for (i, unit) in units.iter_mut().enumerate() {
        ui.group(|ui| {
            ui.horizontal(|ui| {
                ui.with_layout(egui::Layout::left_to_right(egui::Align::Min), |ui| {
                    if ui.text_edit_singleline(&mut unit.name).changed() {
                        units_changed = true;
                    }
                });
                if ui.button("X").clicked() {
                    units_changed = true;
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
                                TextStyle::Small.resolve(&ui.ctx().style()),
                            ),
                        );
                        ui.selectable_value(
                            &mut unit.color,
                            Color::Blue,
                            with_colored_dot(
                                "Blue",
                                Color::Blue,
                                TextStyle::Small.resolve(&ui.ctx().style()),
                            ),
                        );
                        ui.selectable_value(
                            &mut unit.color,
                            Color::Green,
                            with_colored_dot(
                                "Green",
                                Color::Green,
                                TextStyle::Small.resolve(&ui.ctx().style()),
                            ),
                        );
                        ui.selectable_value(
                            &mut unit.color,
                            Color::Colorless,
                            with_colored_dot(
                                "Colorless",
                                Color::Colorless,
                                TextStyle::Small.resolve(&ui.ctx().style()),
                            ),
                        );
                    });
                if unit.color != unit_color_before {
                    units_changed = true;
                }
                if ui.checkbox(&mut unit.fourstar_focus, "4* focus?").changed() {
                    units_changed = true;
                }
            });
        });
    }
    for &i in to_delete.iter().rev() {
        units.remove(i);
    }

    units_changed
}
