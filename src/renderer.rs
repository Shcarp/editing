use std::cell::RefCell;
use std::rc::Rc;

use pathfinder_renderer::options::BuildOptions;
use pathfinder_renderer::scene::Scene;
use wasm_bindgen::JsCast;
use wasm_bindgen_test::console_log;
use web_sys::{HtmlCanvasElement, OffscreenCanvas, WebGl2RenderingContext};
use pathfinder_canvas::{Canvas, CanvasFontContext, CanvasRenderingContext2D};
use pathfinder_renderer::gpu::renderer::Renderer as PathfinderRenderer;
use pathfinder_geometry::vector::vec2i;
use pathfinder_renderer::concurrent::executor::SequentialExecutor;
use pathfinder_renderer::gpu::options::{DestFramebuffer, RendererMode, RendererOptions};
use pathfinder_resources::embedded::EmbeddedResourceLoader;
use pathfinder_webgl::WebGlDevice;

use crate::render_buffer::GLBufferRenderer;

pub struct Renderer {
    canvas: CanvasType,
    pathfinder_renderer: Option<PathfinderRenderer<WebGlDevice>>,
    scene: Scene,
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

        // 初始化 Pathfinder 渲染器
        let framebuffer_size = vec2i(width as i32, height as i32);
        let pathfinder_device = WebGlDevice::new(gl.clone());
        let mode = RendererMode::default_for_device(&pathfinder_device);
        let options = RendererOptions {
            dest: DestFramebuffer::full_window(framebuffer_size),
            background_color: None, // 透明背景
            ..RendererOptions::default()
        };
        let resource_loader = EmbeddedResourceLoader::new();
        let pathfinder_renderer = Some(PathfinderRenderer::new(
            pathfinder_device,
            &resource_loader,
            mode,
            options,
        ));

        let renderer = GLBufferRenderer::new(canvas.clone(), width, height);

        Renderer {
            canvas,
            pathfinder_renderer,
            scene: Scene::new(),
            width,
            height,
            gl_renderer: renderer,
        }
    }

    // 使用 Pathfinder 进行矢量渲染
    pub fn render_vector(&mut self, render_fn: impl FnOnce(&mut CanvasRenderingContext2D)) {
        let framebuffer_size = vec2i(self.width as i32, self.height as i32);
        let pathfinder_canvas = Canvas::new(framebuffer_size.to_f32());
        let font_context = CanvasFontContext::from_system_source();
        let mut ctx = pathfinder_canvas.get_context_2d(font_context);

        render_fn(&mut ctx);

        self.scene = ctx.into_canvas().into_scene();
        if let Some(renderer) = &mut self.pathfinder_renderer {
            self.scene.build_and_render(renderer, BuildOptions::default(), SequentialExecutor);
        }

    }

    // 使用 WebGL 直接渲染
    pub fn render_webgl(&mut self, render_fn: impl FnOnce(&mut WebGl2RenderingContext)) {
        self.gl_renderer.render_to_buffer(render_fn);
        if self.gl_renderer.has_content() {
            self.gl_renderer.flush_to_canvas();
        } else {
            // self.gl_renderer.clear_buffer();
            console_log!("no content");
        }
    }

    // 组合渲染
    pub fn render(
        &mut self,
        vector_fn: impl FnOnce(&mut CanvasRenderingContext2D),
        webgl_fn: impl FnOnce(&mut WebGl2RenderingContext),
    ) {
        // 先进行矢量渲染
        self.render_vector(vector_fn);
        
        // 然后进行 WebGL 渲染
        self.render_webgl(webgl_fn);
    }

    // 调整大小
    pub fn resize(&mut self, width: u32, height: u32) {
        self.width = width;
        self.height = height;
        
        // 更新 Pathfinder 渲染器的视口
        if let Some(renderer) = &mut self.pathfinder_renderer {
            let framebuffer_size = vec2i(width as i32, height as i32);
            renderer.options_mut().dest = DestFramebuffer::full_window(framebuffer_size);
        }
        
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