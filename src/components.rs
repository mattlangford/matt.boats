use gloo::events::EventListener;
use wasm_bindgen::JsCast;
use yew::prelude::*;

use crate::geom::Vec2f;

#[derive(Debug, Clone)]
pub struct ControlState {
    pub x: f64,
    pub y: f64,
    pub scale: f64,

    pub dx: Option<f64>,
    pub dy: Option<f64>,
    pub dscale: Option<f64>,
}
impl Default for ControlState {
    fn default() -> Self {
        Self {
            x: -1.745,
            y: -0.038,
            scale: 0.1789,
            dx: None,
            dy: None,
            dscale: None,
        }
    }
}

#[derive(Properties, PartialEq)]
pub struct ControlPanelProps {
    pub callback: Callback<ControlState>,
    pub window: Vec2f,
}

pub struct ControlPanel {
    state: ControlState,
    listener: Option<EventListener>,
}

enum ControlPanelAction {
    IncX,
    DecX,
    IncY,
    DecY,
    IncScale,
    DecScale,
    Reset,
}
pub struct ControlPanelMessage {
    shift: bool,
    action: ControlPanelAction,
}

impl Component for ControlPanel {
    type Message = ControlPanelMessage;
    type Properties = ControlPanelProps;

    fn create(ctx: &Context<Self>) -> Self {
        let props = ctx.props();
        let state = ControlState::default();
        ctx.props().callback.emit(state.clone());
        Self {
            state: state,
            listener: None,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        self.state.dx = None;
        self.state.dy = None;
        self.state.dscale = None;

        let scale = self.state.scale * if msg.shift { 0.1 } else { 1.0 };
        let dx = scale * 0.25;
        let dy = scale * 0.25;
        let dscale = scale * 0.25;

        type Action = ControlPanelAction;
        match msg.action {
            Action::IncX => {
                self.state.x += dx;
                self.state.dx = Some(dx);
            }
            Action::DecX => {
                self.state.x -= dx;
                self.state.dx = Some(-dx);
            }
            Action::IncY => {
                self.state.y -= dy;
                self.state.dy = Some(dy);
            }
            Action::DecY => {
                self.state.y += dy;
                self.state.dy = Some(-dy);
            }
            Action::IncScale => {
                self.state.scale -= dscale;
                self.state.dscale = Some(dscale);
            }
            Action::DecScale => {
                self.state.scale += dscale;
                self.state.dscale = Some(-dscale);
            }
            Action::Reset => {
                self.state = ControlState::default();
            }
        }
        self.state.scale = self.state.scale.abs().max(f64::EPSILON);
        ctx.props().callback.emit(self.state.clone());
        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let link = ctx.link();
        let props = ctx.props();

        type Action = ControlPanelAction;
        let callback = move |e: web_sys::MouseEvent, action| Self::Message {
            shift: e.shift_key(),
            action: action,
        };

        html! {
            <div class="control-panel">
                <button type="button"
                        id="control-panel-reset"
                        onclick={link.callback(move |e| callback(e, Action::Reset))}>{"○"}</button>
                <button type="button"
                        id="control-panel-inc-x"
                        onclick={link.callback(move |e| callback(e, Action::IncX))}>{"x+"}</button>
                <button type="button"
                        id="control-panel-inc-y"
                        onclick={link.callback(move |e| callback(e, Action::IncY))}>{"y+"}</button>
                <button type="button"
                        id="control-panel-inc-scale"
                        onclick={link.callback(move |e| callback(e, Action::IncScale))}>{"z+"}</button>
                <button type="button"
                        id="control-panel-dec-x"
                        onclick={link.callback(move |e| callback(e, Action::DecX))}>{"x-"}</button>
                <button type="button"
                        id="control-panel-dec-y"
                        onclick={link.callback(move |e| callback(e, Action::DecY))}>{"y-"}</button>
                <button type="button"
                        id="control-panel-dec-scale"
                        onclick={link.callback(move |e| callback(e, Action::DecScale))}>{"z-"}</button>

            </div>
        }
    }

    fn rendered(&mut self, ctx: &Context<Self>, first_render: bool) {
        if !first_render {
            return;
        }

        let onkeypress = ctx.link().batch_callback(|event: KeyboardEvent| {
            type Action = ControlPanelAction;
            let msg = |action| {
                Some(Self::Message {
                    shift: event.shift_key(),
                    action: action,
                })
            };

            match &*event.key() {
                "ArrowRight" => msg(Action::IncX),
                "ArrowLeft" => msg(Action::DecX),
                "ArrowUp" => msg(Action::IncY),
                "ArrowDown" => msg(Action::DecY),
                "x" => msg(Action::IncScale),
                "z" => msg(Action::DecScale),
                _ => None,
            }
        });

        let document = web_sys::window()
            .and_then(|w| w.document())
            .expect("Unable to load document.");
        let listener = EventListener::new(&document, "keydown", move |event| {
            onkeypress.emit(event.dyn_ref::<web_sys::KeyboardEvent>().unwrap().clone());
        });

        self.listener.replace(listener);
    }
}
