pub trait Animatable {
    fn update(&mut self, delta_time: f64);
    fn is_finished(&self) -> bool;
    fn reset(&mut self);
    fn value(&self) -> f64;
}

#[derive(Clone, Debug)]
pub struct Tween {
    start_value: f64,
    end_value: f64,
    duration: f64,
    elapsed_time: f64,
    easing: fn(f64) -> f64,
}

impl Tween {
    pub fn new(start_value: f64, end_value: f64, duration: f64) -> Self {
        Self {
            start_value,
            end_value,
            duration,
            elapsed_time: 0.0,
            easing: |t| t, 
        }
    }

    pub fn with_easing(mut self, easing: fn(f64) -> f64) -> Self {
        self.easing = easing;
        self
    }
}

impl Animatable for Tween {
    fn update(&mut self, delta_time: f64) {
        self.elapsed_time += delta_time;
        if self.elapsed_time > self.duration {
            self.elapsed_time = self.duration;
        }
    }

    fn is_finished(&self) -> bool {
        self.elapsed_time >= self.duration
    }

    fn reset(&mut self) {
        self.elapsed_time = 0.0;
    }

    fn value(&self) -> f64 {
        let t = self.elapsed_time / self.duration;
        let eased_t = (self.easing)(t);
        self.start_value + (self.end_value - self.start_value) * eased_t
    }
}

// 一些常用的缓动函数
pub mod easing {
    pub fn linear(t: f64) -> f64 { t }
    pub fn ease_in_quad(t: f64) -> f64 { t * t }
    pub fn ease_out_quad(t: f64) -> f64 { t * (2.0 - t) }
    pub fn ease_in_out_quad(t: f64) -> f64 {
        if t < 0.5 { 2.0 * t * t } else { -1.0 + (4.0 - 2.0 * t) * t }
    }
}