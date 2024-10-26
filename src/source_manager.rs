use futures::channel::mpsc;
use futures::StreamExt;
use pathfinder_content::pattern::Pattern;
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{window, HtmlImageElement};

use crate::image::Image;

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
        // 检查缓存
        if let Some(pattern) = self.cache.borrow().get(&key) {
            return Ok(pattern.clone());
        } else {
            async {
                let pattern = match key {
                    SourceKey::Url(url) => self.load_from_url(&url).await,
                    SourceKey::ResourceId(id) => self.load_from_resource(&id).await,
                };

                if let Ok(pattern) = pattern {
                    let pattern = Rc::new(RefCell::new(pattern));
                    self.cache.borrow_mut().insert(key.clone(), pattern);
                }
            };
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

        image.set_onload(Some(callback.as_ref().unchecked_ref()));
        image.set_onerror(Some(error_callback.as_ref().unchecked_ref()));
        image.set_src(url);

        // 修改这部分接收逻辑
        rx.next()
            .await
            .ok_or("Channel closed unexpectedly")?
            .map_err(|e| format!("Channel error: {}", e))?;

        // 将 HTMLImageElement 转换为 Pattern
        Image::new(image).into_pattern()
    }

    async fn load_from_resource(&self, id: &str) -> Result<Pattern, String> {
        // 在浏览器环境中，ResourceId 可能指向预加载的资源或特定的资源URL
        // 这里简单地将其视为 URL 处理
        self.load_from_url(id).await
    }

    pub fn clear_cache(&self) {
        self.cache.borrow_mut().clear();
    }

    pub fn remove_from_cache(&self, key: &SourceKey) {
        self.cache.borrow_mut().remove(key);
    }
}
