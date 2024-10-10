use std::cell::{RefCell, Cell};
use std::fmt::Debug;
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::console;

use crate::element::Renderable;
use crate::events::{get_event_system, AppEvent};
use crate::helper::request_animation_frame;
use crate::object_manager::ObjectManager;
use crate::scene_manager::SceneManager;
use crate::scene_manager::SceneManagerOptions;

#[derive(Debug, Clone)]
pub struct App {
    pub object_manager: Rc<RefCell<ObjectManager>>,
    pub scene_manager: Rc<RefCell<SceneManager>>,
    render_requested: Rc<Cell<bool>>,
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
            scene_manager: scene_manager,
            render_requested: Rc::new(Cell::new(false)),
        }
    }

    pub fn init(&mut self) -> Result<(), JsValue> {
        self.scene_manager.borrow_mut().init()?;
        self.scene_manager.borrow_mut().set_context_type("2d")?;

        self.scene_manager.borrow_mut().attach(self);

        let _ = get_event_system().emit(AppEvent::READY.into(), &JsValue::NULL);
        Ok(())
    }

    pub fn request_render(&self) {
        let render_requested = self.render_requested.clone();
        let scene_manager = self.scene_manager.clone();

        let closure = Closure::wrap(Box::new(move || {
            if render_requested.get() {
                console::log_1(&"render".into());
                scene_manager.borrow_mut().render();
                render_requested.set(false);
            }
        }) as Box<dyn FnMut()>);

        if !self.render_requested.get() {
            self.render_requested.set(true);
            request_animation_frame(closure.as_ref().unchecked_ref());
        }

        closure.forget();
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
    pub fn add(&self, mut object: impl Renderable + 'static) {
        object.attach(self);
        self.object_manager.borrow_mut().add(object);
        self.request_render();
    }

    pub fn remove(&self, id: &str) -> Option<Rc<RefCell<Box<dyn Renderable>>>> {
        let res = self.object_manager.borrow_mut().remove(id);
        self.request_render();
        res
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
