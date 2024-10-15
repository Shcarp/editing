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
use helper::request_animation_frame;
use wasm_bindgen::prelude::*;
use web_sys::{console, window};
use std::rc::Rc;
use std::cell::{RefCell, Cell};

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
                let x: f64 = center_x + radius * angle.cos();
                let y = center_y + radius * angle.sin();

                let rect = Rect::new(RectOptions {
                    x,
                    y,
                    ..Default::default()
                });

                app.add(rect);
            }

            let frame_count = Rc::new(Cell::new(0));
            let zoom = Rc::new(Cell::new(0.8));
            let rotation_center_x = Rc::new(Cell::new(500.0)); 
            let rotation_center_y = Rc::new(Cell::new(500.0));

            let f: Rc<RefCell<Option<Closure<dyn FnMut(f64)>>>> = Rc::new(RefCell::new(None));
            let g = f.clone();

            let zoom_clone = zoom.clone();
            let rotation_center_x_clone = rotation_center_x.clone();
            let rotation_center_y_clone = rotation_center_y.clone();

            let app_clone = app.clone();
            *g.borrow_mut() = Some(Closure::wrap(Box::new(move |timestamp: f64| {
                let current_frame = frame_count.get();
                frame_count.set(current_frame + 1);

                // 动态改变缩放比例
                let current_zoom = zoom_clone.get();
                let zoom_factor = (current_frame as f64 * 0.002).sin(); // 增加频率
                let new_zoom = current_zoom + zoom_factor * 0.2; // 增加幅度
                let new_zoom = new_zoom.max(0.5).min(2.0); // 保持缩放范围限制
                zoom_clone.set(new_zoom);
                app_clone.scene_manager.borrow_mut().set_zoom(new_zoom);

                // 动态改变旋转中心
                let frame_f64 = current_frame as f64;
                let new_center_x = 700.0 + (frame_f64 * 0.02).cos() * 100.0;
                let new_center_y = 700.0 + (frame_f64 * 0.02).sin() * 100.0;
                rotation_center_x_clone.set(new_center_x);
                rotation_center_y_clone.set(new_center_y);
                app_clone.scene_manager.borrow_mut().set_center(new_center_x, new_center_y);
                
                let rotation_speed = 0.01;
                app_clone.scene_manager.borrow_mut().update_rotation(rotation_speed);

                app_clone.request_render();
                if frame_count.get() % 100 == 0 {
                    app_clone.history.borrow_mut().ensure_current_unit_finalized();
                    let summary = app_clone.history.borrow().get_history_summary();

                    match summary {
                        Ok(summary) => {
                            console::log_1(&summary);
                        }
                        Err(err) => {
                            console::log_1(&err);
                        }
                    }
                }

                request_animation_frame(f.borrow().as_ref().unwrap().as_ref().unchecked_ref());
            }) as Box<dyn FnMut(f64)>));
            request_animation_frame(g.borrow().as_ref().unwrap().as_ref().unchecked_ref());
        }
        Err(err) => console::log_1(&err),
    }
}
