
use std::cell::RefCell;
use std::rc::Rc;

use pathfinder_renderer::concurrent::rayon::RayonExecutor;
use pathfinder_renderer::options::BuildOptions;
use pathfinder_renderer::scene::Scene;
use wasm_bindgen::{JsCast, JsValue};
use web_sys::{ console, HtmlCanvasElement, OffscreenCanvas, WebGl2RenderingContext};

use pathfinder_canvas::{Canvas, CanvasFontContext, CanvasRenderingContext2D};
use pathfinder_color::ColorF;
use pathfinder_geometry::vector::vec2i;
use pathfinder_renderer::concurrent::executor::SequentialExecutor;
use pathfinder_renderer::gpu::options::{DestFramebuffer, RendererMode, RendererOptions};
use pathfinder_renderer::gpu::renderer::Renderer as PathfinderRenderer;
use pathfinder_resources::embedded::EmbeddedResourceLoader;
use pathfinder_webgl::WebGlDevice;

pub struct Renderer {
    canvas: CanvasType,
    pathfinder_renderer: PathfinderRenderer<WebGlDevice>,
    scene: Scene,
    width: u32,
    height: u32,
}

impl std::fmt::Debug for Renderer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Renderer")
    }
}


impl Renderer {
    pub fn new(canvas: CanvasType, width: u32, height: u32) -> Self {
        let context = match &canvas {
            CanvasType::Html(canvas) => {
                let canvas = canvas.borrow();
                canvas.get_context("webgl2").unwrap().unwrap().dyn_into::<WebGl2RenderingContext>().unwrap()
            },
            CanvasType::Offscreen(canvas) => {
                let canvas = canvas.borrow();
                canvas.get_context("webgl2").unwrap().unwrap().dyn_into::<WebGl2RenderingContext>().unwrap()
            },
        };

        let framebuffer_size = vec2i(width as i32, height as i32);
        let pathfinder_device = WebGlDevice::new(context);

        let mode = RendererMode::default_for_device(&pathfinder_device);
        let options = RendererOptions {
            dest: DestFramebuffer::full_window(framebuffer_size),
            background_color: Some(ColorF::white()),
            ..RendererOptions::default()
        };
        let resource_loader = EmbeddedResourceLoader::new();
        let renderer = PathfinderRenderer::new(pathfinder_device, &resource_loader, mode, options);

        Renderer {
            canvas,
            pathfinder_renderer: renderer,
            scene: Scene::new(),
            width,
            height,
        }
    }

    pub fn render(&mut self, render_fn: impl FnOnce(&mut CanvasRenderingContext2D)) {
        let (width, height) = (self.width, self.height);
        let framebuffer_size = vec2i(width as i32, height as i32);
        let pathfinder_canvas = Canvas::new(framebuffer_size.to_f32());
        
        let font_context = CanvasFontContext::from_system_source();
        let mut ctx = pathfinder_canvas.get_context_2d(font_context);

        render_fn(&mut ctx);

        let mut scene = ctx.into_canvas().into_scene();
        scene.build_and_render(&mut self.pathfinder_renderer, BuildOptions::default(), RayonExecutor);

        self.scene = scene;
    }


}


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

