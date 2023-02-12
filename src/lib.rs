#![allow(unused_imports)]

mod components;
mod geom;
mod model;
mod svg;
mod utils;

use geom::*;
use utils::*;

use gloo::timers::callback::Interval;
use rand::Rng;
use yew::prelude::*;

use nalgebra as na;

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
    camera: model::Camera,
    model: model::Model,

    _frame_update_handle: Interval,
}

pub enum Msg {
    Update(f32),
}

impl Component for App {
    type Message = Msg;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        log!("Creating app.");

        Self {
            camera: model::Camera::new(),
            model: model::Model::load(),
            _frame_update_handle: {
                let link = ctx.link().clone();
                let fps = 15;
                Interval::new(1000 / fps, move || {
                    link.send_message(Msg::Update(1.0 / fps as f32))
                })
            },
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Msg) -> bool {
        match msg {
            Self::Message::Update(dt) => {
                self.model.world_from_model *=
                    na::Rotation3::<f32>::from_euler_angles(0.2 * dt, 0.1 * dt, dt);
                true
            }
        }
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        let window_size = get_window_size().expect("Unable to get window size.");
        let viewbox = get_viewbox().unwrap();

        let style_string = format!("width:{}px;height:{}px", window_size[0], window_size[1]);
        let viewbox_string = format!(
            "{} {} {} {}",
            viewbox.start[0], viewbox.start[1], viewbox.dim[0], viewbox.dim[1]
        );

        let projected = self.model.project(&self.camera);

        html! {
            <div id="container" style={style_string}>
                <svg width="100%" height="100%" viewBox={viewbox_string} preserveAspectRatio="none">
                {
                    for projected.faces.iter().rev().map(|f| { html! {
                        <polygon points={format!("{:.3},{:.3} {:.3},{:.3} {:.3},{:.3}",
                                                 f.a.x, f.a.y, f.b.x, f.b.y, f.c.x, f.c.y)}
                                 fill="black" stroke="white" stroke-width=0.1/>
                    }})
                }
                </svg>
            </div>
        }
    }
}
