use crate::vec::{UVec2, uvec2};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct URect2 {
    pub pos: UVec2,
    pub size: UVec2,
}
pub const fn urect2(pos: UVec2, size: UVec2) -> URect2 {
    URect2 { pos, size }
}

impl URect2 {
    pub const EMPTY: URect2 = URect2 {
        pos: UVec2::ZERO,
        size: UVec2::ZERO,
    };

    pub fn new(pos: UVec2, size: UVec2) -> Self {
        Self { pos, size }
    }

    pub fn try_from_corners(min: UVec2, max: UVec2) -> Option<Self> {
        if min.x <= max.x && min.y <= max.y {
            Some(urect2(min, max - min))
        } else {
            None
        }
    }

    pub fn from_corners(min: UVec2, max: UVec2) -> Self {
        Self::try_from_corners(min, max).expect("Minimum corner must be less than or equal to maximum corner")
    }

    pub fn corners(&self) -> (UVec2, UVec2) {
        (self.pos, self.pos + self.size)
    }

    pub fn area(&self) -> usize {
        self.size.x * self.size.y
    }

    pub fn center(&self) -> UVec2 {
        self.pos + self.size >> 1
    }

    pub fn contains(&self, point: UVec2) -> bool {
        let (min, max) = self.corners();
        point.x >= min.x && point.x < max.x && point.y >= min.y && point.y < max.y
    }

    pub fn intersect(&self, other: &Self) -> Option<Self> {
        let (min_a, max_a) = self.corners();
        let (min_b, max_b) = other.corners();

        let min = uvec2(min_a.x.max(min_b.x), min_a.y.max(min_b.y));
        let max = uvec2(max_a.x.min(max_b.x), max_a.y.min(max_b.y));

        Self::try_from_corners(min, max)
    }

    pub fn union(&self, other: &Self) -> Self {
        let (min_a, max_a) = self.corners();
        let (min_b, max_b) = other.corners();

        let min = uvec2(min_a.x.min(min_b.x), min_a.y.min(min_b.y));
        let max = uvec2(max_a.x.max(max_b.x), max_a.y.max(max_b.y));

        Self::from_corners(min, max)
    }

    pub fn encapsulate(points: &[UVec2]) -> Self {
        assert!(!points.is_empty(), "Cannot encapsulate an empty set of points");

        let min = points.iter().fold(UVec2::MAX, |min, &p| uvec2(min.x.min(p.x), min.y.min(p.y)));
        let max = points.iter().fold(UVec2::MIN, |max, &p| uvec2(max.x.max(p.x), max.y.max(p.y)));
        Self::from_corners(min, max)
    }

    pub fn expand(&self, amount: usize) -> Self {
        let (min, max) = self.corners();
        let min = uvec2(min.x.saturating_sub(amount), min.y.saturating_sub(amount));
        let max = uvec2(max.x.saturating_add(amount), max.y.saturating_add(amount));
        Self::from_corners(min, max)
    }

    pub fn shrink(&self, amount: usize) -> Option<Self> {
        let (min, max) = self.corners();
        let min = uvec2(min.x.saturating_add(amount), min.y.saturating_add(amount));
        let max = uvec2(max.x.saturating_sub(amount), max.y.saturating_sub(amount));
        Self::try_from_corners(min, max)
    }
}
