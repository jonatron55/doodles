use crate::common::vec::{UVec2, uvec2};

#[derive(Clone, Debug)]
pub struct Image {
    size: UVec2,
    data: Vec<u8>,
}

impl Image {
    pub fn new(width: usize, height: usize) -> Self {
        Image {
            size: uvec2(width, height),
            data: vec![0; width * height],
        }
    }

    pub fn new_checkered(size: UVec2, check_size: UVec2) -> Self {
        let mut data = vec![0; size.x * size.y];
        for y in 0..size.y {
            for x in 0..size.x {
                let index = y * size.x + x;

                data[index] = if (x / check_size.x + y / check_size.y) % 2 == 0 {
                    208
                } else {
                    48
                };
            }
        }

        Image { size, data }
    }

    pub fn new_concentric(size: UVec2, ring_width: usize) -> Self {
        let mut data = vec![0; size.x * size.y];
        let center = size / 2;
        for y in 0..size.y {
            for x in 0..size.x {
                let index = y * size.x + x;
                let point = uvec2(x, y);
                let d = point.manhattan_dist(center);

                data[index] = if (d / ring_width) % 2 == 0 { 224 } else { 32 };
            }
        }

        Image { size, data }
    }

    pub fn size(&self) -> UVec2 {
        self.size
    }

    pub fn pixel(&self, point: UVec2) -> f64 {
        let index = point.y * self.size.x + point.x;
        self.data[index] as f64 / 255.0
    }

    pub fn invert(&mut self) {
        for value in &mut self.data {
            *value = 255 - *value;
        }
    }
}
