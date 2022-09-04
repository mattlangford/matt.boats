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

#[derive(Serialize, Deserialize, Default, Clone, Debug)]
pub struct RenderRequest {
    center_x: f64,
    center_y: f64,
    scale: f64,

    window_x: f64,
    window_y: f64,

    steps: usize,

    divergence_threshold: f64,
}
#[derive(Serialize, Deserialize)]
pub struct RenderResponse {
    data: Vec<u8>,
    hist: Vec<usize>,

    window_x: usize,
    window_y: usize,
}

pub struct Worker {
    prev_hist: Vec<usize>,
}

fn index(i: usize, w_x: usize) -> (usize, usize) {
    (i % w_x, i / w_x)
}
fn rindex(x: usize, y: usize, w_x: usize) -> usize {
    x * w_x + y
}

impl gloo::worker::Worker for Worker {
    type Message = ();
    type Input = RenderRequest;
    type Output = RenderResponse;

    fn create(_: &gloo::worker::WorkerScope<Self>) -> Self {
        log!("Creating worker.");
        Worker {
            prev_hist: Vec::new(),
        }
    }

    fn update(&mut self, _: &gloo::worker::WorkerScope<Self>, _: Self::Message) {}

    fn received(
        &mut self,
        scope: &gloo::worker::WorkerScope<Self>,
        state: Self::Input,
        id: gloo::worker::HandlerId,
    ) {
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
        let mut max: u16 = 0;
        let counts: Vec<u16> = (0..data_size)
            .map(offset)
            .map(|c| {
                let mut z = Complex::new(0.0, 0.0);
                let count = (0..state.steps)
                    .take_while(|_| {
                        let f = Complex::new(z.re.abs(), z.im.abs());
                        z = f * f + c;
                        (z.re * z.re + z.im * z.im) < state.divergence_threshold
                    })
                    .count() as u16;
                max = max.max(count);
                count
            })
            .collect();
        let mut hist = vec![0; max as usize + 1];
        for &c in &counts {
            hist[c as usize] += 1;
        }

        if self.prev_hist.len() == hist.len() {
            for (h, p) in hist.iter_mut().zip(self.prev_hist.iter()) {
                *h = (*h + p) / 2;
            }
        }

        let inv_scale_total = 255.0 / (hist.iter().sum::<usize>() as f64);
        let data: Vec<u8> = counts
            .iter()
            .enumerate()
            .map(|(i, &count)| {
                let integral = (0..=count as usize).map(|b| hist[b]).sum::<usize>() as f64;
                (inv_scale_total * integral).round() as u8
            })
            .collect();

        self.prev_hist = hist.clone();

        scope.respond(
            id,
            RenderResponse {
                data: data,
                hist: hist,
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
    image: Option<image::ImageBuffer<image::Luma<u8>, Vec<u8>>>,
    step_size: usize,
    resize_listener: Option<EventListener>,
    bridge: gloo::worker::WorkerBridge<Worker>,
}

pub enum Msg {
    Resize,
    Update(ControlState),
    SetImage(RenderResponse),
    Query((usize, usize)),
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

        let mut request = RenderRequest {
            center_x: -1.745,
            center_y: -0.038,
            scale: 0.1789,
            window_x: window_size[0],
            window_y: window_size[1],
            steps: 32,
            divergence_threshold: 10.0,
        };

        if window_size[1] > window_size[0] {
            request.center_x = -1.8608;
            request.center_y = -0.0035;
            request.scale = 0.0058;
        }
        let bridge = spawner.spawn("worker.js");
        bridge.send(request.clone());

        Self {
            request: request,
            image: None,
            step_size: 4,
            resize_listener: None,
            bridge: bridge,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Msg) -> bool {
        let r = &mut self.request;
        match msg {
            Self::Message::Resize => {
                let window_size = get_window_size()
                    .expect("Unable to get window size.")
                    .cast::<f64>();
                r.window_x = window_size[0];
                r.window_y = window_size[1];
                self.bridge.send(self.request.clone());

                self.step_size = 4;
                false
            }
            Self::Message::Update(msg) => {
                r.center_x = msg.x;
                r.center_y = msg.y;
                r.scale = msg.scale;

                self.step_size = 4;
                log!("{:?}", r);

                self.bridge.send(r.clone());
                false
            }
            Self::Message::SetImage(msg) => {
                let mut steps = r.steps;

                let threshold = median(msg.hist.clone()) / 10;
                let fill_count = msg
                    .hist
                    .iter()
                    .rev()
                    .take(msg.hist.len() / 2)
                    .filter(|&h| *h > threshold)
                    .count();

                if fill_count < 64 {
                    steps += self.step_size;
                }
                if fill_count > 128 {
                    steps -= self.step_size;
                }

                if steps != r.steps && steps > 32 && steps < 500 {
                    self.step_size = (self.step_size * 2).max(32);
                    r.steps = steps;
                    self.bridge.send(r.clone());
                }

                self.image = image::ImageBuffer::from_vec(
                    msg.window_x as u32,
                    msg.window_y as u32,
                    msg.data,
                );
                true
            }
            Self::Message::Query((x, y)) => {
                let px = self
                    .image
                    .as_ref()
                    .map(|img| img.get_pixel(x as u32, y as u32).0[0]);
                log!("x: {} y: {} px: {}", x, y, px.unwrap_or(0));
                false
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let window_size = get_window_size().expect("Unable to get window size.");
        let viewbox = get_viewbox().unwrap();

        let style_string = format!("width:{}px;height:{}px", window_size[0], window_size[1]);

        let data = self.image.as_ref().map(|image| {
            let mut buf = std::io::Cursor::new(Vec::new());
            image
                .write_to(&mut buf, image::ImageOutputFormat::Png)
                .unwrap();
            base64::encode(&buf.into_inner())
        });
        let link = ctx.link();

        html! {
            <div id="container" style={style_string}>
                if data.is_some() {
                    <img src={format!("data:image/png;base64,{}", data.unwrap())}
                         onclick={link.callback(|e: MouseEvent| {
                             Msg::Query((e.offset_x() as usize, e.offset_y() as usize))
                         })}
                    />
                }
                <ControlPanel
                    callback={link.callback(|s| Msg::Update(s))}
                    window={viewbox.dim}
                    x={self.request.center_x}
                    y={self.request.center_y}
                    scale={self.request.scale}
                />
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
