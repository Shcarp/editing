use crate::{object_manager::ObjectManager, renderer::Renderer};
use std::{cell::RefCell, rc::Rc};

#[derive(Debug)]
pub struct SceneManager {
    pub object_manager: Rc<RefCell<ObjectManager>>,
}

impl SceneManager {
    pub fn new(object_manager: Rc<RefCell<ObjectManager>>) -> Self {
        Self { object_manager }
    }

    pub fn render(&self, renderer: &dyn Renderer) {
        renderer.clear_all();

        // renderer.translate(200.0, 200.0);

        let object_manager = self.object_manager.borrow();
        for object in object_manager.get_objects() {
            renderer.save();
            object.borrow_mut().render(renderer);
            renderer.restore()
        }
    }
}
