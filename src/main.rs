#![allow(unused_imports)]

mod geom;
mod map;
mod svg;
mod utils;
use geom::*;
use map::*;
use utils::*;

use gloo::events::EventListener;
use gloo::timers::callback::Interval;
use nalgebra as na;
use wasm_bindgen::JsCast;
use yew::events::Event;
use yew::prelude::*;

use rand::seq::SliceRandom;
use rand::Rng;
use rand::SeedableRng;

use std::collections::HashSet;

fn get_window_size() -> Option<na::Vector2<f32>> {
    let window = web_sys::window().unwrap();
    let w_height = window.inner_height().ok().and_then(|v| v.as_f64());
    let w_width = window.inner_width().ok().and_then(|v| v.as_f64());
    if let (Some(h), Some(w)) = (w_height, w_width) {
        Some(Vec2f::new(w as f32, h as f32))
    } else {
        None
    }
}

const HEIGHT: f32 = 100.0;
fn get_viewbox_size() -> Option<Vec2f> {
    get_window_size().map(|s| Vec2f::new(HEIGHT * s[0] / s[1], HEIGHT))
}

fn get_viewbox() -> Option<AABox> {
    get_viewbox_size().map(|dim| AABox {
        start: -0.5 * dim,
        dim: dim,
    })
}

#[derive(Debug, Default, Clone)]
pub struct ControlState {
    x: f64,
    y: f64,
    scale: f64,

    dx: Option<f64>,
    dy: Option<f64>,
    dscale: Option<f64>,
}

#[derive(Properties, PartialEq)]
pub struct ControlPanelProps {
    pub callback: Callback<ControlState>,
    pub window: Vec2f,
}

struct ControlPanel {
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
struct ControlPanelMessage {
    shift: bool,
    action: ControlPanelAction,
}

impl Component for ControlPanel {
    type Message = ControlPanelMessage;
    type Properties = ControlPanelProps;

    fn create(ctx: &Context<Self>) -> Self {
        let props = ctx.props();
        let state = ControlState {
            scale: 5.0,
            ..ControlState::default()
        };
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
                self.state = ControlState {
                    scale: 5.0,
                    ..ControlState::default()
                };
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

        let link = ctx.link();
        let document = web_sys::window()
            .and_then(|w| w.document())
            .expect("Unable to load document.");
        let listener = EventListener::new(&document, "keydown", move |event| {
            onkeypress.emit(event.dyn_ref::<web_sys::KeyboardEvent>().unwrap().clone());
        });

        self.listener.replace(listener);
    }
}

struct App {
    scale: f64,
    center: Vec2d,
}

enum Msg {
    Update(ControlState),
}

impl Component for App {
    type Message = Msg;
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        Self {
            scale: 1.0,
            center: Vec2d::new(0.0, 0.0),
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Msg) -> bool {
        match msg {
            Self::Message::Update(msg) => {
                log!("{:?} msg", msg);
                self.scale = msg.scale as f64;
                self.center = Vec2d::new(msg.x, msg.y);
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let window_size = get_window_size().expect("Unable to get window size.");
        let viewbox = get_viewbox().unwrap();

        let style_string = format!("width:{}px;height:{}px", window_size[0], window_size[1]);

        let mut img = image::GrayImage::new(window_size[0] as u32, window_size[1] as u32);

        let ratio = (window_size[1] / window_size[0]) as f64;
        let offset = |x, y| {
            (
                self.scale * (x as f64 / window_size[0] as f64 - 0.5) + self.center[0],
                self.scale * ratio * (y as f64 / window_size[1] as f64 - 0.5) + self.center[1],
            )
        };

        let mut max = 0;
        let mut min = 255;
        for ((cx, cy), px) in img
            .enumerate_pixels_mut()
            .map(|(x, y, px)| (offset(x, y), px))
        {
            const MAX_STEPS: usize = 64;
            type Complex = na::Complex<f64>;
            let c = Complex::new(cx, cy);

            let mut z = Complex::new(0.0, 0.0);
            let steps = (0..MAX_STEPS)
                .take_while(|_| {
                    let f = Complex::new(z.re.abs(), z.im.abs());
                    z = f * f + c;
                    (z.re * z.re + z.im * z.im) < 10.0
                })
                .count();

            let v = 255.0 * (steps as f32) / (MAX_STEPS as f32);
            max = max.max(steps);
            min = min.min(steps);
            *px = image::Luma([v as u8]);
        }
        log!("min: {} max: {}", min, max);

        let mut buf = std::io::Cursor::new(Vec::new());
        img.write_to(&mut buf, image::ImageOutputFormat::Png)
            .unwrap();
        let res_base64 = base64::encode(&buf.into_inner());

        html! {
            <div id="container" style={style_string}>
                <img src={format!("data:image/png;base64,{}", res_base64)}/>
                <ControlPanel callback={ctx.link().callback(|s| Msg::Update(s))} window={viewbox.dim}/>
            </div>
        }
    }
}

fn main() {
    log!("Starting model...");
    yew::start_app::<App>();
}
