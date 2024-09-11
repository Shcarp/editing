mod rect;

pub use rect::{Rect, RectOptions};

use nalgebra as na;
use web_sys::CanvasRenderingContext2d;
use std::fmt::Debug;

use std::any::TypeId;

use crate::renderer::Renderer;

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct ObjectId(pub String);

// 渲染 trait
pub trait Renderable: Debug {
    fn render(&self, renderer: &dyn Renderer);
    fn id(&self) -> &ObjectId;
}

pub trait Transformable {
    fn get_transform(&self) -> &na::Matrix1x6<f64>;
    fn set_transform(&mut self, transform: na::Matrix1x6<f64>);

    fn translate(&mut self, dx: f64, dy: f64) {
        let mut new_transform = *self.get_transform();
        new_transform[4] += dx;
        new_transform[5] += dy;
        self.set_transform(new_transform);
    }

    fn rotate(&mut self, angle: f64) {
        let cos = angle.cos();
        let sin = angle.sin();
        let mut new_transform = *self.get_transform();
        let a = new_transform[0];
        let b = new_transform[1];
        let c = new_transform[2];
        let d = new_transform[3];
        new_transform[0] = a * cos + c * sin;
        new_transform[1] = b * cos + d * sin;
        new_transform[2] = c * cos - a * sin;
        new_transform[3] = d * cos - b * sin;
        self.set_transform(new_transform);
    }

    fn scale(&mut self, sx: f64, sy: f64) {
        let mut new_transform = *self.get_transform();
        new_transform[0] *= sx;
        new_transform[1] *= sx;
        new_transform[2] *= sy;
        new_transform[3] *= sy;
        self.set_transform(new_transform);
    }

    fn reset_transform(&mut self) {
        self.set_transform(na::Matrix1x6::new(1.0, 0.0, 0.0, 1.0, 0.0, 0.0));
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

// 为 Renderable trait 实现检查方法
pub fn is_renderable<T: 'static>() -> bool {
    TypeId::of::<T>() == TypeId::of::<dyn Renderable>()
}

// 为 Transformable trait 实现检查方法
pub fn is_transformable<T: 'static>() -> bool {
    TypeId::of::<T>() == TypeId::of::<dyn Transformable>()
}

// 为 RenderContainer trait 实现检查方法
pub fn is_render_container<T: 'static>() -> bool {
    TypeId::of::<T>() == TypeId::of::<dyn RenderContainer<Item = dyn Renderable>>()
}
