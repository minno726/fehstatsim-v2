use egui::Ui;
use summon_simulator::{banner::GenericBanner, types::Color};

#[derive(Clone, PartialEq, Eq)]
pub struct UiUnit {
    name: String,
    color: Color,
    fourstar_focus: bool,
}
#[derive(Clone, PartialEq, Eq)]
pub struct UiBanner {
    name: String,
    starting_rates: (u8, u8),
    has_focus_charges: bool,
    has_spark: bool,
    units: Vec<UiUnit>,
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
    ]
}

pub(crate) fn display_banner(ui: &mut Ui, state: &mut BannerState) {
    let mut custom_banner = state.current.clone();
    custom_banner.name = "Custom".into();
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

    ui.add_enabled_ui(state.current.name == "Custom", |ui| {
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
        display_unit_list(ui, &mut state.current.units);
        if ui.button("+").clicked() {
            state.current.units.push(UiUnit {
                name: "New Unit".into(),
                color: Color::Red,
                fourstar_focus: false,
            });
        }
    });

    ()
}

fn display_unit_list(ui: &mut Ui, units: &mut Vec<UiUnit>) {
    let mut to_delete = Vec::new();
    for (i, unit) in units.iter_mut().enumerate() {
        ui.group(|ui| {
            if ui.button("X").clicked() {
                to_delete.push(i);
            }
            ui.text_edit_singleline(&mut unit.name);
            egui::ComboBox::from_id_source((i, "display_unit_list"))
                .selected_text(format!("{:?}", unit.color))
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut unit.color, Color::Red, "Red");
                    ui.selectable_value(&mut unit.color, Color::Blue, "Blue");
                    ui.selectable_value(&mut unit.color, Color::Green, "Green");
                    ui.selectable_value(&mut unit.color, Color::Colorless, "Colorless");
                });
            ui.checkbox(&mut unit.fourstar_focus, "4* focus?");
        });
    }
    for &i in to_delete.iter().rev() {
        units.remove(i);
    }
}
