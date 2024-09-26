use std::cell::RefCell;
use std::fmt::Debug;
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::spawn_local;
use web_sys::console;
use web_sys::js_sys::Promise;

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

        spawn_local(async move {
            loop {
                let mut render_control = render_control_clone.borrow_mut();
                if let Some(_messages) = render_control.receive_messages().await {
                    let mut scene_manager = scene_manager_clone.borrow_mut();
                    console::log_1(&JsValue::from_str("render"));
                    scene_manager.render(0.0);
                }
            }
        });

        let scene_manager_clone = scene_manager.clone();
        let object_manager_clone = object_manager.clone();
        spawn_local(async move {
            loop {
                let delta_time = scene_manager_clone.borrow_mut().update_time();

                object_manager_clone
                    .borrow_mut()
                    .update_all(delta_time);

                let promise = Promise::new(&mut |resolve, _| {
                    request_animation_frame(&resolve);
                });
                wasm_bindgen_futures::JsFuture::from(promise).await.unwrap();
            }
        });

        Ok(())
    }
}
