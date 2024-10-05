mod qwen;

use std::borrow::Cow;
use std::cell::RefCell;
use std::collections::{HashMap, VecDeque};
use std::fmt::Debug;
use std::rc::Rc;
use futures::channel::mpsc::{channel, Receiver, Sender};
use futures::StreamExt;
use wasm_timer::Instant;
use web_sys::console;

use crate::element::Renderable;

pub use qwen::*;

#[derive(Debug, Clone)]
pub enum AnimationValue {
    Int(i32),
    Float(f64),
    String(String),
    Color((u8, u8, u8, u8)),
    Vector2D((f64, f64)),
    Matrix([f64; 6]),
}

pub trait Animatable {
    fn get_properties(&self, properties: &[String]) -> HashMap<String, AnimationValue> {
        HashMap::new()
    }

    fn set_properties(
        &mut self,
        properties: HashMap<String, AnimationValue>,
    ) -> Result<(), AnimationError> {
        Ok(())
    }

    fn is_animatable(&self) -> bool {
        false
    }
}
pub trait AnimatableExt {
    fn into_renderable(self) -> Rc<RefCell<Box<dyn Renderable>>>;
}

impl AnimatableExt for Rc<RefCell<Box<dyn Animatable>>> {
    fn into_renderable(self) -> Rc<RefCell<Box<dyn Renderable>>> {
        // This is still unsafe, but now it's implemented as a trait method
        unsafe { std::mem::transmute(self) }
    }
}

#[derive(Debug)]
pub enum AnimationError {
    InvalidProperty(Cow<'static, str>),
    UnsupportedPropertyType(Cow<'static, str>),
}

#[derive(Debug)]
pub enum AnimationStatus {
    InProgress(f64),
    Completed,
}

pub trait Animation: Debug {
    fn update(
        &mut self,
        delta: f64,
        current_values: &HashMap<String, AnimationValue>,
    ) -> AnimationStatus;
    fn get_progress_values(&self) -> HashMap<String, AnimationValue>;
    fn get_properties(&self) -> Vec<String>;
}


#[derive(Debug)]
struct AnimationEntry {
    animation: Box<dyn Animation>,
    object_id: String,
}

#[derive(Debug)]
pub struct AnimationManager {
    pub init: bool,
    animations: Vec<AnimationEntry>,
    queued_animations: VecDeque<(String, Box<dyn Animation>)>,
    last_update: Instant,

    sender: Sender<bool>,
    receiver: Receiver<bool>,

    last_send: Instant,
}

impl AnimationManager {
    pub fn new() -> Self {
        let (sender, receiver) = channel(1);
        Self {
            init: false,
            animations: Vec::new(),
            queued_animations: VecDeque::new(),
            last_update: Instant::now(),
            sender,
            receiver,

            last_send: Instant::now(),
        }
    }

    pub fn add_animation(&mut self, object_id: String, animation: Box<dyn Animation>) {
        // console::log_1(&"add_animation".into());
        self.animations.push(AnimationEntry {
            animation,
            object_id,
        });

        if self.init {
            self.sender();
        }
    }

    pub fn queue_animation(&mut self, object_id: String, animation: Box<dyn Animation>) {
        self.queued_animations.push_back((object_id, animation));
    }

    pub fn update(
        &mut self,
        objects: HashMap<String, Rc<RefCell<Box<dyn Renderable>>>>,
    ) -> Result<(), AnimationError> {
        // console::log_1(&"update".into());
        // 如果没有初始化，则进行初始化
        if !self.init {
            console::log_1(&"init".into());
            self.init = true;
            self.sender();
            return Ok(());
        }

        let now = Instant::now();
        let delta = now.duration_since(self.last_update).as_secs_f64();
        self.last_update = now;

        let mut completed_indices = Vec::new();

        for (index, entry) in self.animations.iter_mut().enumerate() {
            if let Some(object) = objects.get(&entry.object_id) {
                let properties = entry.animation.get_properties();
                let current_values = object.borrow().get_properties(&properties);

                match entry.animation.update(delta, &current_values) {
                    AnimationStatus::InProgress(progress) => {
                        let new_values = entry.animation.get_progress_values();
                        object.borrow_mut().set_properties(new_values)?;
                    }
                    AnimationStatus::Completed => {
                        completed_indices.push(index);
                    }
                }
            } else {
                completed_indices.push(index);
            }
        }
        for &index in completed_indices.iter().rev() {
            self.animations.swap_remove(index);
            self.sender();
        }

        while let Some((object_id, animation)) = self.queued_animations.pop_front() {
            self.animations.push(AnimationEntry {
                animation,
                object_id,
            });
            self.sender();
        }
        
        self.sender();

        Ok(())
    }

    pub fn get_active_animation_count(&self) -> usize {
        self.animations.len()
    }

    pub fn clear_all_animations(&mut self) {
        self.animations.clear();
        self.queued_animations.clear();
    }

    pub fn is_empty(&self) -> bool {
        self.animations.is_empty()
    }

    pub async fn listen(&mut self) -> bool {
        self.receiver.next().await.unwrap_or(false)
    }

    pub fn sender(&mut self) {
        // console::log_1(&"sender".into());
        let now = Instant::now();
        if now.duration_since(self.last_send).as_secs_f64() >= 0.008 {
            if let Err(e) = self.sender.try_send(self.is_empty()) {
                console::error_1(&format!("Failed to send animation update: {}", e).into());
            }
            self.last_send = now;
        }
    }
}
