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

use std::collections::HashSet;

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

#[derive(Debug, Default)]
struct GraphNode {
    point: Vec2f,
    score: f64,
    edges: Vec<usize>,
}

#[derive(Debug, Default)]
struct Graph {
    graph: Vec<GraphNode>,
}

impl Graph {
    fn set(&mut self, index: usize, score: f64) {
        if min_in_place(&mut self.graph[index].score, score) {
            for neighbor in self.graph[index].edges.clone() {
                let cost = (self.graph[index].point - self.graph[neighbor].point).norm();
                self.set(neighbor, score + cost as f64);
            }
        }
    }

    fn get(&self, index: usize) -> Vec<usize> {
        let score = self.graph[index].score;
        let mut edges = self.graph[index]
            .edges
            .iter()
            .filter(|&n| self.graph[*n].score < score)
            .copied()
            .collect::<Vec<_>>();
        edges.sort_by(|a, b| {
            self.graph[*a]
                .score
                .partial_cmp(&self.graph[*b].score)
                .expect("Tried to compare a NaN")
        });
        let scores = self.graph[index]
            .edges
            .iter()
            .map(|&n| self.graph[n].score)
            .collect::<Vec<_>>();
        edges
    }
}

#[derive(Debug, Default)]
struct Grid {
    boxes: Vec<AABox>,
    neighbors: Vec<Vec<(usize, bool)>>,
}

impl Grid {
    fn new(viewbox: AABox) -> Self {
        Self {
            boxes: vec![viewbox],
            neighbors: vec![vec![]],
        }
    }

    fn new_subdivided(viewbox: AABox, divides: usize) -> Self {
        let mut grid = Grid::new(viewbox);
        for div in 0..divides {
            let count = grid.boxes.len();
            for i in 0..count {
                grid.split(i);
            }
        }
        grid
    }

    fn into_graph(&self) -> Graph {
        Graph {
            graph: self
                .boxes
                .iter()
                .zip(self.neighbors.iter())
                .map(|(b, ns)| GraphNode {
                    point: b.center(),
                    score: f64::INFINITY,
                    edges: ns
                        .iter()
                        .filter(|(i, valid)| *valid)
                        .map(|(i, _)| *i)
                        .collect(),
                })
                .collect(),
        }
    }

    fn query(&self, point: &Vec2f) -> Result<usize, String> {
        self.boxes
            .iter()
            .enumerate()
            .filter(|(_, b)| point_in_aabox(&point, b))
            .map(|(i, _)| i)
            .next()
            .ok_or(format!("Unable to find point ({}) in grid.", point))
    }

    fn split(&mut self, old_index: usize) {
        let new_index = self.boxes.len();
        let new = self.boxes[old_index].split_mut();
        self.boxes.push(new);

        let old_neighbors = self.neighbors[old_index].clone();
        self.neighbors[old_index].clear();
        self.neighbors.push(Vec::with_capacity(old_neighbors.len()));

        for (i, _) in old_neighbors {
            if aabox_are_adjacent(&self.boxes[new_index], &self.boxes[i]) {
                self.neighbors[i].push((new_index, true));
                self.neighbors[new_index].push((i, true));
            }

            if aabox_are_adjacent(&self.boxes[old_index], &self.boxes[i]) {
                self.neighbors[old_index].push((i, true));
            } else {
                let index = self.neighbors[i]
                    .iter()
                    .position(|(j, _)| *j == old_index)
                    .unwrap();
                self.neighbors[i].swap_remove(index);
            }
        }

        self.neighbors[old_index].push((new_index, true));
        self.neighbors[new_index].push((old_index, true));
    }

    fn render(&self) -> Html {
        html! {
            for self.boxes.iter().enumerate().map(|(i, b)| {
                let center = b.center();
                html!{
                    <>
                        <svg::Rect ..svg::RectProps::from_aabox(b).with_class("gridline")/>
                        <text x={svg::s(center[0])} y={svg::s(center[1])} class="heavy" transform="scale(1,1)">{i}</text>
                        {
                        for self.neighbors[i].iter()
                            .filter(|n| n.1)
                            .map(|n| Line::new_segment(center, self.boxes[n.0].center()))
                            .map(|l| html! { <svg::Line ..svg::LineProps::from_line(&l).with_class("gridline-thin")/> } )
                        }
                    </>
                }
            })
        }
    }
}

#[derive(Debug)]
enum StepResult {
    Step(Vec2f),
    Success,
    Split,
}

