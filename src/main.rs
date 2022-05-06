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

struct CookieManager {
    doc: web_sys::HtmlDocument,
}

impl CookieManager {
    fn create() -> Self {
        let window = web_sys::window().unwrap();
        let document = window.document().unwrap();
        Self {
            doc: document
                .dyn_into::<web_sys::HtmlDocument>()
                .expect("Unable to cast to HtmlDocument."),
        }
    }

    fn save(&self, key: &str, data: &str) {
        let cookie_string = format!("{}={}", key, data);
        self.doc
            .set_cookie(&cookie_string)
            .expect("Unable to set cookie.");
    }

    fn load(&self, key: &str) -> Option<String> {
        let cookies = self.doc.cookie().expect("Unable to load cookies.");
        cookies
            .split(";")
            .find(|c| c.trim().starts_with(key))
            .and_then(|c| c.split_once("="))
            .and_then(|(_, v)| Some(String::from(v)))
    }
}

fn load_image_urls() -> Vec<String> {
    [
        "https://imgur.com/se9hSxe.jpeg",
        "https://imgur.com/Tvh8kPD.jpeg",
        "https://imgur.com/LCUkOWx.jpeg",
        "https://imgur.com/3JrrUbO.jpeg",
        "https://imgur.com/oNTzoBn.jpeg",
        "https://imgur.com/uLaowT7.jpeg",
        "https://imgur.com/YfHyEje.jpeg",
        "https://imgur.com/ZqNZ01Q.jpeg",
        "https://imgur.com/LIsTr9Z.jpeg",
        "https://imgur.com/3TG5xmA.jpeg",
        "https://imgur.com/8zPRRdY.jpeg",
        "https://imgur.com/OoZkItp.jpeg",
        "https://imgur.com/ZarQdl4.jpeg",
        "https://imgur.com/HLFHAep.jpeg",
    ]
    .iter()
    .map(|&s| String::from(s))
    .collect()
}

fn load_background_image_url() -> String {
    let mut urls = load_image_urls();

    let cookies = CookieManager::create();

    let default_seed = || {
        let now = chrono::Local::now();
        now.timestamp_subsec_nanos() as u64
    };

    let c_seed = cookies
        .load("seed")
        .and_then(|s| s.parse::<u64>().ok())
        .unwrap_or(default_seed());
    let c_index = cookies
        .load("index")
        .and_then(|s| s.parse::<usize>().ok())
        .unwrap_or(0);

    let mut rng = rand::rngs::StdRng::seed_from_u64(c_seed);
    urls.shuffle(&mut rng);

    let index = (c_index + 1)
        .checked_rem(urls.len())
        .expect("Invalid length from load_image_urls()");
    log!("Seed: {}, index: {}, url: {}", c_seed, index, urls[index]);

    cookies.save("seed", &format!("{}", c_seed));
    cookies.save("index", &format!("{}", index));
    urls.swap_remove(index)
}

enum Msg {
    Update(f64),
}

struct Model {
    url: String,

    left_top: na::Vector2<f64>,
    velocity: na::Vector2<f64>,

    _update_handle: Interval,
}

impl Component for Model {
    type Message = Msg;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        let s = 500.0;
        Self {
            url: load_background_image_url(),
            left_top: na::vector![100.0, 100.0],
            velocity: na::vector![
                rand::thread_rng().gen_range(-s..s),
                rand::thread_rng().gen_range(-s..s)
            ],
            _update_handle: {
                let link = ctx.link().clone();
                let fps = 30;
                Interval::new(1000 / fps, move || {
                    link.send_message(Msg::Update(1.0 / fps as f64))
                })
            },
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::Update(dt) => {
                let window = web_sys::window().unwrap();
                let document = window.document().unwrap();
                let w_height = window
                    .inner_height()
                    .ok()
                    .and_then(|v| v.as_f64())
                    .unwrap_or(10.0);
                let w_width = window
                    .inner_width()
                    .ok()
                    .and_then(|v| v.as_f64())
                    .unwrap_or(20.0);
                let image = document
                    .get_element_by_id("image")
                    .expect("Unable to find image.");
                let i_height = image.client_height() as f64;
                let i_width = image.client_width() as f64;

                self.left_top += dt * self.velocity;
                let max_height = w_height - i_height;
                let max_width = w_width - i_width;
                if self.left_top[0] < 0.0 || self.left_top[0] >= max_width {
                    self.velocity[0] *= -rand::thread_rng().gen_range(0.9..1.1);
                    self.left_top[0] = self.left_top[0].min(max_width).max(0.0)
                }
                if self.left_top[1] < 0.0 || self.left_top[1] > max_height {
                    self.velocity[1] *= -rand::thread_rng().gen_range(0.9..1.1);
                    self.left_top[1] = self.left_top[1].min(max_height).max(0.0)
                }
                true
            }
        }
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        let style_string = format!("left:{}px;top:{}px;", self.left_top[0], self.left_top[1]);
        html! {
            <>
                <div id="container" style={style_string}>
                    <img id="image" src={String::clone(&self.url)}/>
                </div>
            </>
        }
    }
}

fn main() {
    log!("Starting model...");
    yew::start_app::<Model>();
}
