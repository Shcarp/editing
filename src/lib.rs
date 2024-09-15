mod animation;
mod app;
mod bounding_box;
mod element;
mod event_manager;
mod events;
mod helper;
mod image;
mod object_manager;
mod renderer;
mod scene_manager;
mod render_control;

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
            console::log_1(&JsValue::from_str("init success"));
            app.scene_manager.borrow_mut().pan(100.0, 100.0);

            // 创建100个矩形并添加动画
            for i in 0..100 {
                let mut rect = Rect::new(RectOptions::default());
                
                // 初始位置动画
                let initial_animation = AnimationParams::default()
                    .set_x((i % 10 * 100) as f64)
                    .set_y((i / 10 * 100) as f64)
                    .set_height(80.0)
                    .set_width(80.0)
                    .set_rotation((i as f64) * 3.6);
                rect.animate_to(initial_animation, 3.0, animation::easing::ease_out_quad);
                
                // 第二段动画：缩小并旋转
                let shrink_animation = AnimationParams::default()
                    .set_height(40.0)
                    .set_width(40.0)
                    .set_rotation((i as f64) * 7.2);
                rect.animate_to(shrink_animation, 2.0, animation::easing::ease_in_out_cubic);
                
                // 第三段动画：放大并移动（从第二段动画的结束状态开始）
                let expand_animation = AnimationParams::default()
                    .set_x((i % 10 * 120) as f64)
                    .set_y((i / 10 * 120) as f64)
                    .set_height(100.0)
                    .set_width(100.0)
                    .set_rotation((i as f64) * 7.2); // 保持第二段动画的旋转角度
                rect.animate_to(expand_animation, 2.5, animation::easing::ease_out_elastic);
                
                app.add(rect);
            }

            let _ = app.start_loop();
        }
        Err(err) => console::log_1(&err),
    }
}
