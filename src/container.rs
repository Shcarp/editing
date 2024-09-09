use web_sys::CanvasRenderingContext2d;
use std::fmt::Debug;

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct ObjectId(String);

// 渲染 trait
pub trait Renderable: Debug {
    fn render(&self, context: &CanvasRenderingContext2d);
    fn update(&mut self, delta_time: f32);
    fn get_id(&self) -> &ObjectId;
    fn is_visible(&self) -> bool;
    fn set_visible(&mut self, visible: bool);
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
