use crate::{object_manager::ObjectManager, renderer::Renderer};
use std::{cell::RefCell, rc::Rc};
use wasm_timer::Instant;

#[derive(Debug)]
pub struct SceneManager {
    pub object_manager: Rc<RefCell<ObjectManager>>,
    last_update: Instant,
}

impl SceneManager {
    pub fn new(object_manager: Rc<RefCell<ObjectManager>>) -> Self {
        Self {
            object_manager,
            last_update: Instant::now(),
        }
    }

    pub fn render(&mut self, renderer: &dyn Renderer) {
        let now = Instant::now();
        let delta_time = (now - self.last_update).as_secs_f64();
        // We can't modify self.last_update here because self is a shared reference
        // Instead, we'll need to store the current time and use it later
        self.last_update = now;

        renderer.clear_all();
        renderer.save();
        renderer.translate(100.0, 100.0);

        let object_manager = self.object_manager.borrow();
        for object in object_manager.get_objects() {
            renderer.save();
            object.borrow_mut().render(renderer, delta_time);
            renderer.restore()
        }

        renderer.restore();
    }
}
