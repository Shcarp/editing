use std::cell::{RefCell, Cell};
use std::fmt::Debug;
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

use crate::element::Renderable;
use crate::events::{get_event_system, AppEvent};
use crate::helper::request_animation_frame;
use crate::history::History;
use crate::object_manager::ObjectManager;
use crate::scene_manager::SceneManager;
use crate::scene_manager::SceneManagerOptions;

#[derive(Debug, Clone)]
pub struct App {
    pub history: Rc<RefCell<History>>,
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
            history: Rc::new(RefCell::new(History::new())), 
            object_manager: object_manager,
            scene_manager: scene_manager,
            render_requested: Rc::new(Cell::new(false)),
        }
    }

    pub fn init(&mut self) -> Result<(), JsValue> {
        self.scene_manager.borrow_mut().init()?;
        self.scene_manager.borrow_mut().attach(self);
        self.history.borrow_mut().attach(&self);
        self.object_manager.borrow_mut().attach(self);

        let _ = get_event_system().emit(AppEvent::READY.into(), &JsValue::NULL);
        Ok(())
    }

    pub fn request_render(&self) {
        if self.render_requested.get() {
            return;
        }

        self.render_requested.set(true);
        let render_requested = Rc::clone(&self.render_requested);
        let scene_manager = Rc::clone(&self.scene_manager);

        let closure = Rc::new(RefCell::new(None));
        let closure_clone = Rc::clone(&closure);

        *closure.borrow_mut() = Some(Closure::wrap(Box::new(move || {
            if render_requested.get() {
                scene_manager.borrow_mut().render();
                render_requested.set(false);
            }
            closure_clone.borrow_mut().take();
        }) as Box<dyn FnMut()>));

        request_animation_frame(closure.borrow().as_ref().unwrap().as_ref().unchecked_ref());
    }

    pub fn reset_to_initial_state(&self) {
        self.object_manager.borrow_mut().clear();
        self.scene_manager.borrow_mut().reset_to_initial_state();
    }
}

impl App {
    pub fn add(&self, mut object: impl Renderable + 'static) {
        object.attach(self);
        self.object_manager.borrow_mut().add(Box::new(object));
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
