use enumset::EnumSet;
use summon_simulator::{
    banner::GenericBanner,
    goal::{BudgetGoal, BudgetGoalLimit, Goal, UnitCountGoal, UnitGoal},
    types::{Color, Pool},
};
use yew::prelude::*;

use crate::components::select::Select;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UiUnit {
    pub name: String,
    pub color: Color,
    pub fourstar_focus: bool,
}
#[derive(Debug, Clone, PartialEq, Eq)]
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SingleGoal {
    pub is_quantity_goal: bool,
    pub unit_count_goal: u32,
    pub orb_limit: u32,
    pub unit: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MultiGoal {
    pub unit_count_goals: Vec<u32>,
    pub require_all: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UiGoal {
    pub banner: UiBanner,
    pub is_single: bool,
    pub single: SingleGoal,
    pub multi: MultiGoal,
}

impl UiGoal {
    pub fn new(banner: UiBanner, is_single: bool) -> Self {
        let default_unit = banner.units[0].name.clone();
        let num_possible_units = banner.units.len();
        UiGoal {
            banner,
            is_single,
            single: SingleGoal {
                is_quantity_goal: true,
                unit_count_goal: 1,
                orb_limit: 5,
                unit: default_unit,
            },
            multi: MultiGoal {
                unit_count_goals: vec![0; num_possible_units],
                require_all: true,
            },
        }
    }

    pub fn to_sim_goal(&self) -> Option<Goal> {
        if self.is_single {
            let unit = self
                .banner
                .units
                .iter()
                .find(|u| u.name == self.single.unit)?;
            let pools = if unit.fourstar_focus {
                EnumSet::from(Pool::Focus) | Pool::FourstarFocus
            } else {
                EnumSet::from(Pool::Focus)
            };
            if self.single.is_quantity_goal {
                Some(Goal::Quantity(UnitCountGoal::new(
                    vec![UnitGoal {
                        color: unit.color,
                        copies: self.single.unit_count_goal,
                        pools,
                    }],
                    true,
                )))
            } else {
                Some(Goal::OrbBudget(BudgetGoal {
                    color: unit.color,
                    limit: BudgetGoalLimit::OrbCount(self.single.orb_limit),
                    pools,
                }))
            }
        } else {
            if self.multi.unit_count_goals.iter().all(|&goal| goal == 0) {
                return None;
            }
            let goals = self
                .banner
                .units
                .iter()
                .zip(self.multi.unit_count_goals.iter())
                .map(|(unit, count)| UnitGoal {
                    color: unit.color,
                    copies: *count,
                    pools: if unit.fourstar_focus {
                        EnumSet::from(Pool::Focus) | Pool::FourstarFocus
                    } else {
                        EnumSet::from(Pool::Focus)
                    },
                })
                .collect::<Vec<_>>();
            Some(Goal::Quantity(UnitCountGoal::new(
                goals,
                self.multi.require_all,
            )))
        }
    }
}

#[derive(PartialEq, Properties)]
pub struct BannerSelectProps {
    pub on_banner_changed: Callback<(UiBanner, UiGoal)>,
}

pub struct BannerSelect {
    selected_banner: UiBanner,
    available_banners: Vec<UiBanner>,
    goal: UiGoal,
}

pub enum BannerSelectMsg {
    BannerSelected(UiBanner),
    RatesSelected((u8, u8)),
}

impl Component for BannerSelect {
    type Message = BannerSelectMsg;
    type Properties = BannerSelectProps;

    fn create(ctx: &Context<Self>) -> Self {
        let mut banners = default_banners();
        let selected_banner = banners[0].clone();
        let goal = UiGoal::new(selected_banner.clone(), true);
        banners.insert(0, selected_banner.clone());
        banners[0].name = "Custom".into();

        ctx.props()
            .on_banner_changed
            .emit((selected_banner.clone(), goal.clone()));

        Self {
            selected_banner,
            available_banners: banners,
            goal,
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        html! {
            <div>
                <Select<UiBanner>
                    onchange={ctx.link().callback(BannerSelectMsg::BannerSelected)}
                    values={self.available_banners.clone()}
                    current={self.selected_banner.clone()}
                    to_label={Callback::from(|b: UiBanner| b.name.clone())}/>
                <p>{
                    self.selected_banner.units.iter().map(|el| &*el.name).collect::<Vec<_>>().join(", ")
                }</p>
                <details>
                    <summary>{ "Details" }</summary>
                    <fieldset disabled={self.selected_banner.name != "Custom"} style={"border: none;"}>
                        <Select<(u8, u8)>
                            onchange={ctx.link().callback(BannerSelectMsg::RatesSelected)}
                            values={vec![(3, 3), (4, 2), (8, 0), (5, 3), (6, 0)]}
                            current={self.selected_banner.starting_rates}
                            to_label={Callback::from(|r| match r {
                                (3, 3) => "3%/3% (Standard)".into(),
                                (4, 2) => "4%/2% (Weekly Revival)".into(),
                                (8, 0) => "8%/0% (Legendary/Mythic)".into(),
                                (5, 3) => "5%/3% (Hero Fest)".into(),
                                (6, 0) => "6%/0% (Remix/Double Special)".into(),
                                _ => "INVALID".into(),
                            })}
                            />
                    </fieldset>
                </details>
            </div>
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            BannerSelectMsg::BannerSelected(new_banner) => {
                match (
                    self.selected_banner.name == "Custom",
                    new_banner.name == "Custom",
                ) {
                    (true, true) => false,
                    (false, true) => {
                        // Changing from some other preset to "custom" is just renaming it.
                        self.selected_banner.name = "Custom".into();
                        self.available_banners[0] = self.selected_banner.clone();
                        ctx.props()
                            .on_banner_changed
                            .emit((self.selected_banner.clone(), self.goal.clone()));
                        true
                    }
                    (true, false) => {
                        // Check if we're just switching back from "custom" without
                        // changing anything.
                        self.selected_banner.name = new_banner.name.clone();
                        if self.selected_banner != new_banner {
                            self.selected_banner = new_banner.clone();
                            self.goal = UiGoal::new(self.selected_banner.clone(), true);
                            ctx.props()
                                .on_banner_changed
                                .emit((self.selected_banner.clone(), self.goal.clone()));
                        }
                        true
                    }
                    (false, false) => {
                        self.selected_banner = new_banner;
                        self.goal = UiGoal::new(self.selected_banner.clone(), true);
                        ctx.props()
                            .on_banner_changed
                            .emit((self.selected_banner.clone(), self.goal.clone()));
                        true
                    }
                }
            }
            BannerSelectMsg::RatesSelected(rates) => {
                self.available_banners[0].starting_rates = rates;
                self.selected_banner = self.available_banners[0].clone();
                ctx.props()
                    .on_banner_changed
                    .emit((self.selected_banner.clone(), self.goal.clone()));
                true
            }
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
                    color: Color::Green,
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
