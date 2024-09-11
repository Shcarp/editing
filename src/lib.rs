mod bounding_box;
mod events;
mod image;
mod helper;
mod renderer;
mod element;
mod object_manager;
mod scene_manager;
mod app;

use wasm_bindgen::prelude::*; 
use app::App;
use element::{Rect, RectOptions};

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

#[wasm_bindgen(start)]
pub fn wasm_main() {
    let mut app = App::new("TEST_001".to_string()); 
    
    let init_result =  app.init();
    match init_result {
        Ok(_) => {
            log("init success");
            let rect = Rect::new(RectOptions::default());
            
            app.add(rect);

            let _ = app.start_loop();
        },
        Err(_) =>{
            log("error")
        },
    }
}