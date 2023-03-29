use std::rc::Rc;

use instant::Duration;
use summon_simulator::{banner::GenericBanner, frequency_counter::FrequencyCounter, goal::Goal};
use yew::prelude::*;
use yew_agent::{Bridge, Bridged};

use crate::{
    agent::{SimWorker, SimWorkerInput},
    banner::{BannerSelect, UiBanner, UiGoal},
    results::Results,
};

pub struct App {
    worker: Box<dyn Bridge<SimWorker>>,
    data: Option<FrequencyCounter>,
    banner: Option<GenericBanner>,
    goal: Option<Goal>,
    is_running: bool,
}

pub enum AppMsg {
    RunClicked,
    BannerChanged(UiBanner, UiGoal),
    DataReceived(FrequencyCounter),
}

impl Component for App {
    type Message = AppMsg;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        let cb = {
            let link = ctx.link().clone();
            move |e| link.send_message(Self::Message::DataReceived(e))
        };

        Self {
            worker: SimWorker::bridge(Rc::new(cb)),
            data: None,
            banner: None,
            goal: None,
            is_running: false,
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let can_run = if let (Some(banner), Some(_)) = (&self.banner, &self.goal) {
            banner.is_valid()
        } else {
            false
        };
        html! {
            <>
            <BannerSelect on_banner_changed={ctx.link().callback(|(banner, goal)| AppMsg::BannerChanged(banner, goal))} />
            <button disabled={!can_run} onclick={ctx.link().callback(|_| AppMsg::RunClicked)}>{
                if self.is_running {
                    "Stop"
                } else {
                    "Run"
                }
            }
            </button>
            <p>{ format!("{:?}", &self.banner) }</p>
            <p>{ format!("{:?}", &self.goal) }</p>
            <Results data={self.data.clone()} />
            </>
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            AppMsg::DataReceived(data) => {
                if self.is_running {
                    self.data = Some(data);
                    true
                } else {
                    false
                }
            }
            AppMsg::RunClicked => {
                if !self.is_running {
                    self.start_sim();
                } else {
                    self.stop_sim();
                }
                true
            }
            AppMsg::BannerChanged(banner, goal) => {
                let new_banner = banner.to_sim_banner();
                let new_goal = goal.to_sim_goal();
                if new_banner != self.banner || new_goal != self.goal {
                    self.banner = new_banner;
                    self.goal = new_goal;
                    self.data = None;
                    self.stop_sim();
                    true
                } else {
                    false
                }
            }
        }
    }
}

impl App {
    fn start_sim(&mut self) {
        if let (Some(banner), Some(goal)) = (&self.banner, &self.goal) {
            self.worker.send(SimWorkerInput::Run {
                banner: banner.clone(),
                goal: goal.clone(),
                target_interval: Duration::from_millis(500),
            });
            self.is_running = true;
        }
    }

    fn stop_sim(&mut self) {
        self.worker.send(SimWorkerInput::Stop);
        self.is_running = false;
    }
}
