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

#[derive(Debug)]
struct Collision {
    point: na::Vector2<f64>,
    normal: na::Vector2<f64>,
}

trait Colliable {
    fn is_collided(&self, center: &na::Vector2<f64>, radius: f64) -> bool;
    fn get_collision(&self, center: &na::Vector2<f64>, radius: f64) -> Option<Collision>;
}

impl Colliable for Ball {
    fn is_collided(&self, center: &na::Vector2<f64>, radius: f64) -> bool {
        let dist = (self.center - center).norm();
        dist < radius || dist < self.radius
    }

    fn get_collision(&self, center: &na::Vector2<f64>, radius: f64) -> Option<Collision> {
        None
    }
}

#[derive(Default, Debug)]
struct Line {
    start: na::Vector2<f64>,
    end: na::Vector2<f64>,
}

impl Line {
    fn new(s_x: f64, s_y: f64, e_x: f64, e_y: f64) -> Line {
        Line {
            start: na::vector![s_x, s_y],
            end: na::vector![e_x, e_y],
        }
    }
}

impl Line {
    fn length(&self) -> f64 {
        (self.end - self.start).norm()
    }
    fn direction(&self) -> na::Vector2<f64> {
        self.end - self.start
    }
    fn normal(&self) -> na::Vector2<f64> {
        let dir = self.direction() / self.length();
        na::vector!(dir[1], dir[0])
    }
}

impl Colliable for Line {
    fn is_collided(&self, center: &na::Vector2<f64>, radius: f64) -> bool {
        let to_center = center - self.start;
        if to_center.dot(&self.normal()).abs() > radius {
            return false;
        }

        let along_edge_len = to_center.dot(&self.direction().normalize());
        if along_edge_len < 0.0 || along_edge_len >= self.length() {
            return false;
        }

        return true;
    }

    fn get_collision(&self, center: &na::Vector2<f64>, radius: f64) -> Option<Collision> {
        if !self.is_collided(center, radius) {
            return None;
        }

        let to_center = center - self.start;
        let direction = self.direction().normalize();
        let point = self.start + direction * to_center.dot(&direction);

        let mut normal = self.normal();
        if (center - point).dot(&normal) < 0.0 {
            normal *= -1.0;
        }

        Some(Collision {
            point: point,
            normal: normal,
        })
    }
}

#[derive(Default, Debug)]
struct Rect {
    top: Line,
    left: Line,
    right: Line,
    bottom: Line,
}

impl Rect {
    fn from_diag(tl: &na::Vector2<f64>, br: &na::Vector2<f64>) -> Rect {
        Rect {
            top: Line::new(tl[0], tl[1], br[0], tl[1]),
            left: Line::new(tl[0], tl[1], tl[0], br[1]),
            right: Line::new(br[0], br[1], br[0], tl[1]),
            bottom: Line::new(tl[0], br[1], br[0], br[1]),
        }
    }

    fn get_collision(&self, center: &na::Vector2<f64>, radius: f64) -> Vec<Collision> {
        [&self.top, &self.left, &self.right, &self.bottom]
            .iter()
            .filter_map(|l| l.get_collision(center, radius))
            .collect()
    }
}

struct Model {
    bounds: Rect,
    balls: Vec<Ball>,
    _update_handle: Interval,
}

impl Component for Model {
    type Message = Msg;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        let mut balls: Vec<Ball> = Vec::new();
        balls.push(Ball::new());
        balls.push(Ball::new());
        balls.push(Ball::new());

        Self {
            bounds: Rect::from_diag(&na::vector![0.0, 0.0], &na::vector![1.0, 1.0]),
            balls: balls,
            _update_handle: {
                let link = ctx.link().clone();
                let fps = 17;
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
                    for Collision { point, normal } in
                        self.bounds.get_collision(&ball.center, ball.radius)
                    {
                        let along_normal = ball.velocity.dot(&normal);
                        ball.velocity -= normal * along_normal * 2.0;
                        ball.center = point + normal * ball.radius;
                    }
                }

                for i in 0..self.balls.len() {
                    for j in i + 1..self.balls.len() {
                        if self.balls[i].is_collided(&self.balls[j].center, self.balls[j].radius) {
                            let normal = (self.balls[j].center - self.balls[i].center).normalize();

                            let i_along_normal = self.balls[i].velocity.dot(&normal);
                            let j_along_normal = self.balls[j].velocity.dot(&normal);

                            self.balls[i].velocity -= normal * i_along_normal * 2.0;
                            self.balls[j].velocity -= normal * j_along_normal * 2.0;
                        }
                    }
                }

                true
            }
        }
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        let style_string = get_window_size().map_or(String::from(""), |x| {
            format!("width:{}px;height:{}px;background:grey", x[0], x[1])
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
