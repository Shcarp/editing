use crate::{
    element::Renderable,
    helper::{convert_3x3_to_1x6, get_canvas, get_canvas_css_size, get_window_dpr},
    object_manager::ObjectManager,
    renderer::{Canvas2DRenderer, OffscreenCanvas2DRenderer, Renderer},
};
use nalgebra as na;
use std::{
    cell::RefCell,
    fmt::{Debug, Formatter},
    rc::Rc,
};
use wasm_bindgen::{prelude::Closure, JsCast, JsValue};
use wasm_timer::Instant;
use web_sys::{
    console, window, CanvasRenderingContext2d, HtmlCanvasElement, MouseEvent, OffscreenCanvas,
    OffscreenCanvasRenderingContext2d,
};

#[derive(Debug, Clone)]
pub enum CanvasContextType {
    Canvas2d,
    WebGl2,
}

pub struct SceneManagerOptions {
    pub canvas_id: String,
    pub context_type: Option<CanvasContextType>,
    pub object_manager: Rc<RefCell<ObjectManager>>,
    pub height: Option<u32>,
    pub width: Option<u32>,
    pub device_pixel_ratio: Option<f64>,
}

impl Default for SceneManagerOptions {
    fn default() -> Self {
        let window_dpr = window().unwrap().device_pixel_ratio();
        Self {
            canvas_id: "canvas".to_string(),
            context_type: Some(CanvasContextType::Canvas2d),
            object_manager: Rc::new(RefCell::new(ObjectManager::new())),
            height: Some(1000),
            width: Some(1000),
            device_pixel_ratio: Some(window_dpr),
        }
    }
}

#[derive(Default)]
struct EventHandlers {
    on_mouse_move: Option<Rc<RefCell<dyn Fn(&MouseEvent)>>>,
    on_mouse_down: Option<Rc<RefCell<dyn Fn(&MouseEvent)>>>,
    on_mouse_up: Option<Rc<RefCell<dyn Fn(&MouseEvent)>>>,
    on_mouse_leave: Option<Rc<RefCell<dyn Fn(&MouseEvent)>>>,
}

impl Debug for EventHandlers {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "EventHandlers {{ on_mouse_move, on_mouse_down, on_mouse_up, on_mouse_leave }}")
    }
}

#[derive(Debug, Clone)]
pub struct SceneManager {
    dpr: Option<f64>,
    height: Option<u32>,
    width: Option<u32>,
    context_type: CanvasContextType,
    canvas_id: String,
    canvas: Option<Rc<RefCell<HtmlCanvasElement>>>,
    renderer: Rc<RefCell<Option<Box<dyn Renderer>>>>,
    hit_canvas: Option<Rc<RefCell<OffscreenCanvas>>>,
    hit_renderer: Rc<RefCell<Option<Box<dyn Renderer>>>>,
    object_manager: Rc<RefCell<ObjectManager>>,
    last_update: Instant,

    zoom: f64,
    offset_x: f64,
    offset_y: f64,
    rotation: f64,

    event_handlers: Rc<RefCell<EventHandlers>>,
    event_listeners: Rc<RefCell<Vec<Closure<dyn FnMut(MouseEvent)>>>>,
}

impl Default for SceneManager {
    fn default() -> Self {
        Self::new(SceneManagerOptions::default())
    }
}

impl SceneManager {
    pub fn get_transform(&self) -> na::Matrix1x6<f64> {
        na::Matrix1x6::new(self.zoom, 0.0, 0.0, self.zoom, self.offset_x, self.offset_y)
    }

    pub fn calc_transform(&self) -> na::Matrix1x6<f64> {
        let scale_matrix =
            na::Matrix3::new(self.zoom, 0.0, 0.0, 0.0, self.zoom, 0.0, 0.0, 0.0, 1.0);

        let cos_r = self.rotation.cos();
        let sin_r = self.rotation.sin();
        let rotation_matrix =
            na::Matrix3::new(cos_r, -sin_r, 0.0, sin_r, cos_r, 0.0, 0.0, 0.0, 1.0);

        let translation_matrix = na::Matrix3::new(
            1.0,
            0.0,
            self.offset_x,
            0.0,
            1.0,
            self.offset_y,
            0.0,
            0.0,
            1.0,
        );

        let transform_matrix = scale_matrix * rotation_matrix * translation_matrix;

        convert_3x3_to_1x6(transform_matrix)
    }

