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
    map: Map,
    zoom: bool,
}

enum Msg {
    ZoomToggle,
}

impl Component for BackgroundMap {
    type Message = Msg;
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        let viewbox_size = get_viewbox_size().expect("Unable to get viewBox size.");
        let map = Map::generate_random(viewbox_size[0] as f32, viewbox_size[1] as f32);

        log!(
            "Loaded {} coordinates and {} ports",
            map.coordinates.len(),
            map.ports.len()
        );

        Self {
            map: map,
            zoom: true,
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Self::Message::ZoomToggle => {
                self.zoom = !self.zoom;
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let window_size = get_window_size().expect("Unable to get window size.");
        let scale = if self.zoom { 1.0 } else { 200.0 };
        let viewbox_size = scale * get_viewbox_size().expect("Unable to get viewBox size.");

        let style_string = format!("width:{}px;height:{}px", window_size[0], window_size[1]);

        let viewbox_string = format!(
            "{} {} {} {}",
            -0.5 * viewbox_size[0],
            -0.5 * viewbox_size[1],
            viewbox_size[0],
            viewbox_size[1]
        );

        let point_str = self
            .map
            .coordinates
            .iter()
            .map(|pt| format!("{:.3},{:.3} ", pt[0], pt[1]))
            .collect::<String>();

        let port_size = (scale * 500.0) as f32;

        html! {
            <div id="container" style={style_string} onclick={ctx.link().callback(|_| Self::Message::ZoomToggle )}>
                <svg width="100%" height="100%" viewBox={viewbox_string} preserveAspectRatio="none" style="display: block; transform: scale(1,-1)">
                    <polyline class="land" points={point_str}/>

                    //{
                    //for self.ports.iter().map(|pt| html!{
                    //    <rect class="port"
                    //        x={f(pt[0] - 0.5 * port_size)}
                    //        y={f(pt[1] - 0.5 * port_size)}
                    //        height={f(port_size)}
                    //        width={f(port_size)}/>
                    //    })
                    //}
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
