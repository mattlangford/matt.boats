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
