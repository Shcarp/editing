use into_static_str::IntoStaticStr;

#[derive(IntoStaticStr)]
pub enum AppEvent {
    READY,
}
