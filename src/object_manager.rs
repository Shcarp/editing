use std::{cell::RefCell, collections::HashMap, rc::Rc};
use crate::element::Renderable;

#[derive(Debug)]
pub struct ObjectManager {
    pub objects: HashMap<String, Rc<RefCell<Box<dyn Renderable>>>>,
}

impl ObjectManager {
    pub fn new() -> Self {
        Self {
            objects: HashMap::new(),
        }
    }

    pub fn add<T>(&mut self, object: T) where 
        T:  Renderable + 'static
    {
        let id = object.id().0.clone();
        let value: Rc<RefCell<Box<dyn Renderable>>> = Rc::new(RefCell::new(Box::new(object)));

        self.objects.insert(id, value);
    }

    pub fn remove(&mut self, id: &str) -> Option<Rc<RefCell<Box<dyn Renderable>>>> {
        self.objects.remove(id)
    }

    pub fn get(&self, id: &str) -> Option<Rc<RefCell<Box<dyn Renderable>>>> {
        self.objects.get(id).cloned()
    }

    pub fn contains(&self, id: &str) -> bool {
        self.objects.contains_key(id)
    }

    pub fn len(&self) -> usize {
        self.objects.len()
    }

    pub fn is_empty(&self) -> bool {
        self.objects.is_empty()
    }

    pub fn clear(&mut self) {
        self.objects.clear();
    }

    pub fn iter(&self) -> impl Iterator<Item = (&String, &Rc<RefCell<Box<dyn Renderable>>>)> {
        self.objects.iter()
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = (&String, &mut Rc<RefCell<Box<dyn Renderable>>>)> {
        self.objects.iter_mut()
    }

    pub fn update(&mut self, id: &str, update_fn: impl FnOnce(&mut Rc<RefCell<Box<dyn Renderable>>>)) -> bool {
        if let Some(object) = self.objects.get_mut(id) {
            update_fn(object);
            true
        } else {
            false
        }
    }

    pub fn get_objects(&self) -> Vec<Rc<RefCell<Box<dyn Renderable>>>> {
        self.objects.values().cloned().collect()
    }
}