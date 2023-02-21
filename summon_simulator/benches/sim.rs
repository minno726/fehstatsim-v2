use criterion::{criterion_group, criterion_main, Criterion};
use enumset::EnumSet;
use summon_simulator::{
    banner::StandardBanner,
    goal::{UnitCountGoal, UnitGoal},
    sim::sim_until_goal_many,
    types::{Color, Pool},
};

fn sim_benchmark(c: &mut Criterion) {
    let banner = StandardBanner::Standard {
        focus: [1, 1, 1, 1],
    }
    .as_generic_banner(false);
    let goal = UnitCountGoal::new(
        vec![UnitGoal {
            color: Color::Red,
            copies: 1,
            pools: EnumSet::from(Pool::Focus),
        }],
        true,
    );
    c.bench_function("standard_one_red_focus_10kx", |b| {
        b.iter(|| sim_until_goal_many(&banner, &goal, 10000))
    });
    // To test competitiveness with the old version, which completes 200k iterations in 1-2s on WASM
    // c.bench_function("standard_one_red_focus_200kx", |b| {
    //     b.iter(|| sim_until_goal_many(&banner, &goal, 200000))
    // });
}

criterion_group!(benches, sim_benchmark);
criterion_main!(benches);
