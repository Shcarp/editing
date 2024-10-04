use super::{Dirty, Eventable, ObjectId, Renderable, Transformable};
use crate::{
    animation::{Animatable, Tween},
    helper::{convert_1x6_to_3x3, convert_3x3_to_1x6, get_rotation_matrix},
    render_control::{get_render_control, UpdateBody, UpdateMessage, UpdateType},
    renderer::Renderer,
};
use dirty_setter::DirtySetter;
use nalgebra as na;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use serde_json::Value;

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

#[derive(Debug, Clone, DirtySetter)]
#[derive(Serialize, Deserialize)]
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

    #[serde(skip)]
    pub x_animation: Option<Tween>,
    #[serde(skip)]
    pub y_animation: Option<Tween>,
    #[serde(skip)]
    pub rotation_animation: Option<Tween>,
    #[serde(skip)]
    pub width_animation: Option<Tween>,
    #[serde(skip)]
    pub height_animation: Option<Tween>,
    #[serde(skip)]
    animation_queue: VecDeque<AnimationStage>,
}

#[derive(Debug, Clone)]
struct AnimationStage {
    params: AnimationParams,
    duration: f64,
    easing: fn(f64) -> f64,
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
            animation_queue: VecDeque::new(),
            dirty: true,
        }
    }

    pub fn animate_to(&mut self, params: AnimationParams, duration: f64, easing: fn(f64) -> f64) {
        let stage = AnimationStage {
            params,
            duration,
            easing,
        };
        self.animation_queue.push_back(stage);

        // If this is the only animation in the queue, start it immediately
        if self.animation_queue.len() == 1 {
            self.start_next_animation();
        }
    }

    fn start_next_animation(&mut self) {
        if let Some(stage) = self.animation_queue.front() {
            let AnimationStage {
                params,
                duration,
                easing,
            } = stage;
            if let Some(x) = params.x {
                self.x_animation = Some(Tween::new(self.x, x, *duration, *easing));
            }
            if let Some(y) = params.y {
                self.y_animation = Some(Tween::new(self.y, y, *duration, *easing));
            }
            if let Some(rotation) = params.rotation {
                self.rotation_animation =
                    Some(Tween::new(self.rotation, rotation, *duration, *easing));
            }
            if let Some(width) = params.width {
                self.width_animation = Some(Tween::new(self.width, width, *duration, *easing));
            }
            if let Some(height) = params.height {
                self.height_animation = Some(Tween::new(self.height, height, *duration, *easing));
            }
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
    stroke:  Option<String>,
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
    
    fn update_frame(&mut self, delta_time: f64) {
        self.update_animations(delta_time);
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

impl Rect {
    fn update_animations(&mut self, delta_time: f64) {
        let mut updates = DirtyUpdates::default();
        let mut all_finished = true;

        for (animation, field) in [
            (&mut self.x_animation, &mut updates.x),
            (&mut self.y_animation, &mut updates.y),
            (&mut self.width_animation, &mut updates.width),
            (&mut self.height_animation, &mut updates.height),
            (&mut self.rotation_animation, &mut updates.rotation),
        ] {
            if let Some(ref mut anim) = animation {
                anim.update(delta_time);
                *field = Some(anim.value());
                if anim.is_finished() {
                    *animation = None;
                } else {
                    all_finished = false;
                }
            }
        }

        self.set_multiple(updates);
        
        if all_finished {
            self.animation_queue.pop_front();
            self.start_next_animation();
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct AnimationParams {
    pub x: Option<f64>,
    pub y: Option<f64>,
    pub rotation: Option<f64>,
    pub width: Option<f64>,
    pub height: Option<f64>,
}

impl AnimationParams {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn x(mut self, x: f64) -> Self {
        self.x = Some(x);
        self
    }

    pub fn y(mut self, y: f64) -> Self {
        self.y = Some(y);
        self
    }

    pub fn rotation(mut self, rotation: f64) -> Self {
        self.rotation = Some(rotation);
        self
    }

    pub fn width(mut self, width: f64) -> Self {
        self.width = Some(width);
        self
    }

    pub fn height(mut self, height: f64) -> Self {
        self.height = Some(height);
        self
    }
}