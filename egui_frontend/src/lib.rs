mod app;
pub use app::App;

use summon_simulator::{
    banner::GenericBanner, frequency_counter::FrequencyCounter, goal::Goal, sim::sim,
};

pub struct SimWorker {}

impl gloo_worker::Worker for SimWorker {
    type Message = ();

    type Input = (GenericBanner, Goal, u32);

    type Output = (FrequencyCounter,);

    fn create(scope: &gloo_worker::WorkerScope<Self>) -> Self {
        let _scope = scope;
        Self {}
    }

    fn update(&mut self, scope: &gloo_worker::WorkerScope<Self>, msg: Self::Message) {
        let _scope = scope;
        let _msg = msg;
    }

    fn received(
        &mut self,
        scope: &gloo_worker::WorkerScope<Self>,
        msg: Self::Input,
        id: gloo_worker::HandlerId,
    ) {
        let (banner, goal, iters) = msg;
        let result = sim(&banner, &goal, iters);
        scope.respond(id, (result,));
    }
}
