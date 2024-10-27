use super::{Dirty, Eventable, ObjectId, Renderable};
use crate::history::{HistoryItem, ObjectHistoryItem};
use crate::app::App;
use crate::source_manager::{get_source_manager, SourceKey};
use css_color_parser::Color as CssColor;
use dirty_setter::DirtySetter;
use pathfinder_canvas::{vec2f, CanvasRenderingContext2D, RectF, Transform2F, Vector2F};
use pathfinder_color::ColorU;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

pub struct ImageOptions {
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
}

impl Default for ImageOptions {
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
pub struct ImageElement {
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

    #[dirty_setter]
    pub source: Option<SourceKey>,

    #[serde(skip)]
    app: Option<App>,
    #[serde(skip)]
    cached_transform: Option<Transform2F>,
}

impl ImageElement {
    pub fn new(options: ImageOptions) -> Self {
        let id = ObjectId::new();

        let mut image = ImageElement {
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
            app: None,
            cached_transform: None,
            source: None,
        };

        image.calc_transform();
        image
    }

    pub fn new_from_source_key(source: SourceKey) -> Self {
        let mut image = Self::new(Default::default());
        match get_source_manager().load_source(&source) {
            Ok(pattern) => {
                let data = pattern.borrow().clone();
                image.width = data.size().x() as f64;
                image.height = data.size().y() as f64;
                image.source = Some(source);
            },
            Err(e) => {
                return image;
            }
        }
        image
    }


    pub fn render_fn(&self, ctx: &mut CanvasRenderingContext2D) {
        let current_transform = ctx.transform();
        let transform = current_transform * self.get_transform();
        ctx.set_transform(&transform);
        ctx.set_global_alpha(self.opacity as f32);

        if let Some(source) = self.source.as_ref() {
            match get_source_manager().load_source(source) {
                Ok(pattern) => {
                    let data = pattern.borrow().clone();
                    let rect = RectF::new(Vector2F::zero(), Vector2F::new(self.width as f32, self.height as f32));
                    ctx.draw_image(data, rect);
                },
                Err(e) => {
                    return;
                }
            }
        }

        if self.stroke_width > 0.0 {
            ctx.set_line_width(self.stroke_width as f32);
            let offset = self.stroke_width / 2.0;
            let rect = RectF::new(
                Vector2F::new(offset as f32, offset as f32),
                Vector2F::new(
                    (self.width - self.stroke_width) as f32,
                    (self.height - self.stroke_width) as f32
                )
            );
            let color = self.stroke.parse::<CssColor>().unwrap();
            let color_u = ColorU::new(color.r as u8, color.g as u8, color.b as u8, (color.a * 255.0) as u8);
            ctx.set_stroke_style(color_u);
            ctx.stroke_rect(rect);
        }
    }

}

impl Dirty for ImageElement {
    fn set_dirty(&mut self) {
        self.set_dirty_flag(true);
        self.calc_transform();
    }
    fn set_dirty_flag(&mut self, is_dirty: bool) {
        self.dirty = is_dirty;
    }

    fn is_dirty(&self) -> bool {
        self.dirty
    }
}

#[derive(Debug, Clone, Deserialize)]
struct ImageUpdateBoadyData {
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

impl Renderable for ImageElement {
    fn id(&self) -> &ObjectId {
        return &self.id;
    }

    fn update(&mut self, data: Value) {
        self.update(data);
    }

    fn render(&self, renderer: &mut CanvasRenderingContext2D) {
        self.render_fn(renderer)
    }

    fn position(&self) -> (f64, f64) {
        (self.x, self.y)
    }

    fn attach(&mut self, app: &App) {
        self.app = Some(app.clone());
    }

    fn detach(&mut self) {
        self.app = None;
    }

    fn get_type(&self) -> &str {
        "image"
    }

    fn to_value(&self) -> Value {
        json!(self)
    }
}

impl Eventable for ImageElement {}

impl  ImageElement {
    fn get_transform(&self) -> Transform2F {
        self.cached_transform.clone().unwrap_or(Transform2F::default())
    }

    fn calc_transform(&mut self) -> Transform2F {
        if !self.dirty {
            if let Some(cached) = self.cached_transform {
                return cached;
            }
        }

        let base_transform = Transform2F::from_scale(vec2f(self.scale_x as f32, self.scale_y as f32));
        let translate_transform = base_transform.translate(vec2f(self.x as f32, self.y as f32));
        let final_transform = translate_transform.rotate(self.rotation.to_radians() as f32);

        self.cached_transform = Some(final_transform);
        return final_transform;
    }

}