fn step_search(
    current: Vec2f,
    goal: Vec2f,
    map: &[Vec2f],
    grid: &mut Grid,
) -> Result<StepResult, String> {
    if !intersect_polygon(&Line::new_segment(current, goal), map) {
        return Ok(StepResult::Success);
    }

    let mut graph = grid.into_graph();

    // Populate goal
    let goal_i = grid.query(&goal)?;
    graph.set(goal_i, 0.0);

    let current_i = grid.query(&current)?;

    if graph.graph[current_i].score.is_infinite() {
        let mut to_split = HashSet::<usize>::new();
        for (i, g) in graph.graph.iter().enumerate() {
            // G is a losing cell.
            if g.score.is_finite() {
                continue;
            }

            to_split.insert(i);
            for (n, v) in &grid.neighbors[i] {
                if graph.graph[*n].score.is_infinite() {
                    continue;
                }
                to_split.insert(*n);
            }
        }

        for i in to_split {
            grid.split(i);
        }

        return Ok(StepResult::Split);
    }

    let mut count = 0;
    for to_i in graph.get(current_i) {
        let to_point = grid.boxes[to_i].center();
        if !intersect_polygon(&Line::new_segment(current, to_point), map) {
            return Ok(StepResult::Step(to_point));
        }

        for (_, valid) in &mut grid.neighbors[current_i].iter_mut().filter(|(n, _)| *n == to_i) {
            *valid = false;
        }
        for (n, valid) in &mut grid.neighbors[to_i].iter_mut().filter(|(n, _)| *n == current_i) {
            *valid = false;
        }

        count += 1;
        if count > 10 {
            return Err(String::from("MaxIterations reached"));
        }
    }

    if count == 0 {
        grid.split(current_i);
    }
    return Ok(StepResult::Split);
}

struct App {
    grid: Grid,
    map: Map,
    zoom: bool,

    position: Vec2f,
    goal: Vec2f,
    history: Vec<Vec2f>,

    solution: Vec<Vec2f>,
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
            .map(|pt| 0.98 * pt)
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
            grid: Grid::new_subdivided(viewbox, 1),
            zoom: true,

            position: start,
            goal: goal,
            history: vec![start],

            solution: vec![],
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Self::Message::ZoomToggle => {
                self.zoom = !self.zoom;
                true
            }
            Self::Message::Step => {
                loop {
                    if self.position == self.goal {
                        break;
                    }

                    let result = step_search(
                        self.position,
                        self.goal,
                        &self.map.coordinates,
                        &mut self.grid,
                    );
                    log!("step_search result: {:?}", result);

                    match result {
                        Err(s) => {
                            log!("Error from step_search: {}", s);
                        }
                        Ok(StepResult::Step(p)) => {
                            self.position = p;
                            self.history.push(self.position);
                        }
                        Ok(StepResult::Success) => {
                            self.position = self.goal;
                            self.history.push(self.position);

                            self.solution.push(self.position);
                            loop {
                                let pos = self.solution.last().unwrap();
                                for (i, h) in self.history.iter().enumerate() {
                                    if !intersect_polygon(
                                        &Line::new_segment(*h, *pos),
                                        &self.map.coordinates,
                                    ) {
                                        self.solution.push(*h);
                                        if i == 0 {
                                            return true;
                                        }
                                        break;
                                    }
                                }
                            }
                        }
                        Ok(StepResult::Split) => {}
                    }

                    if self.grid.boxes.iter().any(|b| b.area() < 10.0) {
                        break;
                    }
                }
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let window_size = get_window_size().expect("Unable to get window size.");
        let scale: f32 = if self.zoom { 1.1 } else { 1.2 };
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
                    <svg::Rect ..svg::RectProps::square(&self.position, 400.0 * scale).with_class("position")/>
                    <svg::Rect ..svg::RectProps::square(&self.goal, 400.0 * scale).with_class("goal")/>
                    {
                        for self.history.windows(2)
                            .map(|s| svg::LineProps::from_line(&Line::new_segment(s[0], s[1])).with_class("path"))
                            .map(|props| html! { <svg::Line ..props/> })
                    }
                    {
                        for self.solution.windows(2)
                            .map(|s| svg::LineProps::from_line(&Line::new_segment(s[0], s[1])).with_class("solution"))
                            .map(|props| html! { <svg::Line ..props/> })
                    }

                    {self.grid.render()}
                </svg>
            </div>
        </>
        }
    }
}

fn main() {
    log!("Starting model...");
    yew::start_app::<App>();
}
