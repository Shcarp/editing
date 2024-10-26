mod animation;
mod app;
mod bounding_box;
mod element;
mod event_manager;
mod events;
mod helper;
mod history;
mod image;
mod object_manager;
mod render_control;
mod renderer;
mod scene_manager;
mod source_manager;

use app::App;
use element::{Rect, RectOptions};
use wasm_bindgen::prelude::*;
use web_sys::{window, console};
use wasm_timer::Instant;

#[wasm_bindgen(start)]
pub async fn wasm_main() {
    let mut app = App::new("TEST_001".to_string());

    let init_result = app.init();
    match init_result {
        Ok(_) => {
            let start_time = Instant::now();

            let center_x = 500.0;
            let center_y = 500.0;
            let radius = 400.0;
            let total_rects = 10000;

            let setup_time = start_time.elapsed();
            console::log_1(&format!("Setup time: {:?}", setup_time).into());

            let loop_start = Instant::now();
            for i in 0..total_rects {
                let angle = (i as f64 / total_rects as f64) * 2.0 * std::f64::consts::PI;
                let x: f64 = center_x + radius * angle.cos();
                let y = center_y + radius * angle.sin();

                let rect = Rect::new(RectOptions {
                    x,
                    y,
                    ..Default::default()
                });

                app.add(rect);
            }
            let loop_time = loop_start.elapsed();
            console::log_1(&format!("Loop time: {:?}", loop_time).into());

            let render_start = Instant::now();
            app.request_render();
            let render_time = render_start.elapsed();
            console::log_1(&format!("Render request time: {:?}", render_time).into());

            let total_time = start_time.elapsed();
            console::log_1(&format!("Total execution time: {:?}", total_time).into());
        }
        Err(err) => console::log_1(&err),
    }
}
