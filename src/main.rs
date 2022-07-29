#![allow(unused_imports)]

mod geom;
mod map;
mod svg;
mod utils;
use geom::*;
use map::*;
use utils::*;

use gloo::events::EventListener;
use gloo::timers::callback::Interval;
use nalgebra as na;
use wasm_bindgen::JsCast;
use web_sys::{EventTarget, HtmlInputElement};
use yew::events::Event;
use yew::prelude::*;

use rand::seq::SliceRandom;
use rand::Rng;
use rand::SeedableRng;

use std::collections::HashSet;

const HEIGHT: f32 = 50000.0;

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

#[derive(Properties, PartialEq)]
pub struct ValueSetterProps {
    pub callback: Callback<f64>,

    pub name: String,

    #[prop_or(0.0)]
    pub init: f64,

    #[prop_or(1.0)]
    pub step: f64,
}

struct ValueSetter {
    value: f64,
    name: String,
}
enum ValueSetterMessage {
    Inc,
    Dec,
    Set(f64),
}

impl Component for ValueSetter {
    type Message = ValueSetterMessage;
    type Properties = ValueSetterProps;

    fn create(ctx: &Context<Self>) -> Self {
        let props = ctx.props();
        Self {
            value: props.init,
            name: props.name.clone(),
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Self::Message::Inc => { self.value += ctx.props().step; }
            Self::Message::Dec => { self.value -= ctx.props().step; }
            Self::Message::Set(s) => { self.value = s; }
        }
        ctx.props().callback.emit(self.value);
        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let link = ctx.link();

        // Use batch_callback so if something unexpected happens we can return
        // None and do nothing
        let onchange = link.batch_callback(|e: Event| {
            // When events are created the target is undefined, it's only
            // when dispatched does the target get added.
            let target: Option<EventTarget> = e.target();
            // Events can bubble so this listener might catch events from child
            // elements which are not of type HtmlInputElement
            let input = target
                .and_then(|t| t.dyn_into::<HtmlInputElement>().ok())
                .map(|i| i.value().parse::<f64>().ok())
                .flatten();
            input.map(|v| Self::Message::Set(v))
        });

        html! {
            <table class="value-panel">
                <tbody width="100%">
                    <tr>
                        <td width="25%">
                            <button type="button" class="dec-button"
                                onclick={link.callback(|_| Self::Message::Dec )}>{"-"}</button>
                        </td>
                        <td width="50%">
                            <tr>
                                <div class="value-panel-text">
                                    {"test"}
                                </div>
                            </tr>
                        </td>
                        <td width="25%">
                            <button type="button" style="float:right;" class="inc-button"
                                onclick={link.callback(|_| Self::Message::Inc )}>{"+"}</button>
                        </td>
                    </tr>
                </tbody>
            </table>
        }
    }
}

struct App {
    scale: f64,
    center: Vec2d,
}

enum Msg {
    SetScale(f64),
    SetCenterX(f64),
    SetCenterY(f64)
}

impl Component for App {
    type Message = Msg;
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        Self {
            scale: 1.0,
            center: Vec2d::new(0.0, 0.0),
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Msg) -> bool {
        match msg {
            Msg::SetScale(s) => { self.scale = s; },
            Msg::SetCenterX(x) => { self.center[0] = x; },
            Msg::SetCenterY(y) => { self.center[1] = y; },
        }
        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let window_size = get_window_size().expect("Unable to get window size.");
        let viewbox = get_viewbox().unwrap();

        let style_string = format!("width:{}px;height:{}px", window_size[0], window_size[1]);

        //let mut img = image::GrayImage::new(window_size[0] as u32, window_size[1] as u32);

        //let offset = |x, y| {
        //    (self.scale * (x as f64 / window_size[0] as f64 - 0.5) + self.center[0],
        //     self.scale * (y as f64 / window_size[1] as f64 - 0.5) + self.center[1])
        // };

        //for ((cx, cy), px) in img.enumerate_pixels_mut().map(|(x, y, px)| (offset(x, y), px)) {
        //    const MAX_STEPS: usize = 32;

        //    let mut x = 0.0;
        //    let mut y = 0.0;
        //    let steps = (0..MAX_STEPS).take_while(|_| {
        //        let xn = x;
        //        let yn = y;

        //        x = xn * xn - yn * yn - cx as f64;
        //        y = 2.0 * (xn * yn).abs() - cy as f64;
        //        x * x + y * y < 10.0
        //    }).count();

        //    let v = 255.0 * (steps as f32) / (MAX_STEPS as f32);
        //    *px = image::Luma([v as u8]);
        //}

        //let mut buf = std::io::Cursor::new(Vec::new());
        //img.write_to(&mut buf, image::ImageOutputFormat::Png).unwrap();
        //let res_base64 = base64::encode(&buf.into_inner());

        html! {
            <div id="container" style={style_string}>
                //<img src={format!("data:image/png;base64,{}", res_base64)}/>

                <div id="panel">
                    <ValueSetter name="scale" init=1.0 callback={ctx.link().callback(|v| Msg::SetScale(v))}/>
                </div>

            </div>
        }
    }
}

fn main() {
    log!("Starting model...");
    yew::start_app::<App>();
}
