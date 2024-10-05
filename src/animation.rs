use std::any::{Any, TypeId};
use std::cell::RefCell;
use std::collections::{HashMap, VecDeque};
use std::rc::Rc;
use wasm_timer::Instant;
use std::borrow::Cow;
use std::fmt::Debug;

use crate::element::Renderable;

#[derive(Debug, Clone)]
pub enum AnimationValue {
    Int(i32),
    Float(f64),
    String(String),
    Color((u8, u8, u8, u8)),
    Vector2D((f64, f64)),
    Matrix([f64; 6]),
}


pub trait Animatable: Renderable {
    fn get_properties(&self, properties: &[String]) -> HashMap<String, AnimationValue>;
    fn set_properties(&mut self, properties: HashMap<String, AnimationValue>) -> Result<(), AnimationError>;
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
    
pub trait Animation: Debug {
    fn update(&mut self, delta: f64, current_values: &HashMap<String, AnimationValue>) -> AnimationStatus;
    fn get_current_values(&self) -> HashMap<String, AnimationValue>;
    fn get_properties(&self) -> &[String];
    fn reset(&mut self);
    fn set_duration(&mut self, duration: f64);
    fn get_duration(&self) -> f64;
    fn set_easing(&mut self, easing: Box<dyn Fn(f64) -> f64 + Send + Sync>);
}

#[derive(Debug)]
pub enum AnimationStatus {
    InProgress,
    Completed,
}

#[derive(Debug)]
struct AnimationEntry {
    animation: Box<dyn Animation>,
    object_id: String,
}

#[derive(Debug)]
pub struct AnimationManager {
    animations: Vec<AnimationEntry>,
    queued_animations: VecDeque<(String, Box<dyn Animation>)>,
    last_update: Instant,
}

impl AnimationManager {
    pub fn new() -> Self {
        Self {
            animations: Vec::new(),
            queued_animations: VecDeque::new(),
            last_update: Instant::now(),
        }
    }

    pub fn add_animation(&mut self, object_id: String, animation: Box<dyn Animation>) {
        self.animations.push(AnimationEntry { animation, object_id });
    }

    pub fn queue_animation(&mut self, object_id: String, animation: Box<dyn Animation>) {
        self.queued_animations.push_back((object_id, animation));
    }

    pub fn update(&mut self, objects: HashMap<String, Rc<RefCell<Box<dyn Animatable>>>>) -> Result<(), AnimationError> {
        let now = Instant::now();
        let delta = now.elapsed().as_secs_f64() - self.last_update.elapsed().as_secs_f64();
        self.last_update = now;

        let mut completed_indices = Vec::new();

        for (index, entry) in self.animations.iter_mut().enumerate() {
            if let Some(object) = objects.get(&entry.object_id) {
                let properties = entry.animation.get_properties();
                let current_values = object.borrow().get_properties(properties);

                match entry.animation.update(delta, &current_values) {
                    AnimationStatus::InProgress => {
                        let new_values = entry.animation.get_current_values();
                        object.borrow_mut().set_properties(new_values)?;
                    },
                    AnimationStatus::Completed => {
                        completed_indices.push(index);
                    },
                }
            } else {
                completed_indices.push(index);
            }
        }

        // Remove completed animations in reverse order to maintain correct indices
        for &index in completed_indices.iter().rev() {
            self.animations.swap_remove(index);
        }

        // Add queued animations
        while let Some((object_id, animation)) = self.queued_animations.pop_front() {
            self.animations.push(AnimationEntry { animation, object_id });
        }

        Ok(())
    }

    pub fn get_object_ids(&self) -> Vec<String> {
        self.animations.iter().map(|entry| entry.object_id.clone()).collect()
    }

    pub fn get_active_animation_count(&self) -> usize {
        self.animations.len()
    }

    pub fn clear_all_animations(&mut self) {
        self.animations.clear();
        self.queued_animations.clear();
    }
}