    pub fn set_zoom(&mut self, zoom: f64) {
        self.zoom = zoom.max(0.1).min(10.0); // Limit zoom range
    }

    pub fn set_offset(&mut self, x: f64, y: f64) {
        self.offset_x = x;
        self.offset_y = y;
    }

    pub fn set_rotation(&mut self, rotation: f64) {
        self.rotation = rotation % (2.0 * std::f64::consts::PI);
    }

    pub fn pan(&mut self, dx: f64, dy: f64) {
        self.offset_x += dx;
        self.offset_y += dy;
    }

    pub fn zoom_at(&mut self, x: f64, y: f64, factor: f64) {
        let new_zoom = (self.zoom * factor).max(0.1).min(10.0);
        let zoom_change = new_zoom / self.zoom;
        self.offset_x = x - (x - self.offset_x) * zoom_change;
        self.offset_y = y - (y - self.offset_y) * zoom_change;
        self.zoom = new_zoom;
    }

    pub fn reset_transform(&mut self) {
        self.zoom = 1.0;
        self.offset_x = 0.0;
        self.offset_y = 0.0;
        self.rotation = 0.0;
    }
}

impl SceneManager {
    pub fn new(options: SceneManagerOptions) -> Self {
        Self {
            dpr: options.device_pixel_ratio,
            height: options.height,
            width: options.width,
            context_type: options.context_type.unwrap_or(CanvasContextType::Canvas2d),
            canvas_id: options.canvas_id,
            canvas: None,
            renderer: Rc::new(RefCell::new(None)),
            hit_canvas: None,
            hit_renderer: Rc::new(RefCell::new(None)),
            object_manager: options.object_manager,
            last_update: Instant::now(),
            zoom: 1.0,
            offset_x: 0.0,
            offset_y: 0.0,
            rotation: 0.0,

            event_handlers: Rc::new(RefCell::new(EventHandlers::default())),
            event_listeners: Rc::new(RefCell::new(Vec::new())),
        }
    }
}

impl SceneManager {
    pub fn set_pixel_ratio(&mut self, ratio: f64) -> Result<(), JsValue> {
        if let Some(canvas) = self.canvas.as_ref() {
            let style = canvas.borrow().style();

            let css_width = self.width.unwrap() as f64;
            let css_height = self.height.unwrap() as f64;
            let physical_width = (css_width * ratio) as u32;
            let physical_height = (self.width.unwrap() as f64 * ratio) as u32;

            canvas.borrow_mut().set_width(physical_width);
            canvas.borrow_mut().set_height(physical_height);

            style.set_property("width", &format!("{}px", css_width))?;
            style.set_property("height", &format!("{}px", css_height))?;

            // Update hit_canvas
            if let Some(hit_canvas) = &mut self.hit_canvas {
                hit_canvas.borrow_mut().set_width(physical_width);
                hit_canvas.borrow_mut().set_height(physical_height);
            }

            self.renderer
                .borrow_mut()
                .as_mut()
                .unwrap()
                .scale(ratio, ratio);
            self.hit_renderer
                .borrow_mut()
                .as_mut()
                .unwrap()
                .scale(ratio, ratio);
        }
        self.dpr = Some(ratio);
        Ok(())
    }

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

impl SceneManager {
    pub fn init(&mut self) -> Result<(), JsValue> {
        let dpr = get_window_dpr()?;
        let canvas = get_canvas(&self.canvas_id)?;
        let (css_width, css_height) = get_canvas_css_size(&canvas)?;

        self.width = Some(self.width.unwrap_or(css_width));
        self.height = Some(self.height.unwrap_or(css_height));

        let hit_canvas = OffscreenCanvas::new(
            (self.width.unwrap() as f64 * dpr) as u32,
            (self.height.unwrap() as f64 * dpr) as u32,
        )
        .unwrap();

        // 初始化渲染器
        let (renderer, hit_renderer) = match self.context_type {
            CanvasContextType::Canvas2d => {
                let context: CanvasRenderingContext2d = canvas
                    .get_context("2d")?
                    .ok_or_else(|| JsValue::from_str("Failed to get 2D context"))?
                    .dyn_into::<CanvasRenderingContext2d>()?;

                let renderer = Canvas2DRenderer::create_renderer(context);
                let hit_context: OffscreenCanvasRenderingContext2d = hit_canvas
                    .get_context("2d")?
                    .ok_or_else(|| JsValue::from_str("Failed to get 2D context"))?
                    .dyn_into::<OffscreenCanvasRenderingContext2d>()?;

                let hit_renderer = OffscreenCanvas2DRenderer::create_renderer(hit_context);
                (renderer, hit_renderer)
            }
            _ => return Err(JsValue::from_str("Unsupported context type")),
        };

        self.renderer = renderer;
        self.hit_renderer = hit_renderer;
        self.canvas = Some(Rc::new(RefCell::new(canvas)));
        self.hit_canvas = Some(Rc::new(RefCell::new(hit_canvas)));

        self.set_pixel_ratio(dpr)?;

        self.init_event()?;
        Ok(())
    }
}

impl SceneManager {
    pub fn render(&mut self, delta_time: f64) {
        if let Some(renderer) = self.renderer.borrow_mut().as_mut() {
            self.render_fn(renderer, delta_time, &Self::render_objects);
        }
        self.update_hit_canvas();
    }

