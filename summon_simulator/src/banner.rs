use serde::{Deserialize, Serialize};

use crate::types::Pool;

#[derive(Copy, Clone, Debug)]
pub enum StandardBanner {
    Standard {
        focus: [u8; 4],
    },
    NewHeroes {
        focus: [u8; 4],
        fourstar_focus: [u8; 4],
    },
    NewSeasonal {
        focus: [u8; 4],
        fourstar_focus: [u8; 4],
    },
    WeeklyRevival {
        focus: [u8; 4],
    },
    Legendary,
    HeroFest,
    LegendaryRemix,
    DoubleSpecial {
        fourstar_focus: [u8; 4],
    },
}

impl StandardBanner {
    pub fn as_generic_banner(&self, has_feh_pass: bool) -> GenericBanner {
        use StandardBanner::*;
        match *self {
            Standard { focus } => GenericBanner {
                starting_rates: (3, 3),
                focus_sizes: focus,
                fourstar_focus_sizes: [0, 0, 0, 0],
                has_spark: false,
                has_charges: true,
            },
            NewHeroes {
                focus,
                fourstar_focus,
            } => GenericBanner {
                starting_rates: (3, 3),
                focus_sizes: focus,
                fourstar_focus_sizes: fourstar_focus,
                has_spark: true,
                has_charges: true,
            },
            NewSeasonal {
                focus,
                fourstar_focus,
            } => GenericBanner {
                starting_rates: (3, 3),
                focus_sizes: focus,
                fourstar_focus_sizes: fourstar_focus,
                has_spark: has_feh_pass,
                has_charges: has_feh_pass,
            },
            WeeklyRevival { focus } => GenericBanner {
                starting_rates: (4, 2),
                focus_sizes: focus,
                fourstar_focus_sizes: [0, 0, 0, 0],
                has_spark: false,
                has_charges: true,
            },
            Legendary => GenericBanner {
                starting_rates: (8, 0),
                focus_sizes: [3, 3, 3, 3],
                fourstar_focus_sizes: [0, 0, 0, 0],
                has_spark: has_feh_pass,
                has_charges: false,
            },
            HeroFest => GenericBanner {
                starting_rates: (5, 3),
                focus_sizes: [1, 1, 1, 1],
                fourstar_focus_sizes: [0, 0, 0, 0],
                has_spark: has_feh_pass,
                has_charges: has_feh_pass,
            },
            LegendaryRemix => GenericBanner {
                starting_rates: (6, 0),
                focus_sizes: [2, 2, 2, 2],
                fourstar_focus_sizes: [0, 0, 0, 0],
                has_spark: true,
                has_charges: false,
            },
            DoubleSpecial { fourstar_focus } => GenericBanner {
                starting_rates: (6, 0),
                focus_sizes: [2, 2, 2, 2],
                fourstar_focus_sizes: fourstar_focus,
                has_spark: false,
                has_charges: false,
            },
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GenericBanner {
    pub starting_rates: (u8, u8),
    pub focus_sizes: [u8; 4],
    pub fourstar_focus_sizes: [u8; 4],
    pub has_spark: bool,
    pub has_charges: bool,
}

impl GenericBanner {
    pub fn pool_sizes(&self, pool: Pool) -> [u8; 4] {
        use Pool::*;
        match pool {
            Focus => self.focus_sizes,
            Fivestar => [29, 29, 20, 16],
            FourstarFocus => self.fourstar_focus_sizes,
            FourstarSpecial => [62, 41, 36, 28],
            Common => [41, 44, 35, 44],
        }
    }

    pub fn starting_rates(&self) -> [u8; 5] {
        let focus_rate = self.starting_rates.0;
        let fivestar_rate = self.starting_rates.1;
        let fourstar_focus_rate = if self.fourstar_focus_sizes != [0, 0, 0, 0] {
            3
        } else {
            0
        };
        let fourstar_special_rate = 3;
        let common_rate =
            100 - focus_rate - fivestar_rate - fourstar_focus_rate - fourstar_special_rate;
        [
            focus_rate,
            fivestar_rate,
            fourstar_focus_rate,
            fourstar_special_rate,
            common_rate,
        ]
    }

    pub fn is_valid(&self) -> bool {
        if self.starting_rates.0.saturating_add(self.starting_rates.1) > 100
            || self.starting_rates.0 == 0
        {
            return false;
        }
        for i in 0..4 {
            if self.fourstar_focus_sizes[i] > self.focus_sizes[i] {
                return false;
            }
        }
        if self.focus_sizes == [0, 0, 0, 0] {
            return false;
        }
        true
    }
}
