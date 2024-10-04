use std::cell::RefCell;
use std::fmt::Debug;
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::spawn_local;
use web_sys::console;
use web_sys::js_sys::Promise;
use wasm_timer::Instant;

use crate::element::Renderable;
use crate::events::{get_event_system, AppEvent};
use crate::helper::request_animation_frame;
use crate::object_manager::ObjectManager;
use crate::render_control::get_render_control;
use crate::scene_manager::SceneManager;
use crate::scene_manager::SceneManagerOptions;

#[derive(Debug)]
pub struct App {
    pub object_manager: Rc<RefCell<ObjectManager>>,
    pub scene_manager: Rc<RefCell<SceneManager>>,
}

impl App {
    pub fn new(canvas_id: String) -> Self {
        let object_manager = Rc::new(RefCell::new(ObjectManager::new()));
        let mut options = SceneManagerOptions::default();
        options.canvas_id = canvas_id;
        options.object_manager = object_manager.clone();

        let scene_manager = Rc::new(RefCell::new(SceneManager::new(options)));

        Self {
            object_manager: object_manager,
            scene_manager,
        }
    }

    pub fn init(&mut self) -> Result<(), JsValue> {
        self.scene_manager.borrow_mut().init()?;
        self.scene_manager.borrow_mut().set_context_type("2d")?;
        let _ = get_event_system().emit(AppEvent::READY.into(), &JsValue::NULL);
        Ok(())
    }

    pub fn is_support_type(&self, context_type: &str) -> bool {
        let window = web_sys::window().expect("Should have a window in this context");
        let document = window.document().expect("Should have a document on window");
        let canvas = document
            .create_element("canvas")
            .expect("Should be able to create a canvas");
        let canvas: web_sys::HtmlCanvasElement =
            canvas.dyn_into::<web_sys::HtmlCanvasElement>().unwrap();

        match context_type {
            "2d" => canvas.get_context(context_type).is_ok(),
            "webgl2" => canvas.get_context(context_type).is_ok(),
            _ => false,
        }
    }
}

impl App {
    pub fn add(&self, object: impl Renderable + 'static) {
        self.object_manager.borrow_mut().add(object);
    }

    pub fn remove(&self, id: &str) -> Option<Rc<RefCell<Box<dyn Renderable>>>> {
        self.object_manager.borrow_mut().remove(id)
    }

    pub fn get(&self, id: &str) -> Option<Rc<RefCell<Box<dyn Renderable>>>> {
        self.object_manager.borrow().get(id)
    }

    pub fn contains(&self, id: &str) -> bool {
        self.object_manager.borrow().contains(id)
    }

    pub fn len(&self) -> usize {
        self.object_manager.borrow().len()
    }

    pub fn is_empty(&self) -> bool {
        self.object_manager.borrow().is_empty()
    }

    pub fn clear(&self) {
        self.object_manager.borrow_mut().clear();
    }

    pub fn get_objects(&self) -> Vec<Rc<RefCell<Box<dyn Renderable>>>> {
        let res = self.object_manager.borrow().get_objects().clone();
        res
    }
}

impl App {
    pub fn start_loop(&self) -> Result<(), JsValue> {
        let scene_manager: Rc<RefCell<SceneManager>> = self.scene_manager.clone();
        let object_manager: Rc<RefCell<ObjectManager>> = self.object_manager.clone();
        let render_control = Rc::new(RefCell::new(get_render_control()));

        let render_control_clone = render_control.clone();
        let scene_manager_clone = scene_manager.clone();

        let update_object_manager = object_manager.clone();
        spawn_local(async move {
            let mut loop_count = 0;
            let mut total_receive_time = std::time::Duration::new(0, 0);
            let mut total_update_time = std::time::Duration::new(0, 0);
            let mut total_render_time = std::time::Duration::new(0, 0);
            let mut total_loop_time = std::time::Duration::new(0, 0);

            loop {
                let loop_start = Instant::now();

                let mut render_control = render_control_clone.borrow_mut();
                let receive_start = Instant::now();
                if let Some(_messages) = render_control.receive_messages().await {
                    let receive_duration = receive_start.elapsed();
                    total_receive_time += receive_duration;

                    let update_start = Instant::now();
                    update_object_manager.borrow_mut().update_object_from_message(&_messages);
                    let update_duration = update_start.elapsed();
                    total_update_time += update_duration;

                    let render_start = Instant::now();
                    let scene_manager = scene_manager_clone.borrow_mut();
                    scene_manager.render();
                    let render_duration = render_start.elapsed();
                    total_render_time += render_duration;
                }

                let loop_duration = loop_start.elapsed();
                total_loop_time += loop_duration;

                loop_count += 1;

                if loop_count % 100 == 0 {
                    console::log_1(&format!("Average times over {} loops:", loop_count).into());
                    console::log_1(&format!("Receive messages: {:?}", total_receive_time / loop_count).into());
                    console::log_1(&format!("Update objects: {:?}", total_update_time / loop_count).into());
                    console::log_1(&format!("Render: {:?}", total_render_time / loop_count).into());
                    console::log_1(&format!("Total loop: {:?}", total_loop_time / loop_count).into());
                    console::log_1(&"------------------------".into());
                }
            }
        });

        let scene_manager_clone = scene_manager.clone();
        let object_manager_clone = object_manager.clone();
        spawn_local(async move {
            let mut loop_count = 0;
            let mut total_update_time = std::time::Duration::new(0, 0);
            let mut total_loop_time = std::time::Duration::new(0, 0);
            loop {
                let loop_start = Instant::now();
                let delta_time = scene_manager_clone.borrow_mut().update_time();
                let update_start = Instant::now();
                object_manager_clone.borrow_mut().update_all(delta_time);
                let update_duration = update_start.elapsed();
                total_update_time += update_duration;

                let promise = Promise::new(&mut |resolve, _| {
                    request_animation_frame(&resolve);
                });

                let loop_duration = loop_start.elapsed();
                total_loop_time += loop_duration;

                wasm_bindgen_futures::JsFuture::from(promise).await.unwrap();
                loop_count += 1;

                if loop_count % 100 == 0 {
                    console::log_1(&format!("Average times over {} loops:", loop_count).into());
                    console::log_1(&format!("Update objects: {:?}", total_update_time / loop_count).into());
                    console::log_1(&format!("Total loop: {:?}", total_loop_time / loop_count).into());
                    console::log_1(&"------------------------".into());
                }
            }
        });

        Ok(())
    }
}
