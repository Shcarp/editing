use crate::{helper::convert_1x6_to_3x3, renderer::Renderer};
use nalgebra as na;
use wasm_bindgen::{convert::FromWasmAbi, JsValue};
use web_sys::console;

use super::{Eventable, ObjectId, Renderable, Transformable};

pub struct RectOptions {
    x: f64,
    y: f64,
    width: f64,
    height: f64,
    fill: String,
    stroke: String,
    stroke_width: f64,
    opacity: f64,
    scale_x: f64,
    scale_y: f64,
    skew_x: f64,
    skew_y: f64,
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
            stroke: "black".to_string(),
            stroke_width: 2.0,
            opacity: 1.0,
            scale_x: 1.0,
            scale_y: 1.0,
            skew_x: 0.0,
            skew_y: 0.0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Rect {
    id: ObjectId,
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
    pub fill: String,
    pub stroke: String,
    pub stroke_width: f64,
    pub opacity: f64,
    pub scale_x: f64,
    pub scale_y: f64,
    pub skew_x: f64,
    pub skew_y: f64,
}

impl Rect {
    pub fn new(options: RectOptions) -> Self {
        let id = ObjectId::new();
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
            scale_x: options.scale_x,
            scale_y: options.scale_y,
            skew_x: options.skew_x,
            skew_y: options.skew_y,
        }
    }

    pub fn render_fn(&self, renderer: &dyn Renderer, fill: &str, stroke: &str) {

        let binding = self.get_transform();
        let transform_slice = binding.as_slice();
        if let [a, b, c, d, e, f] = transform_slice {
            renderer.transform(*a, *b, *c, *d, *e, *f);
        }
        renderer.set_global_alpha(self.opacity);
        renderer.draw_rectangle(self.x, self.y, self.width, self.height, fill);
        let offset = self.stroke_width / 2.0;
        renderer.set_stroke_style(stroke);
        renderer.set_line_width(self.stroke_width);
        renderer.stroke_rect(
            self.x + offset,
            self.y + offset,
            self.width - self.stroke_width,
            self.height - self.stroke_width,
        );

        let (center_x, center_y) = (self.width / 2.0, self.height / 2.0);
        renderer.set_fill_style("red");
        renderer.begin_path();
        renderer.arc(center_x, center_y, 5.0, 0.0, 2.0 * std::f64::consts::PI);
        renderer.fill();
    }
}

impl Renderable for Rect {
    fn render(&mut self, renderer: &dyn Renderer) {
        self.rotate(1.0);
        self.render_fn(renderer, &self.fill, &self.stroke)
    }

    fn id(&self) -> &ObjectId {
        return &self.id;
    }
}

impl Eventable for Rect {
    fn render_shadow_box(&mut self, renderer: &dyn Renderer) {
        let color_id = self.id.color_id;
        let fill_color = format!(
            "rgba({},{},{}, {})",
            { color_id[0] },
            { color_id[1] },
            { color_id[2] },
            { color_id[3] }
        );
        self.rotate(1.0);
        self.render_fn(renderer, &fill_color, &fill_color)
    }
}
impl Transformable for Rect {
    fn get_transform(&self) -> nalgebra::Matrix1x6<f64> {
        nalgebra::Matrix1x6::new(
            self.scale_x,
            self.skew_x,
            self.skew_y,
            self.scale_y,
            self.x,
            self.y,
        )
    }

    fn set_transform(&mut self, transform: nalgebra::Matrix1x6<f64>) {
        self.scale_x = transform[0];
        self.skew_x = transform[1];
        self.skew_y = transform[2];
        self.scale_y = transform[3];
        self.x = transform[4];
        self.y = transform[5];
    }

    fn get_center(&self) -> (f64, f64) {
        let transform = convert_1x6_to_3x3(self.get_transform());
        // Calculate the center of the untransformed rectangle
        let center = na::Vector3::new(self.x + self.width / 2.0, self.y + self.height / 2.0, 1.0);
        
        // Apply the transformation to the center point
        let transformed_center = transform * center;

        (transformed_center.x, transformed_center.y)
    }
}
