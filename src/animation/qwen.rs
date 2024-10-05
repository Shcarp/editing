use std::collections::HashMap;
use std::fmt::Debug;

use super::{AnimationValue, Animation, AnimationStatus};

pub struct QwenAnimation {
    properties: HashMap<String, (AnimationValue, AnimationValue)>, // (start, end)
    duration: f64,
    elapsed: f64,
    easing: Box<dyn Fn(f64) -> f64 >,
}

impl Debug for QwenAnimation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "QwenAnimation {{ properties: {:?}, duration: {:?} }}", self.properties, self.duration)
    }
}

impl Animation for QwenAnimation {
    fn update(
        &mut self,
        delta: f64,
        current_values: &HashMap<String, AnimationValue>,
    ) -> AnimationStatus {
        self.elapsed += delta;
        if self.elapsed >= self.duration {
            // Animation completed
            return AnimationStatus::Completed;
        }
        let progress = self.elapsed / self.duration;

        AnimationStatus::InProgress(progress)
    }

    fn get_progress_values(&self) -> HashMap<String, AnimationValue> {
        let raw_progress = if self.duration > 0.0 {
            (self.elapsed / self.duration).clamp(0.0, 1.0)
        } else {
            1.0
        };
        
        let eased_progress = (self.easing)(raw_progress);

        self.properties.iter().map(|(k, (start, end))| {
            let value = match (start, end) {
                (AnimationValue::Int(s), AnimationValue::Int(e)) => {
                    AnimationValue::Int(*s + (((*e as f64 - *s as f64) * eased_progress) as i32))
                },
                (AnimationValue::Float(s), AnimationValue::Float(e)) => {
                    AnimationValue::Float(s + (e - s) * eased_progress)
                },
                (AnimationValue::String(s), AnimationValue::String(e)) => {
                    // For strings, we'll interpolate the length
                    let new_len = s.len() + ((e.len() as f64 - s.len() as f64) * eased_progress) as usize;
                    AnimationValue::String(s.chars().take(new_len).collect())
                },
                (AnimationValue::Color(s), AnimationValue::Color(e)) => {
                    let new_color = (
                        (s.0 as f64 + ((e.0 as f64 - s.0 as f64) * eased_progress)) as u8,
                        (s.1 as f64 + ((e.1 as f64 - s.1 as f64) * eased_progress)) as u8,
                        (s.2 as f64 + ((e.2 as f64 - s.2 as f64) * eased_progress)) as u8,
                        (s.3 as f64 + ((e.3 as f64 - s.3 as f64) * eased_progress)) as u8,
                    );
                    AnimationValue::Color(new_color)
                },
                (AnimationValue::Vector2D(s), AnimationValue::Vector2D(e)) => {
                    let new_vector = (
                        s.0 + (e.0 - s.0) * eased_progress,
                        s.1 + (e.1 - s.1) * eased_progress,
                    );
                    AnimationValue::Vector2D(new_vector)
                },
                (AnimationValue::Matrix(s), AnimationValue::Matrix(e)) => {
                    let mut new_matrix = [0.0; 6];
                    for i in 0..6 {
                        new_matrix[i] = s[i] + (e[i] - s[i]) * eased_progress;
                    }
                    AnimationValue::Matrix(new_matrix)
                },
                _ => start.clone(),
            };
            (k.clone(), value)
        }).collect()
    }

    fn get_properties(&self) -> Vec<String> {
        self.properties.keys().cloned().collect()
    }
}

impl QwenAnimation {
    fn new(duration: f64) -> Self {
        QwenAnimation {
            properties: HashMap::new(),
            duration,
            elapsed: 0.0,
            easing: Box::new(|x| x.powf(2.0)), // Linear easing by default
        }
    }

    fn set_easing(&mut self, easing: Box<dyn Fn(f64) -> f64 >) {
        self.easing = easing;
    }
}

pub struct QwenAnimationBuilder {
    duration: f64,
    properties: HashMap<String, (AnimationValue, AnimationValue)>,
    easing: Option<Box<dyn Fn(f64) -> f64 >>,
}

impl QwenAnimationBuilder {
    pub fn new(duration: f64) -> Self {
        QwenAnimationBuilder {
            duration,
            properties: HashMap::new(),
            easing: None,
        }
    }

    pub fn add_property(mut self, name: &str, start: AnimationValue, end: AnimationValue) -> Self {
        self.properties.insert(name.to_string(), (start, end));
        self
    }

    pub fn set_easing(mut self, easing: Box<dyn Fn(f64) -> f64 >) -> Self {
        self.easing = Some(easing);
        self
    }

    pub fn build(self) -> QwenAnimation {
        let mut animation = QwenAnimation::new(self.duration);
        animation.properties = self.properties;
        if let Some(easing) = self.easing {
            animation.set_easing(easing);
        }
        animation
    }
}