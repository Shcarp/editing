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
use source_manager::{get_source_manager, SourceType};
use wasm_bindgen::prelude::*;
use web_sys::console;
use crate::element::{ImageElement, ImageOptions};

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

            let image = ImageElement::new_from_source_key(key);
            app.add(image);
        }
        Err(err) => console::log_1(&err),
    }
}
