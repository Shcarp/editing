use std::{cell::RefCell, f64::consts::PI, rc::Rc};
use wasm_bindgen::JsValue;
use web_sys::OffscreenCanvasRenderingContext2d;

use super::{
    CompositeOperation, Gradient, Image, ImageData, LineCap, LineJoin, Pattern, PatternRepetition,
    Renderer, TextAlign, TextBaseline,
};

pub struct OffscreenCanvas2DRenderer {
    context: OffscreenCanvasRenderingContext2d,
    locked_fill_color: Option<String>,
    locked_stroke_color: Option<String>,
}

impl std::fmt::Debug for OffscreenCanvas2DRenderer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Canvas2DRenderer")
    }
}

impl OffscreenCanvas2DRenderer {
    pub fn new(context: OffscreenCanvasRenderingContext2d) -> Self {
        OffscreenCanvas2DRenderer {
            context,
            locked_fill_color: None,
            locked_stroke_color: None,
        }
    }

    pub fn create_renderer(
        context: OffscreenCanvasRenderingContext2d,
    ) -> Rc<RefCell<Option<Box<dyn Renderer>>>> {
        Rc::new(RefCell::new(Some(
            Box::new(OffscreenCanvas2DRenderer::new(context)) as Box<dyn Renderer>,
        )))
    }

    fn set_fill_color(&self, color: &str) {
        if let Some(locked_color) = &self.locked_fill_color {
            self.context
                .set_fill_style(&JsValue::from_str(locked_color));
        } else {
            self.context.set_fill_style(&JsValue::from_str(color));
        }
    }

    fn set_stroke_color(&self, color: &str) {
        if let Some(locked_color) = &self.locked_stroke_color {
            self.context
                .set_stroke_style(&JsValue::from_str(locked_color));
        } else {
            self.context.set_stroke_style(&JsValue::from_str(color));
        }
    }
}

impl Renderer for OffscreenCanvas2DRenderer {
    fn clear(&self, x: f64, y: f64, width: f64, height: f64) {
        self.context.clear_rect(x, y, width, height);
    }

    fn clear_all(&self) {
        let canvas = self.context.canvas();
        self.context
            .clear_rect(0.0, 0.0, canvas.width() as f64, canvas.height() as f64);
    }

    fn draw_rectangle(&self, x: f64, y: f64, width: f64, height: f64, color: &str) {
        self.set_fill_color(color);
        self.context.fill_rect(x, y, width, height);
    }

    fn draw_circle(&self, x: f64, y: f64, radius: f64, color: &str) {
        self.set_fill_color(color);
        self.context.begin_path();
        self.context.arc(x, y, radius, 0.0, 2.0 * PI).unwrap();
        self.context.fill();
    }

    fn draw_ellipse(&self, x: f64, y: f64, radius_x: f64, radius_y: f64, color: &str) {
        self.set_fill_color(color);
        self.context.begin_path();
        self.context
            .ellipse(x, y, radius_x, radius_y, 0.0, 0.0, 2.0 * PI)
            .unwrap();
        self.context.fill();
    }

    fn draw_line(&self, x1: f64, y1: f64, x2: f64, y2: f64, color: &str, width: f64) {
        self.set_stroke_color(color);
        self.context.set_line_width(width);
        self.context.begin_path();
        self.context.move_to(x1, y1);
        self.context.line_to(x2, y2);
        self.context.stroke();
    }

    fn draw_polygon(&self, points: &[f64], color: &str) {
        if points.len() < 4 || points.len() % 2 != 0 {
            return;
        }
        // self.context.set_fill_style(&JsValue::from_str(color));
        self.set_fill_color(color);
        self.context.begin_path();
        self.context.move_to(points[0], points[1]);
        for i in (2..points.len()).step_by(2) {
            self.context.line_to(points[i], points[i + 1]);
        }
        self.context.close_path();
        self.context.fill();
    }

    fn begin_path(&self) {
        self.context.begin_path();
    }

    fn move_to(&self, x: f64, y: f64) {
        self.context.move_to(x, y);
    }

    fn line_to(&self, x: f64, y: f64) {
        self.context.line_to(x, y);
    }

    fn bezier_curve_to(&self, cp1x: f64, cp1y: f64, cp2x: f64, cp2y: f64, x: f64, y: f64) {
        self.context.bezier_curve_to(cp1x, cp1y, cp2x, cp2y, x, y);
    }

    fn quadratic_curve_to(&self, cpx: f64, cpy: f64, x: f64, y: f64) {
        self.context.quadratic_curve_to(cpx, cpy, x, y);
    }

    fn arc(&self, x: f64, y: f64, radius: f64, start_angle: f64, end_angle: f64) {
        self.context
            .arc(x, y, radius, start_angle, end_angle)
            .unwrap();
    }

    fn arc_to(&self, x1: f64, y1: f64, x2: f64, y2: f64, radius: f64) {
        self.context.arc_to(x1, y1, x2, y2, radius).unwrap();
    }

    fn close_path(&self) {
        self.context.close_path();
    }

    fn stroke(&self) {
        self.context.stroke();
    }

    fn fill(&self) {
        self.context.fill();
    }

    fn fill_text(&self, text: &str, x: f64, y: f64) {
        self.context.fill_text(text, x, y).unwrap();
    }

    fn stroke_text(&self, text: &str, x: f64, y: f64) {
        self.context.stroke_text(text, x, y).unwrap();
    }

