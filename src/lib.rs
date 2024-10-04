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
use element::{AnimationParams, Rect, RectOptions};
use wasm_bindgen::prelude::*;
use web_sys::console;

#[wasm_bindgen(start)]
pub fn wasm_main() {
    let mut app = App::new("TEST_001".to_string());

    let init_result = app.init();
    match init_result {
        Ok(_) => {
            // app.scene_manager.borrow_mut().set_zoom(0.1);

            let _ = app.start_loop();

            for i in 0..100 {
                let mut rect = Rect::new(RectOptions::default());

                let initial_animation = AnimationParams::default()
                    .x((i % 13 * 100) as f64)
                    .y((i / 13 * 100) as f64)
                    .height(80.0)
                    .width(80.0)
                    .rotation((i as f64) * 3.6);
                rect.animate_to(initial_animation, 3.0, animation::easing::ease_out_quad);

                let shrink_animation = AnimationParams::default()
                    .height(40.0)
                    .width(40.0)
                    .rotation((i as f64) * 7.2);
                rect.animate_to(shrink_animation, 2.0, animation::easing::ease_in_out_cubic);

                let expand_animation = AnimationParams::default()
                    .x((i % 10 * 120) as f64)
                    .y((i / 10 * 120) as f64)
                    .height(100.0)
                    .width(100.0)
                    .rotation((i as f64) * 7.2); 
                rect.animate_to(expand_animation, 2.5, animation::easing::ease_out_quad);

                app.add(rect);
            }

            app.scene_manager.borrow_mut().set_zoom(0.1);

            app.scene_manager.borrow_mut().set_offset(100.0, 100.0);

            // app.scene_manager.borrow_mut().set_rotation(0.5);
        }
        Err(err) => console::log_1(&err),
    }
}
