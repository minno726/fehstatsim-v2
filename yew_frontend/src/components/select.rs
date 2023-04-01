use web_sys::HtmlSelectElement;
use yew::prelude::*;

#[derive(PartialEq, Properties)]
pub struct SelectProps {
    pub onchange: Callback<usize>,
    pub labels: Vec<String>,
    pub value: usize,
    #[prop_or(false)]
    pub disabled: bool,
}

pub struct Select {
    select_ref: NodeRef,
}

pub enum SelectMsg {
    Changed,
}

impl Component for Select {
    type Message = SelectMsg;
    type Properties = SelectProps;

    fn create(_ctx: &Context<Self>) -> Self {
        Self {
            select_ref: NodeRef::default(),
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        html! {
            <select
                ref={self.select_ref.clone()}
                onchange={ctx.link().callback(|_| SelectMsg::Changed)}
                disabled={ctx.props().disabled}>
                { for ctx.props().labels.iter().map(|s| html! {
                    <option>{ s }</option>
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

        if ctx.props().value != old_props.value {
            let selected_index = ctx.props().value;

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
            select_el.set_selected_index(ctx.props().value as i32);
        }
    }
}