    fn stroke_rect(&self, x: f64, y: f64, width: f64, height: f64) {
        self.context.stroke_rect(x, y, width, height);
    }

    fn measure_text(&self, text: &str) -> f64 {
        self.context.measure_text(text).unwrap().width()
    }

    fn draw_image(&self, image: &Image, x: f64, y: f64) {
        let img = image.as_html_image_element();
        self.context
            .draw_image_with_html_image_element(&img, x, y)
            .unwrap();
    }

    fn draw_image_with_size(&self, image: &Image, x: f64, y: f64, width: f64, height: f64) {
        let img = image.as_html_image_element();
        self.context
            .draw_image_with_html_image_element_and_dw_and_dh(&img, x, y, width, height)
            .unwrap();
    }

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
    ) {
        let img = image.as_html_image_element();
        self.context
            .draw_image_with_html_image_element_and_sw_and_sh_and_dx_and_dy_and_dw_and_dh(
                &img, sx, sy, s_width, s_height, dx, dy, d_width, d_height,
            )
            .unwrap();
    }

    fn save(&self) {
        self.context.save();
    }

    fn restore(&self) {
        self.context.restore();
    }

    fn set_transform(&self, a: f64, b: f64, c: f64, d: f64, e: f64, f: f64) {
        self.context.set_transform(a, b, c, d, e, f).unwrap();
    }

    fn transform(&self, a: f64, b: f64, c: f64, d: f64, e: f64, f: f64) {
        self.context.transform(a, b, c, d, e, f).unwrap();
    }

    fn translate(&self, x: f64, y: f64) {
        self.context.translate(x, y).unwrap();
    }

    fn rotate(&self, angle: f64) {
        self.context.rotate(angle).unwrap();
    }

    fn scale(&self, x: f64, y: f64) {
        self.context.scale(x, y).unwrap();
    }

    fn set_fill_style(&self, style: &str) {
        self.set_fill_color(style);
    }

    fn set_stroke_style(&self, style: &str) {
        self.set_stroke_color(style);
    }

    fn set_line_width(&self, width: f64) {
        self.context.set_line_width(width);
    }

    fn set_line_cap(&self, cap: LineCap) {
        let cap_str = cap.into();
        self.context.set_line_cap(cap_str);
    }

    fn set_line_join(&self, join: LineJoin) {
        let join_str = join.into();
        self.context.set_line_join(join_str);
    }

    fn set_miter_limit(&self, limit: f64) {
        self.context.set_miter_limit(limit);
    }

    fn set_shadow_color(&self, color: &str) {
        self.context.set_shadow_color(color);
    }

    fn set_shadow_blur(&self, blur: f64) {
        self.context.set_shadow_blur(blur);
    }

    fn set_shadow_offset_x(&self, offset: f64) {
        self.context.set_shadow_offset_x(offset);
    }

    fn set_shadow_offset_y(&self, offset: f64) {
        self.context.set_shadow_offset_y(offset);
    }

    fn set_font(&self, font: &str) {
        self.context.set_font(font);
    }

    fn set_text_align(&self, align: TextAlign) {
        let align_str = align.into();
        self.context.set_text_align(align_str);
    }

    fn set_text_baseline(&self, baseline: TextBaseline) {
        let baseline_str = baseline.into();
        self.context.set_text_baseline(baseline_str);
    }

    fn set_global_alpha(&self, alpha: f64) {
        self.context.set_global_alpha(alpha);
    }

    fn set_global_composite_operation(&self, operation: CompositeOperation) {
        let operation_str: String = operation.into();
        self.context
            .set_global_composite_operation(&operation_str)
            .unwrap();
    }

    fn create_linear_gradient(&self, x0: f64, y0: f64, x1: f64, y1: f64) -> Box<dyn Gradient> {
        let gradient = self.context.create_linear_gradient(x0, y0, x1, y1);
        Box::new(gradient)
    }

    fn create_radial_gradient(
        &self,
        x0: f64,
        y0: f64,
        r0: f64,
        x1: f64,
        y1: f64,
        r1: f64,
    ) -> Box<dyn Gradient> {
        let gradient = self
            .context
            .create_radial_gradient(x0, y0, r0, x1, y1, r1)
            .unwrap();
        Box::new(gradient)
    }

    fn create_pattern(&self, image: &Image, repetition: PatternRepetition) -> Box<dyn Pattern> {
        let img = image.as_html_image_element();
        let repetition_str = repetition.into();
        let pattern = self
            .context
            .create_pattern_with_html_image_element(&img, repetition_str)
            .unwrap()
            .unwrap();
        Box::new(pattern)
    }

    fn get_image_data(&self, sx: f64, sy: f64, sw: f64, sh: f64) -> ImageData {
        let canvas_image_data = self.context.get_image_data(sx, sy, sw, sh).unwrap();
        ImageData(canvas_image_data)
    }

    fn put_image_data(&self, image_data: &ImageData, dx: f64, dy: f64) {
        self.context.put_image_data(&image_data.0, dx, dy).unwrap();
    }

    fn lock_color(&mut self, color: &str) {
        self.locked_fill_color = Some(color.to_string());
        self.locked_stroke_color = Some(color.to_string());
        self.set_fill_color(color);
        self.set_stroke_color(color);
    }

    fn unlock_color(&mut self) {
        self.locked_fill_color = None;
        self.locked_stroke_color = None;
    }
}
