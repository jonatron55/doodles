// Copyright (c) 2025 Jonathon Burnham Cobb
// Licensed under the MIT-0 license.

use std::{
    io::{BufRead, Cursor, Error as IoError, ErrorKind as IoErrorKind, Seek},
    path::Path,
};

use image::{
    ImageFormat, ImageReader, ImageResult,
    imageops::{FilterType, crop, resize},
};
use rand::{Rng, RngExt, random_bool};

use crate::{
    dir::Axis,
    math::Lerp,
    row_major::IterRowMajor,
    vec::{UVec2, uvec2},
};

#[derive(Clone, Debug)]
pub struct Image {
    size: UVec2,
    data: Vec<u8>,
}

const SMILEY: &[u8] = include_bytes!("../../assets/smiley.png");
const SKULL: &[u8] = include_bytes!("../../assets/skull.png");

/// A a grayscale image with pixel values in the range [0.0, 1.0].
///
/// This is used as a biasing image for various operations. It uses the [`image`] crate under the hood for loading and
/// resizing images, but maintains its own internal representation of the image.
impl Image {
    pub const DEFAULT_LIGHT: f64 = 0.7;
    pub const DEFAULT_DARK: f64 = 0.2;

    /// Constructs a new image with the given width and height, with all pixels initialized to 0.0.
    pub fn new(width: usize, height: usize) -> Self {
        Image {
            size: uvec2(width, height),
            data: vec![0; width * height],
        }
    }

    /// Constructs a new checkerboard image.
    ///
    /// Arguments
    /// ---------
    ///
    /// - `size`: width and height of the image in pixels.
    /// - `light`: value assigned to light checks, in the range [0.0, 1.0].
    /// - `dark`: value assigned to dark checks, in the range [0.0, 1.0].
    /// - `check_size`: width and height of each check in pixels.
    pub fn new_checkered(size: UVec2, light: f64, dark: f64, check_size: UVec2) -> Self {
        let light = (light.clamp(0.0, 1.0) * 255.0).round() as u8;
        let dark = (dark.clamp(0.0, 1.0) * 255.0).round() as u8;

        let mut data = vec![0; size.x * size.y];
        for pos in size.iter_row_major() {
            let index = pos.y * size.x + pos.x;

            data[index] = if (pos.x / check_size.x + pos.y / check_size.y) % 2 == 0 {
                light
            } else {
                dark
            };
        }

        Image { size, data }
    }

    /// Constructs a new checkerboard image with reasonable default parameters.
    pub fn default_checkered(size: UVec2) -> Self {
        Self::new_checkered(
            size,
            Image::DEFAULT_LIGHT,
            Image::DEFAULT_DARK,
            uvec2(size.x / 5, size.y / 3),
        )
    }

    /// Constructs a new image with concentric rings emanating from its center.
    ///
    /// Arguments
    /// ---------
    ///
    /// - `size`: width and height of the image in pixels.
    /// - `light`: value assigned to light rings, in the range [0.0, 1.0].
    /// - `dark`: value assigned to dark rings, in the range [0.0, 1.0].
    /// - `ring_width`: width of each ring in pixels.
    /// - `dist_fn`: a function that computes the distance from the center for a given pixel.
    pub fn new_concentric<F: Fn(UVec2, UVec2) -> f64>(
        size: UVec2,
        light: f64,
        dark: f64,
        ring_width: usize,
        dist_fn: F,
    ) -> Self {
        let light = (light.clamp(0.0, 1.0) * 255.0).round() as u8;
        let dark = (dark.clamp(0.0, 1.0) * 255.0).round() as u8;

        let mut data = vec![0; size.x * size.y];
        let center = size / 2;
        for pos in size.iter_row_major() {
            let index = pos.y * size.x + pos.x;
            let d = dist_fn(pos, center);
            let phase = (d / ring_width as f64).floor() as usize;

            data[index] = if phase % 2 == 0 { light } else { dark };
        }

        Image { size, data }
    }

    /// Constructs a new image with concentric rings within reasonable random parameters.
    pub fn random_concentric<R: Rng>(size: UVec2, rand: &mut R) -> Self {
        Self::new_concentric(
            size,
            Image::DEFAULT_LIGHT,
            Image::DEFAULT_DARK,
            rand.random_range(4..10),
            |pos, center| pos.manhattan_dist(center) as f64,
        )
    }

    /// Constructs a new image with a smiley face emoji, in the given size.
    ///
    /// This is built from an embedded PNG image, and will be cropped and resized to fit the given size.
    pub fn new_smiley(size: UVec2) -> Self {
        let mut reader = ImageReader::new(Cursor::new(SMILEY));
        reader.set_format(ImageFormat::Png);
        Self::read(reader, size).unwrap()
    }

    /// Constructs a new image with a skull emoji, in the given size.
    ///
    /// This is built from an embedded PNG image, and will be cropped and resized to fit the given size.
    pub fn new_skull(size: UVec2) -> Self {
        let mut reader = ImageReader::new(Cursor::new(SKULL));
        reader.set_format(ImageFormat::Png);
        Self::read(reader, size).unwrap()
    }

    /// Constructs a new image with a random graphic.
    pub fn random_graphic<R: Rng>(size: UVec2, rand: &mut R) -> Self {
        let mut img = match rand.random_range(0..2) {
            0 => Self::new_smiley(size),
            _ => Self::new_skull(size),
        };

        if rand.random_bool(0.5) {
            img.invert();
        }

        img
    }

