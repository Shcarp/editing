use futures::channel::mpsc;
use futures::StreamExt;
use serde::{Deserialize, Serialize};
use web_sys::WebGl2RenderingContext;
use web_sys::WebGlTexture;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::Once;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::HtmlImageElement;
use crate::image::Image;

pub enum SourceType {
    ImageUrl(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SourceKey {
    Url(String),
    ResourceId(String),
}

#[derive(Debug, Clone)]
pub struct SourceManager<'a> {
    cache: HashMap<SourceKey, Rc<RefCell<Image<'a>>>>,
    texture_cache: HashMap<SourceKey, Rc<RefCell<WebGlTexture>>>,
}

impl<'a> SourceManager<'a> {
    pub fn new() -> Self {
        Self {
            cache: HashMap::new(),
            texture_cache: HashMap::new(),
        }
    }

    pub fn load_source(&self, key: &SourceKey) -> Result<Rc<RefCell<Image<'a>>>, String> {
        if let Some(image) = self.cache.get(&key) {
            return Ok(image.clone());
        } else {
            Err("Not found".to_string())
        }
    }

    async fn load_from_url(&self, url: &str) -> Result<Image, String> {
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
        Ok(Image::new(image))
    }

    pub async fn load_from_resource(&mut self, id: SourceType) -> Result<SourceKey, String> {
        match id {
            SourceType::ImageUrl(url) => {
                let image = self.load_from_url(&url).await?;
                let key = SourceKey::Url(url.clone());
                let image: Image<'a> = unsafe { std::mem::transmute(image) };
                let image = Rc::new(RefCell::new(image));
                
                self.cache.insert(key.clone(), image);
                Ok(key)
            }
        }
    }

    pub fn get_source_from_key(&mut self, gl: &WebGl2RenderingContext, key: &SourceKey) -> Result<Rc<RefCell<WebGlTexture>>, String> {
        if let Some(texture) = self.texture_cache.get(&key) {
            return Ok(texture.clone());
        } 
        if let Some(image) = self.cache.get(&key) {
            let image = image.borrow().as_html_image_element();
            let texture = gl.create_texture().unwrap();
            gl.pixel_storei(WebGl2RenderingContext::UNPACK_FLIP_Y_WEBGL, 1);
            gl.bind_texture(WebGl2RenderingContext::TEXTURE_2D, Some(&texture));
            gl.tex_image_2d_with_u32_and_u32_and_html_image_element(
                WebGl2RenderingContext::TEXTURE_2D,
                0,                                              // mipmap level
                WebGl2RenderingContext::RGBA as i32,   // internal format
                WebGl2RenderingContext::RGBA,                  // format
                WebGl2RenderingContext::UNSIGNED_BYTE,         // type
                &image,
            ).map_err(|e| e.as_string().unwrap_or_else(|| "Error uploading texture".to_string()))?;

            gl.generate_mipmap(WebGl2RenderingContext::TEXTURE_2D);

            gl.tex_parameteri(
                WebGl2RenderingContext::TEXTURE_2D,
                WebGl2RenderingContext::TEXTURE_MIN_FILTER,
                WebGl2RenderingContext::LINEAR as i32,
            );
            // 这告诉WebGL如果纹理需要被方法时，采用线性插值的方式来进行采样
            gl.tex_parameteri(
                WebGl2RenderingContext::TEXTURE_2D,
                WebGl2RenderingContext::TEXTURE_MAG_FILTER,
                WebGl2RenderingContext::LINEAR as i32,
            );
            // 告诉WebGL如果纹理坐标超出了s坐标的最大/最小值，直接取边界值
            gl.tex_parameteri(
                WebGl2RenderingContext::TEXTURE_2D,
                WebGl2RenderingContext::TEXTURE_WRAP_S,
                WebGl2RenderingContext::CLAMP_TO_EDGE as i32,
            );
            // 告诉WebGL如果纹理坐标超出了t坐标的最大/最小值，直接取边界值
            gl.tex_parameteri(
                WebGl2RenderingContext::TEXTURE_2D,
                WebGl2RenderingContext::TEXTURE_WRAP_T,
                WebGl2RenderingContext::CLAMP_TO_EDGE as i32,
            );
            let rc_texture = Rc::new(RefCell::new(texture));
            self.texture_cache.insert(key.clone(), rc_texture.clone());
            Ok(rc_texture)
        } else {
            Err("Not found".to_string())
        }
    }

}


static INIT: Once = Once::new();
static mut GLOBAL_SOURCE_MANAGER: Option<SourceManager> = None;

pub fn get_source_manager() -> &'static mut SourceManager<'static> {
    unsafe {
        INIT.call_once(|| {
            GLOBAL_SOURCE_MANAGER = Some(SourceManager::new());
        });
        GLOBAL_SOURCE_MANAGER.as_mut().unwrap()
    }
}