    fn update_hit_canvas(&self) {
        let mut binding = self.hit_renderer.borrow_mut();
        let hit_renderer = binding.as_mut().unwrap();
        self.render_fn(hit_renderer, 0.0, &Self::render_hit_objects);
    }

    fn render_fn<F>(&self, renderer: &mut Box<dyn Renderer>, delta_time: f64, render_objects: F)
    where
        F: Fn(&Self, &mut Box<dyn Renderer>, f64),
    {
        renderer.clear_all();
        renderer.save();
        self.prepare_renderer(renderer);
        self.apply_transform(renderer);
        render_objects(self, renderer, delta_time);
        renderer.restore();
    }

    fn prepare_renderer(&self, renderer: &mut Box<dyn Renderer>) {
        let dpr = web_sys::window().unwrap().device_pixel_ratio() as f64;
        renderer.set_line_width(1.0 / dpr);
    }

    fn apply_transform(&self, renderer: &mut Box<dyn Renderer>) {
        let transform = self.calc_transform();
        renderer.transform(
            transform[0],
            transform[1],
            transform[2],
            transform[3],
            transform[4],
            transform[5],
        );
    }

    fn render_objects(&self, renderer: &mut Box<dyn Renderer>, delta_time: f64) {
        let object_manager = self.object_manager.borrow();
        for object in object_manager.get_objects() {
            renderer.save();
            object.borrow_mut().render(&mut **renderer, delta_time);
            renderer.restore();
        }
    }

    fn render_hit_objects(&self, renderer: &mut Box<dyn Renderer>, delta_time: f64) {
        let object_manager = self.object_manager.borrow();
        for object in object_manager.get_objects() {
            let color = object.borrow().id().color();
            let fill_color = format!("rgba({},{},{},{})", color.0, color.1, color.2, color.3);
            renderer.save();
            object
                .borrow_mut()
                .render_hit(&mut **renderer, &fill_color, delta_time);
            renderer.restore();
        }
    }

    pub fn update_time(&mut self) -> f64 {
        let now = Instant::now();
        let delta_time = (now - self.last_update).as_secs_f64();
        self.last_update = now;
        delta_time
    }
}

impl SceneManager {
    pub fn init_event(&mut self) -> Result<(), JsValue> {
        let event_handlers = self.event_handlers.clone();
        let canvas = self
            .canvas
            .as_ref()
            .ok_or_else(|| JsValue::from_str("Canvas not initialized"))?;

        self.create_and_add_event_listeners(canvas.clone(), event_handlers)?;
        self.set_default_event_handlers();

        Ok(())
    }

