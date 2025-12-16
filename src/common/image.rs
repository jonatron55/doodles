pub struct Image {
    width: usize,
    height: usize,
    data: Vec<u8>,
}

impl Image {
    pub fn new(width: usize, height: usize) -> Self {
        Image {
            width,
            height,
            data: vec![0; width * height],
        }
    }

    pub fn new_checkered(width: usize, height: usize, check_size: (usize, usize)) -> Self {
        let mut data = vec![0; width * height];
        for y in 0..height {
            for x in 0..width {
                let index = y * width + x;
                if (x / check_size.0 + y / check_size.1) % 2 == 0 {
                    data[index] = 208;
                } else {
                    data[index] = 48;
                }
            }
        }

        Image {
            width,
            height,
            data,
        }
    }

    pub fn size(&self) -> (usize, usize) {
        (self.width, self.height)
    }

    pub fn pixel(&self, x: usize, y: usize) -> f32 {
        let index = y * self.width + x;
        self.data[index] as f32 / 255.0
    }

    pub fn invert(&mut self) {
        for value in &mut self.data {
            *value = 255 - *value;
        }
    }
}
