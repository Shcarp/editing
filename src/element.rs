mod rect;

pub use rect::{Rect, RectOptions};

use nalgebra as na;
use std::fmt::Debug;
use web_sys::CanvasRenderingContext2d;

use std::any::TypeId;

use crate::helper::{
    convert_1x6_to_3x3, convert_3x3_to_1x6, generate_color_id, generate_id,
    normalize_3x3_if_needed, normalize_if_needed,
};
use crate::log;
use crate::renderer::Renderer;

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
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
}

// 渲染 trait
pub trait Renderable: Debug {
    fn render(&mut self, renderer: &dyn Renderer);
    fn id(&self) -> &ObjectId;
}

// events
pub trait Eventable {
    fn render_shadow_box(&mut self, renderer: &dyn Renderer);
}

pub trait Transformable {
    fn get_transform(&self) -> na::Matrix1x6<f64>;
    fn set_transform(&mut self, transform: na::Matrix1x6<f64>);
    fn get_center(&self) -> (f64, f64);

    fn translate(&mut self, dx: f64, dy: f64) {
        let mut new_transform = self.get_transform();
        new_transform[4] += dx;
        new_transform[5] += dy;
        self.set_transform(new_transform);
    }

    fn rotate(&mut self, angle_degrees: f64) {
        const EPSILON: f64 = 1e-6;
        let (center_x, center_y) = self.get_center(); 
        let angle_radians = angle_degrees.to_radians();

        log(&format!("{},{}", center_x, center_y));

            // 创建平移矩阵（移动到原点）
        let translate_to_origin = na::Matrix3::new(
            1.0, 0.0, 0.0,
            0.0, 1.0, 0.0,
            0.0, 0.0, 1.0
        );

        // 创建平移矩阵（从原点移回）
        let translate_back = na::Matrix3::new(
            1.0, 0.0, 0.0,
            0.0, 1.0, 0.0,
            0.0, 0.0, 1.0
        );


        let rotation_matrix = if angle_radians.abs() < EPSILON {
            let sin = angle_radians;
            let cos = 1.0 - 0.5 * angle_radians * angle_radians;
            na::Matrix3::new(
                cos, -sin, 0.0, 
                sin, cos, 0.0,
                0.0, 0.0, 1.0
            )
        } else {
            let (sin, cos) = angle_radians.sin_cos();
            na::Matrix3::new(
                cos, -sin, 0.0, 
                sin, cos, 0.0, 
                0.0, 0.0, 1.0
            )
        };

        let current_transform = convert_1x6_to_3x3(self.get_transform());
        let new_transform = current_transform * translate_to_origin * rotation_matrix * translate_back;
        let normalized_transform = convert_3x3_to_1x6(normalize_3x3_if_needed(new_transform));
        self.set_transform(normalized_transform);
    }

    fn scale(&mut self, sx: f64, sy: f64) {
        let mut new_transform = self.get_transform();
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

pub trait Collidable {
    fn collides_with(&self, other: &dyn Collidable) -> bool;
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

pub fn is_collidable<T: 'static>() -> bool {
    TypeId::of::<T>() == TypeId::of::<dyn Collidable>()
}
