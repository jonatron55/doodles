// Copyright (c) 2025 Jonathon Burnham Cobb
// Licensed under the MIT-0 license.

use std::{
    fmt::{Display, Formatter, Result as FmtResult},
    ops::{
        Add, AddAssign, BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, Div,
        DivAssign, Mul, MulAssign, Not, Rem, RemAssign, Shl, ShlAssign, Shr, ShrAssign, Sub,
        SubAssign,
    },
    str::FromStr,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct UVec2 {
    pub x: usize,
    pub y: usize,
}

pub const fn uvec2(x: usize, y: usize) -> UVec2 {
    UVec2 { x, y }
}

impl UVec2 {
    pub const fn new(x: usize, y: usize) -> Self {
        UVec2 { x, y }
    }

    pub const fn smear(n: usize) -> Self {
        UVec2 { x: n, y: n }
    }

    pub const fn zero() -> Self {
        Self::smear(0)
    }

    pub const fn one() -> Self {
        Self::smear(1)
    }

    pub fn abs_diff(&self, other: Self) -> Self {
        let dx = self.x.abs_diff(other.x);
        let dy = self.y.abs_diff(other.y);
        UVec2 { x: dx, y: dy }
    }

    pub fn euclidean_distance_squared(&self, other: Self) -> f64 {
        let d = self.abs_diff(other);
        let (x, y) = (d.x as f64, d.y as f64);
        x * x + y * y
    }

    pub fn euclidean_distance(&self, other: Self) -> f64 {
        self.euclidean_distance_squared(other).sqrt()
    }

    pub fn manhattan_distance(&self, other: Self) -> usize {
        let d = self.abs_diff(other);
        d.x + d.y
    }

    pub fn chebyshev_distance(&self, other: Self) -> usize {
        let d = self.abs_diff(other);
        d.x.max(d.y)
    }

    pub fn saturating_add(&self, other: Self) -> Self {
        UVec2 {
            x: self.x.saturating_add(other.x),
            y: self.y.saturating_add(other.y),
        }
    }

    pub fn saturating_sub(&self, other: Self) -> Self {
        UVec2 {
            x: self.x.saturating_sub(other.x),
            y: self.y.saturating_sub(other.y),
        }
    }

    pub fn saturating_mul(&self, rhs: usize) -> Self {
        UVec2 {
            x: self.x.saturating_mul(rhs),
            y: self.y.saturating_mul(rhs),
        }
    }

    pub fn saturating_div(&self, rhs: usize) -> Self {
        UVec2 {
            x: self.x.saturating_div(rhs),
            y: self.y.saturating_div(rhs),
        }
    }

    pub fn wrapping_add(&self, other: Self) -> Self {
        UVec2 {
            x: self.x.wrapping_add(other.x),
            y: self.y.wrapping_add(other.y),
        }
    }

    pub fn wrapping_sub(&self, other: Self) -> Self {
        UVec2 {
            x: self.x.wrapping_sub(other.x),
            y: self.y.wrapping_sub(other.y),
        }
    }

    pub fn wrapping_mul(&self, rhs: usize) -> Self {
        UVec2 {
            x: self.x.wrapping_mul(rhs),
            y: self.y.wrapping_mul(rhs),
        }
    }

    pub fn wrapping_div(&self, rhs: usize) -> Self {
        UVec2 {
            x: self.x.wrapping_div(rhs),
            y: self.y.wrapping_div(rhs),
        }
    }

    pub fn checked_add(&self, other: Self) -> Option<Self> {
        Some(UVec2 {
            x: self.x.checked_add(other.x)?,
            y: self.y.checked_add(other.y)?,
        })
    }

    pub fn checked_sub(&self, other: Self) -> Option<Self> {
        Some(UVec2 {
            x: self.x.checked_sub(other.x)?,
            y: self.y.checked_sub(other.y)?,
        })
    }

    pub fn checked_mul(&self, rhs: usize) -> Option<Self> {
        Some(UVec2 {
            x: self.x.checked_mul(rhs)?,
            y: self.y.checked_mul(rhs)?,
        })
    }

    pub fn checked_div(&self, rhs: usize) -> Option<Self> {
        Some(UVec2 {
            x: self.x.checked_div(rhs)?,
            y: self.y.checked_div(rhs)?,
        })
    }
}

impl Into<(usize, usize)> for UVec2 {
    fn into(self) -> (usize, usize) {
        (self.x, self.y)
    }
}

impl From<(u8, u8)> for UVec2 {
    fn from(value: (u8, u8)) -> Self {
        UVec2 {
            x: value.0 as usize,
            y: value.1 as usize,
        }
    }
}

impl From<(u16, u16)> for UVec2 {
    fn from(value: (u16, u16)) -> Self {
        UVec2 {
            x: value.0 as usize,
            y: value.1 as usize,
        }
    }
}

impl From<(u32, u32)> for UVec2 {
    fn from(value: (u32, u32)) -> Self {
        UVec2 {
            x: value.0 as usize,
            y: value.1 as usize,
        }
    }
}

impl From<(usize, usize)> for UVec2 {
    fn from(value: (usize, usize)) -> Self {
        UVec2 {
            x: value.0,
            y: value.1,
        }
    }
}

// Arithmetic Operations

impl Add for UVec2 {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        UVec2 {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}

impl AddAssign for UVec2 {
    fn add_assign(&mut self, other: Self) {
        self.x += other.x;
        self.y += other.y;
    }
}

impl Sub for UVec2 {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        UVec2 {
            x: self.x - other.x,
            y: self.y - other.y,
        }
    }
}

impl SubAssign for UVec2 {
    fn sub_assign(&mut self, other: Self) {
        self.x -= other.x;
        self.y -= other.y;
    }
}

impl Mul<usize> for UVec2 {
    type Output = Self;

    fn mul(self, rhs: usize) -> Self::Output {
        UVec2 {
            x: self.x * rhs,
            y: self.y * rhs,
        }
    }
}

impl Mul<UVec2> for usize {
    type Output = UVec2;

    fn mul(self, rhs: UVec2) -> Self::Output {
        UVec2 {
            x: self * rhs.x,
            y: self * rhs.y,
        }
    }
}

impl MulAssign<usize> for UVec2 {
    fn mul_assign(&mut self, rhs: usize) {
        self.x *= rhs;
        self.y *= rhs;
    }
}

impl Div<usize> for UVec2 {
    type Output = Self;

    fn div(self, rhs: usize) -> Self::Output {
        UVec2 {
            x: self.x / rhs,
            y: self.y / rhs,
        }
    }
}

impl DivAssign<usize> for UVec2 {
    fn div_assign(&mut self, rhs: usize) {
        self.x /= rhs;
        self.y /= rhs;
    }
}

impl Rem<usize> for UVec2 {
    type Output = Self;

    fn rem(self, rhs: usize) -> Self::Output {
        UVec2 {
            x: self.x % rhs,
            y: self.y % rhs,
        }
    }
}

impl RemAssign<usize> for UVec2 {
    fn rem_assign(&mut self, rhs: usize) {
        self.x %= rhs;
        self.y %= rhs;
    }
}

// Bitwise Operations

impl BitAnd for UVec2 {
    type Output = Self;
    fn bitand(self, rhs: Self) -> Self {
        Self {
            x: self.x & rhs.x,
            y: self.y & rhs.y,
        }
    }
}

impl BitAndAssign for UVec2 {
    fn bitand_assign(&mut self, rhs: Self) {
        *self = *self & rhs;
    }
}

impl BitOr for UVec2 {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self {
        Self {
            x: self.x | rhs.x,
            y: self.y | rhs.y,
        }
    }
}

impl BitOrAssign for UVec2 {
    fn bitor_assign(&mut self, rhs: Self) {
        *self = *self | rhs;
    }
}

impl BitXor for UVec2 {
    type Output = Self;
    fn bitxor(self, rhs: Self) -> Self {
        Self {
            x: self.x ^ rhs.x,
            y: self.y ^ rhs.y,
        }
    }
}

impl BitXorAssign for UVec2 {
    fn bitxor_assign(&mut self, rhs: Self) {
        *self = *self ^ rhs;
    }
}

impl Not for UVec2 {
    type Output = Self;
    fn not(self) -> Self {
        Self {
            x: !self.x,
            y: !self.y,
        }
    }
}

impl Shl<usize> for UVec2 {
    type Output = Self;
    fn shl(self, rhs: usize) -> Self {
        Self {
            x: self.x << rhs,
            y: self.y << rhs,
        }
    }
}

impl ShlAssign<usize> for UVec2 {
    fn shl_assign(&mut self, rhs: usize) {
        *self = *self << rhs;
    }
}

impl Shr<usize> for UVec2 {
    type Output = Self;
    fn shr(self, rhs: usize) -> Self {
        Self {
            x: self.x >> rhs,
            y: self.y >> rhs,
        }
    }
}

impl ShrAssign<usize> for UVec2 {
    fn shr_assign(&mut self, rhs: usize) {
        *self = *self >> rhs;
    }
}

// Display and parsing

impl Display for UVec2 {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "({}, {})", self.x, self.y)
    }
}

impl FromStr for UVec2 {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim();
        if let Some((x_str, y_str)) = s
            .strip_prefix('(')
            .and_then(|s| s.strip_suffix(')'))
            .and_then(|s| {
                let mut parts = s.split(',');
                if let (Some(x), Some(y)) = (parts.next(), parts.next())
                    && parts.next().is_none()
                {
                    Some((x.trim(), y.trim()))
                } else {
                    None
                }
            })
        {
            if let (Ok(x), Ok(y)) = (x_str.parse::<usize>(), y_str.parse::<usize>()) {
                return Ok(UVec2 { x, y });
            }
        }
        Err(())
    }
}
