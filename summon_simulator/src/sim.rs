use rand::{prelude::Distribution, Rng, SeedableRng};
use rand_xoshiro::Xoshiro128Plus;

use crate::{
    banner::GenericBanner,
    frequency_counter::FrequencyCounter,
    goal::{BudgetGoal, BudgetGoalLimit, Goal, UnitCountGoal},
    types::{Color, Pool},
    weightedindex::{WeightedIndexColor, WeightedIndexPool},
};

#[derive(Copy, Clone, Debug)]
struct Status {
    total_pulled: u32,
    orbs_spent: u32,
    pity_count: u32,
    focus_charges: u32,
}

struct DistributionCache {
    color_dists: Vec<WeightedIndexColor>,
    pool_dists: Vec<WeightedIndexPool>,
}

impl DistributionCache {
    pub fn new(banner: &GenericBanner) -> Self {
        let mut pool_dists = Vec::new();
        for i in 0..=24 {
            pool_dists.push(get_pool_dist(banner.starting_rates(), i, true));
        }
        for i in 0..=24 {
            pool_dists.push(get_pool_dist(banner.starting_rates(), i, false));
        }
        let color_dists = vec![
            get_color_dist(banner.pool_sizes(Pool::Focus)),
            get_color_dist(banner.pool_sizes(Pool::Fivestar)),
            get_color_dist(banner.pool_sizes(Pool::FourstarFocus)),
            get_color_dist(banner.pool_sizes(Pool::FourstarSpecial)),
            get_color_dist(banner.pool_sizes(Pool::Common)),
        ];
        Self {
            color_dists,
            pool_dists,
        }
    }

    pub fn get_pool_dist(&self, pity_incr: u32, focus_charge_active: bool) -> &WeightedIndexPool {
        let pity_incr = pity_incr.min(24);
        if focus_charge_active {
            &self.pool_dists[pity_incr as usize]
        } else {
            &self.pool_dists[pity_incr as usize + 25]
        }
    }

    pub fn get_color_dist(&self, pool: Pool) -> &WeightedIndexColor {
        &self.color_dists[pool as usize]
    }
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

pub struct Sim {
    banner: GenericBanner,
    goal: Goal,
    data: FrequencyCounter,
}

impl Sim {
    pub fn new(banner: GenericBanner, goal: Goal) -> Self {
        Self {
            banner,
            goal,
            data: FrequencyCounter::new(),
        }
    }

    pub fn sim(&mut self, iters: u32) -> &mut Self {
        let new_data = match &self.goal {
            Goal::Quantity(goal) => sim_until_goal_many(&self.banner, goal, iters),
            Goal::OrbBudget(goal) => sim_orb_budget_many(&self.banner, goal, iters),
        };

        self.data.combine(new_data);
        self
    }

    pub fn data(&self) -> &FrequencyCounter {
        &self.data
    }