    /// Constructs a new image with a gradient from `from` to `to`, in the given size.
    ///
    /// Arguments
    /// ---------
    ///
    /// - `axis`: the axis along which the gradient will be applied (horizontal or vertical).
    /// - `from`: the value at the start of the gradient, in the range [0.0, 1.0].
    /// - `to`: the value at the end of the gradient, in the range [0.0, 1.0].
    /// - `size`: width and height of the image in pixels.
    pub fn new_gradient(axis: Axis, from: f64, to: f64, size: UVec2) -> Self {
        let mut data = vec![0; size.x * size.y];
        for pos in size.iter_row_major() {
            let index = pos.y * size.x + pos.x;
            let t = match axis {
                Axis::Horizontal => pos.x as f64 / (size.x - 1) as f64,
                Axis::Vertical => pos.y as f64 / (size.y - 1) as f64,
            };
            data[index] = (f64::lerp(from, to, t).clamp(0.0, 1.0) * 255.0).round() as u8;
        }

        Image { size, data }
    }

    /// Constructs a new image with a zero-to-one gradient along a random axis.
    pub fn random_gradient<R: Rng>(size: UVec2, rand: &mut R) -> Self {
        let axis = if rand.random_bool(0.5) {
            Axis::Horizontal
        } else {
            Axis::Vertical
        };
        let (from, to) = if random_bool(0.5) { (0.0, 1.0) } else { (1.0, 0.0) };

        Self::new_gradient(axis, from, to, size)
    }

    /// Constructs a new image from the given file path, in the given size.
    ///
    /// The image will be cropped and resized to fit the given size, while maintaining its aspect ratio.
    pub fn from_file(path: &Path, size: UVec2) -> ImageResult<Self> {
        Self::read(ImageReader::open(path)?, size)
    }

    /// Returns the width and height of the image in pixels.
    pub fn size(&self) -> UVec2 {
        self.size
    }

    /// Returns the value of the pixel at the given coordinates, in the range [0.0, 1.0].
    pub fn pixel(&self, point: UVec2) -> f64 {
        let index = point.y * self.size.x + point.x;
        self.data[index] as f64 / 255.0
    }

    /// Inverts the pixel values of the image, such that light pixels become dark and dark pixels become light.
    pub fn invert(&mut self) {
        for value in &mut self.data {
            *value = 255 - *value;
        }
    }

    /// Constructs a new image from the given string.
    ///
    /// The string will first be interpreted as a file path. If the file exists, it will be loaded, cropped, and resized
    /// to fit the given size. If the file does not exist, but the string matches one of the following special cases, a
    /// corresponding image will be generated:
    ///
    /// - "smiley": a smiley face emoji (see [`Self::new_smiley`]).
    /// - "skull": a skull emoji (see [`Self::new_skull`]).
    /// - "checkered({width},{height})": a checkerboard pattern with the given check size (see [`Self::new_checkered`]).
    /// - "concentric{width}": concentric rings with the given ring width (see [`Self::new_concentric`]).
    /// - "hgrad": a horizontal gradient from black to white (see [`Self::new_gradient`]).
    /// - "vgrad": a vertical gradient from black to white (see [`Self::new_gradient`]).
    ///
    /// If neither a file nor a special case is found, an error will be returned.
    pub fn from_str(s: &str, size: UVec2) -> ImageResult<Self> {
        let path = Path::new(s);

        if path.exists() {
            Image::from_file(path, size)
        } else {
            if s == "smiley" {
                Ok(Self::new_smiley(size))
            } else if s == "skull" {
                Ok(Self::new_skull(size))
            } else if s.starts_with("checkered")
                && let Ok(check_size) = s["checkered".len()..].parse::<UVec2>()
            {
                Ok(Self::new_checkered(
                    size,
                    Image::DEFAULT_LIGHT,
                    Image::DEFAULT_DARK,
                    check_size,
                ))
            } else if s.starts_with("concentric")
                && let Ok(ring_width) = s["concentric".len()..].parse::<usize>()
            {
                Ok(Self::new_concentric(
                    size,
                    Image::DEFAULT_LIGHT,
                    Image::DEFAULT_DARK,
                    ring_width,
                    |pos, center| pos.manhattan_dist(center) as f64,
                ))
            } else if s == "hgrad" {
                Ok(Self::new_gradient(Axis::Horizontal, 0.0, 1.0, size))
            } else if s == "vgrad" {
                Ok(Self::new_gradient(Axis::Vertical, 0.0, 1.0, size))
            } else {
                return Err(IoError::from(IoErrorKind::NotFound).into());
            }
        }
    }

    fn read<R: BufRead + Seek>(r: ImageReader<R>, size: UVec2) -> ImageResult<Self> {
        let mut img = r.decode()?.to_luma8();
        let img_aspect = img.width() as f64 / img.height() as f64;
        let target_aspect = size.x as f64 / size.y as f64;

        // Crop to fill the target size while maintaining aspect ratio
        let crop_size = if img_aspect > target_aspect {
            uvec2((img.height() as f64 * target_aspect) as usize, img.height() as usize)
        } else {
            uvec2(img.width() as usize, (img.width() as f64 / target_aspect) as usize)
        };

        let crop_origin = uvec2(img.width() as usize - crop_size.x, img.height() as usize - crop_size.y) / 2;

        img = crop(
            &mut img,
            crop_origin.x as u32,
            crop_origin.y as u32,
            crop_size.x as u32,
            crop_size.y as u32,
        )
        .to_image();

        img = resize(&img, size.x as u32, size.y as u32, FilterType::Lanczos3);

        Ok(Image {
            size: uvec2(img.width() as usize, img.height() as usize),
            data: img.into_raw(),
        })
    }
}
