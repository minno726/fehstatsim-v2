use criterion::{criterion_group, criterion_main, Criterion};
use enumset::EnumSet;
use summon_simulator::{
    banner::BannerType,
    goal::{UnitCountGoal, UnitGoal},
    sim::{sim_until_goal_many},
    types::{Color, Pool},
};

fn sim_benchmark(c: &mut Criterion) {
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
    c.bench_function("standard_one_red_focus_1000x", |b| {
        b.iter(|| sim_until_goal_many(&banner, goal.clone(), 1000))
    });
}

criterion_group!(benches, sim_benchmark);
criterion_main!(benches);
