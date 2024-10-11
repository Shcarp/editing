mod animation;
mod app;
mod bounding_box;
mod element;
mod event_manager;
mod events;
mod helper;
mod image;
mod object_manager;
mod render_control;
mod renderer;
mod scene_manager;
mod history;

use app::App;
use element::{Rect, RectOptions};
use wasm_bindgen::prelude::*;
use web_sys::console;

#[wasm_bindgen(start)]
pub async fn wasm_main() {
    let mut app = App::new("TEST_001".to_string());

    let init_result = app.init();
    match init_result {
        Ok(_) => {
            let center_x = 500.0;
            let center_y = 500.0;
            let radius = 400.0;
            let total_rects = 100;

            for i in 0..total_rects {
                let angle = (i as f64 / total_rects as f64) * 2.0 * std::f64::consts::PI;
                let x = center_x + radius * angle.cos();
                let y = center_y + radius * angle.sin();

                let rect = Rect::new(RectOptions {
                    x,
                    y,
                    ..Default::default()
                });

                app.add(rect);
            }

            app.scene_manager.borrow_mut().set_zoom(0.8);
            app.scene_manager.borrow_mut().set_offset(100.0, 100.0);
        }
        Err(err) => console::log_1(&err),
    }
}
