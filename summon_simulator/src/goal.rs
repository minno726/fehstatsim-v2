use enumset::EnumSet;
use serde::{Deserialize, Serialize};

use crate::types::{Color, Pool};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Goal {
    Quantity(UnitCountGoal),
    OrbBudget(BudgetGoal),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UnitCountGoal {
    pub units: Vec<UnitGoal>,
    pub need_all: bool,
    colors: EnumSet<Color>,
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub struct UnitGoal {
    pub color: Color,
    pub copies: u32,
    pub pools: EnumSet<Pool>,
}

impl UnitCountGoal {
    pub fn new(units: Vec<UnitGoal>, need_all: bool) -> Self {
        let mut result = Self {
            units,
            need_all,
            colors: EnumSet::new(),
        };
        result.calculate_colors();
        result
    }

    pub fn colors(&self) -> EnumSet<Color> {
        self.colors
    }

    fn calculate_colors(&mut self) {
        self.colors = self
            .units
            .iter()
            .filter(|unit| unit.copies > 0)
            .fold(EnumSet::new(), |set, unit| set | unit.color);
    }

    pub fn pull(&mut self, pool: Pool, color: Color, unit_index: u8) {
        for (idx, unit) in self
            .units
            .iter_mut()
            .filter(|unit| unit.color == color && unit.pools.contains(pool) && unit.copies > 0)
            .enumerate()
        {
            if idx as u8 == unit_index {
                unit.copies -= 1;
                self.calculate_colors();
                return;
            }
        }
    }

    pub fn finished(&self) -> bool {
        if self.need_all {
            self.units.iter().all(|unit| unit.copies == 0)
        } else {
            self.units.iter().any(|unit| unit.copies == 0)
        }
    }
}

#[derive(PartialEq, Eq, Copy, Clone, Debug, Serialize, Deserialize)]

pub enum BudgetGoalLimit {
    OrbCount(u32),
    UntilSpark,
}

#[derive(PartialEq, Eq, Copy, Clone, Debug, Serialize, Deserialize)]
pub struct BudgetGoal {
    pub color: Color,
    pub limit: BudgetGoalLimit,
    pub pools: EnumSet<Pool>,
}
