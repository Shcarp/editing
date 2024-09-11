use std::{cell::RefCell,  rc::Rc};
use crate::{ object_manager::ObjectManager, renderer::Renderer};


#[derive(Debug)]
pub struct SceneManager {
    pub object_manager: Rc<RefCell<ObjectManager>>,
}

impl SceneManager {
    pub fn new(object_manager: Rc<RefCell<ObjectManager>>) -> Self {
        Self {
            object_manager,
        }
    }

    pub fn render(&self, renderer: &dyn Renderer) {
        let object_manager = self.object_manager.borrow();
        for object in object_manager.get_objects() {
            renderer.save();
            object.borrow().render(renderer);
            renderer.restore()
        }
    }
}
