use std::collections::HashMap;
use crate::container::Renderable;


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
}