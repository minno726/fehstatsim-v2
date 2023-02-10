use enumset::EnumSet;

use crate::types::{Color, Pool};

#[derive(Clone, Debug)]
pub enum Goal {
    Quantity(UnitCountGoal),
    OrbBudget(BudgetGoal),
}

#[derive(Clone, Debug)]
pub struct UnitCountGoal {
    pub units: Vec<UnitGoal>,
}

#[derive(Copy, Clone, Debug)]
pub struct UnitGoal {
    pub color: Color,
    pub copies: u32,
    pub pools: EnumSet<Pool>,
}

impl UnitCountGoal {
    pub fn colors(&self) -> EnumSet<Color> {
        self.units
            .iter()
            .fold(EnumSet::new(), |set, val| set | val.color)
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
                return;
            }
        }
    }

    pub fn finished(&self) -> bool {
        self.units.iter().all(|unit| unit.copies == 0)
    }
}

#[derive(Copy, Clone, Debug)]
pub struct BudgetGoal {
    pub color: Color,
    pub limit: u32,
    pub pools: EnumSet<Pool>,
}
