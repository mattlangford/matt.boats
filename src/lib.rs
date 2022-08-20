#![allow(unused_imports)]

mod components;
mod geom;
mod map;
mod svg;
mod utils;

use components::*;
use geom::*;
use map::*;
use utils::*;

use gloo::events::EventListener;
use wasm_bindgen::JsCast;
use yew::prelude::*;

use nalgebra as na;

use rand::seq::SliceRandom;
use rand::Rng;
use rand::SeedableRng;

use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(Serialize, Deserialize, Default, Clone)]
pub struct RenderRequest {
    center_x: f64,
    center_y: f64,
    scale: f64,

    window_x: f64,
    window_y: f64,

    steps: u8,

    divergence_threshold: f64,
}
#[derive(Serialize, Deserialize)]
pub struct RenderResponse {
    data: Vec<u16>,
    min: u16,
    max: u16,

    window_x: usize,
    window_y: usize,
}

pub struct Worker {}

// (3, 2) => (1, 1)
fn index(i: usize, w_x: usize) -> (usize, usize) {
    (i % w_x, i / w_x)
}
// (1, 1, 2) => (3)
fn rindex(x: usize, y: usize, w_x: usize) -> usize {
    x * w_x + y
}

impl gloo::worker::Worker for Worker {
    type Message = ();
    type Input = RenderRequest;
    type Output = RenderResponse;

    fn create(_: &gloo::worker::WorkerScope<Self>) -> Self {
        log!("Creating worker.");
        Worker {}
    }

    fn update(&mut self, _: &gloo::worker::WorkerScope<Self>, _: Self::Message) {
        log!("Update");
    }

    fn received(
        &mut self,
        scope: &gloo::worker::WorkerScope<Self>,
        state: Self::Input,
        id: gloo::worker::HandlerId,
    ) {
        let data: Vec<u16> = vec![0; state.window_x as usize * state.window_y as usize];
        type Complex = na::Complex<f64>;

        let ratio = state.window_y / state.window_x;
        let offset = |i: usize| {
            let (x, y) = index(i, state.window_x as usize);
            Complex::new(
                state.scale * (x as f64 / state.window_x as f64 - 0.5) + state.center_x,
                state.scale * ratio * (y as f64 / state.window_y as f64 - 0.5) + state.center_y,
            )
        };

        let data_size = state.window_x as usize * state.window_y as usize;

        let mut max = u16::MIN;
        let mut min = u16::MAX;

        let data = (0..data_size).map(offset).map(|c| {
            let mut z = Complex::new(0.0, 0.0);
            let count = (0..state.steps)
                .take_while(|_| {
                    let f = Complex::new(z.re.abs(), z.im.abs());
                    z = f * f + c;
                    (z.re * z.re + z.im * z.im) < state.divergence_threshold
                })
                .count() as u16;

            max = max.max(count);
            min = min.max(count);
            count
        });

        scope.respond(
            id,
            RenderResponse {
                data: data.collect(),
                min: min,
                max: max,
                window_x: state.window_x as usize,
                window_y: state.window_y as usize,
            },
        );
    }
}

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

pub struct App {
    request: RenderRequest,
    data: String,

    resize_listener: Option<EventListener>,
    bridge: gloo::worker::WorkerBridge<Worker>,
}

pub enum Msg {
    Resize,
    Update(ControlState),
    SetImage(RenderResponse),
}

impl Component for App {
    type Message = Msg;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        log!("Creating app.");

        let mut spawner = <Worker as gloo::worker::Spawnable>::spawner();
        let link = ctx.link().clone();
        spawner.callback(move |resp| link.send_message(Self::Message::SetImage(resp)));

        let window_size = get_window_size()
            .expect("Unable to get window size.")
            .cast::<f64>();
        let request = RenderRequest {
            center_x: -1.745,
            center_y: -0.038,
            scale: 0.1789,
            window_x: window_size[0],
            window_y: window_size[1],
            steps: 8,
            divergence_threshold: 10.0,
        };

        Self {
            request: request,
            data: String::new(),
            resize_listener: None,
            bridge: spawner.spawn("worker.js"),
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Msg) -> bool {
        log!("Update app.");

        match msg {
            Self::Message::Resize => {
                let window_size = get_window_size()
                    .expect("Unable to get window size.")
                    .cast::<f64>();
                self.request.window_x = window_size[0];
                self.request.window_y = window_size[1];
                self.request.steps = 16;
                self.bridge.send(self.request.clone());
                false
            }
            Self::Message::Update(msg) => {
                self.request.center_x = msg.x;
                self.request.center_y = msg.y;
                self.request.scale = msg.scale;
                self.request.steps = 16;
                self.bridge.send(self.request.clone());
                false
            }
            Self::Message::SetImage(msg) => {
                if self.request.steps < 64 {
                    self.request.steps = self.request.steps.saturating_mul(2);
                    self.bridge.send(self.request.clone());
                }

                let range = (msg.max - msg.min) as f32;
                let mut img =
                    image::GrayImage::from_fn(msg.window_x as u32, msg.window_y as u32, |y, x| {
                        let count = msg.data[rindex(x as usize, y as usize, msg.window_x as usize)];
                        let scaled = 255.0 * ((count - msg.min) as f32 / range);
                        image::Luma([scaled as u8])
                    });

                let mut buf = std::io::Cursor::new(Vec::new());
                img.write_to(&mut buf, image::ImageOutputFormat::Png)
                    .unwrap();
                self.data = base64::encode(&buf.into_inner());
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let window_size = get_window_size().expect("Unable to get window size.");
        let viewbox = get_viewbox().unwrap();

        let style_string = format!("width:{}px;height:{}px", window_size[0], window_size[1]);

        html! {
            <div id="container" style={style_string}>
                if !self.data.is_empty() {
                    <img src={format!("data:image/png;base64,{}", self.data)}/>
                }
                <ControlPanel callback={ctx.link().callback(|s| Msg::Update(s))} window={viewbox.dim}/>
            </div>
        }
    }

    fn rendered(&mut self, ctx: &Context<Self>, first_render: bool) {
        if !first_render {
            return;
        }

        let link = ctx.link();
        let window = web_sys::window().unwrap();
        let resize = ctx.link().callback(|_| Self::Message::Resize);
        let listener = EventListener::new(&window, "resize", move |event| {
            resize.emit(());
        });
        self.resize_listener.replace(listener);
    }
}