    fn create_and_add_event_listeners(
        &mut self,
        canvas: Rc<RefCell<HtmlCanvasElement>>,
        event_handlers: Rc<RefCell<EventHandlers>>,
    ) -> Result<(), JsValue> {
        let event_types = ["mousemove", "mousedown", "mouseup", "mouseleave"];

        for event_type in event_types.iter() {
            let closure = self.create_event_closure(event_handlers.clone(), event_type);
            canvas
                .borrow_mut()
                .add_event_listener_with_callback(event_type, closure.as_ref().unchecked_ref())?;
            self.event_listeners.borrow_mut().push(closure);
        }

        Ok(())
    }

    fn create_event_closure(
        &self,
        event_handlers: Rc<RefCell<EventHandlers>>,
        event_type: &'static str,
    ) -> Closure<dyn FnMut(MouseEvent)> {
        Closure::wrap(Box::new(move |event: MouseEvent| {
            let handlers = event_handlers.borrow();
            let handler = match event_type {
                "mousemove" => &handlers.on_mouse_move,
                "mousedown" => &handlers.on_mouse_down,
                "mouseup" => &handlers.on_mouse_up,
                "mouseleave" => &handlers.on_mouse_leave,
                _ => return,
            };
            if let Some(handler) = handler {
                handler.borrow()(&event);
            }
        }) as Box<dyn FnMut(MouseEvent)>)
    }

    fn set_default_event_handlers(&mut self) {
        let self_clone = self.clone();
        self.set_on_mouse_move(move |event| {
            // console::log_1(&format!("mousemove: {:?}", event).into());
            self_clone.get_trigger_object(&event);
        });
        self.set_on_mouse_down(|event| {
            // console::log_1(&format!("mousedown: {:?}", event).into());
        });
        self.set_on_mouse_up(|event| {
            // console::log_1(&format!("mouseup: {:?}", event).into());
        });
        self.set_on_mouse_leave(|event| {
            // console::log_1(&format!("mouseleave: {:?}", event).into());
        });
    }

    // Methods to set event handlers
    pub fn set_on_mouse_move(&mut self, handler: impl Fn(&MouseEvent) + 'static) {
        self.event_handlers.borrow_mut().on_mouse_move = Some(Rc::new(RefCell::new(handler)));
    }

    pub fn set_on_mouse_down(&mut self, handler: impl Fn(&MouseEvent) + 'static) {
        self.event_handlers.borrow_mut().on_mouse_down = Some(Rc::new(RefCell::new(handler)));
    }

    pub fn set_on_mouse_up(&mut self, handler: impl Fn(&MouseEvent) + 'static) {
        self.event_handlers.borrow_mut().on_mouse_up = Some(Rc::new(RefCell::new(handler)));
    }

    pub fn set_on_mouse_leave(&mut self, handler: impl Fn(&MouseEvent) + 'static) {
        self.event_handlers.borrow_mut().on_mouse_leave = Some(Rc::new(RefCell::new(handler)));
    }

    // Add a cleanup method
    pub fn cleanup(&mut self) {
        if let Some(canvas) = &self.canvas {
            for listener in self.event_listeners.borrow_mut().drain(..) {
                canvas
                    .borrow_mut()
                    .remove_event_listener_with_callback(
                        "mousemove",
                        listener.as_ref().unchecked_ref(),
                    )
                    .unwrap();
            }
        }
    }

    fn get_trigger_object(&self, event: &MouseEvent) -> Option<Rc<RefCell<dyn Renderable>>> {
        console::log_1(&event);
        // 从事件中获取点击的 位置, 然后从 hit_canvas 中获取点击的像素
        // let x = event.client_x() as f64;
        // let y = event.client_y() as f64;
        // let point = na::Vector3::new(x, y, 0.0);

        // let pixel_data = self
        //     .hit_renderer
        //     .borrow_mut()
        //     .as_ref()
        //     .unwrap()
        //     .get_image_data(x, y, 1.0, 1.0);

        // console::log_1(&format!("{:?}", pixel_data.0.data()).into());

        None
    }
}

impl Drop for SceneManager {
    fn drop(&mut self) {
        self.cleanup();
    }
}
