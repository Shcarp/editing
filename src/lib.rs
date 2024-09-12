mod app;
mod bounding_box;
mod element;
mod events;
mod helper;
mod image;
mod object_manager;
mod renderer;
mod scene_manager;

use app::App;
use element::{Rect, RectOptions, Transformable};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

#[wasm_bindgen(start)]
pub fn wasm_main() {
    let mut app = App::new("TEST_001".to_string());

    let init_result = app.init();
    match init_result {
        Ok(_) => {
            log("init success");
            let mut rect = Rect::new(RectOptions::default());
            rect.translate(100.0, 100.0);
            app.add(rect);
            let _ = app.start_loop();
        }
        Err(_) => log("error"),
    }
}
