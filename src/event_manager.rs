use std::collections::HashMap;

pub struct EventManager {
    listeners: HashMap<String, Vec<Box<dyn Fn()>>>,
}

impl EventManager {
    pub fn new() -> Self {
        EventManager {
            listeners: HashMap::new(),
        }
    }

    pub fn add_listener(&mut self, event_type: &str, callback: Box<dyn Fn()>) {
        self.listeners
            .entry(event_type.to_string())
            .or_insert_with(Vec::new)
            .push(callback);
    }

    pub fn remove_listener(&mut self, event_type: &str) {
        self.listeners.remove(event_type);
    }

    pub fn trigger(&self, event_type: &str) {
        if let Some(callbacks) = self.listeners.get(event_type) {
            for callback in callbacks {
                callback();
            }
        }
    }
}
