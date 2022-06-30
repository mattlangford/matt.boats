#![allow(unused_imports)]

mod geom;
mod map;
mod utils;
use geom::*;
use map::*;
use utils::*;

use gloo::timers::callback::Interval;
use nalgebra as na;
use wasm_bindgen::JsCast;
use yew::prelude::*;

use rand::seq::SliceRandom;
use rand::Rng;
use rand::SeedableRng;

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
        let scale = if self.zoom { 1.0 } else { 1.2 };
        let viewbox_size = get_viewbox_size().expect("Unable to get viewBox size.");

        let style_string = format!("width:{}px;height:{}px", window_size[0], window_size[1]);

        let corner = Vec2f::new(0.5 * viewbox_size[0] as f32, 0.5 * viewbox_size[1] as f32);
        let viewbox_string = format!(
            "{} {} {} {}",
            -scale * corner[0],
            -scale * corner[1],
            2.0 * scale * corner[0],
            2.0 * scale * corner[1]
        );

        let point_str = self
            .map
            .coordinates
            .iter()
            .map(|pt| format!("{:.3},{:.3} ", pt[0], pt[1]))
            .collect::<String>();

        let port_size = (scale * 500.0) as f32;

        let edges = generate_edges(viewbox_size[0] as f32, viewbox_size[1] as f32);
        let edge_points = edges.iter().flat_map(|l| generate_points_on_line(10, l));

        html! {
            <div id="container" style={style_string} onclick={ctx.link().callback(|_| Self::Message::ZoomToggle )}>
                <svg width="100%" height="100%" viewBox={viewbox_string} preserveAspectRatio="none" style="display: block; transform: scale(1,-1)">
                    <polyline class="land" points={point_str}/>
                    //{
                    //for self.map.coordinates.iter().map(|pt| html!{
                    //    <circle
                    //        cx={f(pt[0])}
                    //        cy={f(pt[1])}
                    //        r="0.5%"
                    //        />
                    //    })
                    //}

                    {
                    for self.map.ports.iter().map(|pt| html!{
                        <rect class="port"
                            x={f(pt[0] - 0.5 * port_size)}
                            y={f(pt[1] - 0.5 * port_size)}
                            height={f(port_size)}
                            width={f(port_size)}/>
                        })
                    }

                    {
                    for edge_points.filter(|pt| !point_in_polygon(&pt, &self.map.coordinates)).map(|pt| html!{
                        <circle
                            cx={f(pt[0])}
                            cy={f(pt[1])}
                            r="0.5%"
                            fill="yellow"/>
                        })
                    }

                    <rect fill="none" stroke="red" stroke-width="0.5%"
                        x={f(-0.5 * self.map.width_m)}
                        y={f(-0.5 * self.map.height_m)}
                        height={f(self.map.height_m)}
                        width={f(self.map.width_m)}/>
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
