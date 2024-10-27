use crate::{
    app::App, element::Renderable, history::{ElementHistoryItem, HistoryItem}
};
use glam::DVec2;
use serde_json::Value;
use std::{
    cell::RefCell,
    collections::{HashMap, VecDeque},
    rc::Rc,
};

#[derive(Debug)]
struct ObjectData {
    object: Rc<RefCell<Box<dyn Renderable>>>,
    last_update: f64,
    position: DVec2,
}

#[derive(Debug)]
pub struct ObjectManager {
    app: Option<App>,
    objects: HashMap<String, ObjectData>,
    update_queue: VecDeque<String>,
    total_time: f64,
}

impl ObjectManager {
    pub fn new() -> Self {
        Self {
            objects: HashMap::new(),
            update_queue: VecDeque::new(),
            total_time: 0.0,
            app: None,
        }
    }

    pub fn attach(&mut self, app: &App) {
        self.app = Some(app.clone());
    }

    pub fn add(&mut self, mut object: Box<dyn Renderable>) {
        if let Some(app) = &self.app {
            object.attach(app);
            let id = object.id().value().to_string();
            let object_id = object.id().value().to_string();
            let object_type = object.get_type().to_string();
            let object_value = object.to_value();
            let position = DVec2::new(object.position().0 as f64, object.position().1 as f64);
            let object_data = ObjectData {
                object: Rc::new(RefCell::new(object)),
                last_update: self.total_time,
                position,
            };
    
            self.objects.insert(id.clone(), object_data);
            self.update_queue.push_back(id);
            let item = ElementHistoryItem::new(object_id, object_type, object_value);
            app.history.borrow_mut().push(HistoryItem::AddElement(item));
        }

    }

    pub fn remove(&mut self, id: &str) -> Option<Rc<RefCell<Box<dyn Renderable>>>> {
        if let Some(app) = &self.app {
            if let Some(object_data) = self.objects.remove(id) {
                self.update_queue.retain(|queue_id| queue_id != id);
    
                let object = object_data.object;
                let object_id = object.borrow().id().value().to_string();
                let object_type = object.borrow().get_type().to_string();
                let object_value = object.borrow().to_value();
                let item = ElementHistoryItem::new(object_id, object_type, object_value);
                app.history.borrow_mut().push(HistoryItem::RemoveElement(item));
    
                Some(object)
            } else {
                None
            }
        } else {
            None
        }
    }

    pub fn get(&self, id: &str) -> Option<Rc<RefCell<Box<dyn Renderable>>>> {
        if let Some(object_data) = self.objects.get(id) {
            Some(object_data.object.clone())
        } else {
            None
        }
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
        self.update_queue.clear();
    }

    pub fn iter(&self) -> impl Iterator<Item = (&String, &Rc<RefCell<Box<dyn Renderable>>>)> {
        self.objects.iter().map(|(id, data)| (id, &data.object))
    }

    pub fn get_objects(&self) -> Vec<Rc<RefCell<Box<dyn Renderable>>>> {
        self.objects
            .iter()
            .map(|(_, data)| data.object.clone())
            .collect()
    }

    pub fn update_object(&mut self, id: String, data: Value) {
        if let Some(object_data) = self.objects.get_mut(&id) {
            let mut object = object_data.object.borrow_mut();
            object.update(data);
        }
    }
}
