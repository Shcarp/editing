use crate::{animation::{Animatable, Tween}, helper::{convert_1x6_to_3x3, convert_3x3_to_1x6, get_rotation_matrix}, renderer::Renderer};
use nalgebra as na;

use super::{Eventable, ObjectId, Renderable, Transformable};
use wasm_timer::Instant;

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
    rotation: f64,
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
            rotation: 0.0
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
    pub rotation: f64,
    pub x_animation: Option<Tween>,
    pub y_animation: Option<Tween>,
    pub rotation_animation: Option<Tween>,
    pub width_animation: Option<Tween>,
    pub height_animation: Option<Tween>,
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
            rotation: options.rotation,
            x_animation: None,
            y_animation: None,
            rotation_animation: None,
            width_animation: None,
            height_animation: None,
        }
    }

    pub fn animate_to(&mut self, params: AnimationParams, duration: f64) {
        if let Some(x) = params.x {
            self.x_animation = Some(Tween::new(self.x, x, duration));
        }
        if let Some(y) = params.y {
            self.y_animation = Some(Tween::new(self.y, y, duration));
        }
        if let Some(rotation) = params.rotation {
            self.rotation_animation = Some(Tween::new(self.rotation, rotation, duration));
        }
        if let Some(width) = params.width {
            self.width_animation = Some(Tween::new(self.width, width, duration));
        }
        if let Some(height) = params.height {
            self.height_animation = Some(Tween::new(self.height, height, duration));
        }
    }

    pub fn render_fn(&self, renderer: &dyn Renderer, fill: &str, stroke: &str) {
        let binding = self.calc_transform();
        let transform_slice = binding.as_slice();
        if let [a, b, c, d, e, f] = transform_slice {
            renderer.transform(*a, *b, *c, *d, *e, *f);
        }
    
        renderer.set_global_alpha(self.opacity);

        renderer.draw_rectangle(0.0, 0.0, self.width, self.height, fill);

        let offset = self.stroke_width / 2.0;
        renderer.set_stroke_style(stroke);
        renderer.set_line_width(self.stroke_width);
        renderer.stroke_rect(
            offset,
            offset,
            self.width - self.stroke_width,
            self.height - self.stroke_width,
        );
    
        let center_x = self.width / 2.0;
        let center_y = self.height / 2.0;
        renderer.set_fill_style("red");
        renderer.begin_path();
        renderer.arc(center_x, center_y, 5.0, 0.0, 2.0 * std::f64::consts::PI);
        renderer.fill();
    
    }
}

impl Renderable for Rect {
    fn render(&mut self, renderer: &dyn Renderer, delta_time: f64) {
        self.update_animations(delta_time);
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
        self.rotation += 1.0;
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

    fn get_center(&self) -> (f64, f64) {
        let transform = convert_1x6_to_3x3(self.get_transform());
        let center = na::Vector3::new(self.width / 2.0,  self.height / 2.0, 1.0);
        let transformed_center = transform * center;
        (transformed_center.x, transformed_center.y)
    }
    
    fn calc_transform(&self) -> na::Matrix1x6<f64> {
        let base_transform = self.get_transform();
        let (translate_x, translate_y) = (base_transform[4], base_transform[5]);
        
        let scale_skew_matrix = na::Matrix3::new(
            base_transform[0], base_transform[1], 0.0,
            base_transform[2], base_transform[3], 0.0,
            0.0, 0.0, 1.0
        );

        let translate_to_center = na::Matrix3::new(
            1.0, 0.0, self.width / 2.0, 
            0.0, 1.0, self.height / 2.0, 
            0.0, 0.0, 1.0
        );

        let translate_from_center = na::Matrix3::new(
            1.0, 0.0, -self.width / 2.0,
            0.0, 1.0, -self.height / 2.0,
            0.0, 0.0, 1.0
        );

        let rotation = get_rotation_matrix(self.rotation.to_radians());

        let transform_matrix = scale_skew_matrix * translate_to_center * rotation * translate_from_center;

        let mut final_transform = convert_3x3_to_1x6(transform_matrix);
        final_transform[4] += translate_x;
        final_transform[5] += translate_y;

        final_transform
    }

    fn set_rotation(&mut self, angle_degrees: f64) {
        self.rotation = angle_degrees % 360.0;
    }
    
    fn set_position(&mut self, x: f64, y: f64) {
        self.x = x;
        self.y = y;
    }
    
    fn set_scale(&mut self, sx: f64, sy: f64) {
        self.scale_x = sx;
        self.scale_y = sy;
    }
    
    fn apply_transform(&mut self, transform: nalgebra::Matrix1x6<f64>) {
        self.scale_x = transform[0];
        self.skew_x = transform[1];
        self.skew_y = transform[2];
        self.scale_y = transform[3];
        self.x = transform[4];
        self.y = transform[5];

        let angle_radians = (self.skew_y / self.scale_x).atan();
        self.rotation = angle_radians.to_degrees();
    }
    
    fn get_rotation(&self) -> f64 {
        self.rotation
    }
    
    fn get_position(&self) -> (f64, f64) {
        (self.x, self.y)
    }
    
    fn get_scale(&self) -> (f64, f64) {
       (self.scale_x, self.scale_y)
    }
}

impl Rect {
    fn update_animation(animation: &mut Option<Tween>, delta_time: f64) -> Option<f64> {
        if let Some(ref mut anim) = animation {
            anim.update(delta_time);
            let value = anim.value();
            if anim.is_finished() {
                *animation = None;
            }
            Some(value)
        } else {
            None
        }
    }

    fn update_animations(&mut self, delta_time: f64) {
        if let Some(x) = Self::update_animation(&mut self.x_animation, delta_time) {
            self.x = x;
        }
        if let Some(y) = Self::update_animation(&mut self.y_animation, delta_time) {
            self.y = y;
        }
        if let Some(rotation) = Self::update_animation(&mut self.rotation_animation, delta_time) {
            self.rotation = rotation;
        }
        if let Some(width) = Self::update_animation(&mut self.width_animation, delta_time) {
            self.width = width;
        }
        if let Some(height) = Self::update_animation(&mut self.height_animation, delta_time) {
            self.height = height;
        }
    }
}

pub struct AnimationParams {
    pub x: Option<f64>,
    pub y: Option<f64>,
    pub rotation: Option<f64>,
    pub width: Option<f64>,
    pub height: Option<f64>,
}

impl AnimationParams {
    pub fn set_x(mut self, x: f64) -> Self {
        self.x = Some(x);
        self
    }

    pub fn set_y(mut self, y: f64) -> Self {
        self.y = Some(y);
        self
    }

    pub fn set_rotation(mut self, rotation: f64) -> Self {
        self.rotation = Some(rotation);
        self
    }

    pub fn set_width(mut self, width: f64) -> Self {
        self.width = Some(width);
        self
    }

    pub fn set_height(mut self, height: f64) -> Self {
        self.height = Some(height);
        self
    }
}

impl Default for AnimationParams {
    fn default() -> Self {
        Self {
            x: None,
            y: None,
            rotation: None,
            width: None,
            height: None,
        }
    }
}
