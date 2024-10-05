use std::{any::Any, collections::HashMap};

use super::{Dirty, Eventable, ObjectId, Renderable, Transformable};
use crate::{
    animation::{Animatable, AnimationError, AnimationValue},
    helper::{convert_1x6_to_3x3, convert_3x3_to_1x6, get_rotation_matrix},
    render_control::{get_render_control, UpdateBody, UpdateMessage, UpdateType},
    renderer::Renderer,
};
use dirty_setter::DirtySetter;
use nalgebra as na;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use web_sys::console;

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
            rotation: 0.0,
        }
    }
}

#[derive(Debug, Clone, DirtySetter, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct Rect {
    id: ObjectId,
    dirty: bool,
    #[dirty_setter]
    pub x: f64,
    #[dirty_setter]
    pub y: f64,
    #[dirty_setter]
    pub width: f64,
    #[dirty_setter]
    pub height: f64,
    #[dirty_setter]
    pub fill: String,
    #[dirty_setter]
    pub stroke: String,
    #[dirty_setter]
    pub stroke_width: f64,
    #[dirty_setter]
    pub opacity: f64,
    #[dirty_setter]
    pub scale_x: f64,
    #[dirty_setter]
    pub scale_y: f64,
    #[dirty_setter]
    pub skew_x: f64,
    #[dirty_setter]
    pub skew_y: f64,
    #[dirty_setter]
    pub rotation: f64,
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
            dirty: true,
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
    }
}

impl Dirty for Rect {
    fn set_dirty(&mut self) {
        self.set_dirty_flag(true);
    }
    fn set_dirty_flag(&mut self, is_dirty: bool) {
        self.dirty = is_dirty;
    }

    fn is_dirty(&self) -> bool {
        self.dirty
    }
}

#[derive(Debug, Clone, Deserialize)]
struct RectUpdateBoadyData {
    x: Option<f64>,
    y: Option<f64>,
    width: Option<f64>,
    height: Option<f64>,
    fill: Option<String>,
    stroke: Option<String>,
    stroke_width: Option<f64>,
    opacity: Option<f64>,
    scale_x: Option<f64>,
    scale_y: Option<f64>,
    skew_x: Option<f64>,
    skew_y: Option<f64>,
    rotation: Option<f64>,
}

impl Renderable for Rect {
    fn id(&self) -> &ObjectId {
        return &self.id;
    }

    fn update(&mut self, data: Value) {
        self.update(data);
    }

    fn render(&self, renderer: &dyn Renderer) {
        self.render_fn(renderer, &self.fill, &self.stroke)
    }

    fn position(&self) -> (f64, f64) {
        (self.x, self.y)
    }
}

