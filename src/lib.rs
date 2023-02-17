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

struct MouseDrag {
    start: (i32, i32),
    current: (i32, i32),
}

pub struct App {
    camera: model::Camera,
    model: model::Model,

    drag: Option<MouseDrag>,
    momentum: geom::Vec2f,

    _frame_update_handle: Interval,
}

pub enum Msg {
    MouseDown((i32, i32)),
    MouseMove((i32, i32)),
    MouseUp,
    Scroll(f32),
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
            drag: None,
            momentum: geom::Vec2f::new(0.0, 0.0),
            _frame_update_handle: {
                let link = ctx.link().clone();
                let fps = 24;
                Interval::new(1000 / fps, move || {
                    link.send_message(Msg::Update(1.0 / fps as f32))
                })
            },
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Msg) -> bool {
        match msg {
            Self::Message::Update(dt) => {
                if let Some(drag) = &mut self.drag {
                    let dx = drag.current.0 - drag.start.0;
                    let dy = drag.current.1 - drag.start.1;

                    self.momentum += geom::Vec2f::new(dx as f32, dy as f32);

                    drag.start = drag.current;
                }

                self.momentum *= 0.0375 / dt;

                if self.momentum.norm() > 1.0 {
                    let pitch = 1E-3 * self.momentum.y;
                    let yaw = -1E-3 * self.momentum.x;
                    self.model
                        .rotate(na::Rotation3::<f32>::from_euler_angles(0.0, pitch, yaw));

                    log!("rpy: {:?}", self.model.rotation().euler_angles());
                }
                return true;
            }
            Self::Message::MouseDown((x, y)) => {
                self.drag = Some(MouseDrag {
                    start: (x, y),
                    current: (x, y),
                });
            }
            Self::Message::MouseMove((x, y)) => {
                if let Some(drag) = &mut self.drag {
                    drag.current = (x, y);
                }
            }
            Self::Message::Scroll(s) => {
                self.camera.world_from_camera *= na::Translation3::<f32>::from(-1E-2 * s * geom::Vec3f::z());
            }
            Self::Message::MouseUp => {
                self.drag = None;
            }
        }
        return false;
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let window_size = get_window_size().expect("Unable to get window size.");
        let viewbox = get_viewbox().unwrap();

        let style_string = format!("width:{}px;height:{}px", window_size[0], window_size[1]);
        let viewbox_string = format!(
            "{} {} {} {}",
            viewbox.start[0], viewbox.start[1], viewbox.dim[0], viewbox.dim[1]
        );

        let projected = self.model.project(&self.camera);

        let onmousedown = ctx.link().callback(|event: MouseEvent| {
            if event.which() == 3 { return Self::Message::MouseUp; } // right click
            Self::Message::MouseDown((event.client_x(), event.client_y()))
        });
        let onmousemove = ctx.link().callback(|event: MouseEvent| {
            Self::Message::MouseMove((event.client_x(), event.client_y()))
        });
        let onmouseup = ctx.link().callback(|_: MouseEvent| Self::Message::MouseUp);
        let onmouseout = ctx.link().batch_callback(move |event: MouseEvent| {
            if event.client_x() < 0
                || event.client_y() < 0
                || event.client_x() >= window_size.x as i32
                || event.client_y() >= window_size.y as i32
            {
                return Some(Self::Message::MouseUp);
            }
            None
        });

        let onwheel = ctx.link().callback(|event: WheelEvent| {
            Self::Message::Scroll(event.delta_y() as f32)
        });

        html! {
            <div id="container" style={style_string} {onmousedown} {onmousemove} {onmouseup} {onmouseout} {onwheel}>
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
