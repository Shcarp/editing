mod canvas_2d_renderer;
mod offscreen_canvas_2d_renderer;

use std::fmt::Debug;

use crate::image::Image;

pub use canvas_2d_renderer::Canvas2DRenderer;
pub use offscreen_canvas_2d_renderer::OffscreenCanvas2DRenderer;

pub trait Renderer: Debug {
    // 清除方法
    fn clear(&self, x: f64, y: f64, width: f64, height: f64);
    fn clear_all(&self);

    // 基本形状绘制
    fn draw_rectangle(&self, x: f64, y: f64, width: f64, height: f64, color: &str);
    fn draw_circle(&self, x: f64, y: f64, radius: f64, color: &str);
    fn draw_ellipse(&self, x: f64, y: f64, radius_x: f64, radius_y: f64, color: &str);
    fn draw_line(&self, x1: f64, y1: f64, x2: f64, y2: f64, color: &str, width: f64);
    fn draw_polygon(&self, points: &[f64], color: &str);

    // 路径绘制
    fn begin_path(&self);
    fn move_to(&self, x: f64, y: f64);
    fn line_to(&self, x: f64, y: f64);
    fn bezier_curve_to(&self, cp1x: f64, cp1y: f64, cp2x: f64, cp2y: f64, x: f64, y: f64);
    fn quadratic_curve_to(&self, cpx: f64, cpy: f64, x: f64, y: f64);
    fn arc(&self, x: f64, y: f64, radius: f64, start_angle: f64, end_angle: f64);
    fn arc_to(&self, x1: f64, y1: f64, x2: f64, y2: f64, radius: f64);
    fn close_path(&self);
    fn stroke(&self);
    fn fill(&self);

    fn stroke_rect(&self, x: f64, y: f64, width: f64, height: f64);

    // 文本绘制
    fn fill_text(&self, text: &str, x: f64, y: f64);
    fn stroke_text(&self, text: &str, x: f64, y: f64);
    fn measure_text(&self, text: &str) -> f64;

    // 图像绘制
    fn draw_image(&self, image: &Image, x: f64, y: f64);
    fn draw_image_with_size(&self, image: &Image, x: f64, y: f64, width: f64, height: f64);
    fn draw_image_clip(
        &self,
        image: &Image,
        sx: f64,
        sy: f64,
        s_width: f64,
        s_height: f64,
        dx: f64,
        dy: f64,
        d_width: f64,
        d_height: f64,
    );

    // 状态管理
    fn save(&self);
    fn restore(&self);
    fn set_transform(&self, a: f64, b: f64, c: f64, d: f64, e: f64, f: f64);
    fn transform(&self, a: f64, b: f64, c: f64, d: f64, e: f64, f: f64);
    fn translate(&self, x: f64, y: f64);
    fn rotate(&self, angle: f64);
    fn scale(&self, x: f64, y: f64);

    // 样式设置
    fn set_fill_style(&self, style: &str);
    fn set_stroke_style(&self, style: &str);
    fn set_line_width(&self, width: f64);
    fn set_line_cap(&self, cap: LineCap);
    fn set_line_join(&self, join: LineJoin);
    fn set_miter_limit(&self, limit: f64);
    fn set_shadow_color(&self, color: &str);
    fn set_shadow_blur(&self, blur: f64);
    fn set_shadow_offset_x(&self, offset: f64);
    fn set_shadow_offset_y(&self, offset: f64);
    fn set_font(&self, font: &str);
    fn set_text_align(&self, align: TextAlign);
    fn set_text_baseline(&self, baseline: TextBaseline);
    fn set_global_alpha(&self, alpha: f64);
    fn set_global_composite_operation(&self, operation: CompositeOperation);

    // 渐变和图案
    fn create_linear_gradient(&self, x0: f64, y0: f64, x1: f64, y1: f64) -> Box<dyn Gradient>;
    fn create_radial_gradient(
        &self,
        x0: f64,
        y0: f64,
        r0: f64,
        x1: f64,
        y1: f64,
        r1: f64,
    ) -> Box<dyn Gradient>;
    fn create_pattern(&self, image: &Image, repetition: PatternRepetition) -> Box<dyn Pattern>;

    // 像素操作
    fn get_image_data(&self, sx: f64, sy: f64, sw: f64, sh: f64) -> ImageData;
    fn put_image_data(&self, image_data: &ImageData, dx: f64, dy: f64);
}

// 辅助类型定义
pub trait Gradient {
    fn add_gradient_color_stop(&self, offset: f64, color: &str);
}

// pub struct Pattern;
pub trait Pattern {
    fn set_pattern_transform(&self, a: f64, b: f64, c: f64, d: f64, e: f64, f: f64);
}

#[derive(Debug)]
pub struct ImageData(pub web_sys::ImageData);

pub enum LineCap {
    Butt,
    Round,
    Square,
}

