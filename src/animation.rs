

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
    pub fn new(start_value: f64, end_value: f64, duration: f64, easing: fn(f64) -> f64) -> Self {
        Self {
            start_value,
            end_value,
            duration,
            elapsed_time: 0.0,
            easing: easing,
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
    use std::f64::consts::PI;
    pub fn linear(t: f64) -> f64 {
        t
    }
    pub fn ease_in_quad(t: f64) -> f64 {
        t * t
    }
    pub fn ease_out_quad(t: f64) -> f64 {
        t * (2.0 - t)
    }
    pub fn ease_in_out_quad(t: f64) -> f64 {
        if t < 0.5 {
            2.0 * t * t
        } else {
            -1.0 + (4.0 - 2.0 * t) * t
        }
    }
    pub fn ease_in_cubic(t: f64) -> f64 {
        t * t * t
    }
    pub fn ease_out_cubic(t: f64) -> f64 {
        let t2 = t * t;
        t2 * (3.0 - 2.0 * t)
    }

    pub fn ease_in_out_cubic(t: f64) -> f64 {
        if t < 0.5 {
            4.0 * t * t * t
        } else {
            (t - 1.0) * (2.0 * t - 2.0) * (2.0 * t - 2.0) + 1.0
        }
    }
    
    pub fn ease_in_elastic(t: f64) -> f64 {
        if t == 0.0 || t == 1.0 {
            return t;
        }
        let p = 0.3;
        -(2.0_f64.powf(10.0 * (t - 1.0))) * ((t - 1.0 - p / 4.0) * (2.0 * PI) / p).sin()
    }
    
    pub fn ease_out_elastic(t: f64) -> f64 {
        if t == 0.0 || t == 1.0 {
            return t;
        }
        let p = 0.3;
        2.0_f64.powf(-10.0 * t) * (t - p / 4.0) * (2.0 * PI / p).sin() + 1.0
    }
    
    
}
