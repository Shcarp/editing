use std::{cell::RefCell, fmt::Debug, rc::Rc};
use serde_json::Value;
use web_sys::{console, js_sys};
use wasm_timer::Instant;
use crate::{app::App, helper::create_element};
use wasm_bindgen::prelude::*;
use serde::{Serialize, Deserialize};

#[derive(Clone, Debug)]
pub struct ObjectHistoryItem {
    pub undo_data: Value, 
    pub redo_data: Value,
    pub timestamp: f64,
    pub object_id: String,
}

impl ObjectHistoryItem {
    pub fn new(object_id: String, undo_data: Value, redo_data: Value) -> Self {
        Self {
            undo_data,
            redo_data,
            timestamp: js_sys::Date::now(),
            object_id,
        }
    }
}

#[derive(Clone, Debug)]
pub struct SceneHistoryItem {
    pub undo_data: Value,
    pub redo_data: Value,
    pub timestamp: f64,
}

impl SceneHistoryItem {
    pub fn new(undo_data: Value, redo_data: Value) -> Self {
        Self {
            undo_data,
            redo_data,
            timestamp: js_sys::Date::now(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct ElementHistoryItem {
    pub element_id: String,
    pub element_type: String,
    pub element_data: Value,
    pub timestamp: f64,
}

impl ElementHistoryItem {
    pub fn new(element_id: String, element_type: String, element_data: Value) -> Self {
        Self {
            element_id,
            element_type,
            element_data,
            timestamp: js_sys::Date::now(),
        }
    }
}

#[derive(Debug)]
pub enum HistoryItem {
    ObjectUpdate(ObjectHistoryItem),
    SceneUpdate(SceneHistoryItem),
    AddElement(ElementHistoryItem),
    RemoveElement(ElementHistoryItem),
}

pub struct HistoryUnit {
    items: Vec<HistoryItem>,
    timestamp: f64,
}

#[derive(Clone)]
pub struct History {
    app: Option<App>,
    undo_stack: Rc<RefCell<Vec<HistoryUnit>>>,
    redo_stack: Rc<RefCell<Vec<HistoryUnit>>>,
    current_unit: Rc<RefCell<Option<HistoryUnit>>>,
    last_push_time: Rc<RefCell<Instant>>,

    is_undoing: bool,
    is_redoing: bool,
}

impl Debug for History {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "History {{")?;
        writeln!(f, "  undo_stack: [")?;
        for (i, unit) in self.undo_stack.borrow().iter().enumerate() {
            writeln!(f, "    Unit {}: {} items", i, unit.items.len())?;
        }
        writeln!(f, "  ],")?;
        writeln!(f, "  redo_stack: [")?;
        for (i, unit) in self.redo_stack.borrow().iter().enumerate() {
            writeln!(f, "    Unit {}: {} items", i, unit.items.len())?;
        }
        write!(f, "  ]")?;
        write!(f, "}}")
    }
}

impl History {
    pub fn new() -> Self {
        Self {
            undo_stack: Rc::new(RefCell::new(Vec::new())),
            redo_stack: Rc::new(RefCell::new(Vec::new())),
            app: None,
            current_unit: Rc::new(RefCell::new(None)),
            last_push_time: Rc::new(RefCell::new(Instant::now())),

            is_undoing: false,
            is_redoing: false,
        }
    }

    pub fn attach(&mut self, app: &App) {
        self.app = Some(app.clone());
    }
}

impl History {
    pub fn get_history_summary(&self) -> Result<JsValue, JsValue> {
        let mut summary = Vec::new();
        for unit in self.undo_stack.borrow().iter() {
            let mut item_counts = std::collections::HashMap::new();
            for item in &unit.items {
                *item_counts.entry(match item {
                    HistoryItem::ObjectUpdate(_) => "Object updates",
                    HistoryItem::SceneUpdate(_) => "Scene updates",
                    HistoryItem::AddElement(_) => "Added elements",
                    HistoryItem::RemoveElement(_) => "Removed elements",
                }).or_insert(0) += 1;
            }

            let description = if unit.items.len() == 1 {
                match &unit.items[0] {
                    HistoryItem::ObjectUpdate(item) => format!("Object update: {}", item.object_id),
                    HistoryItem::SceneUpdate(_) => "Scene update".to_string(),
                    HistoryItem::AddElement(item) => format!("Add element: {}", item.element_id),
                    HistoryItem::RemoveElement(item) => format!("Remove element: {}", item.element_id),
                }
            } else {
                let details: Vec<String> = item_counts.iter()
                    .map(|(k, v)| format!("{}: {}", k, v))
                    .collect();
                format!("Multiple updates: {}", details.join(", "))
            };

            summary.push(HistorySummaryItem {
                timestamp: unit.timestamp,
                description,
                item_count: unit.items.len(),
            });
        }
        serde_wasm_bindgen::to_value(&summary).map_err(|e| e.into())
    }

    pub fn push(&mut self, item: HistoryItem) {
        if self.is_undoing || self.is_redoing {
            return;
        }

        let now = Instant::now();
        let should_finalize = {
            let current_unit = self.current_unit.borrow();
            let last_push_time = self.last_push_time.borrow();
            current_unit.is_none() || now.duration_since(*last_push_time).as_secs_f64() > 0.5
        };

        if should_finalize {
            self.finalize_current_unit();
            *self.current_unit.borrow_mut() = Some(HistoryUnit { 
                items: vec![item], 
                timestamp: js_sys::Date::now(),
            });
        } else {
            self.current_unit.borrow_mut().as_mut().unwrap().items.push(item);
        }

        self.redo_stack.borrow_mut().clear();

        *self.last_push_time.borrow_mut() = now;
    }

    pub fn finalize_current_unit(&mut self) {
        let mut current_unit = self.current_unit.borrow_mut();
        if let Some(unit) = current_unit.take() {
            if !unit.items.is_empty() {
                self.undo_stack.borrow_mut().push(unit);
            }
        }
    }

    pub fn ensure_current_unit_finalized(&mut self) {
        self.finalize_current_unit();
    }

    fn apply_history_unit(&self, app: &App, unit: &HistoryUnit, is_undo: bool) {
        let items_iter: Box<dyn Iterator<Item = &HistoryItem>> = if is_undo {
            Box::new(unit.items.iter().rev())
        } else {
            Box::new(unit.items.iter())
        };

        for item in items_iter {
            match item {
                HistoryItem::ObjectUpdate(item) => {
                    let data = if is_undo { &item.undo_data } else { &item.redo_data };
                    app.object_manager.borrow_mut().update_object(item.object_id.clone(), data.clone());
                }
                HistoryItem::SceneUpdate(item) => {
                    let data = if is_undo { &item.undo_data } else { &item.redo_data };
                    app.scene_manager.borrow_mut().update_scene(data.clone());
                }
                HistoryItem::AddElement(item) => {
                    if is_undo {
                        app.object_manager.borrow_mut().remove(&item.element_id);
                    } else {
                        let data = item.element_data.clone();
                        let element_type = item.element_type.clone();

                        match create_element(&element_type, &data) {
                            Ok(element) => {
                                app.object_manager.borrow_mut().add(element);
                            },
                            Err(e) => console::error_1(&format!("Failed to create element: {:?}", e).into()),
                        }
                    }
                }
                HistoryItem::RemoveElement(item) => {
                    if is_undo {
                        let data = item.element_data.clone();
                        let element_type = item.element_type.clone();

                        match create_element(&element_type, &data) {
                            Ok(element) => {
                                app.object_manager.borrow_mut().add(element);
                            },
                            Err(e) => console::error_1(&format!("Failed to create element: {:?}", e).into()),
                        }
                    } else {
                        app.object_manager.borrow_mut().remove(&item.element_id);
                    }
                }
            }
        }
    }

    pub fn undo(&mut self) -> bool {
        self.is_undoing = true;
        self.ensure_current_unit_finalized();
        if let Some(app) = &self.app {
            let mut undo_stack = self.undo_stack.borrow_mut();
            let mut redo_stack = self.redo_stack.borrow_mut();
            
            if let Some(unit) = undo_stack.pop() {
                self.apply_history_unit(app, &unit, true);
                redo_stack.push(unit);
                app.request_render();
                return true;
            }
        }
        self.is_undoing = false;
        false
    }

    pub fn redo(&mut self) -> bool {
        self.is_redoing = true;
        self.ensure_current_unit_finalized();
        if let Some(app) = &self.app {
            let mut undo_stack = self.undo_stack.borrow_mut();
            let mut redo_stack = self.redo_stack.borrow_mut();
            
            if let Some(unit) = redo_stack.pop() {
                self.apply_history_unit(app, &unit, false);
                undo_stack.push(unit);
                app.request_render();
                return true;
            }
        }
        self.is_redoing = false;
        false
    }

    pub fn undo_to_time(&mut self, target_time: f64) -> bool {
        self.is_undoing = true;
        self.ensure_current_unit_finalized();
        if let Some(app) = &self.app {
            let mut undo_stack = self.undo_stack.borrow_mut();
            let mut redo_stack = self.redo_stack.borrow_mut();
            let target_index = undo_stack
                .iter()
                .position(|unit| unit.timestamp <= target_time)
                .unwrap_or(0);
            let units_to_undo: Vec<_> = undo_stack.drain(target_index..).rev().collect();
            redo_stack.extend(units_to_undo);
            self.apply_operations_to_current_state(app, &undo_stack, true);

            app.request_render();
            return true;
        }
        self.is_undoing = false;
        false
    }

    pub fn redo_to_time(&mut self, target_time: f64) -> bool {
        self.is_redoing = true;
        if let Some(app) = &self.app {
            let mut undo_stack = self.undo_stack.borrow_mut();
            let mut redo_stack = self.redo_stack.borrow_mut();
            let target_index = redo_stack
                .iter()
                .position(|unit| unit.timestamp > target_time)
                .unwrap_or(redo_stack.len());

            let units_to_redo: Vec<_> = redo_stack.drain(..target_index).collect();
            undo_stack.extend(units_to_redo);

            self.apply_operations_to_current_state(app, &undo_stack, false);

            app.request_render();
            return true;
        }
        self.is_redoing = false;
        false
    }

    fn apply_operations_to_current_state(&self, app: &App, operations: &[HistoryUnit], is_undo: bool) {
        app.reset_to_initial_state();
        for unit in operations {
            self.apply_history_unit(app, unit, is_undo);
        }
    }
    
    pub fn can_undo(&self) -> bool {
        !self.undo_stack.borrow().is_empty() || self.current_unit.borrow().is_some()
    }

    pub fn can_redo(&self) -> bool {
        !self.redo_stack.borrow().is_empty()
    }

    pub fn clear(&mut self) {
        self.undo_stack.borrow_mut().clear();
        self.redo_stack.borrow_mut().clear();
        *self.current_unit.borrow_mut() = None;
        *self.last_push_time.borrow_mut() = Instant::now();
    }
}

#[derive(Serialize, Deserialize)]
struct HistorySummaryItem {
    timestamp: f64,
    description: String,
    item_count: usize,
}
