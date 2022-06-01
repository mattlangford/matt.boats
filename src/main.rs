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

const HEIGHT: f64 = 1.0;

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

#[derive(Debug, Clone)]
struct Ball {
    center: na::Vector2<f64>,
    velocity: na::Vector2<f64>,
    mass: f64,
    radius: f64,
}

impl Ball {
    fn radius_from_mass(mass: f64) -> f64 {
        1E-4 * mass
    }

    fn from(c_x: f64, c_y: f64, v_x: f64, v_y: f64, m: f64) -> Self {
        Self {
            center: na::vector![c_x, c_y],
            velocity: na::vector![v_x, v_y],
            mass: m,
            radius: Ball::radius_from_mass(m),
        }
    }
    fn new() -> Self {
        let s = 3.0;
        let m = 1000.0;
        let velocity = na::vector![
            rand::thread_rng().gen_range(-s..s),
            rand::thread_rng().gen_range(-s..s)
        ];

        let mass = rand::thread_rng().gen_range(0.5..1.0) * m;
        Self {
            center: na::vector![HEIGHT, HEIGHT] * 0.5,
            velocity: velocity,
            mass: mass,
            radius: Ball::radius_from_mass(m),
        }
    }

    fn render(&self) -> Html {
        html! {
            <circle
                cx={format!("{:.3}", self.center[0])}
                cy={format!("{:.3}", self.center[1])}
                r={format!("{:.3}", self.radius)}
                fill="none"
                stroke="white"
                stroke-width={format!("{}", 0.1 * self.radius)}
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
    fn is_collided(&self, center: &na::Vector2<f64>, radius: f64) -> bool {
        self.get_collision(center, radius).is_some()
    }
    fn get_collision(&self, center: &na::Vector2<f64>, radius: f64) -> Option<Collision>;
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
    edges: [Line; 4],
    normals: [na::Vector2<f64>; 4],
}

impl Rect {
    fn from_diag(tl: &na::Vector2<f64>, br: &na::Vector2<f64>) -> Rect {
        Rect {
            edges: [
                Line::new(tl[0], tl[1], br[0], tl[1]), // top
                Line::new(tl[0], tl[1], tl[0], br[1]), // left
                Line::new(br[0], br[1], br[0], tl[1]), // right
                Line::new(tl[0], br[1], br[0], br[1]), // bottom
            ],
            normals: [
                na::vector![0.0, 1.0],
                na::vector![1.0, 0.0],
                na::vector![-1.0, 0.0],
                na::vector![0.0, -1.0],
            ],
        }
    }

    fn get_collisions(
        &self,
        center: na::Vector2<f64>,
        radius: f64,
    ) -> impl Iterator<Item = Collision> + '_ {
        self.edges
            .iter()
            .zip(self.normals)
            .filter_map(move |(l, n)| {
                if let Some(c) = l.get_collision(&center, radius) {
                    Some(Collision {
                        point: c.point,
                        normal: n,
                    })
                } else {
                    None
                }
            })
    }
}

impl Colliable for Ball {
    fn get_collision(&self, center: &na::Vector2<f64>, radius: f64) -> Option<Collision> {
        let diff = center - self.center;
        let distance = diff.norm().max(1E-3);
        if distance >= self.radius + radius {
            return None;
        }

        let normal = diff / distance;

        Some(Collision {
            point: center - radius * normal,
            normal: normal,
        })
    }
}

struct Model {
    balls: Vec<Ball>,
    _update_handle: Interval,
}

enum Msg {
    Update(f64),
    Add,
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
                let fps = 17;
                Interval::new(1000 / fps, move || {
                    link.send_message(Msg::Update(1.0 / fps as f64))
                })
            },
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::Add => {
                self.balls.push(Ball::new());

                true
            }
            Msg::Update(dt) => {
                let bounds = Rect::from_diag(&na::vector![0.0, 0.0], &get_viewbox_size().unwrap());
                for ball in self.balls.iter_mut() {
                    const G: f64 = 0.2 * 9.8;
                    let accel = na::vector![0.0, G];

                    ball.center += ball.velocity * dt + 0.5 * accel * dt * dt;

                    for Collision { point, normal } in
                        bounds.get_collisions(ball.center.clone(), ball.radius)
                    {
                        let along_normal = ball.velocity.dot(&normal);
                        ball.velocity -= normal * along_normal * 2.0;
                        ball.center = point + normal * ball.radius;

                        ball.velocity *= 0.999;
                    }

                    ball.velocity += accel * dt;

                    //if ball.velocity.norm() * dt > ball.radius {
                    //    ball.velocity = ball.velocity.normalize() * 0.9 * dt * ball.radius;
                    //}

                    //ball.velocity = ball.velocity.map(|b| if b.abs() < 1.0 { 0.0 } else { b });
                }

                for i in 0..self.balls.len() {
                    for j in 0..self.balls.len() {
                        if i == j {
                            continue;
                        }

                        if let Some(Collision { point, normal }) =
                            self.balls[i].get_collision(&self.balls[j].center, self.balls[j].radius)
                        {
                            let disp = (self.balls[i].center - self.balls[j].center).norm();
                            let needed = self.balls[i].radius + self.balls[j].radius;
                            self.balls[i].center -= normal * 0.5 * (needed - disp);
                            self.balls[j].center += normal * 0.5 * (needed - disp);

                            let i_mass = self.balls[i].mass;
                            let j_mass = self.balls[j].mass;
                            let total_mass = i_mass + j_mass;

                            let tangent = na::vector![-normal[1], normal[0]];
                            let vi_normal = self.balls[i].velocity.dot(&normal);
                            let vj_normal = self.balls[j].velocity.dot(&normal);
                            let vi_tangent = self.balls[i].velocity.dot(&tangent);
                            let vj_tangent = self.balls[j].velocity.dot(&tangent);

                            let new_vi_normal = (vi_normal * (i_mass - j_mass)
                                + 2.0 * j_mass * vj_normal)
                                / total_mass;
                            let new_vj_normal = (vj_normal * (j_mass - i_mass)
                                + 2.0 * i_mass * vi_normal)
                                / total_mass;

                            self.balls[i].velocity = new_vi_normal * normal + vi_tangent * tangent;
                            self.balls[j].velocity = new_vj_normal * normal + vj_tangent * tangent;
                        }
                    }
                }

                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let window_size = get_window_size().expect("Unable to get window size.");
        let style_string = format!(
            "width:{}px;height:{}px;background:grey",
            window_size[0], window_size[1]
        );

        let viewbox_size = get_viewbox_size().expect("Unable to get viewBox size.");
        let viewbox_string = format!("0 0 {} {}", viewbox_size[0], viewbox_size[1]);

        html! {
            <>
                <div id="container"
                    style={style_string}
                    onclick={ctx.link().callback(|_| Msg::Add)}
                >
                    <svg width="100%" height="100%" viewBox={viewbox_string}>
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
