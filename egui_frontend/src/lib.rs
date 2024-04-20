mod app;
mod banner;
mod goal;
mod results;
pub use app::App;

use gloo_console::log;
use instant::Instant;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use summon_simulator::{
    banner::GenericBanner, frequency_counter::FrequencyCounter, goal::Goal, sim,
};

#[derive(Debug)]
pub enum SimWorkerMessage {
    Continue,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum SimWorkerInput {
    Run {
        banner: GenericBanner,
        goal: Goal,
        target_interval: Duration,
    },
    Stop,
}

pub struct SimWorker {
    sim: Option<sim::Sim>,
    target_interval: Option<Duration>,
    id: Option<gloo_worker::HandlerId>,
    num_iters: u32,
    running: bool,
}

impl gloo_worker::Worker for SimWorker {
    type Message = SimWorkerMessage;

    type Input = SimWorkerInput;

    type Output = FrequencyCounter;

    fn create(scope: &gloo_worker::WorkerScope<Self>) -> Self {
        let _scope = scope;
        Self {
            sim: None,
            target_interval: None,
            num_iters: 0,
            id: None,
            running: false,
        }
    }

    fn update(&mut self, scope: &gloo_worker::WorkerScope<Self>, msg: Self::Message) {
        match msg {
            SimWorkerMessage::Continue => {
                if !self.running {
                    return;
                }
                match (&mut self.sim, self.target_interval, self.id) {
                    (Some(sim), Some(interval), Some(id)) => {
                        log!("Performing ", self.num_iters as f64, " iterations.");
                        let start = Instant::now();
                        sim.sim(self.num_iters);
                        let duration = Instant::now().duration_since(start);
                        log!("Simulation took ", duration.as_secs_f64() * 1000.0, " ms.");
                        // Don't send back data if the simulation finished too quickly, since
                        // there will be much less data in this set than there will be in the
                        // following ones.
                        if duration.as_secs_f64() * 2.0 > interval.as_secs_f64() {
                            scope.respond(id, sim.data().clone());
                        }
                        // Update number of iterations to aim for the requested result interval
                        let new_iters = (self.num_iters as f64 * interval.as_secs_f64()
                            / duration.as_secs_f64())
                        .round() as u32;
                        // ...but don't ramp up more than 10x each time, to avoid a very fast
                        // first iteration resulting in a very imprecise second
                        self.num_iters = new_iters.min(self.num_iters.saturating_mul(10));
                    }
                    _ => panic!("Received Continue message without parameters"),
                }

                // If I send this message directly instead of putting it in an immediate callback,
                // external messages like Stop seem to be choked out and never processed.
                let scope = scope.clone();
                gloo_timers::callback::Timeout::new(0, move || {
                    scope.send_message(SimWorkerMessage::Continue);
                })
                .forget();
            }
        }
    }

    fn received(
        &mut self,
        scope: &gloo_worker::WorkerScope<Self>,
        msg: Self::Input,
        id: gloo_worker::HandlerId,
    ) {
        match msg {
            SimWorkerInput::Run {
                banner,
                goal,
                target_interval: interval,
            } => {
                self.sim = Some(sim::Sim::new(banner, goal));
                self.num_iters = 100;
                self.target_interval = Some(interval);
                self.id = Some(id);
                self.running = true;
                scope.send_message(SimWorkerMessage::Continue);
            }
            SimWorkerInput::Stop => self.running = false,
        }
    }
}
