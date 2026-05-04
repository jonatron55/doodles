use crate::{
    rect::URect2,
    vec::{UVec2, uvec2},
};

pub trait IterRowMajor {
    fn iter_row_major(&self) -> impl Iterator<Item = UVec2>;
}

impl IterRowMajor for UVec2 {
    fn iter_row_major(&self) -> impl Iterator<Item = UVec2> {
        (0..self.y).flat_map(move |y| (0..self.x).map(move |x| uvec2(x, y)))
    }
}

impl IterRowMajor for &UVec2 {
    fn iter_row_major(&self) -> impl Iterator<Item = UVec2> {
        (*self).iter_row_major()
    }
}

impl IterRowMajor for URect2 {
    fn iter_row_major(&self) -> impl Iterator<Item = UVec2> {
        let (min, max) = self.corners();
        (min.y..max.y).flat_map(move |y| (min.x..max.x).map(move |x| uvec2(x, y)))
    }
}

impl IterRowMajor for &URect2 {
    fn iter_row_major(&self) -> impl Iterator<Item = UVec2> {
        (*self).iter_row_major()
    }
}
