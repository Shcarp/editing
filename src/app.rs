use std::cell::RefCell;
use std::fmt::Debug;
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::spawn_local;

use web_sys::console;
use web_sys::js_sys::Promise;

use crate::animation::AnimationManager;
use crate::element::Renderable;
use crate::events::{get_event_system, AppEvent};
use crate::helper::request_animation_frame;
use crate::object_manager::ObjectManager;
use crate::render_control::get_render_control;
use crate::render_control::UpdateMessage;
use crate::scene_manager::SceneManager;
use crate::scene_manager::SceneManagerOptions;

#[derive(Debug)]
pub struct App {
    pub animation_manager: Rc<RefCell<AnimationManager>>,
    pub object_manager: Rc<RefCell<ObjectManager>>,
    pub scene_manager: Rc<RefCell<SceneManager>>,
}

impl App {
    pub fn new(canvas_id: String) -> Self {
        let object_manager = Rc::new(RefCell::new(ObjectManager::new()));
        let mut options = SceneManagerOptions::default();
        options.canvas_id = canvas_id;
        options.object_manager = object_manager.clone();

        let scene_manager = Rc::new(RefCell::new(SceneManager::new(options)));

        Self {
            object_manager: object_manager,
            scene_manager,
            animation_manager: Rc::new(RefCell::new(AnimationManager::new())),
        }
    }

    pub fn init(&mut self) -> Result<(), JsValue> {
        self.scene_manager.borrow_mut().init()?;
        self.scene_manager.borrow_mut().set_context_type("2d")?;
        let _ = get_event_system().emit(AppEvent::READY.into(), &JsValue::NULL);
        Ok(())
    }

    pub fn is_support_type(&self, context_type: &str) -> bool {
        let window = web_sys::window().expect("Should have a window in this context");
        let document = window.document().expect("Should have a document on window");
        let canvas = document
            .create_element("canvas")
            .expect("Should be able to create a canvas");
        let canvas: web_sys::HtmlCanvasElement =
            canvas.dyn_into::<web_sys::HtmlCanvasElement>().unwrap();

        match context_type {
            "2d" => canvas.get_context(context_type).is_ok(),
            "webgl2" => canvas.get_context(context_type).is_ok(),
            _ => false,
        }
    }
}

impl App {
    pub fn add(&self, object: impl Renderable + 'static) {
        self.object_manager.borrow_mut().add(object);
    }

    pub fn remove(&self, id: &str) -> Option<Rc<RefCell<Box<dyn Renderable>>>> {
        self.object_manager.borrow_mut().remove(id)
    }

    pub fn get(&self, id: &str) -> Option<Rc<RefCell<Box<dyn Renderable>>>> {
        self.object_manager.borrow().get(id)
    }

    pub fn contains(&self, id: &str) -> bool {
        self.object_manager.borrow().contains(id)
    }

    pub fn len(&self) -> usize {
        self.object_manager.borrow().len()
    }

    pub fn is_empty(&self) -> bool {
        self.object_manager.borrow().is_empty()
    }

    pub fn clear(&self) {
        self.object_manager.borrow_mut().clear();
    }

    pub fn get_objects(&self) -> Vec<Rc<RefCell<Box<dyn Renderable>>>> {
        let res = self.object_manager.borrow().get_objects().clone();
        res
    }
}

impl App {
    pub async fn start_loop(&self) -> Result<(), JsValue> {
        let scene_manager = self.scene_manager.clone();
        let object_manager = self.object_manager.clone();
        let render_control = Rc::new(RefCell::new(get_render_control()));
        let animation_manager = self.animation_manager.clone();

        // 创建一个 Promise 来等待所有任务创建完成
        let setup_promise = Promise::new(&mut |resolve, _| {
            // 渲染任务
            {
                let render_control = render_control.clone();
                let scene_manager = scene_manager.clone();
                let object_manager = object_manager.clone();
                
                spawn_local(async move {
                    loop {
                        if let Some(messages) = render_control.borrow_mut().receive_messages().await {
                            object_manager.borrow_mut().update_object_from_message(&messages);
                            scene_manager.borrow().render();
                        }
                    }
                })
            };

            {
                let object_manager = object_manager.clone();
                let animation_manager = animation_manager.clone();
                
                spawn_local(async move {
                    loop {
                        let update = || {
                            let objects = object_manager.borrow().get_animatables();
                            let objects_map = objects
                                .into_iter()
                                .map(|object| (object.borrow().id().value().to_string(), object.clone()))
                                .collect();
                            let _ = animation_manager.borrow_mut().update(objects_map);
                        };

                        let promise = Promise::new(&mut |resolve, _| {

                            request_animation_frame(&resolve);
                        });

                        wasm_bindgen_futures::JsFuture::from(promise).await.unwrap();

                        update();
                    }
                })
            };

            let window = web_sys::window().expect("no global `window` exists");
            let closure = Closure::once_into_js(move || {
                resolve.call0(&JsValue::NULL).unwrap();
            });
            window.set_timeout_with_callback_and_timeout_and_arguments_0(
                closure.as_ref().unchecked_ref(),
                10
            ).expect("failed to set timeout");
        });

        // 等待 setup_promise 完成
        wasm_bindgen_futures::JsFuture::from(setup_promise).await?;

        get_render_control().add_message(UpdateMessage::ForceUpdate);
        let _ = get_event_system().emit("start_loop", &JsValue::NULL);

        Ok(())
    }
}
