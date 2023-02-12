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

struct Circle {
    circle: geom::Circle,
    depth: f32,
    rgb: [u8; 3],
}

impl Circle {
    const MAX_RADIUS: f32 = 5.0;

    fn new_depth() -> f32 {
        let mut rng = rand::thread_rng();
        // Zero is close, One is far
        let max_range = 0.8;
        let min_range = 0.1;
        rng.gen::<f32>() * (max_range - min_range) + min_range
    }

    fn new(center: geom::Vec2f) -> Circle {
        let mut rng = rand::thread_rng();
        let depth = Circle::new_depth();
        Circle {
            circle: geom::Circle::new(center, (1.0 - depth) * Self::MAX_RADIUS),
            depth: depth,
            rgb: rng.gen(),
        }
    }
    fn new_outside(viewport: &geom::AABox) -> Circle {
        let mut rng = rand::thread_rng();
        let depth = Circle::new_depth();
        let radius = (1.0 - depth) * Self::MAX_RADIUS;
        let x = viewport.bottom_right().x + radius;
        let y = HEIGHT * 2.0 * (rng.gen::<f32>() - 0.5);

        Circle {
            circle: geom::Circle::new(Vec2f::new(x, y), radius),
            depth: depth,
            rgb: rng.gen(),
        }
    }

    fn to_props(&self) -> svg::CircleProps {
        let grey = 1.0 - 0.8 * self.depth;
        let rgb = [
            (255.0 * grey) as u8,
            (255.0 * grey) as u8,
            (255.0 * grey) as u8,
        ];
        svg::CircleProps::from_circle(&self.circle).with_fill(rgb)
    }

    fn update(&mut self, dt: f32) {
        let mut rng = rand::thread_rng();
        let max_rate = 40.0;
        let min_rate = 30.0;
        let velocity = (1.0 - self.depth) * (max_rate - min_rate) + min_rate;
        self.circle.center[0] -= dt * velocity;
    }
}

pub struct App {
    _frame_update_handle: Interval,
    _spawn_update_handle: Interval,
    circles: Vec<Circle>,
    size: usize,
}

pub enum Msg {
    Update(f32),
    Spawn(),
}

impl Component for App {
    type Message = Msg;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        log!("Creating app.");

        let viewbox = get_viewbox().expect("Unable to load viewbox.");

        //let mut circles =
        //    geom::generate_random_points(1000, &viewbox.top_left(), &viewbox.bottom_right())
        //        .iter()
        //        .map(|&pt| Circle::new(pt))
        //        .collect::<Vec<Circle>>();
        //circles.sort_by_key(|c| (1E3 * (1.0 - c.depth)) as u32);

        Self {
            _frame_update_handle: {
                let link = ctx.link().clone();
                let fps = 24;
                Interval::new(1000 / fps, move || {
                    link.send_message(Msg::Update(1.0 / fps as f32))
                })
            },
            _spawn_update_handle: {
                let link = ctx.link().clone();
                Interval::new(1000, move || link.send_message(Msg::Spawn()))
            },
            size: 500,
            circles: Vec::new(),
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Msg) -> bool {
        let viewbox = get_viewbox().unwrap();
        match msg {
            Self::Message::Update(dt) => {
                self.circles.iter_mut().for_each(|c| c.update(dt));
                self.circles
                    .retain(|c| !geom::circle_fully_outside_aabox(&c.circle, &viewbox));
                true
            }
            Self::Message::Spawn() => {
                for i in 0..self.size {
                    self.circles.push(Circle::new_outside(&viewbox));
                }

                self.circles.sort_by_key(|c| (1E3 * (1.0 - c.depth)) as u32);
                log!("{} svgs", self.circles.len());
                false
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let window_size = get_window_size().expect("Unable to get window size.");
        let viewbox = get_viewbox().unwrap();

        let style_string = format!("width:{}px;height:{}px", window_size[0], window_size[1]);
        let viewbox_string = format!(
            "{} {} {} {}",
            viewbox.start[0], viewbox.start[1], viewbox.dim[0], viewbox.dim[1]
        );

        let link = ctx.link();

        html! {
            <div id="container" style={style_string}>
                <svg width="100%" height="100%" viewBox={viewbox_string} preserveAspectRatio="none">
                { for self.circles.iter().map(|c| { html! { <svg::Circle ..c.to_props()/> } }) }
                </svg>
            </div>
        }
    }
}