impl Eventable for Rect {}

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
        let center = na::Vector3::new(self.width / 2.0, self.height / 2.0, 1.0);
        let transformed_center = transform * center;
        (transformed_center.x, transformed_center.y)
    }

    fn calc_transform(&self) -> na::Matrix1x6<f64> {
        let base_transform = self.get_transform();
        let (translate_x, translate_y) = (base_transform[4], base_transform[5]);

        let scale_skew_matrix = na::Matrix3::new(
            base_transform[0],
            base_transform[1],
            0.0,
            base_transform[2],
            base_transform[3],
            0.0,
            0.0,
            0.0,
            1.0,
        );

        let translate_to_center = na::Matrix3::new(
            1.0,
            0.0,
            self.width / 2.0,
            0.0,
            1.0,
            self.height / 2.0,
            0.0,
            0.0,
            1.0,
        );

        let translate_from_center = na::Matrix3::new(
            1.0,
            0.0,
            -self.width / 2.0,
            0.0,
            1.0,
            -self.height / 2.0,
            0.0,
            0.0,
            1.0,
        );

        let rotation = get_rotation_matrix(self.rotation.to_radians());

        let transform_matrix =
            scale_skew_matrix * translate_to_center * rotation * translate_from_center;

        let mut final_transform = convert_3x3_to_1x6(transform_matrix);
        final_transform[4] += translate_x;
        final_transform[5] += translate_y;

        final_transform
    }

    fn set_rotation(&mut self, angle_degrees: f64) {
        self.set_rotation(angle_degrees % 360.0);
    }

    fn set_position(&mut self, x: f64, y: f64) {
        self.set_x(x);
        self.set_y(y);
    }

    fn set_scale(&mut self, sx: f64, sy: f64) {
        self.set_scale_x(sx);
        self.set_scale_y(sy);
    }

    fn set_skew(&mut self, skew_x: f64, skew_y: f64) {
        self.set_skew_x(skew_x);
        self.set_skew_y(skew_y);
    }

    fn apply_transform(&mut self, transform: nalgebra::Matrix1x6<f64>) {
        self.set_x(transform[4]);
        self.set_y(transform[5]);
        self.set_scale(transform[0], transform[3]);
        self.set_skew(transform[1], transform[2]);

        let angle_radians = (self.skew_y / self.scale_x).atan();
        self.set_rotation(angle_radians.to_degrees());
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

impl Animatable for Rect {
    fn get_properties(&self, properties: &[String]) -> HashMap<String, AnimationValue> {
        let mut result = HashMap::new();

        for property in properties {
            match property.as_str() {
                "x" => result.insert("x".to_string(), AnimationValue::Float(self.x)),
                "y" => result.insert("y".to_string(), AnimationValue::Float(self.y)),
                "width" => result.insert("width".to_string(), AnimationValue::Float(self.width)),
                "height" => result.insert("height".to_string(), AnimationValue::Float(self.height)),
                "fill" => result.insert(
                    "fill".to_string(),
                    AnimationValue::String(self.fill.clone()),
                ),
                "stroke" => result.insert(
                    "stroke".to_string(),
                    AnimationValue::String(self.stroke.clone()),
                ),
                "stroke_width" => result.insert(
                    "stroke_width".to_string(),
                    AnimationValue::Float(self.stroke_width),
                ),
                "opacity" => {
                    result.insert("opacity".to_string(), AnimationValue::Float(self.opacity))
                }
                "scale_x" => {
                    result.insert("scale_x".to_string(), AnimationValue::Float(self.scale_x))
                }
                "scale_y" => {
                    result.insert("scale_y".to_string(), AnimationValue::Float(self.scale_y))
                }
                "skew_x" => result.insert("skew_x".to_string(), AnimationValue::Float(self.skew_x)),
                "skew_y" => result.insert("skew_y".to_string(), AnimationValue::Float(self.skew_y)),
                "rotation" => {
                    result.insert("rotation".to_string(), AnimationValue::Float(self.rotation))
                }
                _ => None,
            };
        }

        result
    }

    fn set_properties(
        &mut self,
        properties: HashMap<String, AnimationValue>,
    ) -> Result<(), AnimationError> {
        let mut dirty_properties = DirtyUpdates::default();
        for (property, value) in properties {
            match (property.as_str(), value) {
                ("x", AnimationValue::Float(v)) => dirty_properties.x = Some(v),
                ("y", AnimationValue::Float(v)) => dirty_properties.y = Some(v),
                ("width", AnimationValue::Float(v)) => dirty_properties.width = Some(v),
                ("height", AnimationValue::Float(v)) => dirty_properties.height = Some(v),
                ("fill", AnimationValue::String(v)) => dirty_properties.fill = Some(v),
                ("stroke", AnimationValue::String(v)) => dirty_properties.stroke = Some(v),
                ("stroke_width", AnimationValue::Float(v)) => {
                    dirty_properties.stroke_width = Some(v)
                }
                ("opacity", AnimationValue::Float(v)) => dirty_properties.opacity = Some(v),
                ("scale_x", AnimationValue::Float(v)) => dirty_properties.scale_x = Some(v),
                ("scale_y", AnimationValue::Float(v)) => dirty_properties.scale_y = Some(v),
                ("skew_x", AnimationValue::Float(v)) => dirty_properties.skew_x = Some(v),
                ("skew_y", AnimationValue::Float(v)) => dirty_properties.skew_y = Some(v),
                ("rotation", AnimationValue::Float(v)) => dirty_properties.rotation = Some(v),
                _ => return Err(AnimationError::InvalidProperty(property.into())),
            }
        }

        self.set_multiple(dirty_properties);
        Ok(())
    }

    fn is_animatable(&self) -> bool {
        true
    }
}
