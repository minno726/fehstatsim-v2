use std::array;

use memoize::memoize;
use rand::{distributions::WeightedIndex, prelude::Distribution, rngs::SmallRng, Rng, SeedableRng};

use crate::{
    banner::GenericBanner,
    frequency_counter::FrequencyCounter,
    goal::{BudgetGoal, Goal, UnitCountGoal},
    types::{Color, Pool},
};

#[derive(Copy, Clone, Debug)]
struct Status {
    total_pulled: u32,
    orbs_spent: u32,
    pity_count: u32,
    focus_charges: u32,
}

impl Status {
    fn update(&mut self, pool: Pool, session_orb_count: u32) {
        self.total_pulled += 1;
        self.orbs_spent += match session_orb_count {
            1 => 5,
            2..=4 => 4,
            5 => 3,
            _ => panic!("Invalid num_pulled"),
        };

        // Pity rate: reset for a focus, subtract 2% worth for off-focus, increment otherwise
        self.pity_count = match pool {
            Pool::Focus => 0,
            Pool::Fivestar => self.pity_count.saturating_sub(20),
            _ => self.pity_count + 1,
        };

        // Focus charges: reset for a focus unit while charges are active, increment for off-focus
        self.focus_charges = match (pool, self.focus_charges) {
            (Pool::Fivestar, _) => (self.focus_charges + 1).min(3),
            (Pool::Focus, 3) => 0,
            _ => self.focus_charges,
        };
    }
}

pub fn sim(banner: &GenericBanner, goal: &Goal, iters: u32) -> FrequencyCounter {
    match goal {
        Goal::Quantity(goal) => sim_until_goal_many(banner, goal, iters),
        Goal::OrbBudget(goal) => sim_orb_budget_many(banner, goal, iters),
    }
}

pub fn sim_until_goal_many(
    banner: &GenericBanner,
    goal: &UnitCountGoal,
    iters: u32,
) -> FrequencyCounter {
    let mut counter = FrequencyCounter::new();
    for _ in 0..iters {
        let result = sim_until_goal(banner, goal.clone());
        counter[result] += 1;
    }
    counter
}

pub fn sim_until_goal(banner: &GenericBanner, mut goal: UnitCountGoal) -> u32 {
    let mut status = Status {
        total_pulled: 0,
        orbs_spent: 0,
        pity_count: 0,
        focus_charges: 0,
    };
    let has_common_unit = goal
        .units
        .iter()
        .any(|unit| unit.pools.contains(Pool::Common));
    let mut rng = SmallRng::from_rng(&mut rand::thread_rng()).unwrap();
    'sim: loop {
        let mut num_pulled = 0;
        let session = make_session(banner, &status, &mut rng);
        for (i, &(pool, color)) in session.iter().enumerate() {
            if goal.colors().contains(color) || (num_pulled == 0 && i == 4) {
                num_pulled += 1;
                status.update(pool, num_pulled);

                if has_common_unit || pool != Pool::Common {
                    let unit_index = rng.gen_range(0..banner.pool_sizes(pool)[color as usize]);
                    goal.pull(pool, color, unit_index);
                    if goal.finished() {
                        break 'sim;
                    }
                }

                // Don't finish the session if a spark is enough to reach the goal
                if banner.has_spark
                    && status.total_pulled == 40
                    && goal.units.iter().map(|unit| unit.copies).sum::<u32>() == 1
                {
                    break;
                }
            }
        }
        debug_assert!(num_pulled >= 1 && num_pulled <= 5);
        // Spark, if possible
        if banner.has_spark && status.total_pulled >= 40 && (status.total_pulled - num_pulled) < 40
        {
            // If there's one unit with more copies required left than the others, pick that one
            let max_copies_needed = goal.units.iter().map(|unit| unit.copies).max().unwrap();
            let mut spark_candidates = goal
                .units
                .iter_mut()
                .filter(|unit| unit.copies == max_copies_needed)
                .collect::<Vec<_>>();
            // If there are multiples, then just pick the first one
            spark_candidates[0].copies -= 1;
            if goal.finished() {
                break 'sim;
            }
        }
    }

    status.orbs_spent
}

pub fn sim_orb_budget_many(
    banner: &GenericBanner,
    goal: &BudgetGoal,
    iters: u32,
) -> FrequencyCounter {
    let mut counter = FrequencyCounter::new();
    for _ in 0..iters {
        let result = sim_orb_budget(banner, goal);
        counter[result] += 1;
    }
    counter
}

