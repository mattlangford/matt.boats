use wasm_bindgen::JsCast;
use yew::prelude::*;

use rand::seq::SliceRandom;
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

enum Msg {}

struct Model {
    url: String,
}

impl Component for Model {
    type Message = Msg;
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        Self {
            url: load_background_image_url(),
        }
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        html! {
            <div class="container">
                <img src={String::clone(&self.url)}/>
            </div>
        }
    }
}

fn main() {
    log!("Starting model...");
    yew::start_app::<Model>();
}
