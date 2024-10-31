use wasm_bindgen::JsCast;
use web_sys::WebGl2RenderingContext;

use crate::renderer::CanvasType;

struct RenderBuffer {
    width: u32,
    height: u32,
}

impl RenderBuffer {
    fn new(gl: &WebGl2RenderingContext, width: u32, height: u32) -> Self {
        RenderBuffer {
            width,
            height,
        }
    }

    fn bind(&self, gl: &WebGl2RenderingContext) {
        gl.bind_framebuffer(WebGl2RenderingContext::FRAMEBUFFER, None);
        gl.viewport(0, 0, self.width as i32, self.height as i32);
    }


    fn resize(&mut self, gl: &WebGl2RenderingContext, width: u32, height: u32) {
        self.width = width;
        self.height = height;
        gl.viewport(0, 0, width as i32, height as i32);
    }
}

pub struct GLBufferRenderer {
    gl: WebGl2RenderingContext,
    buffer: RenderBuffer,
}

impl GLBufferRenderer {
    pub fn new(canvas: impl Into<CanvasType>, width: u32, height: u32) -> Self {
        let canvas = canvas.into();
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

        // 启用基本特性
        gl.enable(WebGl2RenderingContext::DEPTH_TEST);
        gl.enable(WebGl2RenderingContext::BLEND);
        gl.blend_func(
            WebGl2RenderingContext::SRC_ALPHA,
            WebGl2RenderingContext::ONE_MINUS_SRC_ALPHA,
        );

        // 创建缓冲区
        let buffer = RenderBuffer::new(&gl, width, height);

        GLBufferRenderer { gl, buffer }
    }

    pub fn render_to_buffer(&mut self, render_fn: impl FnOnce(&mut WebGl2RenderingContext)) {
        // self.buffer.bind(&self.gl);
        render_fn(&mut self.gl);
    }

    // pub fn flush_to_canvas(&self) {
    //     // 绑定画布作为渲染目标
    //     self.gl
    //         .bind_framebuffer(WebGl2RenderingContext::DRAW_FRAMEBUFFER, None);

    //     // 绑定缓冲区作为读取源
    //     self.gl.bind_framebuffer(
    //         WebGl2RenderingContext::READ_FRAMEBUFFER,
    //         Some(&self.buffer.resolve_framebuffer),
    //     );

    //     // 复制缓冲区内容到画布
    //     self.gl.blit_framebuffer(
    //         0,
    //         0,
    //         self.buffer.width as i32,
    //         self.buffer.height as i32,
    //         0,
    //         0,
    //         self.buffer.width as i32,
    //         self.buffer.height as i32,
    //         WebGl2RenderingContext::COLOR_BUFFER_BIT,
    //         WebGl2RenderingContext::NEAREST,
    //     );
    // }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.buffer.resize(&self.gl, width, height);
    }

    // 获取 WebGL 上下文的引用
    pub fn gl(&self) -> &WebGl2RenderingContext {
        &self.gl
    }

    // 获取缓冲区的尺寸
    pub fn size(&self) -> (u32, u32) {
        (self.buffer.width, self.buffer.height)
    }
}