pub fn sim_orb_budget(banner: &GenericBanner, goal: &BudgetGoal) -> u32 {
    let mut status = Status {
        total_pulled: 0,
        orbs_spent: 0,
        pity_count: 0,
        focus_charges: 0,
    };
    let mut num_goal_units_pulled = 0;
    let is_common_unit = goal.pools.contains(Pool::Common);
    let mut rng = SmallRng::from_rng(&mut rand::thread_rng()).unwrap();
    loop {
        let mut num_pulled = 0;
        let session = make_session(banner, &status, &mut rng);
        for (i, &(pool, color)) in session.iter().enumerate() {
            let next_orb_cost = match num_pulled {
                0 => 5,
                1..=3 => 4,
                4 => 3,
                _ => panic!("Invalid num_pulled"),
            };
            if status.orbs_spent + next_orb_cost > goal.limit {
                break;
            }
            if goal.color == color || (num_pulled == 0 && i == 4) {
                num_pulled += 1;
                status.update(pool, num_pulled);

                if (is_common_unit || pool != Pool::Common)
                    && goal.color == color
                    && goal.pools.contains(pool)
                {
                    let unit_index = rng.gen_range(0..banner.pool_sizes(pool)[color as usize]);
                    if unit_index == 0 {
                        num_goal_units_pulled += 1;
                    }
                }
            }
        }
        debug_assert!(num_pulled >= 1 && num_pulled <= 5);
        // Spark, if possible
        if banner.has_spark && status.total_pulled >= 40 && (status.total_pulled - num_pulled) < 40
        {
            num_goal_units_pulled += 1;
        }

        if status.orbs_spent + 5 > goal.limit {
            break;
        }
    }

    num_goal_units_pulled
}

fn make_session(banner: &GenericBanner, status: &Status, rng: &mut SmallRng) -> [(Pool, Color); 5] {
    let rates = get_rates(
        banner.starting_rates(),
        status.pity_count / 5,
        banner.has_charges && status.focus_charges >= 3,
    );

    let pool_dist = WeightedIndex::new(&rates).unwrap();

    array::from_fn(|_| {
        let pool = pool_dist.sample(rng);
        let pool = pool.try_into().unwrap();

        let color_dist = WeightedIndex::new(&banner.pool_sizes(pool)).unwrap();
        let color = color_dist.sample(rng);
        let color = color.try_into().unwrap();

        (pool, color)
    })
}

#[memoize(CustomHasher: fxhash::FxHashMap, HasherInit: fxhash::FxHashMap::default())]
fn get_rates(starting_rates: [u8; 5], pity_incr: u32, focus_charge_active: bool) -> [f64; 5] {
    let pity_pct = pity_incr as f64 * 0.005;
    let mut rates: [f64; 5] = std::array::from_fn(|i| starting_rates[i] as f64 / 100.0);
    let fivestar_total = rates[Pool::Focus as usize] + rates[Pool::Fivestar as usize];
    if pity_incr >= 24 {
        rates[Pool::Focus as usize] = rates[Pool::Focus as usize] / fivestar_total;
        rates[Pool::Fivestar as usize] = rates[Pool::Fivestar as usize] / fivestar_total;
        rates[Pool::FourstarFocus as usize] = 0.0;
        rates[Pool::FourstarSpecial as usize] = 0.0;
        rates[Pool::Common as usize] = 0.0;
    } else {
        rates[Pool::Focus as usize] += pity_pct * rates[Pool::Focus as usize] / fivestar_total;
        rates[Pool::Fivestar as usize] +=
            pity_pct * rates[Pool::Fivestar as usize] / fivestar_total;
        rates[Pool::FourstarFocus as usize] -=
            pity_pct * rates[Pool::FourstarFocus as usize] / (1.0 - fivestar_total);
        rates[Pool::FourstarSpecial as usize] -=
            pity_pct * rates[Pool::FourstarSpecial as usize] / (1.0 - fivestar_total);
        rates[Pool::Common as usize] -=
            pity_pct * rates[Pool::Common as usize] / (1.0 - fivestar_total);
    }

    if focus_charge_active {
        rates[Pool::Focus as usize] += rates[Pool::Fivestar as usize];
        rates[Pool::Fivestar as usize] = 0.0;
    }

    debug_assert!((rates.iter().sum::<f64>() - 1.0).abs() < 0.00000001);

    rates
}

#[cfg(test)]
mod test {
    use enumset::EnumSet;

    use crate::{banner::BannerType, goal::UnitGoal};

    use super::*;

    fn median(counter: &FrequencyCounter) -> u32 {
        let total = counter.iter().sum();
        let mut cum_total = 0;
        counter
            .iter()
            .enumerate()
            .find(|&(_, el)| {
                cum_total += *el;
                cum_total * 2 >= total
            })
            .unwrap()
            .0 as u32
    }

