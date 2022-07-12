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
    neighbors: Vec<Vec<(usize, f32)>>,
}

struct CellRef<'a> {
    index: usize,
    cell: &'a AABox,
    neighbors: &'a Vec<(usize, f32)>,
}

impl Grid {
    fn new(viewbox: AABox) -> Self {
        Self {
            boxes: vec![viewbox],
            neighbors: vec![vec![]],
        }
    }

    fn make_cell(&self, index: usize) -> CellRef {
        CellRef {
            index: index,
            cell: &self.boxes[index],
            neighbors: &self.neighbors[index],
        }
    }

    //fn query(&self, point: &Vec2f) -> Option<CellRef> {
    //    self.boxes.iter().enumerate().filter(|(_, b)| point_in_aabox(&point, b)).map(|(i, _)| self.make_cell(i)).next()
    //}
    fn query(&self, point: &Vec2f) -> Option<usize> {
        self.boxes
            .iter()
            .enumerate()
            .filter(|(_, b)| point_in_aabox(&point, b))
            .map(|(i, _)| i)
            .next()
    }

    fn neighbors<'a>(&'a self, cell: &'a CellRef) -> impl Iterator<Item = (f32, CellRef<'a>)> {
        cell.neighbors.iter().map(|&(i, s)| (s, self.make_cell(i)))
    }

    fn split(&mut self, old_index: usize) {
        let new_index = self.boxes.len();
        let new = self.boxes[old_index].split_mut();
        self.boxes.push(new);

        let distance_guess = |lhs: &AABox, rhs: &AABox| (lhs.center() - rhs.center()).norm();

        let num_old_neighbors = self.neighbors[old_index].len();
        let old_neighbors = self.neighbors[old_index].split_off(num_old_neighbors);
        self.neighbors.push(Vec::with_capacity(num_old_neighbors));

        for (i, _) in old_neighbors {
            if aabox_are_adjacent(&self.boxes[new_index], &self.boxes[i]) {
                let dist = distance_guess(&self.boxes[new_index], &self.boxes[i]);
                self.neighbors[i].push((new_index, dist));
                self.neighbors[new_index].push((i, dist));
            }

            if aabox_are_adjacent(&self.boxes[old_index], &self.boxes[i]) {
                let dist = distance_guess(&self.boxes[old_index], &self.boxes[i]);
                self.neighbors[old_index].push((i, dist));
            } else {
                let index = self.neighbors[i]
                    .iter()
                    .position(|&(j, _)| j == old_index)
                    .unwrap();
                self.neighbors[i].swap_remove(index);
            }
        }

        let dist = distance_guess(&self.boxes[new_index], &self.boxes[old_index]);
        self.neighbors[old_index].push((new_index, dist));
        self.neighbors[new_index].push((old_index, dist));
    }

    fn render(&self) -> Html {
        html! {
            for self.boxes.iter().map(|b|
                html!{ <svg::Rect ..svg::RectProps::from_aabox(b).with_class("gridline")/> })
        }
    }
}

#[derive(Debug)]
enum StepResult {
    Step(Vec2f),
    Failure(String),
    Success,
    Split,
}

// TODO: Split should return error if splitting too much, need to check if points are inside polygon or not

fn step_search(
    current: Vec2f,
    goal: Vec2f,
    valid: impl Fn(Line) -> bool,
    grid: &mut Grid,
) -> StepResult {
    let maybe_current_cell = grid.query(&current);
    if maybe_current_cell.is_none() {
        return StepResult::Failure("Unable to find current cell.".into());
    }
    let current_cell = maybe_current_cell.unwrap();

    if !valid(Line::new_segment(
        current,
        grid.boxes[current_cell].center(),
    )) {
        grid.split(current_cell);
        return StepResult::Split;
    }

    let maybe_goal_cell = grid.query(&goal);
    if maybe_goal_cell.is_none() {
        return StepResult::Failure("Unable to find goal cell.".into());
    }
    let goal_cell = maybe_goal_cell.unwrap();

    if !valid(Line::new_segment(grid.boxes[goal_cell].center(), goal)) {
        grid.split(goal_cell);
        return StepResult::Split;
    }

    StepResult::Failure("TODO".into())
}

struct App {
    grid: Grid,
    map: Map,
    zoom: bool,

    position: Vec2f,
    goal: Vec2f,
}

enum Msg {
    ZoomToggle,
    Step,
}

impl Component for App {
    type Message = Msg;
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        let viewbox = get_viewbox().expect("Unable to get viewBox.");
        let map = Map::generate_random(&viewbox);

        let edges = viewbox.edges();
        let start = edges
            .iter()
            .flat_map(|l| generate_points_on_line(10, l))
            .filter(|pt| !point_in_polygon(&pt, &map.coordinates))
            .next()
            .unwrap_or(map.ports[1]);
        let goal = map.ports[0];

        log!(
            "Loaded {} coordinates and {} ports",
            map.coordinates.len(),
            map.ports.len()
        );
        Self {
            map: map,
            grid: Grid::new(viewbox),
            zoom: true,

            position: start,
            goal: goal,
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Self::Message::ZoomToggle => {
                self.zoom = !self.zoom;
                true
            }
            Self::Message::Step => {
                let valid = |line: Line| {
                    self.map
                        .coordinates
                        .iter()
                        .zip(ring_iter(self.map.coordinates.iter(), 1))
                        .all(|(start, end)| intersect_segment(&line, &start, &end).is_none())
                };
                log!(
                    "Step_search result: {:?}",
                    step_search(self.position, self.goal, valid, &mut self.grid)
                );
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

        html! {
        <>
            <div id="container" style={style_string}
                onclick={ctx.link().callback(|_| Self::Message::Step )}>
                <svg width="100%" height="100%" viewBox={viewbox_string} preserveAspectRatio="none" class="svgstyle">
                    <polyline class="land" points={point_str}/>

                    {
                    for self.map.ports.iter()
                        .map(|pt| svg::RectProps::square(pt, 500.0 * scale).with_class("port"))
                        .map(|props| html! { <svg::Rect ..props/> })
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
