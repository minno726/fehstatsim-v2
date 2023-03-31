use std::sync::atomic::{AtomicU64, Ordering};

use web_sys::HtmlInputElement;
use yew::prelude::*;

static NEXT_ID: AtomicU64 = AtomicU64::new(0);

pub struct Toggle {
    id: AttrValue,
    checkbox_ref: NodeRef,
}

#[derive(PartialEq, Properties)]
pub struct ToggleProps {
    pub onchange: Callback<bool>,
    pub label: AttrValue,
    #[prop_or(false)]
    pub value: bool,
}

pub enum ToggleMsg {
    Changed,
}

impl Component for Toggle {
    type Message = ToggleMsg;
    type Properties = ToggleProps;

    fn create(_ctx: &Context<Self>) -> Self {
        Self {
            id: AttrValue::from(format!(
                "__Checkbox_{}",
                NEXT_ID.fetch_add(1, Ordering::SeqCst)
            )),
            checkbox_ref: NodeRef::default(),
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        html! {
            <>
            <input id={self.id.clone()} type="checkbox" ref={self.checkbox_ref.clone()} onchange={ctx.link().callback(|_| ToggleMsg::Changed)} />
            <label for={self.id.clone()}>{ ctx.props().label.clone() }</label>
            </>
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        gloo_console::log!(self
            .checkbox_ref
            .cast::<HtmlInputElement>()
            .expect("Checkbox::update")
            .checked());
        match msg {
            ToggleMsg::Changed => {
                ctx.props().onchange.emit(
                    self.checkbox_ref
                        .cast::<HtmlInputElement>()
                        .expect("Checkbox::update")
                        .checked(),
                );
                false
            }
        }
    }

    fn changed(&mut self, ctx: &Context<Self>, old_props: &Self::Properties) -> bool {
        let mut rerender = false;

        if old_props.value != ctx.props().value {
            self.checkbox_ref
                .cast::<HtmlInputElement>()
                .expect("Checkbox::changed")
                .set_checked(ctx.props().value);
            rerender |= true;
        }

        rerender
    }

    fn rendered(&mut self, ctx: &Context<Self>, first_render: bool) {
        if first_render {
            self.checkbox_ref
                .cast::<HtmlInputElement>()
                .expect("Checkbox::changed")
                .set_checked(ctx.props().value);
        }
    }
}
