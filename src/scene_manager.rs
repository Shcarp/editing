use crate::{
    app::App,
    element::Renderable,
    helper::{
        get_canvas, get_canvas_css_size, get_window_dpr,
    },
    history::{HistoryItem, SceneHistoryItem},
    object_manager::ObjectManager,
    renderer::{CanvasType, Renderer},   
};
use pathfinder_canvas::{vec2f, CanvasRenderingContext2D, Transform2F, Vector2F};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{
    cell::{Cell, RefCell},
    collections::HashMap,
    fmt::{Debug, Formatter},
    rc::Rc,
};
use wasm_bindgen::{prelude::Closure, JsCast, JsValue};

use web_sys::{console, window, HtmlCanvasElement, MouseEvent};


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
    canvas_id: String,
    canvas: Option<Rc<RefCell<HtmlCanvasElement>>>,
    renderer: Rc<RefCell<Option<Renderer>>>,
    object_manager: Rc<RefCell<ObjectManager>>,

    zoom: f64,
    offset_x: f64,
    offset_y: f64,
    rotation: f64,

    center_x: f64,
    center_y: f64,

    event_handlers: Rc<RefCell<EventHandlers>>,
    event_listeners: Rc<RefCell<HashMap<String, Closure<dyn FnMut(MouseEvent)>>>>,

    cached_transform: Cell<Option<Transform2F>>,
    transform_dirty: Cell<bool>,

    app: Option<App>,
}

impl Default for SceneManager {
    fn default() -> Self {
        Self::new(SceneManagerOptions::default())
    }
}

impl SceneManager {
    pub fn new(options: SceneManagerOptions) -> Self {
        Self {
            dpr: options.device_pixel_ratio,
            height: options.height,
            width: options.width,
            canvas_id: options.canvas_id,
            canvas: None,
            renderer: Rc::new(RefCell::new(None)),
            object_manager: options.object_manager,
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
            console::log_1(&JsValue::from_str(&format!("physical_width: {}, physical_height: {}", physical_width, physical_height)));

            self.width = Some(physical_width);
            self.height = Some(physical_height);
        }
        self.dpr = Some(ratio);
        Ok(())
    }

}

impl SceneManager {
    pub fn init(&mut self) -> Result<(), JsValue> {
        let dpr = get_window_dpr()?;
        let canvas: HtmlCanvasElement = get_canvas(&self.canvas_id)?;
        let (css_width, css_height) = get_canvas_css_size(&canvas)?;

        self.width = Some(self.width.unwrap_or(css_width));
        self.height = Some(self.height.unwrap_or(css_height));

        let canvas_rc = Rc::new(RefCell::new(canvas));
        
        self.canvas = Some(canvas_rc.clone());

        self.set_pixel_ratio(dpr * 2.0)?;
        let canvas_type = CanvasType::Html(canvas_rc);
        let renderer = Renderer::new(canvas_type, self.width.unwrap(), self.height.unwrap());
        self.renderer = Rc::new(RefCell::new(Some(renderer)));
        self.init_event()?;
        Ok(())
    }
}

impl SceneManager {
    pub fn render(&self) {
        if let Some(renderer) = self.renderer.borrow_mut().as_mut() {
            renderer.render(|ctx| self.render_scene(ctx));
        }
    }

    fn render_scene(&self, ctx: &mut CanvasRenderingContext2D) {
        self.prepare_renderers(ctx);
        self.render_objects(ctx);
        ctx.restore();
    }

    fn prepare_renderers(&self, ctx: &mut CanvasRenderingContext2D) {
        let dpr = web_sys::window().unwrap().device_pixel_ratio();
        ctx.clear();
        ctx.save();
        ctx.scale(vec2f(dpr as f32, dpr as f32));
        ctx.set_line_width(1.0 / dpr as f32);
        let translate_transform = Transform2F::from_translation(vec2f(self.center_x as f32, self.center_y as f32));
        let final_transform = translate_transform * self.calc_transform();
        ctx.set_transform(&final_transform);
    }

    fn render_objects(&self, ctx: &mut CanvasRenderingContext2D) {
        let object_manager = self.object_manager.borrow();
        for object in object_manager.get_objects() {
            let object_borrow = object.borrow();

            ctx.save();
            object_borrow.render(ctx);
            ctx.restore();
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
        // let canvas = self.canvas.as_ref()?;
        // let rect = canvas.borrow().get_bounding_client_rect();
        // let dpr = self.dpr.unwrap_or(1.0);

        // let canvas_x = (event.client_x() as f64 - rect.left()) * dpr;
        // let canvas_y = (event.client_y() as f64 - rect.top()) * dpr;

        // let transform = convert_1x6_to_3x3(self.calc_transform());
        // let inverse_transform = transform.try_inverse()?;

        // let original_point = inverse_transform * na::Vector3::new(canvas_x, canvas_y, 1.0);
        // let (original_x, original_y) = (original_point[0] as f64, original_point[1] as f64);

        // let mut hit_renderer = self.hit_renderer.borrow_mut();
        // let hit_renderer = hit_renderer.as_mut().unwrap();

        // let pixel_data = hit_renderer.get_pixel_color(original_x, original_y);

        // let object_id = ObjectId::get_id_by_color([pixel_data[0], pixel_data[1], pixel_data[2], pixel_data[3]])?;

        // self.object_manager.borrow().get(&object_id)
        None
    }
}

impl SceneManager {
    pub fn calc_transform(&self) -> Transform2F {
        if !self.transform_dirty.get() {
            if let Some(cached) = self.cached_transform.get() {
                return cached;
            }
        }

        let transform = Transform2F::from_scale(self.zoom as f32);
        let new_transform = transform.translate(Vector2F::new(self.offset_x as f32, self.offset_y as f32));
        let final_transform = new_transform.rotate(self.rotation.to_radians() as f32);

        self.cached_transform.set(Some(final_transform));
        self.transform_dirty.set(false);

        final_transform
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
            app.history
                .borrow_mut()
                .push(HistoryItem::SceneUpdate(item));
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

impl Drop for SceneManager {
    fn drop(&mut self) {
        self.cleanup();
    }
}
