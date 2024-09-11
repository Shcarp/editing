use nalgebra as na;
use super::{Renderable, Transformable, ObjectId};


pub struct RectOptions {
    x: f64,
    y: f64,
    width: f64,
    height: f64,
    fill: String,
    stroke: String,
    stroke_width: f64,
    opacity: f64,
    transform: na::Matrix1x6<f64>,
}

// 为 RectOptions 实现 Default

impl Default for RectOptions {
    fn default() -> Self {
        Self { 
            x: 0.0, 
            y: 0.0, 
            width: 100.0, 
            height: 100.0, 
            fill: "blue".to_string(), 
            stroke: "transparent".to_string(), 
            stroke_width: 0.0, 
            opacity: 1.0, 
            transform: na::Matrix1x6::new(1.0, 0.0, 0.0, 1.0, 100.0,0.0)
        }
    }
}

#[derive(Debug)]
pub struct Rect {
    id: ObjectId,
    x: f64,
    y: f64,
    width: f64,
    height: f64,
    fill: String,
    stroke: String,
    stroke_width: f64,
    opacity: f64,
    transform: na::Matrix1x6<f64>,
}

impl Rect {
    pub fn new(options: RectOptions) -> Self {
        let id = ObjectId("Ces".to_owned());
        Rect {
            id,
            x: options.x,
            y: options.y,
            width: options.width,
            height: options.height,
            fill: options.fill,
            stroke: options.stroke,
            stroke_width: options.stroke_width,
            opacity: options.opacity,
            transform: options.transform
        }
    }
}

impl Renderable for Rect {
    fn render(&self, renderer: &dyn crate::renderer::Renderer) {
        let transform_slice = self.transform.as_slice();
        if let [a, b, c, d, e, f] = transform_slice {
            renderer.transform(*a, *b, *c, *d, *e, *f);
        }

        renderer.set_global_alpha(self.opacity);
        renderer.set_stroke_style(&self.stroke);
        renderer.set_line_width(self.stroke_width);
        renderer.begin_path();
        renderer.draw_rectangle(self.x, self.y, self.width, self.height, &self.fill);
        renderer.stroke()
    }

    fn id(&self) -> &ObjectId {
        return &self.id;
    }
}

impl Transformable for Rect {
    fn get_transform(&self) -> &nalgebra::Matrix1x6<f64> {
        return &self.transform;
    }

    fn set_transform(&mut self, transform: nalgebra::Matrix1x6<f64>) {
        self.transform = transform
    }
}




