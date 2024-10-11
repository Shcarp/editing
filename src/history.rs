use std::{cell::RefCell, fmt::Debug, rc::Rc, time::Instant};
use serde_json::Value;
use crate::app::App;

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
            timestamp: Instant::now().elapsed().as_secs_f64(),
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
            timestamp: Instant::now().elapsed().as_secs_f64(),
        }
    }
}

enum HistoryItemType {
    ObjectUpdate(ObjectHistoryItem),
    SceneUpdate(SceneHistoryItem),
}

pub struct HistoryUnit {
    items: Vec<HistoryItemType>,
}

#[derive(Clone)]
pub struct History {
    app: Option<App>,
    undo_stack: Rc<RefCell<Vec<HistoryUnit>>>,
    redo_stack: Rc<RefCell<Vec<HistoryUnit>>>,
    current_unit: Rc<RefCell<Option<HistoryUnit>>>,
    last_push_time: Rc<RefCell<Instant>>,
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
        }
    }

    pub fn attach(&mut self, app: &App) {
        self.app = Some(app.clone());
    }
}

impl History {
    fn push_undo(&mut self, item: HistoryItemType) {
        let now = Instant::now();
        let should_finalize = {
            let current_unit = self.current_unit.borrow();
            let last_push_time = self.last_push_time.borrow();
            current_unit.is_none() || now.duration_since(*last_push_time).as_secs_f64() > 0.5
        };

        if should_finalize {
            self.finalize_current_unit();
            *self.current_unit.borrow_mut() = Some(HistoryUnit { items: vec![item] });
        } else {
            self.current_unit.borrow_mut().as_mut().unwrap().items.push(item);
        }

        *self.last_push_time.borrow_mut() = now;
    }

    fn push_redo(&mut self, item: HistoryItemType) {
        let now = Instant::now();
        let should_finalize = {
            let current_unit = self.current_unit.borrow();
            let last_push_time = self.last_push_time.borrow();
            current_unit.is_none() || now.duration_since(*last_push_time).as_secs_f64() > 0.5
        };

        if should_finalize {
            self.finalize_current_unit();
            *self.current_unit.borrow_mut() = Some(HistoryUnit { items: vec![item] });
        } else {
            self.current_unit.borrow_mut().as_mut().unwrap().items.push(item);
        }

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

    pub fn undo(&mut self) {
        self.ensure_current_unit_finalized();
    }

    pub fn redo(&mut self) {
        self.ensure_current_unit_finalized();
        // Implement redo logic here
    }
}
