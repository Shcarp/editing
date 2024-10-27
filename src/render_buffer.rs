use std::cell::RefCell;
use std::rc::Rc;

use wasm_bindgen::JsCast;
use wasm_bindgen_test::console_log;
use web_sys::{
    HtmlCanvasElement, OffscreenCanvas, WebGl2RenderingContext, WebGlFramebuffer, WebGlRenderbuffer,
     WebGlTexture,
};

use crate::renderer::CanvasType;

struct RenderBuffer {
    // 解析后的帧缓冲区
    resolve_framebuffer: WebGlFramebuffer,
    resolve_texture: WebGlTexture,

    width: u32,
    height: u32,
}

impl RenderBuffer {
    fn new(gl: &WebGl2RenderingContext, width: u32, height: u32) -> Self {
        let resolve_framebuffer = gl.create_framebuffer().expect("Failed to create framebuffer");
        let resolve_texture = gl.create_texture().expect("Failed to create texture");

        gl.bind_framebuffer(WebGl2RenderingContext::FRAMEBUFFER, Some(&resolve_framebuffer));
        gl.bind_texture(WebGl2RenderingContext::TEXTURE_2D, Some(&resolve_texture));

            // 设置纹理参数
        gl.tex_parameteri(
            WebGl2RenderingContext::TEXTURE_2D,
            WebGl2RenderingContext::TEXTURE_MIN_FILTER,
            WebGl2RenderingContext::NEAREST as i32,
        );
        gl.tex_parameteri(
            WebGl2RenderingContext::TEXTURE_2D,
            WebGl2RenderingContext::TEXTURE_MAG_FILTER,
            WebGl2RenderingContext::NEAREST as i32,
        );

        // 初始化纹理数据
        gl.tex_image_2d_with_i32_and_i32_and_i32_and_format_and_type_and_opt_u8_array(
            WebGl2RenderingContext::TEXTURE_2D,
            0,
            WebGl2RenderingContext::RGBA as i32,
            width as i32,
            height as i32,
            0,
            WebGl2RenderingContext::RGBA,
            WebGl2RenderingContext::UNSIGNED_BYTE,
            None,
        ).expect("Failed to initialize texture");

        gl.framebuffer_texture_2d(
            WebGl2RenderingContext::FRAMEBUFFER,
            WebGl2RenderingContext::COLOR_ATTACHMENT0,
            WebGl2RenderingContext::TEXTURE_2D,
            Some(&resolve_texture),
            0,
        );

        let status = gl.check_framebuffer_status(WebGl2RenderingContext::FRAMEBUFFER);
        if status != WebGl2RenderingContext::FRAMEBUFFER_COMPLETE {
            panic!("Resolve framebuffer is not complete: {}", status);
        }

        RenderBuffer {
            resolve_framebuffer,
            resolve_texture,
            width,
            height,
        }
    }

    fn bind(&self, gl: &WebGl2RenderingContext) {
        gl.bind_framebuffer(WebGl2RenderingContext::FRAMEBUFFER, Some(&self.resolve_framebuffer));
        gl.viewport(0, 0, self.width as i32, self.height as i32);
    }

    fn resolve(&self, gl: &WebGl2RenderingContext) {
        gl.bind_framebuffer(WebGl2RenderingContext::DRAW_FRAMEBUFFER, Some(&self.resolve_framebuffer));
        gl.blit_framebuffer(
            0, 0, self.width as i32, self.height as i32,
            0, 0, self.width as i32, self.height as i32,
            WebGl2RenderingContext::COLOR_BUFFER_BIT,
            WebGl2RenderingContext::NEAREST,
        );
    }

    fn clear(&self, gl: &WebGl2RenderingContext) {
        gl.bind_framebuffer(WebGl2RenderingContext::FRAMEBUFFER, Some(&self.resolve_framebuffer));
        gl.clear_color(0.0, 0.0, 0.0, 0.0);
        gl.clear(
            WebGl2RenderingContext::COLOR_BUFFER_BIT 
            | WebGl2RenderingContext::DEPTH_BUFFER_BIT 
            | WebGl2RenderingContext::STENCIL_BUFFER_BIT
        );
    }

    fn resize(&mut self, gl: &WebGl2RenderingContext, width: u32, height: u32) {
        self.width = width;
        self.height = height;

        // 重新设置解析纹理
        gl.bind_texture(WebGl2RenderingContext::TEXTURE_2D, Some(&self.resolve_texture));
        gl.tex_image_2d_with_i32_and_i32_and_i32_and_format_and_type_and_opt_u8_array (
            WebGl2RenderingContext::TEXTURE_2D,
            0,
            WebGl2RenderingContext::RGBA as i32,
            width as i32,
            height as i32,
            0,
            WebGl2RenderingContext::RGBA,
            WebGl2RenderingContext::UNSIGNED_BYTE,
            None,
        )
        .unwrap();
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
        self.buffer.bind(&self.gl);
        render_fn(&mut self.gl);
    }

    pub fn flush_to_canvas(&self) {
        // 绑定画布作为渲染目标
        self.gl.bind_framebuffer(WebGl2RenderingContext::DRAW_FRAMEBUFFER, None);
        
        // 绑定缓冲区作为读取源
        self.gl.bind_framebuffer(
            WebGl2RenderingContext::READ_FRAMEBUFFER,
            Some(&self.buffer.resolve_framebuffer),
        );

        // 复制缓冲区内容到画布
        self.gl.blit_framebuffer(
            0,
            0,
            self.buffer.width as i32,
            self.buffer.height as i32,
            0,
            0,
            self.buffer.width as i32,
            self.buffer.height as i32,
            WebGl2RenderingContext::COLOR_BUFFER_BIT,
            WebGl2RenderingContext::NEAREST,
        );
    }

    pub fn clear_buffer(&mut self) {
        self.buffer.clear(&self.gl);
    }

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

    pub fn has_content(&self) -> bool {
        // 绑定帧缓冲区以便读取
        self.gl.bind_framebuffer(
            WebGl2RenderingContext::READ_FRAMEBUFFER,
            Some(&self.buffer.resolve_framebuffer),
        );
        
        // 创建一个缓冲区来存储像素数据
        let pixel_count = (self.buffer.width * self.buffer.height * 4) as usize;
        let mut pixels = vec![0u8; pixel_count];
        
        // 读取像素数据
        self.gl.read_pixels_with_opt_u8_array(
            0, 0,
            self.buffer.width as i32,
            self.buffer.height as i32,
            WebGl2RenderingContext::RGBA,
            WebGl2RenderingContext::UNSIGNED_BYTE,
            Some(&mut pixels),
        ).expect("Failed to read pixels");
        
        // 检查是否所有像素都是0（完全透明）
        !pixels.iter().all(|&x| x == 0)
    }
}
