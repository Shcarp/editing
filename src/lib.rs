use wasm_bindgen::prelude::*;
use web_sys::{HtmlCanvasElement, CanvasRenderingContext2d, WebGl2RenderingContext};

static mut CANVAS_DEFAULT_ID: &str = "canvas";

#[wasm_bindgen(start)]
pub fn main_js() -> Result<(), JsValue> {
    console_error_panic_hook::set_once();
    Ok(())
}

pub enum CanvasContext {
    Canvas2d(CanvasRenderingContext2d),
    WebGl2(WebGl2RenderingContext),
    // WebGPU(GpuCanvasContext), // 未来可能添加
}

pub struct App {
    pub canvas: HtmlCanvasElement,
}

pub enum CanvasContextType {
    Canvas2d,
    Webgl2,
    WebGPU,
}

impl App {
    pub fn new() -> Self {
        let window = web_sys::window().unwrap();
        let document = window.document().unwrap();

        let canvas = document.get_element_by_id(unsafe { CANVAS_DEFAULT_ID }).unwrap();
        let canvas = canvas.dyn_into::<HtmlCanvasElement>().unwrap();
        Self { canvas }
    }

    pub fn new_with_id(id: &str) -> Self {
        let window = web_sys::window().unwrap();
        let document = window.document().unwrap();

        let canvas = document.get_element_by_id(id).unwrap();
        let canvas = canvas.dyn_into::<HtmlCanvasElement>().unwrap();

        Self { canvas }
    }

    pub fn get_context(&self, context_type: &str) -> Result<CanvasContext, JsValue> {
        match context_type {
            "2d" => Ok(CanvasContext::Canvas2d(
                self.canvas.get_context("2d")?
                    .ok_or("Failed to get 2D context")?
                    .dyn_into::<CanvasRenderingContext2d>()?
            )),
            "webgl2" => Ok(CanvasContext::WebGl2(
                self.canvas.get_context("webgl2")?
                    .ok_or("Failed to get WebGL2 context")?
                    .dyn_into::<WebGl2RenderingContext>()?
            )),
            _ => Err(JsValue::from_str("Unsupported context type")),
        }
    }
    pub fn get_2d_context(&self) -> Result<CanvasRenderingContext2d, JsValue> {
        Ok(self.canvas
            .get_context("2d")?
            .ok_or_else(|| JsValue::from_str("Failed to get 2D context"))?
            .dyn_into::<CanvasRenderingContext2d>()?)
    }

    pub fn get_webgl2_context(&self) -> Result<WebGl2RenderingContext, JsValue> {
        Ok(self.canvas
            .get_context("webgl2")?
            .ok_or_else(|| JsValue::from_str("Failed to get WebGL2 context"))?
            .dyn_into::<WebGl2RenderingContext>()?)
    }
}
