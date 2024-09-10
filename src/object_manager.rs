use std::collections::HashMap;
use crate::{container::Renderable, renderer::Renderer};


#[derive(Debug)]
pub struct ObjectManager {
    pub objects: HashMap<String, Box<dyn Renderable>>,
}

impl ObjectManager {
    pub fn new() -> Self {
        Self {
            objects: HashMap::new(),
        }
    }

    pub fn render(&self, renderer: &dyn Renderer) {
        for object in self.objects.values() {
            object.render(renderer);
        }
    }
}