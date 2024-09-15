mod rect;

pub use rect::{AnimationParams, Rect, RectOptions};

use nalgebra as na;
use std::fmt::Debug;
use web_sys::CanvasRenderingContext2d;

use std::any::TypeId;

use crate::helper::generate_id;
use crate::render_control::{get_render_control, RenderMessage};
use crate::renderer::Renderer;

use serde::{Deserialize, Serialize};

use std::collections::HashMap;
use once_cell::sync::Lazy;
use rand::Rng;

static mut ID_COLOR_MAP: Lazy<(HashMap<String, [u8; 4]>, HashMap<[u8; 4], String>)> = 
    Lazy::new(|| (HashMap::new(), HashMap::new()));

#[derive(Clone, PartialEq, Eq, Hash, Debug, Serialize, Deserialize)]
pub struct ObjectId {
    id: String,
    color_id: [u8; 4],
}

impl ObjectId {
    pub fn new() -> Self {
        let id = generate_id();
        let color_id = Self::generate_unique_color_id(&id);
        Self { id, color_id }
    }

    pub fn value(&self) -> &str {
        &self.id
    }

    pub fn color(&self) -> (u8, u8, u8, u8) {
        (
            self.color_id[0],
            self.color_id[1],
            self.color_id[2],
            self.color_id[3],
        )
    }

    fn generate_unique_color_id(id: &str) -> [u8; 4] {
        unsafe {
            loop {
                let color_id = Self::generate_random_color();
                if !ID_COLOR_MAP.1.contains_key(&color_id) {
                    ID_COLOR_MAP.0.insert(id.to_string(), color_id);
                    ID_COLOR_MAP.1.insert(color_id, id.to_string());
                    return color_id;
                }
            }
        }
    }

    fn generate_random_color() -> [u8; 4] {
        let mut rng: rand::prelude::ThreadRng = rand::thread_rng();
        [
            rng.gen(),
            rng.gen(),
            rng.gen(),
            255, // 保持 alpha 通道为 255
        ]
    }

    pub fn get_id_by_color(color: [u8; 4]) -> Option<String> {
        unsafe { ID_COLOR_MAP.1.get(&color).cloned() }
    }

    pub fn get_color_by_id(id: &str) -> Option<[u8; 4]> {
        unsafe { ID_COLOR_MAP.0.get(id).cloned() }
    }
}

pub trait Renderable: Debug {
    fn id(&self) -> &ObjectId;
    fn update(&mut self, delta_time: f64);
    fn render(&mut self, renderer: &dyn Renderer, delta_time: f64);
    fn render_hit(&mut self, renderer: &dyn Renderer, hit_color: &str, delta_time: f64);

    fn set_dirty(&mut self) {
        self.set_dirty_flag(true);
        get_render_control().add_message(RenderMessage::Update(self.id().value().to_owned()));
    }

    fn set_dirty_flag(&mut self, is_dirty: bool);
    fn is_dirty(&self) -> bool;
}

pub trait Dirty: Renderable {}

pub trait Eventable {
    // fn on(&mut self, event_type: &str, callback: Box<dyn Fn()>);
    // fn off(&mut self, event_type: &str);
    // fn trigger(&mut self, event_type: &str);
}

pub trait Transformable {
    fn get_transform(&self) -> na::Matrix1x6<f64>;
    fn calc_transform(&self) -> na::Matrix1x6<f64>;

    fn get_center(&self) -> (f64, f64);

    fn set_rotation(&mut self, angle_degrees: f64);
    fn set_position(&mut self, x: f64, y: f64);
    fn set_scale(&mut self, sx: f64, sy: f64);
    fn set_skew(&mut self, skew_x: f64, skew_y: f64);
    fn apply_transform(&mut self, transform: na::Matrix1x6<f64>);
    fn get_rotation(&self) -> f64;
    fn get_position(&self) -> (f64, f64);
    fn get_scale(&self) -> (f64, f64);

    fn reset_transform(&mut self) {
        self.apply_transform(na::Matrix1x6::new(1.0, 0.0, 0.0, 1.0, 0.0, 0.0));
    }
}

// 容器 trait
pub trait RenderContainer: Debug {
    type Item: Renderable;

    fn add(&mut self, item: Self::Item);
    fn remove(&mut self, id: &ObjectId) -> Option<Self::Item>;
    fn get(&self, id: &ObjectId) -> Option<&Self::Item>;
    fn get_mut(&mut self, id: &ObjectId) -> Option<&mut Self::Item>;
    fn render_all(&self, context: &CanvasRenderingContext2d);
    fn update_all(&mut self, delta_time: f32);
}

pub trait Collidable {
    fn collides_with(&self, other: &dyn Collidable) -> bool;
}

pub fn is_renderable<T: 'static>() -> bool {
    TypeId::of::<T>() == TypeId::of::<dyn Renderable>()
}

pub fn is_transformable<T: 'static>() -> bool {
    TypeId::of::<T>() == TypeId::of::<dyn Transformable>()
}

pub fn is_render_container<T: 'static>() -> bool {
    TypeId::of::<T>() == TypeId::of::<dyn RenderContainer<Item = dyn Renderable>>()
}

pub fn is_collidable<T: 'static>() -> bool {
    TypeId::of::<T>() == TypeId::of::<dyn Collidable>()
}
