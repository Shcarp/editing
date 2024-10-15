use crate::{
    app::App, element::{ObjectId, Renderable}, helper::{
        convert_1x6_to_3x3, convert_3x3_to_1x6, get_canvas, get_canvas_css_size, get_window_dpr,
    }, history::{HistoryItem, SceneHistoryItem}, object_manager::ObjectManager, renderer::{Canvas2DRenderer, OffscreenCanvas2DRenderer, Renderer}
};
use nalgebra as na;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{
    cell::{Cell, RefCell},
    collections::HashMap,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SceneDirtyData {
    pub zoom: f64,
    pub offset_x: f64,
    pub offset_y: f64,
    pub rotation: f64,
    pub height: u32,
    pub width: u32,
    pub dpr: f64,
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
            height: None,
            width: None,
            device_pixel_ratio: Some(window_dpr),
        }
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

    center_x: f64,
    center_y: f64,

    event_handlers: Rc<RefCell<EventHandlers>>,
    event_listeners: Rc<RefCell<HashMap<String, Closure<dyn FnMut(MouseEvent)>>>>,

    cached_transform: Cell<Option<na::Matrix1x6<f64>>>,
    transform_dirty: Cell<bool>,
    
    app: Option<App>,
}

impl Default for SceneManager {
    fn default() -> Self {
        Self::new(SceneManagerOptions::default())
    }
}

impl SceneManager {
    pub fn calc_transform(&self) -> na::Matrix1x6<f64> {
        if !self.transform_dirty.get() {
            if let Some(cached) = self.cached_transform.get() {
                return cached;
            }
        }

        let scale_matrix =
            na::Matrix3::new(self.zoom, 0.0, 0.0, 0.0, self.zoom, 0.0, 0.0, 0.0, 1.0);

        let cos_r = self.rotation.cos();
        let sin_r = self.rotation.sin();
        let rotation_matrix =
            na::Matrix3::new(cos_r, -sin_r, 0.0, sin_r, cos_r, 0.0, 0.0, 0.0, 1.0);

        let translation_matrix = na::Matrix3::new(
            1.0,
            0.0,
            self.offset_x ,
            0.0,
            1.0,
            self.offset_y,
            0.0,
            0.0,
            1.0,
        );

        let transform_matrix = scale_matrix * rotation_matrix * translation_matrix;
        let result = convert_3x3_to_1x6(transform_matrix);

        self.cached_transform.set(Some(result));
        self.transform_dirty.set(false);

        result
    }

    pub fn set_zoom(&mut self, zoom: f64) {
        let old_data = self.get_dirty_data();
        self.zoom = zoom.max(0.1).min(10.0); // Limit zoom range
        let new_data = self.get_dirty_data();
        self.set_transform_direct(old_data, new_data);
    }

    pub fn set_offset(&mut self, x: f64, y: f64) {
        let old_data = self.get_dirty_data();
        self.offset_x = x;
        self.offset_y = y;
        let new_data = self.get_dirty_data();
        self.set_transform_direct(old_data, new_data);
    }

    pub fn set_rotation(&mut self, rotation: f64) {
        let old_data = self.get_dirty_data();
        self.rotation = rotation % (2.0 * std::f64::consts::PI);
        let new_data = self.get_dirty_data();
        self.set_transform_direct(old_data, new_data);
    }

    pub fn pan(&mut self, dx: f64, dy: f64) {
        let old_data = self.get_dirty_data();
        self.offset_x += dx;
        self.offset_y += dy;
        let new_data = self.get_dirty_data();
        self.set_transform_direct(old_data, new_data);
    }

    pub fn zoom_at(&mut self, x: f64, y: f64, factor: f64) {
        let old_data = self.get_dirty_data();
        let new_zoom = (self.zoom * factor).max(0.1).min(10.0);
        let zoom_change = new_zoom / self.zoom;
        self.offset_x = x - (x - self.offset_x) * zoom_change;
        self.offset_y = y - (y - self.offset_y) * zoom_change;
        self.zoom = new_zoom;
        let new_data = self.get_dirty_data();
        self.set_transform_direct(old_data, new_data);
    }

    pub fn reset_transform(&mut self) {
        let old_data = self.get_dirty_data();
        self.zoom = 1.0;
        self.offset_x = 0.0;
        self.offset_y = 0.0;
        self.rotation = 0.0;
        let new_data = self.get_dirty_data();
        self.set_transform_direct(old_data, new_data);
    }

    pub fn set_transform_direct(&self, old_data: SceneDirtyData, new_data: SceneDirtyData) {
        self.transform_dirty.set(true);
        if let Some(app) = &self.app {
            let item = SceneHistoryItem::new(
                serde_json::to_value(old_data).unwrap(),
                serde_json::to_value(new_data).unwrap(),
            );
            app.history.borrow_mut().push(HistoryItem::SceneUpdate(item));
            app.request_render();
        }
    }

