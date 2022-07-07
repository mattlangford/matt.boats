use yew::function_component;
use yew::prelude::*;

use crate::geom;

fn s<T: std::fmt::Display>(v: T) -> String {
    format!("{:.5}", v)
}

#[derive(PartialEq, Properties, Default)]
pub struct Style {
    #[prop_or(String::from("black"))]
    pub stroke: String,
    #[prop_or(String::from("0.1%"))]
    pub stroke_width: String,
    #[prop_or(String::from("black"))]
    pub fill: String,
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
    pub fn square_centered_at_with_class(x: f32, y: f32, dim: f32, class: &str) -> RectProps {
        RectProps {
            x: x - 0.5 * dim,
            y: y - 0.5 * dim,
            width: dim,
            height: dim,
            class: Some(String::from(class)),
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
