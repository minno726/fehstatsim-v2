use crate::banner::UiBanner;

use enumset::EnumSet;
use summon_simulator::{
    goal::{Goal, UnitCountGoal, UnitGoal},
    types::{Color, Pool},
};
use sycamore::prelude::*;

pub struct SingleGoal<'a> {
    pub is_quantity_goal: &'a Signal<bool>,
    pub unit_count_goal: &'a Signal<u32>,
    pub orb_limit: &'a Signal<u32>,
    pub unit: &'a Signal<String>,
}

pub struct MultiGoal<'a> {
    pub unit_count_goals: &'a Signal<Vec<RcSignal<u32>>>,
    pub require_all: &'a Signal<bool>,
}

pub struct GoalState<'a> {
    pub is_single: &'a Signal<bool>,
    pub single: &'a Signal<SingleGoal<'a>>,
    pub multi: &'a Signal<MultiGoal<'a>>,
}

impl<'a> GoalState<'a> {
    pub fn to_goal(&self) -> Option<Goal> {
        let sample_goal = Goal::Quantity(UnitCountGoal::new(
            vec![UnitGoal {
                color: Color::Red,
                copies: 1,
                pools: EnumSet::from(Pool::Focus),
            }],
            true,
        ));

        Some(sample_goal)
    }
}

#[component(inline_props)]
pub fn GoalSelector<'a, G: Html>(
    cx: Scope<'a>,
    banner: &'a Signal<UiBanner<'a>>,
    goal: &'a Signal<GoalState<'a>>,
) -> View<G> {
    view! { cx,
        select (bind:value=goal.get().single.get().unit) {
            Indexed(
                iterable=banner.get().units,
                view=|cx, unit| view! { cx,
                    option {
                        (unit.get().name)
                    }
                }
            )
        }
    }
}