    pub fn into_data(self) -> FrequencyCounter {
        self.data
    }
}

pub fn sim_until_goal_many(
    banner: &GenericBanner,
    goal: &UnitCountGoal,
    iters: u32,
) -> FrequencyCounter {
    let mut counter = FrequencyCounter::new();
    let cache = DistributionCache::new(banner);
    let mut rng = Xoshiro128Plus::from_rng(&mut rand::thread_rng()).unwrap();
    for _ in 0..iters {
        let result = sim_until_goal(banner, goal.clone(), &mut rng, &cache);
        counter[result] += 1;
    }
    counter
}

fn sim_until_goal(
    banner: &GenericBanner,
    mut goal: UnitCountGoal,
    rng: &mut impl Rng,
    cache: &DistributionCache,
) -> u32 {
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
    'sim: loop {
        let mut num_pulled = 0;
        let session = make_session(banner, &status, rng, cache);
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
        debug_assert!((1..=5).contains(&num_pulled));
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
    let cache = DistributionCache::new(banner);
    let mut rng = Xoshiro128Plus::from_rng(&mut rand::thread_rng()).unwrap();
    for _ in 0..iters {
        let result = sim_orb_budget(banner, goal, &mut rng, &cache);
        counter[result] += 1;
    }
    counter
}

fn sim_orb_budget(
    banner: &GenericBanner,
    goal: &BudgetGoal,
    rng: &mut impl Rng,
    cache: &DistributionCache,
) -> u32 {
    let mut status = Status {
        total_pulled: 0,
        orbs_spent: 0,
        pity_count: 0,
        focus_charges: 0,
    };
    let mut num_goal_units_pulled = 0;
    let is_common_unit = goal.pools.contains(Pool::Common);
    loop {
        let mut num_pulled = 0;
        let session = make_session(banner, &status, rng, cache);
        for (i, &(pool, color)) in session.iter().enumerate() {
            let next_orb_cost = match num_pulled {
                0 => 5,
                1..=3 => 4,
                4 => 3,
                _ => panic!("Invalid num_pulled"),
            };
            if let BudgetGoalLimit::OrbCount(limit) = goal.limit {
                if status.orbs_spent + next_orb_cost > limit {
                    break;
                }
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
        debug_assert!((1..=5).contains(&num_pulled));
        // Spark, if possible
        if banner.has_spark && status.total_pulled >= 40 && (status.total_pulled - num_pulled) < 40
        {
            num_goal_units_pulled += 1;
            if goal.limit == BudgetGoalLimit::UntilSpark {
                break;
            }
        }

        if let BudgetGoalLimit::OrbCount(limit) = goal.limit {
            if status.orbs_spent + 5 > limit {
                break;
            }
        }
    }

    num_goal_units_pulled
}

fn make_session(
    banner: &GenericBanner,
    status: &Status,
    rng: &mut impl Rng,
    cache: &DistributionCache,
) -> [(Pool, Color); 5] {
    let pool_dist = cache.get_pool_dist(
        status.pity_count / 5,
        banner.has_charges && status.focus_charges >= 3,
    );

    let mut gen = || {
        let pool = pool_dist.sample(rng);
        let color_dist = cache.get_color_dist(pool);
        let color = color_dist.sample(rng);

        (pool, color)
    };
    [gen(), gen(), gen(), gen(), gen()]
}

fn get_color_dist(pool_sizes: [u8; 4]) -> WeightedIndexColor {
    WeightedIndexColor::new(pool_sizes)
}

fn get_pool_dist(
    starting_rates: [u8; 5],
    pity_incr: u32,
    focus_charge_active: bool,
) -> WeightedIndexPool {
    let pity_pct = pity_incr as f32 * 0.005;
    let mut rates: [f32; 5] = std::array::from_fn(|i| starting_rates[i] as f32 / 100.0);
    let fivestar_total = rates[Pool::Focus as usize] + rates[Pool::Fivestar as usize];
    if pity_incr >= 24 {
        rates[Pool::Focus as usize] /= fivestar_total;
        rates[Pool::Fivestar as usize] /= fivestar_total;
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

    debug_assert!((rates.iter().sum::<f32>() - 1.0).abs() < 0.0000001);

    WeightedIndexPool::new(rates)
}

#[cfg(test)]
mod test {
    use enumset::EnumSet;

    use crate::{banner::StandardBanner, goal::UnitGoal};

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

    fn standard() -> (GenericBanner, Goal) {
        let banner = StandardBanner::Standard {
            focus: [1, 1, 1, 1],
        }
        .as_generic_banner(false);
        let goal = Goal::Quantity(UnitCountGoal::new(
            vec![UnitGoal {
                color: Color::Red,
                copies: 1,
                pools: EnumSet::from(Pool::Focus),
            }],
            true,
        ));
        (banner, goal)
    }

    #[test]
    fn test_distribution_focus_charges() {
        let (mut banner, goal) = standard();
        let results = Sim::new(banner.clone(), goal.clone())
            .sim(10000)
            .data()
            .clone();

        banner.has_charges = false;
        let results_without_focus_charges = Sim::new(banner, goal).sim(10000).data().clone();
        let medians = dbg!(
            high_percentile(&results),
            high_percentile(&results_without_focus_charges)
        );
        assert!(medians.0 <= medians.1);
    }

    #[test]
    fn test_distribution_revival_rates() {
        let (mut banner, goal) = standard();
        let results = Sim::new(banner.clone(), goal.clone())
            .sim(10000)
            .data()
            .clone();

        banner.starting_rates = (4, 2);
        let results_with_higher_rate = Sim::new(banner, goal).sim(10000).data().clone();
        let medians = dbg!(median(&results_with_higher_rate), median(&results));
        assert!(medians.0 <= medians.1);
    }

    #[test]
    fn test_distribution_smaller_focus_pool() {
        let (mut banner, goal) = standard();
        let results = Sim::new(banner.clone(), goal.clone())
            .sim(10000)
            .data()
            .clone();

        banner.focus_sizes = [1, 1, 1, 0];
        let results_with_fewer_focuses = Sim::new(banner, goal).sim(10000).data().clone();
        let medians = dbg!(median(&results_with_fewer_focuses), median(&results));
        assert!(medians.0 <= medians.1);
    }

    #[test]
    fn test_distribution_goal_in_common_pool() {
        let (banner, mut goal) = standard();
        let results = Sim::new(banner.clone(), goal.clone())
            .sim(10000)
            .data()
            .clone();

        match goal {
            Goal::Quantity(ref mut goal) => goal.units[0].pools |= Pool::Common,
            Goal::OrbBudget(_) => {}
        }
        let results_with_common_pool = Sim::new(banner, goal).sim(10000).data().clone();
        let medians = dbg!(median(&results_with_common_pool), median(&results));
        assert!(medians.0 <= medians.1);
    }

    #[test]
    fn test_distribution_spark() {
        let (mut banner, goal) = standard();

        banner.has_spark = true;
        let results_with_spark = Sim::new(banner, goal).sim(10000).data().clone();
        assert!(dbg!(results_with_spark.len()) <= 201);
    }

    #[test]
    fn test_distribution_spark_multiple() {
        let (mut banner, mut goal) = standard();

        match goal {
            Goal::Quantity(ref mut goal) => goal.units[0].copies = 2,
            Goal::OrbBudget(_) => {}
        }
        let results_with_extra_copy = Sim::new(banner.clone(), goal.clone())
            .sim(10000)
            .data()
            .clone();
        banner.has_spark = true;
        let results_with_extra_copy_and_spark = Sim::new(banner, goal).sim(10000).data().clone();
        let medians = dbg!(
            median(&results_with_extra_copy_and_spark),
            median(&results_with_extra_copy)
        );
        assert!(medians.0 <= medians.1);
    }

    #[test]
    fn test_distribution_fourstar_focus() {
        let (mut banner, mut goal) = standard();
        let results = Sim::new(banner.clone(), goal.clone())
            .sim(10000)
            .data()
            .clone();

        match goal {
            Goal::Quantity(ref mut goal) => goal.units[0].pools |= Pool::FourstarFocus,
            Goal::OrbBudget(_) => {}
        }
        banner.fourstar_focus_sizes = [1, 0, 0, 0];
        let results_with_fourstar_focus = Sim::new(banner.clone(), goal.clone())
            .sim(10000)
            .data()
            .clone();

        banner.fourstar_focus_sizes = [1, 0, 0, 1];
        let results_with_extra_fourstar_focus = Sim::new(banner, goal).sim(10000).data().clone();

        let medians = dbg!(
            median(&results_with_fourstar_focus),
            median(&results_with_extra_fourstar_focus),
            median(&results)
        );
        assert!(medians.0 <= medians.1 && medians.1 <= medians.2);
    }

    #[test]
    fn test_distribution_any_all() {
        let (mut banner, mut goal) = standard();
        banner.focus_sizes = [2, 0, 1, 1];
        let basic_results = Sim::new(banner.clone(), goal.clone())
            .sim(10000)
            .data()
            .clone();

        match goal {
            Goal::Quantity(ref mut goal) => goal.units.push(UnitGoal {
                color: Color::Red,
                copies: 1,
                pools: EnumSet::from(Pool::Focus),
            }),
            Goal::OrbBudget(_) => {}
        }
        let results_needing_multiple_colors = Sim::new(banner.clone(), goal.clone())
            .sim(10000)
            .data()
            .clone();

        match goal {
            Goal::Quantity(ref mut goal) => goal.need_all = false,
            Goal::OrbBudget(_) => {}
        }
        let results_accepting_multiple_colors = Sim::new(banner, goal).sim(10000).data().clone();
        let medians = dbg!(
            median(&results_accepting_multiple_colors),
            median(&basic_results),
            median(&results_needing_multiple_colors),
        );
        // Pulling for either of the two is slightly worse than half as expensive as pulling for just one.
        assert!(medians.0 <= medians.1);
        assert!(medians.1 <= medians.0 * 2);

        // Pulling for both of the two is less than twice as expensive as pulling for just one, because you
        // first pull for either and then pull for just one, and that first phase is cheaper.
        assert!(medians.1 <= medians.2);
        assert!(medians.2 <= medians.1 * 2);
    }

    #[test]
    fn test_budget() {
        let banner = StandardBanner::Standard {
            focus: [1, 1, 1, 1],
        }
        .as_generic_banner(false);
        let goal = Goal::OrbBudget(BudgetGoal {
            color: Color::Red,
            limit: BudgetGoalLimit::OrbCount(200),
            pools: EnumSet::from(Pool::Focus),
        });
        let results = Sim::new(banner.clone(), goal.clone())
            .sim(10000)
            .data()
            .clone();

        {
            let mut banner = banner.clone();
            banner.has_spark = true;
            let results_with_spark = Sim::new(banner, goal.clone()).sim(10000).data().clone();
            assert!(results_with_spark[0] == 0);
            let medians = dbg!(median(&results), median(&results_with_spark));
            assert!(medians.0 <= medians.1);
        }

        {
            let mut goal = goal.clone();
            match goal {
                Goal::OrbBudget(ref mut goal) => goal.limit = BudgetGoalLimit::OrbCount(1500),
                Goal::Quantity(_) => {}
            }
            let results_with_many = Sim::new(banner, goal).sim(10000).data().clone();
            let median = dbg!(median(&results_with_many));
            assert!(median >= 10);
            assert!(median <= 11);
        }
    }
}
