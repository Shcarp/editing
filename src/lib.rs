mod animation;
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
use element::{Rect, RectOptions, AnimationParams};
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
            let rect_animation = AnimationParams::default()
                .set_x(300.0)
                .set_y(300.0)
                .set_height(400.0)
                .set_width(400.0)
                .set_rotation(60.0);
            // rect.set_position(200.0, 200.0);
            rect.animate_to(rect_animation, 5.0);
            app.add(rect);
            let _ = app.start_loop();
        }
        Err(_) => log("error"),
    }
}
