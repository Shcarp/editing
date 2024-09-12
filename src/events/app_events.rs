pub enum AppEvent {
    READY,
}

impl Into<&'static str> for AppEvent {
    fn into(self) -> &'static str {
        match self {
            AppEvent::READY => "ready",
        }
    }
}
