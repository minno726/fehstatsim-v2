use instant::{Duration, Instant};
use serde::{Deserialize, Serialize};
use summon_simulator::{
    banner::GenericBanner, frequency_counter::FrequencyCounter, goal::Goal, sim,
};
use yew_agent::{HandlerId, Public, WorkerLink};

pub struct SimWorker {
    link: WorkerLink<Self>,
    running: bool,
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

pub enum SimWorkerMessage {
    Continue {
        sim: sim::Sim,
        interval: Duration,
        id: yew_agent::HandlerId,
        num_iters: u32,
        first_run: bool,
    },
}

impl yew_agent::Worker for SimWorker {
    type Input = SimWorkerInput;
    type Message = SimWorkerMessage;
    type Output = FrequencyCounter;
    type Reach = Public<Self>;

    fn create(link: WorkerLink<Self>) -> Self {
        Self {
            link,
            running: false,
        }
    }

    fn update(&mut self, msg: Self::Message) {
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
                gloo_console::log!(format!("Performing {} iterations.", num_iters as f64));
                let start = Instant::now();
                sim.sim(num_iters);
                let duration = Instant::now().duration_since(start);
                gloo_console::log!(format!(
                    "Simulation took {} ms.",
                    duration.as_secs_f64() * 1000.0
                ));

                if !first_run {
                    self.link.respond(id, sim.data().clone());
                }
                // Update number of iterations to aim for the requested result interval
                num_iters = (num_iters as f64 * interval.as_secs_f64() / duration.as_secs_f64())
                    .round() as u32;

                // If I send this message directly instead of putting it in an immediate callback,
                // a RefCell inside of the WorkerLink will still be borrowed.
                let link = self.link.clone();
                gloo_timers::callback::Timeout::new(0, move || {
                    link.send_message(SimWorkerMessage::Continue {
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

    fn handle_input(&mut self, msg: Self::Input, id: HandlerId) {
        match msg {
            SimWorkerInput::Run {
                banner,
                goal,
                target_interval: interval,
            } => {
                let sim = sim::Sim::new(banner, goal);
                self.running = true;
                let link = self.link.clone();
                gloo_timers::callback::Timeout::new(0, move || {
                    link.send_message(SimWorkerMessage::Continue {
                        sim,
                        interval,
                        id,
                        num_iters: 1000,
                        first_run: true,
                    });
                })
                .forget();
            }
            SimWorkerInput::Stop => self.running = false,
        }
    }

    fn name_of_resource() -> &'static str {
        "worker.js"
    }
}
