use gloo_console::log;
use instant::{Duration, Instant};
use serde::{Deserialize, Serialize};
use summon_simulator::{
    banner::GenericBanner, frequency_counter::FrequencyCounter, goal::Goal, sim,
};

pub enum SimWorkerMessage {
    Continue {
        sim: sim::Sim,
        interval: Duration,
        id: gloo_worker::HandlerId,
        num_iters: u32,
        first_run: bool,
    },
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
    running: bool,
}

impl gloo_worker::Worker for SimWorker {
    type Message = SimWorkerMessage;

    type Input = SimWorkerInput;

    type Output = FrequencyCounter;

    fn create(scope: &gloo_worker::WorkerScope<Self>) -> Self {
        let _scope = scope;
        Self { running: false }
    }

    fn update(&mut self, scope: &gloo_worker::WorkerScope<Self>, msg: Self::Message) {
        let _scope = scope;
        match msg {
            SimWorkerMessage::Continue {
                mut sim,
                interval,
                id,
                mut num_iters,
                first_run,
            } => {
                if !self.running {
                    return;
                }
                log!(format!("Performing {} iterations.", num_iters as f64));
                let start = Instant::now();
                sim.sim(num_iters);
                let duration = Instant::now().duration_since(start);
                log!(format!(
                    "Simulation took {} ms.",
                    duration.as_secs_f64() * 1000.0
                ));

                if !first_run {
                    scope.respond(id, sim.data().clone());
                }
                // Update number of iterations to aim for the requested result interval
                num_iters = (num_iters as f64 * interval.as_secs_f64() / duration.as_secs_f64())
                    .round() as u32;

                // If I send this message directly instead of putting it in an immediate callback,
                // external messages like Stop seem to be choked out and never processed.
                let scope = scope.clone();
                gloo_timers::callback::Timeout::new(0, move || {
                    scope.send_message(SimWorkerMessage::Continue {
                        sim,
                        interval,
                        id,
                        num_iters,
                        first_run: false,
                    });
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
                let sim = sim::Sim::new(banner, goal);
                self.running = true;
                scope.send_message(SimWorkerMessage::Continue {
                    sim,
                    interval,
                    id,
                    num_iters: 1000,
                    first_run: true,
                });
            }
            SimWorkerInput::Stop => self.running = false,
        }
    }
}
