use std::cell::RefCell;
use std::rc::Rc;

use wasm_bindgen::JsCast;
use web_sys::{HtmlCanvasElement, OffscreenCanvas, WebGl2RenderingContext};

use crate::{render_buffer::GLBufferRenderer, shader_program::ShaderProgram};

pub struct Renderer {
    canvas: CanvasType,
    width: u32,
    height: u32,
    gl_renderer: GLBufferRenderer,
}

impl std::fmt::Debug for Renderer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Renderer {{ canvas: {:?}}}", self.canvas)
    }
}

impl Renderer {
    pub fn new(canvas: CanvasType, width: u32, height: u32) -> Self {
        let gl = match &canvas {
            CanvasType::Html(canvas) => {
                let canvas = canvas.borrow();
                canvas
                    .get_context("webgl2")
                    .unwrap()
                    .unwrap()
                    .dyn_into::<WebGl2RenderingContext>()
                    .unwrap()
            }
            CanvasType::Offscreen(canvas) => {
                let canvas = canvas.borrow();
                canvas
                    .get_context("webgl2")
                    .unwrap()
                    .unwrap()
                    .dyn_into::<WebGl2RenderingContext>()
                    .unwrap()
            }
        };
        
        let renderer = GLBufferRenderer::new(canvas.clone(), width, height);

        Renderer {
            canvas,
            width,
            height,
            gl_renderer: renderer,
        }
    }

    // 使用 WebGL 直接渲染
    pub fn render_webgl(&mut self, render_fn: impl FnOnce(&mut WebGl2RenderingContext)) {
        self.gl_renderer.render_to_buffer(render_fn);
        // if self.gl_renderer.has_content() {
        //     self.gl_renderer.flush_to_canvas();
        // }
    }

    // 组合渲染
    pub fn render(
        &mut self,
        render_fn: impl FnOnce(&mut WebGl2RenderingContext),
    ) {
        self.render_webgl(render_fn);
    }

    // 调整大小
    pub fn resize(&mut self, width: u32, height: u32) {
        self.width = width;
        self.height = height;
        
        self.gl_renderer.resize(width, height);
    }
}

// Canvas 类型定义
#[derive(Debug, Clone)]
pub enum CanvasType {
    Html(Rc<RefCell<HtmlCanvasElement>>),
    Offscreen(Rc<RefCell<OffscreenCanvas>>),
}

impl From<HtmlCanvasElement> for CanvasType {
    fn from(canvas: HtmlCanvasElement) -> Self {
        CanvasType::Html(Rc::new(RefCell::new(canvas)))
    }
}

impl From<OffscreenCanvas> for CanvasType {
    fn from(canvas: OffscreenCanvas) -> Self {
        CanvasType::Offscreen(Rc::new(RefCell::new(canvas)))
    }
}

pub struct RenderContext<'a> {
    pub gl: &'a WebGl2RenderingContext,
    pub global_transform: glam::DMat3,
}

impl RenderContext<'_> {
    pub fn calc_transform(&self, transform: glam::DMat3) -> glam::DMat3 {
        self.global_transform * transform
    }

    pub fn get_shader_program(&self) -> &'static ShaderProgram {
        ShaderProgram::instance(self.gl)
    }
}

