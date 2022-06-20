#![allow(unused_imports)]

mod map;
use map::Map;

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

const HEIGHT: f64 = 50000.0;

fn f<T: std::fmt::Display>(v: T) -> String {
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

struct BackgroundMap {
    points: Vec<na::Vector2<f32>>,
    ports: Vec<na::Vector2<f32>>,
}

impl Component for BackgroundMap {
    type Message = ();
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        let map = Map::generate().unwrap();
        let viewbox_size = get_viewbox_size().expect("Unable to get viewBox size.");
        let points = map.center_and_crop(
            rand::thread_rng().gen_range(0..map.num_points()),
            viewbox_size[0] as f32,
            viewbox_size[1] as f32,
        );

        let mut rng = rand::thread_rng();
        let mut ports = Vec::<na::Vector2<f32>>::new();
        for _ in 0..50 {
            let pt = points.choose(&mut rng).unwrap();
            if ports.iter().any(|&other| (other - pt).norm() < 5000.0) {
                continue;
            }
            ports.push(*pt);
        }

        log!("{} points", points.len());
        Self {
            points: points,
            ports: ports,
        }
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        let window_size = get_window_size().expect("Unable to get window size.");
        let viewbox_size = get_viewbox_size().expect("Unable to get viewBox size.");

        let style_string = format!("width:{}px;height:{}px", window_size[0], window_size[1]);

        let viewbox_string = format!(
            "{} {} {} {}",
            -0.5 * viewbox_size[0],
            -0.5 * viewbox_size[1],
            viewbox_size[0],
            viewbox_size[1]
        );

        let point_str = self
            .points
            .iter()
            .map(|pt| format!("{:.3},{:.3} ", pt[0], pt[1]))
            .collect::<String>();

        let port_size = 500.0;

        html! {
            <div id="container" style={style_string}>
                <svg width="100%" height="100%" viewBox={viewbox_string} preserveAspectRatio="none" style="display: block; transform: scale(1,-1)">
                    <polyline class="land" points={point_str}/>
                    {for self.ports.iter().map(|pt| html!{ <rect class="port" x={f(pt[0] - 0.5 * port_size)} y={f(pt[1] - 0.5 * port_size)} height={f(port_size)} width={f(port_size)}/> })}
                </svg>
            </div>
        }
    }
}

fn main() {
    log!("Starting model...");
    //log!("{:?}", get_window_size());
    yew::start_app::<BackgroundMap>();
}
