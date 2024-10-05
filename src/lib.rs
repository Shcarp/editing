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

use animation::{AnimationValue, QwenAnimationBuilder};
use app::App;
use element::{Rect, RectOptions, Renderable};
use wasm_bindgen::prelude::*;
use web_sys::console;

#[wasm_bindgen(start)]
pub async fn wasm_main() {
    let mut app = App::new("TEST_001".to_string());

    let init_result = app.init();
    match init_result {
        Ok(_) => {
            let _ = app.start_loop().await;

            for i in 0..100 {
                let rect = Rect::new(RectOptions::default());

                let initial_animation = QwenAnimationBuilder::new(3.0)
                    .add_property("x", AnimationValue::Float(0.0), AnimationValue::Float((i % 13 * 100) as f64))
                    .add_property("y", AnimationValue::Float(0.0), AnimationValue::Float((i / 13 * 100) as f64))
                    .add_property("height", AnimationValue::Float(80.0), AnimationValue::Float(80.0))
                    .add_property("width", AnimationValue::Float(80.0), AnimationValue::Float(80.0))
                    .add_property("rotation", AnimationValue::Float(0.0), AnimationValue::Float((i as f64) * 3.6))
                    .set_easing(Box::new(helper::easing::ease_in_out_quad))
                    .build();
                
                app.animation_manager.borrow_mut().add_animation(rect.id().value().to_string(), Box::new(initial_animation));

                app.add(rect);
            }

            app.scene_manager.borrow_mut().set_zoom(0.8);

            app.scene_manager.borrow_mut().set_offset(100.0, 100.0);
        }
        Err(err) => console::log_1(&err),
    }
}
