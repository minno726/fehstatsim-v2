use std::array;

use rand::{distributions::WeightedIndex, prelude::Distribution, Rng};

use crate::{
    banner::GenericBanner,
    frequency_counter::FrequencyCounter,
    goal::UnitCountGoal,
    types::{Color, Pool},
};

#[derive(Copy, Clone, Debug)]
struct Status {
    orbs_spent: u32,
    pity_count: u32,
    focus_charges: u32,
}

pub fn sim_until_goal_many(
    banner: &GenericBanner,
    goal: UnitCountGoal,
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
        orbs_spent: 0,
        pity_count: 0,
        focus_charges: 0,
    };
    'sim: loop {
        let mut num_pulled = 0;
        let session = make_session(banner, &status);
        for (i, &(pool, color)) in session.iter().enumerate() {
            if goal.colors().contains(color) || (num_pulled == 0 && i == 4) {
                num_pulled += 1;
                status.orbs_spent += match num_pulled {
                    1 => 5,
                    2..=4 => 4,
                    5 => 3,
                    _ => panic!("Invalid num_pulled"),
                };

                status.pity_count = match pool {
                    Pool::Focus => 0,
                    Pool::Fivestar => status.pity_count.saturating_sub(20),
                    _ => status.pity_count + 1,
                };

                status.focus_charges = match (pool, status.focus_charges, banner.has_charges) {
                    (Pool::Fivestar, _, true) => (status.focus_charges + 1).min(3),
                    (Pool::Focus, 3, true) => 0,
                    _ => status.focus_charges,
                };

                let unit_index =
                    rand::thread_rng().gen_range(0..banner.pool_sizes(pool)[color as usize]);
                goal.pull(pool, color, unit_index);
                if goal.finished() {
                    break 'sim;
                }
            }
        }
    }

    status.orbs_spent
}

fn make_session(banner: &GenericBanner, status: &Status) -> [(Pool, Color); 5] {
    let mut rng = rand::thread_rng();
    let starting_rates = banner.starting_rates();
    let pity_pct = (status.pity_count / 5) as f64 * 0.005;

    let mut rates: [f64; 5] = std::array::from_fn(|i| starting_rates[i] as f64 / 100.0);
    let fivestar_total = rates[Pool::Focus as usize] + rates[Pool::Fivestar as usize];
    rates[Pool::Focus as usize] += pity_pct * rates[Pool::Focus as usize] / fivestar_total;
    rates[Pool::Fivestar as usize] += pity_pct * rates[Pool::Fivestar as usize] / fivestar_total;
    rates[Pool::FourstarFocus as usize] -=
        pity_pct * rates[Pool::FourstarFocus as usize] / (1.0 - fivestar_total);
    rates[Pool::FourstarSpecial as usize] -=
        pity_pct * rates[Pool::FourstarSpecial as usize] / (1.0 - fivestar_total);
    rates[Pool::Common as usize] -=
        pity_pct * rates[Pool::Common as usize] / (1.0 - fivestar_total);

    debug_assert!((rates.iter().sum::<f64>() - 1.0).abs() < 0.00000001);

    let pool_dist = WeightedIndex::new(&rates).unwrap();

    array::from_fn(|_| {
        let pool = pool_dist.sample(&mut rng);
        let pool = pool.try_into().unwrap();

        let color_dist = WeightedIndex::new(&banner.pool_sizes(pool)).unwrap();
        let color = color_dist.sample(&mut rng);
        let color = color.try_into().unwrap();

        (pool, color)
    })
}

#[cfg(test)]
mod test {
    use enumset::EnumSet;

    use crate::{banner::BannerType, goal::UnitGoal};

    use super::*;

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
        let results = sim_until_goal_many(&banner, goal, 10000);
        dbg!(results);
        //assert!(false);
    }
}
