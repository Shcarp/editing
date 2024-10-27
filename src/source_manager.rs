use futures::channel::mpsc;
use futures::StreamExt;
use pathfinder_content::pattern::Pattern;
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::Once;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::HtmlImageElement;

use crate::image::Image;

// 资源类型
pub enum SourceType {
    ImageUrl(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SourceKey {
    Url(String),
    ResourceId(String),
}

#[derive(Debug, Clone)]
pub struct SourceManager {
    cache: Rc<RefCell<HashMap<SourceKey, Rc<RefCell<Pattern>>>>>,
}

impl SourceManager {
    pub fn new() -> Self {
        Self {
            cache: Rc::new(RefCell::new(HashMap::new())),
        }
    }

    pub fn load_source(&self, key: &SourceKey) -> Result<Rc<RefCell<Pattern>>, String> {
        if let Some(pattern) = self.cache.borrow().get(&key) {
            return Ok(pattern.clone());
        } else {
            Err("Not found".to_string())
        }
    }

    async fn load_from_url(&self, url: &str) -> Result<Pattern, String> {
        let (mut tx, mut rx) = mpsc::channel(1);

        let image =
            HtmlImageElement::new().map_err(|_| "Failed to create image element".to_string())?;

        let mut tx_clone = tx.clone();
        let callback = Closure::once(Box::new(move || {
            let _ = tx_clone.try_send(Ok(()));
        }) as Box<dyn FnOnce()>);

        let error_callback = Closure::once(Box::new(move || {
            let _ = tx.try_send(Err("Failed to load image".to_string()));
        }) as Box<dyn FnOnce()>);

        image.set_cross_origin(Some("anonymous"));

        image.set_onload(Some(callback.as_ref().unchecked_ref()));
        image.set_onerror(Some(error_callback.as_ref().unchecked_ref()));
        image.set_src(url);

        rx.next()
            .await
            .ok_or("Channel closed unexpectedly")?
            .map_err(|e| format!("Channel error: {}", e))?;

        // 将 HTMLImageElement 转换为 Pattern
        Image::new(image).into_pattern()
    }

    pub async fn load_from_resource(&self, id: SourceType) -> Result<SourceKey, String> {
        match id {
            SourceType::ImageUrl(url) => {
                let pattern = self.load_from_url(&url).await?;
                let key = SourceKey::Url(url);
                let pattern = Rc::new(RefCell::new(pattern));
                
                self.cache.borrow_mut().insert(key.clone(), pattern.clone());
                
                Ok(key)
            }
        }
    }

    pub fn clear_cache(&self) {
        self.cache.borrow_mut().clear();
    }

    pub fn remove_from_cache(&self, key: &SourceKey) {
        self.cache.borrow_mut().remove(key);
    }
}


static INIT: Once = Once::new();
static mut GLOBAL_SOURCE_MANAGER: Option<SourceManager> = None;

pub fn get_source_manager() -> &'static mut SourceManager {
    unsafe {
        INIT.call_once(|| {
            GLOBAL_SOURCE_MANAGER = Some(SourceManager::new());
        });
        GLOBAL_SOURCE_MANAGER.as_mut().unwrap()
    }
}