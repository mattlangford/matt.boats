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

fn get_viewbox_size() -> Option<Vec2f> {
    get_window_size().map(|s| Vec2f::new(HEIGHT * s[0] / s[1], HEIGHT))
}

fn get_viewbox() -> Option<AABox> {
    get_viewbox_size().map(|dim| AABox {
        start: -0.5 * dim,
        dim: dim,
    })
}

struct Grid {
    boxes: Vec<AABox>,
    neighbors: Vec<Vec<usize>>,
}

impl Grid {
    fn new(viewbox: AABox) -> Self {
        Self {
            boxes: vec![viewbox],
            neighbors: vec![vec![]],
        }
    }

    fn split(&mut self, old_index: usize) {
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
    }

    fn render(&self) -> Html {
        html! {
            for self.boxes.iter().map(|b|
                html!{ <svg::Rect ..svg::RectProps::from_aabox(b).with_class("gridline")/> })
        }
    }
}

struct App {
    grid: Grid,
    map: Map,
    zoom: bool,
}

enum Msg {
    ZoomToggle,
}

impl Component for App {
    type Message = Msg;
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        let viewbox = get_viewbox().expect("Unable to get viewBox.");
        let map = Map::generate_random(&viewbox);
        let mut grid = Grid::new(viewbox);
        grid.split(0);
        grid.split(1);
        grid.split(0);

        log!(
            "Loaded {} coordinates and {} ports",
            map.coordinates.len(),
            map.ports.len()
        );
        Self {
            map: map,
            grid: grid,
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
        let viewbox = get_viewbox().unwrap();

        let style_string = format!("width:{}px;height:{}px", window_size[0], window_size[1]);

        let viewbox_string = format!(
            "{} {} {} {}",
            scale * viewbox.start[0],
            scale * viewbox.start[1],
            scale * viewbox.dim[0],
            scale * viewbox.dim[1]
        );

        let point_str = self
            .map
            .coordinates
            .iter()
            .map(|pt| format!("{:.3},{:.3} ", pt[0], pt[1]))
            .collect::<String>();

        let edges = viewbox.edges();
        let edge_points = edges.iter().flat_map(|l| generate_points_on_line(10, l));

        html! {
        <>
            <div id="container" style={style_string}
                onclick={ctx.link().callback(|_| Self::Message::ZoomToggle )}>
                <svg width="100%" height="100%" viewBox={viewbox_string} preserveAspectRatio="none" class="svgstyle">
                    <polyline class="land" points={point_str}/>

                    {
                    for self.map.ports.iter()
                        .map(|pt| svg::RectProps::square(pt, 500.0 * scale).with_class("port"))
                        .map(|props| html! { <svg::Rect ..props/> })
                    }

                    {
                    for edge_points.filter(|pt| !point_in_polygon(&pt, &self.map.coordinates)).map(|pt|
                        html!{ <circle cx={f(pt[0])} cy={f(pt[1])} r="0.5%" fill="yellow"/> })
                    }

                    <svg::Rect ..svg::RectProps::from_aabox(&viewbox).with_class("outline")/>

                    {self.grid.render()}
                </svg>
            </div>
        </>
        }
    }
}

fn main() {
    log!("Starting model...");
    //log!("{:?}", get_window_size());
    yew::start_app::<App>();
}
