use egui::{text::TextWrapping, TextStyle, Ui, Widget};
use egui_extras::{Column, TableBuilder};
use enumset::EnumSet;
use summon_simulator::{
    goal::{BudgetGoal, BudgetGoalLimit, Goal, UnitCountGoal, UnitGoal},
    types::Pool,
};

use crate::{app::with_colored_dot, banner::UiBanner};

pub struct SingleGoal {
    pub is_quantity_goal: bool,
    pub unit_count_goal: u32,
    pub orb_limit: u32,
    pub unit_idx: usize,
}

pub struct MultiGoal {
    pub unit_count_goals: Vec<u32>,
    pub require_all: bool,
}

pub struct GoalState {
    pub banner: UiBanner,
    pub is_single: bool,
    pub single: SingleGoal,
    pub multi: MultiGoal,
}

impl GoalState {
    pub fn new(banner: UiBanner, is_single: bool) -> Self {
        let num_possible_units = banner.units.len();
        GoalState {
            banner,
            is_single,
            single: SingleGoal {
                is_quantity_goal: true,
                unit_count_goal: 1,
                orb_limit: 5,
                unit_idx: 0,
            },
            multi: MultiGoal {
                unit_count_goals: vec![0; num_possible_units],
                require_all: true,
            },
        }
    }

    pub fn set_banner(&mut self, banner: UiBanner) {
        if banner.units.len() != self.multi.unit_count_goals.len() {
            *self = Self::new(banner, self.is_single);
        } else {
            self.banner = banner;
        }
    }

    pub fn to_sim_goal(&self) -> Option<Goal> {
        if self.is_single {
            let unit = &self.banner.units[self.single.unit_idx];
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
                .filter_map(|(unit, count)| {
                    Some(UnitGoal {
                        color: unit.color,
                        copies: if *count > 0 { *count } else { return None },
                        pools: if unit.fourstar_focus {
                            EnumSet::from(Pool::Focus) | Pool::FourstarFocus
                        } else {
                            EnumSet::from(Pool::Focus)
                        },
                    })
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

    if state.banner.to_sim_banner().is_none() {
        ui.label("Invalid banner");
        return false;
    }

    ui.horizontal(|ui| {
        if ui
            .selectable_value(&mut state.is_single, true, "Single unit")
            .changed()
        {
            goal_changed = true;
        }
        if ui
            .selectable_value(&mut state.is_single, false, "Multiple units")
            .changed()
        {
            goal_changed = true;
        }
    });

    if state.is_single {
        let selected_unit_before = state.single.unit_idx;
        egui::ComboBox::from_label("Unit")
            .selected_text(with_colored_dot(
                &state.banner.units[state.single.unit_idx].name,
                state.banner.units[state.single.unit_idx].color,
                TextStyle::Button.resolve(&ui.ctx().style()),
            ))
            .show_ui(ui, |ui| {
                for (i, unit) in state.banner.units.iter().enumerate() {
                    ui.selectable_value(
                        &mut state.single.unit_idx,
                        i,
                        with_colored_dot(
                            &unit.name,
                            unit.color,
                            TextStyle::Button.resolve(&ui.ctx().style()),
                        ),
                    );
                }
            });
        if selected_unit_before != state.single.unit_idx {
            goal_changed = true;
        }
        ui.horizontal(|ui| {
            if ui
                .selectable_value(&mut state.single.is_quantity_goal, true, "# of copies")
                .changed()
            {
                goal_changed = true;
            }
            if ui
                .selectable_value(&mut state.single.is_quantity_goal, false, "# of orbs")
                .changed()
            {
                goal_changed = true;
            }
        });

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
            .column(Column::exact(270.0))
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
                        cb_text.wrap = TextWrapping::truncate_at_width(270.0);
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
                            goal_changed = true;
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
