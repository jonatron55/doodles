// Copyright (c) 2025 Jonathon Burnham Cobb
// Licensed under the MIT-0 license.

/// Trait for types that support linear interpolation.
pub trait Lerp<T: Copy + PartialOrd> {
    /// Linearly interpolates between `a` and `b` by `t`.
    ///
    /// This function does not clamp the interpolant `t` between `0` and `1`. For an interpolant outside that range, the
    /// result will extrapolate beyond the endpoints.
    fn lerp(a: Self, b: Self, t: T) -> T;

    /// Linearly interpolates between `a` and `b` by `t`, without exceeding either endpoint.
    ///
    /// This function clamps the interpolant `t` between `0` and `1`, ensuring the result always lies within the range
    /// defined by `a` and `b`.
    fn saturating_lerp(a: Self, b: Self, t: T) -> T;

    /// Computes the interpolant `t` such that `lerp(a, b, t) == value`.
    fn inverse_lerp(a: Self, b: Self, value: Self) -> T;
}

impl Lerp<f32> for f32 {
    fn lerp(a: f32, b: f32, t: f32) -> f32 {
        // Unlike `a + (b - a) * t`, this formulation will always exactly interpolate the endpoints when `t` is exactly
        // 0 or 1.
        a * (1.0 - t) + b * t
    }

    fn saturating_lerp(a: f32, b: f32, t: f32) -> f32 {
        Self::lerp(a, b, t.clamp(0.0, 1.0))
    }

    fn inverse_lerp(a: f32, b: f32, value: f32) -> f32 {
        (value - a) / (b - a)
    }
}

impl Lerp<f64> for f64 {
    fn lerp(a: f64, b: f64, t: f64) -> f64 {
        // Unlike `a + (b - a) * t`, this formulation will always exactly interpolate the endpoints when `t` is exactly
        // 0 or 1.
        a * (1.0 - t) + b * t
    }

    fn saturating_lerp(a: f64, b: f64, t: f64) -> f64 {
        Self::lerp(a, b, t.clamp(0.0, 1.0))
    }

    fn inverse_lerp(a: f64, b: f64, value: f64) -> f64 {
        (value - a) / (b - a)
    }
}
