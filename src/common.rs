// Copyright (c) 2025 Jonathon Burnham Cobb
// Licensed under the MIT-0 license.

pub mod borders;
pub mod color;
pub mod dir;
pub mod image;
pub mod term;

pub trait Lerp<T: Copy + PartialOrd> {
    fn lerp(a: T, b: T, t: T) -> T;
    fn saturating_lerp(a: T, b: T, t: T) -> T;
}

impl Lerp<f32> for f32 {
    fn lerp(a: f32, b: f32, t: f32) -> f32 {
        a * (1.0 - t) + b * t
    }

    fn saturating_lerp(a: f32, b: f32, t: f32) -> f32 {
        Self::lerp(a, b, t.clamp(0.0, 1.0))
    }
}

impl Lerp<f64> for f64 {
    fn lerp(a: f64, b: f64, t: f64) -> f64 {
        a * (1.0 - t) + b * t
    }

    fn saturating_lerp(a: f64, b: f64, t: f64) -> f64 {
        Self::lerp(a, b, t.clamp(0.0, 1.0))
    }
}
