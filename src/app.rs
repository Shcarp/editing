use wasm_bindgen::prelude::*;
use web_sys::{HtmlCanvasElement, CanvasRenderingContext2d, WebGl2RenderingContext, js_sys::Function};
use std::fmt::Debug;
use std::collections::HashMap;

use crate::object_manager::ObjectManager;

pub enum CanvasContext {
    Canvas2d(CanvasRenderingContext2d),
    WebGl2(WebGl2RenderingContext),
}

#[derive(Debug)]
pub enum CanvasContextType {
    Canvas2d,
    WebGl2,   
}


#[derive(Debug)]
#[wasm_bindgen]
pub struct App {
    canvas_id: String,
    canvas: Option<HtmlCanvasElement>,
    injected_methods: HashMap<String, Function>,
    context_type: CanvasContextType,
    object_manager: ObjectManager,
    // texture_manager: TextureManager,
    // shader_manager: ShaderManager,
}

#[wasm_bindgen]
impl App {
    #[wasm_bindgen(constructor)]
    pub fn new(canvas_id: String) -> Self {
        Self {
            canvas_id,
            canvas: None,
            injected_methods: HashMap::new(),
            context_type: CanvasContextType::Canvas2d,
            object_manager: ObjectManager::new(),
        }
    }

    #[wasm_bindgen]
    pub fn init(&mut self) -> Result<(), JsValue> {
        let window = web_sys::window().unwrap();
        let document = window.document().unwrap();

        let canvas = document.get_element_by_id(&self.canvas_id).unwrap();
        let canvas = canvas.dyn_into::<HtmlCanvasElement>().unwrap();

        self.canvas = Some(canvas);
        self.adjust_for_pixel_ratio()?;
        Ok(())
    }

    #[wasm_bindgen]
    pub fn get_pixel_ratio(&self) -> f64 {
        let window = web_sys::window().expect("Should have a window in this context");
        window.device_pixel_ratio()
    }

    #[wasm_bindgen]
    pub fn adjust_for_pixel_ratio(&self) -> Result<(), JsValue> {
        let ratio = self.get_pixel_ratio();
        self.set_pixel_ratio(ratio)
    }

    #[wasm_bindgen]
    pub fn is_support_type(&self, context_type: &str) -> bool {
        let window = web_sys::window().expect("Should have a window in this context");
        let document = window.document().expect("Should have a document on window");
        let canvas = document.create_element("canvas").expect("Should be able to create a canvas");
        let canvas: web_sys::HtmlCanvasElement = canvas.dyn_into::<web_sys::HtmlCanvasElement>().unwrap();

        match context_type {
            "2d" => canvas.get_context(context_type).is_ok(),
            "webgl2" => canvas.get_context(context_type).is_ok(),
            _ => false,
        }
    }

    #[wasm_bindgen]
    pub fn inject_method(&mut self, method_name: &str, method: JsValue) -> Result<(), JsValue> {
        let function = Function::from(method);
        self.injected_methods.insert(method_name.to_string(), function);
        Ok(())
    }

    #[wasm_bindgen]
    pub fn call_injected_method(&self, method_name: &str, args: &JsValue) -> Result<JsValue, JsValue> {
        if let Some(method) = self.injected_methods.get(method_name) {
            method.call1(&JsValue::NULL, args)
        } else {
            Err(JsValue::from_str(&format!("Method '{}' not found", method_name)))
        }
    }

    #[wasm_bindgen]
    pub fn set_context_type(&mut self, context_type: &str) -> Result<(), JsValue> {
        let context_type = match context_type {
            "2d" => CanvasContextType::Canvas2d,
            "webgl2" => CanvasContextType::WebGl2,
            _ => return Err(JsValue::from_str("Unsupported context type")),
        };
        self.context_type = context_type;
        Ok(())
    }
}

impl App {
    pub fn get_context(&self) -> Result<CanvasContext, JsValue> {
        match self.context_type {
            CanvasContextType::Canvas2d => Ok(CanvasContext::Canvas2d(
                self.canvas.as_ref().ok_or("Canvas not found")?.get_context("2d")?
                    .ok_or("Failed to get 2D context")?
                    .dyn_into::<CanvasRenderingContext2d>()?
            )),
            CanvasContextType::WebGl2 => Ok(CanvasContext::WebGl2(
                self.canvas.as_ref().ok_or("Canvas not found")?.get_context("webgl2")?
                    .ok_or("Failed to get WebGL2 context")?
                    .dyn_into::<WebGl2RenderingContext>()?
            )),
            _ => Err(JsValue::from_str("Unsupported context type")),
        }
    }

    pub fn set_pixel_ratio(&self, ratio: f64) -> Result<(), JsValue> {
        let context = self.get_context()?;

        if let Some(canvas) = self.canvas.as_ref() {
            let style = canvas.style();
            let css_width = style.get_property_value("width")?.parse::<f64>().unwrap_or(300.0);
            let css_height = style.get_property_value("height")?.parse::<f64>().unwrap_or(150.0);

            canvas.set_width((css_width * ratio) as u32);
            canvas.set_height((css_height * ratio) as u32);

            style.set_property("width", &format!("{}px", css_width))?;
            style.set_property("height", &format!("{}px", css_height))?;

            match context {
                CanvasContext::Canvas2d(context) => context.scale(ratio, ratio)?,
                CanvasContext::WebGl2(context) => context.viewport(0, 0, canvas.width() as i32, canvas.height() as i32),
            }
        }
        Ok(())
    }

    pub fn start_loop(&self) {
        let window = web_sys::window().unwrap();
        // let request_animation_frame = window.request_animation_frame(callback).unwrap();
        // 调用 object_manager 的 render_all 方法
        // self.object_manager.render_all(&self.get_context().unwrap());
    }
}