    pub fn set_height(&mut self, height: u32) {
        let old_data = self.get_dirty_data();
        self.height = Some(height);
        let new_data = self.get_dirty_data();
        self.set_transform_direct(old_data, new_data);
    }

    pub fn set_width(&mut self, width: u32) {
        let old_data = self.get_dirty_data();
        self.width = Some(width);
        let new_data = self.get_dirty_data();
        self.set_transform_direct(old_data, new_data);
    }

    pub fn set_dpr(&mut self, dpr: f64) {
        let old_data = self.get_dirty_data();
        self.dpr = Some(dpr);
        let new_data = self.get_dirty_data();
        self.set_transform_direct(old_data, new_data);
    }

    pub fn update_rotation(&mut self, rotation_speed: f64) {
        let old_data = self.get_dirty_data();
        self.rotation += rotation_speed;
        let new_data = self.get_dirty_data();
        self.set_transform_direct(old_data, new_data);
    }

    // 设置旋转中心
    pub fn set_center(&mut self, x: f64, y: f64) {
        let old_data = self.get_dirty_data();
        self.center_x = x;
        self.center_y = y;
        let new_data = self.get_dirty_data();
        self.set_transform_direct(old_data, new_data);
    }

    fn get_dirty_data(&self) -> SceneDirtyData {
        SceneDirtyData {
            zoom: self.zoom,
            offset_x: self.offset_x,
            offset_y: self.offset_y,
            rotation: self.rotation,
            height: self.height.unwrap(),
            width: self.width.unwrap(),
            dpr: self.dpr.unwrap(),
        }
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

            center_x: 0.0,
            center_y: 0.0,

            event_handlers: Rc::new(RefCell::new(EventHandlers::default())),
            event_listeners: Rc::new(RefCell::new(HashMap::new())),

            cached_transform: Cell::new(None),
            transform_dirty: Cell::new(true),

            app: None,
        }
    }

    pub fn attach(&mut self, app: &App) {
        self.app = Some(app.clone());
    }

    pub fn detach(&mut self) {
        self.app = None;
    }

    pub fn update_scene(&mut self, data: Value) {
        let dirty_data: SceneDirtyData = serde_json::from_value(data).unwrap();

        self.set_zoom(dirty_data.zoom);
        self.set_offset(dirty_data.offset_x, dirty_data.offset_y);
        self.set_rotation(dirty_data.rotation);
        self.set_height(dirty_data.height);
        self.set_width(dirty_data.width);
        self.set_dpr(dirty_data.dpr);
    }

    pub fn reset_to_initial_state(&mut self) {
        self.set_zoom(1.0);
        self.set_offset(0.0, 0.0);
        self.set_rotation(0.0);
        self.set_height(self.height.unwrap());
        self.set_width(self.width.unwrap());
        self.set_dpr(self.dpr.unwrap());
    }
}

impl SceneManager {
    pub fn set_pixel_ratio(&mut self, ratio: f64) -> Result<(), JsValue> {
        // let (css_width, css_height) = get_canvas_css_size(&canvas)?;
        if let Some(canvas) = self.canvas.as_ref() {
            let size_canvas = get_canvas(&self.canvas_id)?;
            let (css_width, css_height) = get_canvas_css_size(&size_canvas)?;

            let physical_width = (css_width as f64 * ratio) as u32;
            let physical_height = (css_height as f64 * ratio) as u32;

            canvas.borrow_mut().set_width(physical_width);
            canvas.borrow_mut().set_height(physical_height);

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

        self.set_pixel_ratio(dpr * 2.0)?;

        self.init_event()?;
        Ok(())
    }
}

impl SceneManager {
    pub fn render(&self) {
        let mut renderer = self.renderer.borrow_mut();
        let mut hit_renderer = self.hit_renderer.borrow_mut();

        if let (Some(renderer), Some(hit_renderer)) = (renderer.as_mut(), hit_renderer.as_mut()) {
            self.render_scene(renderer, hit_renderer);
        }
    }

    fn render_scene(&self, renderer: &mut Box<dyn Renderer>, hit_renderer: &mut Box<dyn Renderer>) {
        self.prepare_renderers(renderer, hit_renderer);
        self.render_objects(renderer, hit_renderer);
        self.restore_renderers(renderer, hit_renderer);
    }

