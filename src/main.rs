#![allow(unused_imports)]

mod geom;
mod map;
mod svg;
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

const HEIGHT: f32 = 50000.0;

fn f<T: std::fmt::Display>(v: T) -> String {
    format!("{:.5}", v)
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

fn get_viewbox_size() -> Option<na::Vector2<f32>> {
    get_window_size().map(|s| Vec2f::new(HEIGHT * s[0] / s[1], HEIGHT))
}

struct Boxes {
    boxes: Vec<AABox>,
    neighbors: Vec<Vec<usize>>,
}

enum BoxesMsg {
    Split(usize),
}

impl Component for Boxes {
    type Message = BoxesMsg;
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        let viewbox_size = get_viewbox_size().expect("Unable to get viewBox size.");
        let initial_boxes = vec![AABox {
            start: -0.5 * viewbox_size,
            dim: viewbox_size,
        }];

        Self {
            boxes: initial_boxes,
            neighbors: vec![vec![]],
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            BoxesMsg::Split(old_index) => {
                let new_index = self.boxes.len();
                let new = self.boxes[old_index].split_mut();
                self.boxes.push(new);

                let old_neighbors = self.neighbors[old_index].clone();
                self.neighbors.push(Vec::with_capacity(old_neighbors.len()));
                self.neighbors[old_index].clear();
                for i in old_neighbors {
                    if aabox_are_adjacent(&self.boxes[new_index], &self.boxes[i]) {
                        self.neighbors[i].push(new_index);
                        self.neighbors[new_index].push(i);
                    }

                    if aabox_are_adjacent(&self.boxes[old_index], &self.boxes[i]) {
                        self.neighbors[old_index].push(i);
                    } else {
                        let index = self.neighbors[i]
                            .iter()
                            .position(|&j| j == old_index)
                            .unwrap();
                        self.neighbors[i].swap_remove(index);
                    }
                }

                self.neighbors[old_index].push(new_index);
                self.neighbors[new_index].push(old_index);

                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let window_size = get_window_size().expect("Unable to get window size.");
        let viewbox_size = get_viewbox_size().expect("Unable to get viewBox size.");

        let style_string = format!("width:{}px;height:{}px", window_size[0], window_size[1]);

        let viewbox_string = format!("0.0 0.0 {} {}", viewbox_size[0], viewbox_size[1],);
        log!("Neighbors: {:?}", self.neighbors);

        html! {
        <>
            {
            for self.neighbors.iter().enumerate().flat_map(|(i1, neighs)| neighs.iter().map(move |i2| {
                let start = self.boxes[i1].center();
                let end = self.boxes[*i2].center();
                html!{
                    <line
                        x1={f(start[0])}
                        y1={f(start[1])}
                        x2={f(end[0])}
                        y2={f(end[1])}
                        stroke="yellow"
                        stroke-width={format!("{:.5}%", 1.0 * (end - start).norm() / viewbox_size.norm())}
                    />
                }
            }))
            }
            {
            for self.boxes.iter().enumerate().map(|(i, b)|
                html!{
                    <rect class="gridline"
                        x={f(b.start[0])}
                        y={f(b.start[1])}
                        width={f(b.dim[0])}
                        height={f(b.dim[1])}
                        onclick={ctx.link().callback(move |_| Self::Message::Split(i))}
                    />
                }
            )
            }
        </>
        }
    }
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
        let map = Map::generate_random(viewbox_size[0], viewbox_size[1]);

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
        let scale: f32 = if self.zoom { 1.0 } else { 1.2 };
        let viewbox_size = get_viewbox_size().expect("Unable to get viewBox size.");

        let style_string = format!("width:{}px;height:{}px", window_size[0], window_size[1]);

        let corner = Vec2f::new(0.5 * viewbox_size[0], 0.5 * viewbox_size[1]);
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

        let edges = generate_edges(viewbox_size[0], viewbox_size[1]);
        let edge_points = edges.iter().flat_map(|l| generate_points_on_line(10, l));

        html! {
        <>
            <div id="container" style={style_string}
                oncontextmenu={ctx.link().callback(|_| Self::Message::ZoomToggle )}>
                <svg width="100%"
                     height="100%"
                     viewBox={viewbox_string}
                     preserveAspectRatio="none"
                     class="svgstyle">
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
                    for self.map.ports.iter().map(|pt|
                        html!{<svg::Rect ..svg::RectProps::square_centered_at_with_class(pt[0], pt[1], 500.0 * scale, "port")/>
                    })
                    }
                    //{
                    //for (0..10).map(|i| (i as f32 + 0.5)/ 10.0).map(|t| html! {
                    //  <>
                    //    <svg::Line x1={-corner[0]} y1={2.0 * t * corner[1] - corner[1]}
                    //               x2={corner[0]} y2={2.0 * t * corner[1] - corner[1]}
                    //               class={Some("gridline".to_string())}/>
                    //    <svg::Line y1={-corner[1]} x1={2.0 * t * corner[0] - corner[0]}
                    //               y2={corner[1]} x2={2.0 * t * corner[0] - corner[0]}
                    //               class={Some("gridline".to_string())}/>
                    //  </>
                    //})
                    //}
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
                    <Boxes/>
                </svg>
            </div>
        </>
        }
    }
}

fn main() {
    log!("Starting model...");
    //log!("{:?}", get_window_size());
    yew::start_app::<BackgroundMap>();
}
