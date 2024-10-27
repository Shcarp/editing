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
mod renderer;
mod scene_manager;
mod source_manager;
mod render_buffer;

use app::App;
use source_manager::{get_source_manager, SourceType};
use wasm_bindgen::prelude::*;
use web_sys::console;
use crate::element::ImageElement;

const IMAGE_URL: &str = "/assets/openart-image_QE-Sr6RL_1729828917359_raw.jpg";

#[wasm_bindgen(start)]
pub async fn wasm_main() {
    let mut app = App::new("TEST_001".to_string());

    let init_result = app.init();
    match init_result {
        Ok(_) => {
            let key = get_source_manager()
                .load_from_resource(SourceType::ImageUrl(IMAGE_URL.to_string()))
                .await.unwrap();

            app.scene_manager.borrow_mut().set_center(500.0, 500.0);

            // 创建100张图片并添加到app中
            let size = 1360.0;
            for i in 0..2 {
                let mut image = ImageElement::new_from_source_key(key.clone());
                let x = (i % 10) as f32 * size;
                let y = (i / 10) as f32 * size;
                image.set_height(size);
                image.set_width(size);
                image.set_x(x);
                image.set_y(y);
                image.set_rotation(i as f32 * 10.0);
                image.set_stroke("red".to_string());
                image.set_stroke_width(10.0);

                app.add(image);
            }
        }
        Err(err) => console::log_1(&err),
    }
}