    fn prepare_renderers(
        &self,
        renderer: &mut Box<dyn Renderer>,
        hit_renderer: &mut Box<dyn Renderer>,
    ) {
        let dpr = web_sys::window().unwrap().device_pixel_ratio() as f64;
        let transform = self.calc_transform();

        for r in &mut [renderer, hit_renderer] {
            r.clear_all();
            r.save();
            r.set_line_width(1.0 / dpr);
            
            // Translate to the rotation center
            r.translate(self.center_x, self.center_y);
            
            // Apply the transformation
            r.transform(
                transform[0],
                transform[1],
                transform[2],
                transform[3],
                transform[4],
                transform[5],
            );
            
            // Translate back from the rotation center
            r.translate(-self.center_x, -self.center_y);
        }
    }

    fn render_objects(
        &self,
        renderer: &mut Box<dyn Renderer>,
        hit_renderer: &mut Box<dyn Renderer>,
    ) {
        let object_manager = self.object_manager.borrow();
        for object in object_manager.get_objects() {
            let object_borrow = object.borrow();

            // 渲染到主画布
            renderer.save();
            object_borrow.render(&mut **renderer);
            renderer.restore();

            // 渲染到hit测试画布
            let color = object_borrow.id().color();
            let fill_color = format!("rgba({},{},{},{})", color.0, color.1, color.2, color.3);
            hit_renderer.save();
            hit_renderer.lock_color(&fill_color);
            object_borrow.render(&mut **hit_renderer);
            hit_renderer.unlock_color();
            hit_renderer.restore();
        }
    }

    fn restore_renderers(
        &self,
        renderer: &mut Box<dyn Renderer>,
        hit_renderer: &mut Box<dyn Renderer>,
    ) {
        renderer.restore();
        hit_renderer.restore();
    }

    pub fn update_time(&mut self) -> f64 {
        let now = Instant::now();
        let delta_time = (now - self.last_update).as_secs_f64();
        self.last_update = now;
        delta_time
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
        write!(
            f,
            "EventHandlers {{ on_mouse_move, on_mouse_down, on_mouse_up, on_mouse_leave }}"
        )
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
            self.event_listeners
                .borrow_mut()
                .insert(event_type.to_string(), closure);
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
            self_clone.get_trigger_object(&event);
        });
        let self_clone_down = self.clone();
        self.set_on_mouse_down(move |event| {
            if let Some(obj) = self_clone_down.get_trigger_object(&event) {
                console::log_1(&format!("mousedown: {:#?}", obj).into());
            }
        });
        let self_clone_up = self.clone();
        self.set_on_mouse_up(move |event| {
            if let Some(obj) = self_clone_up.get_trigger_object(&event) {
                console::log_1(&format!("mouseup: {:#?}", obj).into());
            }
        });
        let self_clone_leave = self.clone();
        self.set_on_mouse_leave(move |event| {
            if let Some(obj) = self_clone_leave.get_trigger_object(&event) {
                console::log_1(&format!("mouseleave: {:#?}", obj).into());
            }
        });
    }

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
            for (event_type, listener) in self.event_listeners.borrow_mut().drain() {
                match canvas.borrow_mut().remove_event_listener_with_callback(
                    &event_type,
                    listener.as_ref().unchecked_ref(),
                ) {
                    Ok(_) => console::log_1(
                        &format!("Successfully removed {} event listener", event_type).into(),
                    ),
                    Err(e) => console::error_1(
                        &format!("Failed to remove {} event listener: {:?}", event_type, e).into(),
                    ),
                }
            }
        } else {
            console::warn_1(&"Canvas not found during cleanup".into());
        }
    }

    fn get_trigger_object(&self, event: &MouseEvent) -> Option<Rc<RefCell<Box<dyn Renderable>>>> {
        let canvas = self.canvas.as_ref()?;
        let rect = canvas.borrow().get_bounding_client_rect();
        let dpr = self.dpr.unwrap_or(1.0);

        let canvas_x = (event.client_x() as f64 - rect.left()) * dpr;
        let canvas_y = (event.client_y() as f64 - rect.top()) * dpr;

        let transform = convert_1x6_to_3x3(self.calc_transform());
        let inverse_transform = transform.try_inverse()?;

        let original_point = inverse_transform * na::Vector3::new(canvas_x, canvas_y, 1.0);
        let (original_x, original_y) = (original_point[0] as f64, original_point[1] as f64);

        let binding = self.hit_renderer.borrow();
        let hit_renderer = binding.as_ref()?;
        let pixel_data = hit_renderer.get_image_data(original_x, original_y, 1.0, 1.0);

        let color_id = pixel_data.0.data();
        let object_id =
            ObjectId::get_id_by_color([color_id[0], color_id[1], color_id[2], color_id[3]])?;

        self.object_manager.borrow().get(&object_id)
    }
}

impl Drop for SceneManager {
    fn drop(&mut self) {
        self.cleanup();
    }
}