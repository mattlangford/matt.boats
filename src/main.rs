mod map;

use gloo::timers::callback::Interval;
use nalgebra as na;
use wasm_bindgen::JsCast;
use yew::prelude::*;

//use rand::seq::SliceRandom;
//use rand::Rng;
//use rand::SeedableRng;

// Dish out to gloo::console since it doesn't format the inputs.
macro_rules! log {
    ($($arg:tt)+) => (
        gloo::console::log!(format!($($arg)+));
    );
}

const HEIGHT: f64 = 5.0;

fn f(v: f64) -> String {
    format!("{:.5}", v)
}

fn get_window_size() -> Option<na::Vector2<f64>> {
    let window = web_sys::window().unwrap();
    let w_height = window.inner_height().ok().and_then(|v| v.as_f64());
    let w_width = window.inner_width().ok().and_then(|v| v.as_f64());
    if let (Some(h), Some(w)) = (w_height, w_width) {
        Some(na::vector!(w, h))
    } else {
        None
    }
}

fn get_viewbox_size() -> Option<na::Vector2<f64>> {
    get_window_size().map(|s| na::vector![HEIGHT * s[0] / s[1], HEIGHT])
}

struct Model {
    t: f64,
    _update_handle: Interval,
}

enum Msg {
    Update(f64),
}

impl Component for Model {
    type Message = Msg;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        Self {
            t: 0.0,
            _update_handle: {
                let link = ctx.link().clone();
                let fps = 15;
                Interval::new(1000 / fps, move || {
                    link.send_message(Msg::Update(1.0 / fps as f64))
                })
            },
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::Update(dt) => {
                self.t += dt;
                true
            }
        }
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        let window_size = get_window_size().expect("Unable to get window size.");
        html! {
            <>
                <div id="container" style="background-image: url('assets/test.png');">
                    {"hello world!"}
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
