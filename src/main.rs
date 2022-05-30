use gloo::timers::callback::Interval;
use nalgebra as na;
use wasm_bindgen::JsCast;
use yew::prelude::*;

use rand::seq::SliceRandom;
use rand::Rng;
use rand::SeedableRng;

// Dish out to gloo::console since it doesn't format the inputs.
macro_rules! log {
    ($($arg:tt)+) => (
        gloo::console::log!(format!($($arg)+));
    );
}

fn get_window_size() -> Option<na::Vector2<f64>> {
    let window = web_sys::window().unwrap();
    let document = window.document().unwrap();
    let w_height = window.inner_height().ok().and_then(|v| v.as_f64());
    let w_width = window.inner_width().ok().and_then(|v| v.as_f64());
    if let (Some(h), Some(w)) = (w_height, w_width) {
        Some(na::vector!(w, h))
    } else {
        None
    }
}

enum Msg {
    Update(f64),
}

struct Ball {
    center: na::Vector2<f64>,
    velocity: na::Vector2<f64>,
    mass: f64,
    radius: f64,
}

impl Ball {
    fn new() -> Self {
        let s = 0.5;
        let velocity = na::vector![
            rand::thread_rng().gen_range(-s..s),
            rand::thread_rng().gen_range(-s..s)
        ];
        Self {
            center: na::vector![0.5, 0.5],
            velocity: velocity,
            mass: 1.0,
            radius: 0.1,
        }
    }

    fn render(&self) -> Html {
        html! {
            <circle
                cx={self.center[0].to_string()}
                cy={self.center[1].to_string()}
                r={self.radius.to_string()}
                fill={ format!("rgb({}, {}, {})",
                               255.0 * self.mass,
                               255.0 * self.mass,
                               255.0 * self.mass) }
            />
        }
    }
}

struct Model {
    balls: Vec<Ball>,
    _update_handle: Interval,
}

impl Component for Model {
    type Message = Msg;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        let mut balls: Vec<Ball> = Vec::new();
        balls.push(Ball::new());
        Self {
            balls: balls,
            _update_handle: {
                let link = ctx.link().clone();
                let fps = 30;
                Interval::new(1000 / fps, move || {
                    link.send_message(Msg::Update(1.0 / fps as f64))
                })
            },
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::Update(dt) => {
                for ball in self.balls.iter_mut() {
                    ball.center += ball.velocity * dt;
                }
                true
            }
        }
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        let style_string = get_window_size().map_or(String::from(""), |x| {
            format!("width:{}px;height:{}px", x[0], x[1])
        });

        html! {
            <>
                <div id="container" style={style_string}>
                    <svg width="100%" height="100%" viewBox="0 0 1 1">
                        { for self.balls.iter().map(Ball::render) }
                    </svg>
                </div>
            </>
        }
    }
}

fn main() {
    log!("Starting model...");
    log!("{:?}", get_window_size());
    yew::start_app::<Model>();
}
