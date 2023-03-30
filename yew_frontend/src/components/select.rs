use std::marker::PhantomData;

use web_sys::HtmlSelectElement;
use yew::prelude::*;

#[derive(PartialEq, Properties)]
pub struct SelectProps<T: PartialEq> {
    pub onchange: Callback<usize>,
    pub values: Vec<T>,
    pub to_label: Callback<T, String>,
    pub current: usize,
    #[prop_or(false)]
    pub disabled: bool,
}

pub struct Select<T> {
    select_ref: NodeRef,
    _phantomdata: PhantomData<T>,
}

pub enum SelectMsg {
    Changed,
}

impl<T: 'static + PartialEq + Clone> Component for Select<T> {
    type Message = SelectMsg;
    type Properties = SelectProps<T>;

    fn create(_ctx: &Context<Self>) -> Self {
        Self {
            select_ref: NodeRef::default(),
            _phantomdata: PhantomData::default(),
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        html! {
            <select
                ref={self.select_ref.clone()}
                onchange={ctx.link().callback(|_| SelectMsg::Changed)}
                disabled={ctx.props().disabled}>
                { for ctx.props().values.iter().map(|s| html! {
                    <option>{ ctx.props().to_label.emit(s.clone()) }</option>
                }) }
            </select>
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            SelectMsg::Changed => {
                let idx = self
                    .select_ref
                    .cast::<HtmlSelectElement>()
                    .expect("SelectMsg::Changed")
                    .selected_index();
                if idx >= 0 {
                    ctx.props().onchange.emit(idx as usize);
                }
                false
            }
        }
    }

    fn changed(&mut self, ctx: &Context<Self>, old_props: &Self::Properties) -> bool {
        let mut rerender = false;

        if ctx.props().current != old_props.current {
            let selected_index = ctx.props().current;

            let select_el = self
                .select_ref
                .cast::<HtmlSelectElement>()
                .expect("select_el");
            select_el.set_selected_index(selected_index as i32);

            rerender |= true;
        }

        if ctx.props().disabled != old_props.disabled {
            rerender |= true;
        }

        rerender
    }

    fn rendered(&mut self, ctx: &Context<Self>, first_render: bool) {
        if first_render {
            let select_el = self
                .select_ref
                .cast::<HtmlSelectElement>()
                .expect("select_el");
            let selected_label = ctx
                .props()
                .to_label
                .emit(ctx.props().values[ctx.props().current].clone());
            select_el.set_value(&selected_label);
        }
    }
}
