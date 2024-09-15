mod rect;

pub use rect::{AnimationParams, Rect, RectOptions};

use nalgebra as na;
use std::borrow::BorrowMut;
use std::fmt::Debug;
use web_sys::CanvasRenderingContext2d;

use std::any::TypeId;

use crate::helper::{generate_color_id, generate_id};
use crate::render_control::{get_render_control, RenderMessage};
use crate::renderer::Renderer;

use serde::{Deserialize, Serialize};

#[derive(Clone, PartialEq, Eq, Hash, Debug, Serialize, Deserialize)]
pub struct ObjectId {
    id: String,
    color_id: [i32; 4],
}

impl ObjectId {
    pub fn new() -> Self {
        let id = generate_id();
        let color_id = generate_color_id();
        Self { id, color_id }
    }

    pub fn value(&self) -> String {
        self.id.clone()
    }

    pub fn color(&self) -> (i32, i32, i32, i32) {
        (
            self.color_id[0],
            self.color_id[1],
            self.color_id[2],
            self.color_id[3],
        )
    }
}

pub trait Renderable: Debug {
    fn id(&self) -> &ObjectId;
    fn update(&mut self, delta_time: f64);
    fn render(&mut self, renderer: &dyn Renderer, delta_time: f64);
    fn render_hit(&mut self, renderer: &dyn Renderer, hit_color: &str, delta_time: f64);

    fn set_dirty(&mut self) {
        self.set_dirty_flag(true);
        get_render_control().add_message(RenderMessage::Update(self.id().value()));
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
