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

use app::App;
use element::{Rect, RectOptions};
use wasm_bindgen::prelude::*;
use web_sys::console;

#[wasm_bindgen(start)]
pub fn wasm_main() {
    let mut app = App::new("TEST_001".to_string());

    let init_result = app.init();
    match init_result {
        Ok(_) => {
            app.scene_manager.borrow_mut().set_zoom(0.2);

            let _ = app.start_loop();

            let rows = 100;
            let cols = 100;
            let cell_size = 31.25;

            for i in 0..(rows * cols) {
                let mut rect = Rect::new(RectOptions::default());

                let row = (i / cols) as f64;
                let col = (i % cols) as f64;

                rect.set_x(col * cell_size + cell_size / 2.0)
                    .set_y(row * cell_size + cell_size / 2.0)
                    .set_height(cell_size)
                    .set_width(cell_size)
                    .set_rotation((i as f64) * 3.6);

                app.add(rect);
            }
        }
        Err(err) => console::log_1(&err),
    }
}
