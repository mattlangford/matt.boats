#![allow(dead_code)]

use yew::function_component;
use yew::prelude::*;

use crate::geom;
use crate::utils::*;

pub fn s<T: std::fmt::Display>(v: T) -> String {
    format!("{:.5}", v)
}

#[derive(PartialEq, Properties)]
pub struct LineProps {
    pub x1: f32,
    pub y1: f32,
    pub x2: f32,
    pub y2: f32,
    #[prop_or_default]
    pub class: Option<String>,
}

impl LineProps {
    pub fn from_line(line: &geom::Line) -> Self {
        let start = line.start();
        let end = line.end();
        Self {
            x1: start[0],
            y1: start[1],
            x2: end[0],
            y2: end[1],
            class: None,
        }
    }

    pub fn with_class(mut self, class: &str) -> Self {
        self.class = Some(String::from(class));
        self
    }
}

#[function_component(Line)]
pub fn line(props: &LineProps) -> Html {
    html! {
        <line x1={s(props.x1)} x2={s(props.x2)} y1={s(props.y1)} y2={s(props.y2)}
              class={props.class.clone().unwrap_or_default()}/>
    }
}

#[derive(PartialEq, Properties)]
pub struct RectProps {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    #[prop_or_default]
    pub class: Option<String>,
}

impl RectProps {
    pub fn square(center: &geom::Vec2f, dim: f32) -> RectProps {
        RectProps {
            x: center.x - 0.5 * dim,
            y: center.y - 0.5 * dim,
            width: dim,
            height: dim,
            class: None,
        }
    }

    pub fn from_aabox(b: &geom::AABox) -> RectProps {
        RectProps {
            x: b.start[0],
            y: b.start[1],
            width: b.dim[0],
            height: b.dim[1],
            class: None,
        }
    }

    pub fn with_class(mut self, class: &str) -> RectProps {
        self.class = Some(String::from(class));
        self
    }
}

#[function_component(Rect)]
pub fn rect(props: &RectProps) -> Html {
    html! {
        <rect
            class={props.class.clone().unwrap_or_default()}
            x={s(props.x)}
            y={s(props.y)}
            height={s(props.height)}
            width={s(props.width)}/>
    }
}

#[derive(PartialEq, Properties)]
pub struct CircleProps {
    pub x: f32,
    pub y: f32,
    pub radius: f32,
    #[prop_or_default]
    pub class: String,
    #[prop_or([255, 255, 255])]
    pub fill: [u8; 3],
    #[prop_or(1.0)]
    pub alpha: f32,
    #[prop_or_default]
    pub filter: String,
}

impl CircleProps {
    pub fn new(pt: geom::Vec2f, r: f32) -> Self {
        Self {
            x: pt[0],
            y: pt[1],
            radius: r,
            class: String::new(),
            fill: [0, 0, 0],
            alpha: 1.0,
            filter: String::new()
        }
    }
    pub fn from_circle(circle: &geom::Circle) -> Self {
        Self::new(circle.center, circle.radius)
    }

    pub fn with_class(mut self, class: &str) -> Self {
        self.class = String::from(class);
        self
    }

    pub fn with_fill(mut self, fill: [u8; 3]) -> Self {
        self.fill = fill;
        self
    }
    pub fn with_alpha(mut self, alpha: f32) -> Self {
        self.alpha = alpha;
        self
    }
    pub fn with_filter(mut self, filter: &str) -> Self {
        self.filter = String::from(filter);
        self
    }
}

#[function_component(Circle)]
pub fn circle(props: &CircleProps) -> Html {
    let [r, g, b] = props.fill;
    let fill = format!("rgba({}, {}, {}, {})", r, g, b, (255.0 * props.alpha) as u8);
    html! {
        <circle
            cx={s(props.x)}
            cy={s(props.y)}
            r={s(props.radius)}
            class={props.class.clone()}
            fill={fill}
            filter={props.filter.clone()}
        />
    }
}
