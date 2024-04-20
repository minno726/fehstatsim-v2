use egui::{text::TextWrapping, TextStyle, Ui, Widget};
use egui_extras::{Column, TableBuilder};
use enumset::EnumSet;
use summon_simulator::{
    goal::{BudgetGoal, BudgetGoalLimit, Goal, UnitCountGoal, UnitGoal},
    types::Pool,
};

use crate::{app::with_colored_dot, banner::UiBanner};

struct SingleGoal {
    is_quantity_goal: bool,
    unit_count_goal: u32,
    orb_limit: u32,
    unit: String,
}

struct MultiGoal {
    unit_count_goals: Vec<u32>,
    require_all: bool,
}

pub struct GoalState {
    pub banner: UiBanner,
    pub is_single: bool,
    single: SingleGoal,
    multi: MultiGoal,
}

impl GoalState {
    pub fn new(banner: UiBanner, is_single: bool) -> Self {
        let default_unit = banner.units[0].name.clone();
        let num_possible_units = banner.units.len();
        GoalState {
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

pub(crate) fn display_goal(ui: &mut Ui, state: &mut GoalState) -> bool {
    let mut goal_changed = false;

    if ui
        .radio_value(&mut state.is_single, true, "Single unit")
        .changed()
    {
        goal_changed = true;
    }
    if ui
        .radio_value(&mut state.is_single, false, "Multiple units")
        .changed()
    {
        goal_changed = true;
    }

    if state.is_single {
        let selected_unit_before = state.single.unit.clone();
        egui::ComboBox::from_label("Unit")
            .selected_text(with_colored_dot(
                &state.single.unit,
                state
                    .banner
                    .units
                    .iter()
                    .find_map(|unit| (unit.name == state.single.unit).then(|| unit.color))
                    .unwrap(),
                TextStyle::Button.resolve(&ui.ctx().style()),
            ))
            .show_ui(ui, |ui| {
                for unit in &state.banner.units {
                    ui.selectable_value(
                        &mut state.single.unit,
                        unit.name.clone(),
                        with_colored_dot(
                            &unit.name,
                            unit.color,
                            TextStyle::Small.resolve(&ui.ctx().style()),
                        ),
                    );
                }
            });
        if selected_unit_before != state.single.unit {
            goal_changed = true;
        }
        if ui
            .radio_value(&mut state.single.is_quantity_goal, true, "# of copies")
            .changed()
        {
            goal_changed = true;
        }
        if ui
            .radio_value(&mut state.single.is_quantity_goal, false, "# of orbs")
            .changed()
        {
            goal_changed = true;
        }

        if state.single.is_quantity_goal {
            let suffix = if state.single.unit_count_goal == 1 {
                " Copy"
            } else {
                " Copies"
            };
            if egui::Slider::new(&mut state.single.unit_count_goal, 1..=99)
                .suffix(suffix)
                .logarithmic(true)
                .ui(ui)
                .changed()
            {
                goal_changed = true;
            }
        } else {
            if egui::Slider::new(&mut state.single.orb_limit, 5..=10000)
                .suffix(" Orbs")
                .logarithmic(true)
                .ui(ui)
                .changed()
            {
                goal_changed = true;
            }
        }
    } else {
        ui.horizontal(|ui| {
            if ui
                .selectable_value(&mut state.multi.require_all, true, "All of these")
                .changed()
            {
                goal_changed = true;
            }
            if ui
                .selectable_value(&mut state.multi.require_all, false, "Any of these")
                .changed()
            {
                goal_changed = true;
            }
        });
        TableBuilder::new(ui)
            .column(Column::exact(200.0))
            .column(Column::remainder())
            .body(|body| {
                body.rows(24.0, state.banner.units.len(), |mut row| {
                    let i = row.index();
                    row.col(|ui| {
                        let mut enabled = state.multi.unit_count_goals[i] > 0;
                        let mut cb_text = with_colored_dot(
                            &state.banner.units[i].name,
                            state.banner.units[i].color,
                            TextStyle::Body.resolve(&ui.ctx().style()),
                        );
                        cb_text.wrap = TextWrapping::truncate_at_width(200.0);
                        if ui
                            .checkbox(
                                &mut enabled,
                                //&state.banner.units[i].name,
                                cb_text,
                            )
                            .changed()
                        {
                            if !enabled {
                                state.multi.unit_count_goals[i] = 0;
                            } else {
                                state.multi.unit_count_goals[i] = 1;
                            }
                        }
                    });
                    row.col(|ui| {
                        if state.multi.unit_count_goals[i] > 0 {
                            let suffix = if state.multi.unit_count_goals[i] == 1 {
                                " Copy"
                            } else {
                                " Copies"
                            };
                            if egui::DragValue::new(&mut state.multi.unit_count_goals[i])
                                .clamp_range(1..=99)
                                .suffix(suffix)
                                .ui(ui)
                                .changed()
                            {
                                goal_changed = true;
                            }
                        }
                    });
                })
            })
    }

    goal_changed
}
