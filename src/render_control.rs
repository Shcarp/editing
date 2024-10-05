use futures::{
    channel::mpsc::{channel, Receiver, Sender},
    StreamExt,
};
use serde_json::Value;
use std::collections::VecDeque;
use std::fmt::Debug;
use std::sync::Once;
use wasm_bindgen::JsValue;
use wasm_bindgen_futures::spawn_local;
use wasm_timer::Instant;
use web_sys::console;

static INIT: Once = Once::new();
static mut GLOBAL_RENDER_CONTROL: Option<RenderControl> = None;

pub fn get_render_control() -> &'static mut RenderControl {
    unsafe {
        INIT.call_once(|| {
            GLOBAL_RENDER_CONTROL = Some(RenderControl::new());
        });
        GLOBAL_RENDER_CONTROL.as_mut().unwrap()
    }
}

pub struct RenderControl {
    sender: Sender<Vec<UpdateMessage>>,
    receiver: Receiver<Vec<UpdateMessage>>,
    buffer: VecDeque<UpdateMessage>,
    last_flush: Instant,
    flush_interval: f64,
}

impl RenderControl {
    pub fn new() -> Self {
        let (sender, receiver) = channel(1);
        Self {
            sender,
            receiver,
            buffer: VecDeque::new(),
            last_flush: Instant::now(),
            flush_interval: 0.008, // 8ms
        }
    }

    pub fn add_message(&mut self, message: UpdateMessage) {
        match message {
            UpdateMessage::ForceUpdate => {
                self.flush();
                self.buffer.clear();
            }
            UpdateMessage::Update(update_body) => {
                // Insert the message into the buffer based on priority
                let insert_position = self
                    .buffer
                    .iter()
                    .position(|m| {
                        if let UpdateMessage::Update(existing_body) = m {
                            existing_body.priority < update_body.priority
                        } else {
                            false
                        }
                    })
                    .unwrap_or(self.buffer.len());
                self.buffer
                    .insert(insert_position, UpdateMessage::Update(update_body));
                self.flush_if_needed();
            }
        }
    }

    fn flush_if_needed(&mut self) {
        let elapsed = self.last_flush.elapsed().as_secs_f64();
        let current_time = Instant::now().elapsed().as_secs_f64();

        if (elapsed - current_time) >= self.flush_interval {
            self.flush();
        }
    }

    fn flush(&mut self) {
        if !self.buffer.is_empty() {
            let messages: Vec<UpdateMessage> = self.buffer.drain(..).collect();
            let sender: Sender<Vec<UpdateMessage>> = self.sender.clone();
            spawn_local(async move {
                if let Err(e) = sender.clone().try_send(messages) {
                    console::log_1(&JsValue::from_str(&format!(
                        "Failed to send messages: {:#?}",
                        e
                    )));
                }
            });
            self.last_flush = Instant::now();
        }
    }

    pub async fn receive_messages(&mut self) -> Option<Vec<UpdateMessage>> {
        self.receiver.next().await
    }
}

#[derive(Clone, Debug)]
pub enum UpdateType {
    ObjectUpdate(String),
    SceneUpdate,
}

#[derive(Clone, Debug)]
pub enum UpdateMessage {
    ForceUpdate,
    Update(UpdateBody),
}

#[derive(Clone, Debug)]
pub struct UpdateBody {
    pub update_type: UpdateType,
    pub data: Value,
    pub timestamp: f64,
    pub priority: u8,
}

impl UpdateBody {
    pub fn new(update_type: UpdateType, data: Value) -> Self {
        Self {
            update_type,
            data,
            timestamp: Instant::now().elapsed().as_secs_f64(),
            priority: 0, // 默认优先级为0
        }
    }
}
