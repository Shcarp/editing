mod app_events;

pub use app_events::*;

use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::Once;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsValue;
use web_sys::js_sys::Function;

static INIT: Once = Once::new();
static mut GLOBAL_EVENT_SYSTEM: Option<EventSystem> = None;

pub fn get_event_system() -> &'static EventSystem {
    unsafe {
        INIT.call_once(|| {
            GLOBAL_EVENT_SYSTEM = Some(EventSystem::new());
        });
        GLOBAL_EVENT_SYSTEM.as_ref().unwrap()
    }
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

#[wasm_bindgen]
#[derive(Debug, Clone)]
pub struct EventData {
    event_type: String,
    payload: JsValue,
    timestamp: f64,
}

#[wasm_bindgen]
impl EventData {
    #[wasm_bindgen(constructor)]
    pub fn new(event_type: String, payload: JsValue, timestamp: f64) -> Self {
        Self {
            event_type,
            payload,
            timestamp,
        }
    }

    #[wasm_bindgen(getter)]
    pub fn event_type(&self) -> String {
        self.event_type.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn payload(&self) -> JsValue {
        self.payload.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn timestamp(&self) -> f64 {
        self.timestamp
    }
}

pub struct EventSystem {
    events: RefCell<HashMap<String, Vec<Function>>>,
}

impl EventSystem {
    pub fn new() -> Self {
        console_error_panic_hook::set_once();
        Self {
            events: RefCell::new(HashMap::new()),
        }
    }

    pub fn add_listener(&self, event_name: &str, callback: &Function) -> Result<(), JsValue> {
        self.events
            .borrow_mut()
            .entry(event_name.to_string())
            .or_insert_with(Vec::new)
            .push(callback.clone());

        log(&format!("Added listener for event: {}", event_name));
        Ok(())
    }

    pub fn emit(&self, event_name: &str, payload: &JsValue) -> Result<(), JsValue> {
        let window = web_sys::window().expect("no global `window` exists");
        let event_data = EventData::new(
            event_name.to_string(),
            payload.clone(),
            window.performance().unwrap().now(),
        );

        if let Some(listeners) = self.events.borrow().get(event_name) {
            for listener in listeners {
                listener.call1(&JsValue::NULL, &JsValue::from(event_data.clone()))?;
            }
            log(&format!(
                "Emitted event: {} with {} listeners",
                event_name,
                listeners.len()
            ));
        } else {
            log(&format!("No listeners for event: {}", event_name));
        }
        Ok(())
    }

    pub fn remove_listener(&self, event_name: &str, callback: &Function) -> Result<(), JsValue> {
        let mut events = self.events.borrow_mut();
        if let Some(listeners) = events.get_mut(event_name) {
            let initial_count = listeners.len();
            listeners.retain(|l| l != callback);
            let removed_count = initial_count - listeners.len();
            if removed_count > 0 {
                log(&format!(
                    "Removed {} listener(s) for event: {}",
                    removed_count, event_name
                ));
            } else {
                log(&format!(
                    "No matching listener found for event: {}",
                    event_name
                ));
            }
        } else {
            log(&format!("No listeners found for event: {}", event_name));
        }
        Ok(())
    }

    pub fn clear_listeners(&self, event_name: &str) -> Result<(), JsValue> {
        self.events.borrow_mut().remove(event_name);
        log(&format!("Cleared all listeners for event: {}", event_name));
        Ok(())
    }

    pub fn get_listener_count(&self, event_name: &str) -> usize {
        self.events.borrow().get(event_name).map_or(0, |v| v.len())
    }
}
