use summon_simulator::{
    banner::{GenericBanner, StandardBanner},
    types::Color,
};
use sycamore::prelude::*;

pub struct UiBanner<'a> {
    pub name: String,
    pub starting_rates: &'a Signal<(u8, u8)>,
    pub units: &'a Signal<Vec<RcSignal<UiBannerUnit>>>,
}

#[derive(PartialEq, Eq, Clone)]
pub struct UiBannerUnit {
    pub name: RcSignal<String>,
    pub color: RcSignal<Color>,
    pub fourstar_focus: RcSignal<bool>,
}

impl<'a> UiBanner<'a> {
    pub fn to_generic_banner(&self) -> Option<GenericBanner> {
        let sample_banner = StandardBanner::Standard {
            focus: [1, 1, 1, 1],
        }
        .as_generic_banner(false);

        Some(sample_banner)
    }
}

#[component(inline_props)]
pub fn BannerSelector<'a, G: Html>(cx: Scope<'a>, banner: &'a Signal<UiBanner<'a>>) -> View<G> {
    let onclick = move |_| {
        let count = banner.get().units.get().len();
        banner
            .get()
            .units
            .modify()
            .push(create_rc_signal(UiBannerUnit {
                name: create_rc_signal(format!("Unit {}", count + 1)),
                color: create_rc_signal(Color::Red),
                fourstar_focus: create_rc_signal(false),
            }))
    };

    view! { cx,
        p { (banner.get().name) }
        p { (format!("({}, {})", banner.get().starting_rates.get().0, banner.get().starting_rates.get().1)) }
        Indexed(
            iterable=banner.get().units,
            view=move |cx, unit| view! { cx,
                BannerUnitSelector(unit=unit)
            }
        )
        button (on:click=onclick) { "+" }
    }
}

#[component(inline_props)]
pub fn BannerUnitSelector<'a, G: Html>(cx: Scope<'a>, unit: RcSignal<UiBannerUnit>) -> View<G> {
    let name = unit.get().name.clone();
    let color = unit.get().color.clone();
    let fourstar_focus = unit.get().fourstar_focus.clone();

    view! { cx,
        div {
            p { (name) }
            p { (format!("{:?}", *color.get())) }
            p { (fourstar_focus) }
        }
    }
}