impl Into<&'static str> for LineCap {
    fn into(self) -> &'static str {
        match self {
            LineCap::Butt => "butt",
            LineCap::Round => "round",
            LineCap::Square => "square",
        }
    }
}

impl From<LineCap> for String {
    fn from(cap: LineCap) -> Self {
        let str_slice: &'static str = cap.into();
        str_slice.to_string()
    }
}

pub enum LineJoin {
    Miter,
    Round,
    Bevel,
}

impl Into<&'static str> for LineJoin {
    fn into(self) -> &'static str {
        match self {
            LineJoin::Miter => "miter",
            LineJoin::Round => "round",
            LineJoin::Bevel => "bevel",
        }
    }
}

impl From<LineJoin> for String {
    fn from(join: LineJoin) -> Self {
        let str_slice: &'static str = join.into();
        str_slice.to_string()
    }
}

pub enum TextAlign {
    Start,
    End,
    Left,
    Right,
    Center,
}

impl Into<&'static str> for TextAlign {
    fn into(self) -> &'static str {
        match self {
            TextAlign::Start => "start",
            TextAlign::End => "end",
            TextAlign::Left => "left",
            TextAlign::Right => "right",
            TextAlign::Center => "center",
        }
    }
}

impl From<TextAlign> for String {
    fn from(align: TextAlign) -> Self {
        let str_slice: &'static str = align.into();
        str_slice.to_string()
    }
}

pub enum TextBaseline {
    Top,
    Hanging,
    Middle,
    Alphabetic,
    Ideographic,
    Bottom,
}

impl Into<&'static str> for TextBaseline {
    fn into(self) -> &'static str {
        match self {
            TextBaseline::Top => "top",
            TextBaseline::Hanging => "hanging",
            TextBaseline::Middle => "middle",
            TextBaseline::Alphabetic => "alphabetic",
            TextBaseline::Ideographic => "ideographic",
            TextBaseline::Bottom => "bottom",
        }
    }
}

impl From<TextBaseline> for String {
    fn from(baseline: TextBaseline) -> Self {
        let str_slice: &'static str = baseline.into();
        str_slice.to_string()
    }
}

pub enum CompositeOperation {
    SourceOver,
    SourceIn,
    SourceOut,
    SourceAtop,
    DestinationOver,
    DestinationIn,
    DestinationOut,
    DestinationAtop,
    Lighter,
    Copy,
    Xor,
    Multiply,
    Screen,
    Overlay,
    Darken,
    Lighten,
    ColorDodge,
    ColorBurn,
    HardLight,
    SoftLight,
    Difference,
    Exclusion,
    Hue,
    Saturation,
    Color,
    Luminosity,
}

impl Into<&'static str> for CompositeOperation {
    fn into(self) -> &'static str {
        match self {
            CompositeOperation::SourceOver => "source-over",
            CompositeOperation::SourceIn => "source-in",
            CompositeOperation::SourceOut => "source-out",
            CompositeOperation::SourceAtop => "source-atop",
            CompositeOperation::DestinationOver => "destination-over",
            CompositeOperation::DestinationIn => "destination-in",
            CompositeOperation::DestinationOut => "destination-out",
            CompositeOperation::DestinationAtop => "destination-atop",
            CompositeOperation::Lighter => "lighter",
            CompositeOperation::Copy => "copy",
            CompositeOperation::Xor => "xor",
            CompositeOperation::Multiply => "multiply",
            CompositeOperation::Screen => "screen",
            CompositeOperation::Overlay => "overlay",
            CompositeOperation::Darken => "darken",
            CompositeOperation::Lighten => "lighten",
            CompositeOperation::ColorDodge => "color-dodge",
            CompositeOperation::ColorBurn => "color-burn",
            CompositeOperation::HardLight => "hard-light",
            CompositeOperation::SoftLight => "soft-light",
            CompositeOperation::Difference => "difference",
            CompositeOperation::Exclusion => "exclusion",
            CompositeOperation::Hue => "hue",
            CompositeOperation::Saturation => "saturation",
            CompositeOperation::Color => "color",
            CompositeOperation::Luminosity => "luminosity",
        }
    }
}

impl From<CompositeOperation> for String {
    fn from(operation: CompositeOperation) -> Self {
        let str_slice: &'static str = operation.into();
        str_slice.to_string()
    }
}

pub enum PatternRepetition {
    Repeat,
    RepeatX,
    RepeatY,
    NoRepeat,
}

impl Into<&'static str> for PatternRepetition {
    fn into(self) -> &'static str {
        match self {
            PatternRepetition::Repeat => "repeat",
            PatternRepetition::RepeatX => "repeat-x",
            PatternRepetition::RepeatY => "repeat-y",
            PatternRepetition::NoRepeat => "no-repeat",
        }
    }
}

impl From<PatternRepetition> for String {
    fn from(repetition: PatternRepetition) -> Self {
        let str_slice: &'static str = repetition.into();
        str_slice.to_string()
    }
}