    fn high_percentile(counter: &FrequencyCounter) -> u32 {
        let total: u32 = counter.iter().sum();
        let mut cum_total = 0;
        counter
            .iter()
            .enumerate()
            .find(|&(_, el)| {
                cum_total += *el;
                cum_total * 10 >= total * 9
            })
            .unwrap()
            .0 as u32
    }

    #[test]
    fn test_distribution() {
        let banner = BannerType::Standard {
            focus: [1, 1, 1, 1],
        }
        .as_generic_banner(false);
        let goal = UnitCountGoal {
            units: vec![UnitGoal {
                color: Color::Red,
                copies: 1,
                pools: EnumSet::from(Pool::Focus),
            }],
        };
        let results = sim_until_goal_many(&banner, &goal, 1000);

        {
            let mut banner = banner.clone();
            banner.has_charges = false;
            let results_without_focus_charges = sim_until_goal_many(&banner, &goal, 1000);
            let medians = dbg!(
                high_percentile(&results),
                high_percentile(&results_without_focus_charges)
            );
            assert!(medians.0 <= medians.1);
        }

        {
            let mut banner = banner.clone();
            banner.starting_rates = (4, 2);
            let results_with_higher_rate = sim_until_goal_many(&banner, &goal, 1000);
            let medians = dbg!(median(&results_with_higher_rate), median(&results));
            assert!(medians.0 <= medians.1);
        }

        {
            let mut banner = banner.clone();
            banner.focus_sizes = [1, 1, 1, 0];
            let results_with_fewer_focuses = sim_until_goal_many(&banner, &goal, 1000);
            let medians = dbg!(median(&results_with_fewer_focuses), median(&results));
            assert!(medians.0 <= medians.1);
        }

        {
            let mut goal = goal.clone();
            goal.units[0].pools |= Pool::Common;
            let results_with_common_pool = sim_until_goal_many(&banner, &goal, 1000);
            let medians = dbg!(median(&results_with_common_pool), median(&results));
            assert!(medians.0 <= medians.1);
        }

        {
            let mut banner = banner.clone();
            banner.has_spark = true;
            let results_with_spark = sim_until_goal_many(&banner, &goal, 1000);
            assert!(dbg!(results_with_spark.len()) <= 201);
        }

        {
            let mut goal = goal.clone();
            let mut banner = banner.clone();
            goal.units[0].copies = 2;
            let results_with_extra_copy = sim_until_goal_many(&banner, &goal, 1000);
            banner.has_spark = true;
            let results_with_extra_copy_and_spark = sim_until_goal_many(&banner, &goal, 1000);
            let medians = dbg!(
                median(&results_with_extra_copy_and_spark),
                median(&results_with_extra_copy)
            );
            assert!(medians.0 <= medians.1);
        }

        {
            let mut goal = goal.clone();
            let mut banner = banner.clone();
            goal.units[0].pools |= Pool::FourstarFocus;
            banner.fourstar_focus_sizes = [1, 0, 0, 0];
            let results_with_fourstar_focus = sim_until_goal_many(&banner, &goal, 1000);
            banner.fourstar_focus_sizes = [1, 0, 0, 1];
            let results_with_extra_fourstar_focus = sim_until_goal_many(&banner, &goal, 1000);
            let medians = dbg!(
                median(&results_with_fourstar_focus),
                median(&results_with_extra_fourstar_focus),
                median(&results)
            );
            assert!(medians.0 <= medians.1 && medians.1 <= medians.2);
        }
    }

    #[test]
    fn test_budget() {
        let banner = BannerType::Standard {
            focus: [1, 1, 1, 1],
        }
        .as_generic_banner(false);
        let goal = BudgetGoal {
            color: Color::Red,
            limit: 200,
            pools: EnumSet::from(Pool::Focus),
        };
        let results = sim_orb_budget_many(&banner, &goal, 1000);

        {
            let mut banner = banner.clone();
            banner.has_spark = true;
            let results_with_spark = sim_orb_budget_many(&banner, &goal, 1000);
            assert!(results_with_spark[0] == 0);
            let medians = dbg!(median(&results), median(&results_with_spark));
            assert!(medians.0 <= medians.1);
        }

        {
            let mut goal = goal.clone();
            goal.limit = 1500;
            let results_with_many = sim_orb_budget_many(&banner, &goal, 1000);
            let median = dbg!(median(&results_with_many));
            assert!(median >= 10);
            assert!(median <= 11);
        }
    }
}
