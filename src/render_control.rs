use std::sync::Once;
use futures::{channel::mpsc::{channel, Receiver, Sender}, StreamExt};
use wasm_bindgen::JsValue;
use wasm_timer::Instant;
use web_sys::console;
use std::collections::VecDeque;
use wasm_bindgen_futures::spawn_local;

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
    sender: Sender<Vec<RenderMessage>>,
    receiver: Receiver<Vec<RenderMessage>>,
    buffer: VecDeque<RenderMessage>,
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

    pub fn add_message(&mut self, message: RenderMessage) {
        match message {
            RenderMessage::ForceUpdate => {
                self.flush();
                self.buffer.clear();
            },
            _ => {
                self.buffer.push_back(message);
                self.flush_if_needed();
            }
        }
    }

    fn flush_if_needed(&mut self) {
        if self.last_flush.elapsed().as_secs_f64() >= self.flush_interval {
            self.flush();
        }
    }

    fn flush(&mut self) {
        if !self.buffer.is_empty() {
            let messages: Vec<RenderMessage> = self.buffer.drain(..).collect();
            let sender = self.sender.clone();
            spawn_local(async move {
                if let Err(e) = sender.clone().try_send(messages) {
                    console::log_1(&JsValue::from_str(&format!("Failed to send messages: {:?}", e)));
                }
            });
            self.last_flush = Instant::now();
        }
    }

    pub async fn receive_messages(&mut self) -> Option<Vec<RenderMessage>> {
        self.receiver.next().await
    }
}

#[derive(Clone, Debug)]
pub enum RenderMessage {
    Update(String),
    ForceUpdate,
}