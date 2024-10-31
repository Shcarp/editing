use super::{Dirty, Eventable, ObjectId, Renderable};
use crate::helper::create_shader_program;
use crate::history::{HistoryItem, ObjectHistoryItem};
use crate::app::App;
use crate::renderer::RenderContext;
use crate::source_manager::{get_source_manager, SourceKey};
use dirty_setter::DirtySetter;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use wasm_bindgen::JsValue;
use web_sys::{console, js_sys, WebGl2RenderingContext, WebGlProgram};
use css_color_parser::Color as CssColor;

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
    cached_transform: Option<glam::DMat3>,

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
            Ok(data) => {
                image.width = data.borrow().width() as f64;
                image.height = data.borrow().height() as f64;
                image.source = Some(source);
            },
            Err(e) => {
                return image;
            }
        }
        image
    }


    pub fn render_fn(&self, ctx: &RenderContext) {
        let current_transform = ctx.calc_transform(self.get_transform());

        let webgl = ctx.gl;

        if let Some(source) = self.source.as_ref() {
            match get_source_manager().get_source_from_key(ctx.gl, source) {
                Ok(texture) => {
                    let program = create_shader_program(&webgl).unwrap();
                    webgl.use_program(Some(&program));
                    webgl.active_texture(WebGl2RenderingContext::TEXTURE0);
                    webgl.bind_texture(WebGl2RenderingContext::TEXTURE_2D, Some(&texture.borrow()));

                    let transform_location = webgl.get_uniform_location(&program, "uTransform");
                    let transform_array: [f32; 9] = current_transform.to_cols_array().iter().map(|&x| x as f32).collect::<Vec<f32>>().try_into().unwrap();
                    webgl.uniform_matrix3fv_with_f32_array(
                        transform_location.as_ref(),
                        false,
                        &transform_array
                    );
                    // 设置顶点数据
                    let vertices = [
                        0.0, 0.0,  // 左下
                        1.0, 0.0,  // 右下
                        0.0, 1.0,  // 左上
                        1.0, 1.0,  // 右上
                    ];
                
                    let vertex_buffer = webgl.create_buffer();
                    webgl.bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, vertex_buffer.as_ref());
                    unsafe {
                        let vertices_array = js_sys::Float32Array::view(&vertices);
                        webgl.buffer_data_with_array_buffer_view(
                            WebGl2RenderingContext::ARRAY_BUFFER,
                            &vertices_array,
                            WebGl2RenderingContext::STATIC_DRAW
                        );
                    }
                    
                    // 设置顶点属性
                    let position_location = webgl.get_attrib_location(&program, "aPosition");
                    webgl.vertex_attrib_pointer_with_i32(
                        position_location as u32,
                        2,
                        WebGl2RenderingContext::FLOAT,
                        false,
                        0,
                        0
                    );
                    webgl.enable_vertex_attrib_array(position_location as u32);
                    
                    // 绘制
                    webgl.draw_arrays(
                        WebGl2RenderingContext::TRIANGLE_STRIP,
                        0,
                        4
                    );
                    
                },
                Err(e) => {
                    console::log_1(&format!("get_source_from_key error: {:?}", e).into());
                    return;
                }
            }
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

    fn render(&self, renderer: &RenderContext) {
        self.render_fn(renderer)
    }

    fn position(&self) -> (f64, f64) {
        (self.x, self.y)
    }

    fn set_position(&mut self, x: f64, y: f64) {
        self.x = x;
        self.y = y;
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
    fn get_transform(&self) -> glam::DMat3 {
        self.cached_transform.clone().unwrap_or(glam::DMat3::default())
    }

    fn calc_transform(&mut self) -> glam::DMat3 {
        if !self.dirty {
            if let Some(cached) = self.cached_transform {
                return cached;
            }
        }

        let center = glam::DVec2::new(
            (self.width / 2.0) as f64,
            (self.height / 2.0) as f64
        );
        
        let final_transform = glam::DMat3::from_translation(glam::DVec2::new(self.x as f64, self.y as f64))
            * glam::DMat3::from_translation(center)
            * glam::DMat3::from_angle(self.rotation.to_radians() as f64)
            * glam::DMat3::from_scale(glam::DVec2::new(self.scale_x as f64, self.scale_y as f64))
            * glam::DMat3::from_translation(-center);

        self.cached_transform = Some(final_transform);
        final_transform
    }
}
