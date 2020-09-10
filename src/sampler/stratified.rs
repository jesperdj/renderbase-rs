// Copyright 2020 Jesper de Jong
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use std::iter::FusedIterator;

use rand::Rng;
use rand_xoshiro::rand_core::SeedableRng;
use rand_xoshiro::Xoshiro128Plus;

use crate::rectangle::Rectangle;
use crate::sampler::{PixelSample, Sampler, SampleTile};

#[derive(Clone, Debug)]
pub struct StratifiedSampler {
    rectangle: Rectangle,
    sqrt_samples_per_pixel: u32,

    jitter: bool,
}

#[derive(Clone, Debug)]
pub struct StratifiedSampleTileIterator {
    sampler_rectangle: Rectangle,
    sqrt_samples_per_pixel: u32,

    tile_width: u32,
    tile_height: u32,

    tile_left: u32,
    tile_top: u32,

    jitter: bool,
}

#[derive(Clone, Debug)]
pub struct StratifiedSampleTile {
    tile_rectangle: Rectangle,
    sqrt_samples_per_pixel: u32,

    px: u32,
    py: u32,
    sx: u32,
    sy: u32,

    jitter: bool,
    rng: Xoshiro128Plus,
}

// ===== StratifiedSampler =====================================================================================================================================

impl StratifiedSampler {
    pub fn new(rectangle: Rectangle, sqrt_samples_per_pixel: u32, jitter: bool) -> StratifiedSampler {
        StratifiedSampler { rectangle, sqrt_samples_per_pixel, jitter }
    }
}

impl Sampler for StratifiedSampler {
    type Tile = StratifiedSampleTile;
    type TileIter = StratifiedSampleTileIterator;

    fn rectangle(&self) -> &Rectangle {
        &self.rectangle
    }

    fn tiles(&self, tile_count_x: u32, tile_count_y: u32) -> StratifiedSampleTileIterator {
        assert!(tile_count_x > 0, "tile_count_x must be greater than zero");
        assert!(tile_count_x <= self.rectangle.width(),
                "tile_count_x must be less than or equal to rectangle width, but {} > {}", tile_count_x, self.rectangle.width());
        assert!(tile_count_y > 0, "tile_count_y must be greater than zero");
        assert!(tile_count_y <= self.rectangle.height(),
                "tile_count_y must be less than or equal to rectangle height, but {} > {}", tile_count_y, self.rectangle.height());

        StratifiedSampleTileIterator::new(&self.rectangle, self.sqrt_samples_per_pixel, tile_count_x, tile_count_y, self.jitter)
    }
}

// ===== StratifiedSampleTileIterator ==========================================================================================================================

impl StratifiedSampleTileIterator {
    fn new(sampler_rectangle: &Rectangle, sqrt_samples_per_pixel: u32, tile_count_x: u32, tile_count_y: u32, jitter: bool) -> StratifiedSampleTileIterator {
        let sampler_rectangle = sampler_rectangle.clone();

        // Determine tile size so that tile_width, _height times tile_count_x, _y is always >= total width, height
        let tile_width = (sampler_rectangle.width() - 1) / tile_count_x + 1;
        let tile_height = (sampler_rectangle.height() - 1) / tile_count_y + 1;

        let tile_left = sampler_rectangle.left;
        let tile_top = sampler_rectangle.top;

        StratifiedSampleTileIterator { sampler_rectangle, sqrt_samples_per_pixel, tile_width, tile_height, tile_left, tile_top, jitter }
    }
}

impl Iterator for StratifiedSampleTileIterator {
    type Item = StratifiedSampleTile;

    fn next(&mut self) -> Option<StratifiedSampleTile> {
        if self.tile_top < self.sampler_rectangle.bottom {
            let tile_right = u32::min(self.tile_left + self.tile_width, self.sampler_rectangle.right);
            let tile_bottom = u32::min(self.tile_top + self.tile_height, self.sampler_rectangle.bottom);
            let tile_rectangle = Rectangle::new(self.tile_left, self.tile_top, tile_right, tile_bottom);

            let tile = StratifiedSampleTile::new(tile_rectangle, self.sqrt_samples_per_pixel, self.jitter);

            self.tile_left = tile_right;
            if self.tile_left >= self.sampler_rectangle.right {
                self.tile_left = self.sampler_rectangle.left;
                self.tile_top = tile_bottom;
            }

            Some(tile)
        } else {
            None
        }
    }
}

impl FusedIterator for StratifiedSampleTileIterator {}

// ===== StratifiedSampleTile ==================================================================================================================================

impl StratifiedSampleTile {
    fn new(tile_rectangle: Rectangle, sqrt_samples_per_pixel: u32, jitter: bool) -> StratifiedSampleTile {
        let (px, py) = (tile_rectangle.left, if !tile_rectangle.is_empty() { tile_rectangle.top } else { tile_rectangle.bottom });
        let (sx, sy) = (0, 0);
        let rng = Xoshiro128Plus::from_entropy();

        StratifiedSampleTile { tile_rectangle, sqrt_samples_per_pixel, px, py, sx, sy, jitter, rng }
    }
}

impl SampleTile for StratifiedSampleTile {
    fn rectangle(&self) -> &Rectangle {
        &self.tile_rectangle
    }
}

impl Iterator for StratifiedSampleTile {
    type Item = PixelSample;

    fn next(&mut self) -> Option<PixelSample> {
        if self.py < self.tile_rectangle.bottom {
            let (jitter_x, jitter_y) = if self.jitter { self.rng.gen() } else { (0.5, 0.5) };
            let pixel_x = self.px as f32 + (self.sx as f32 + jitter_x) / self.sqrt_samples_per_pixel as f32;
            let pixel_y = self.py as f32 + (self.sy as f32 + jitter_y) / self.sqrt_samples_per_pixel as f32;

            self.sx += 1;
            if self.sx >= self.sqrt_samples_per_pixel {
                self.sx = 0;
                self.sy += 1;
                if self.sy >= self.sqrt_samples_per_pixel {
                    self.sy = 0;
                    self.px += 1;
                    if self.px >= self.tile_rectangle.right {
                        self.px = self.tile_rectangle.left;
                        self.py += 1;
                    }
                }
            }

            Some(PixelSample::new(pixel_x, pixel_y))
        } else {
            None
        }
    }
}

impl FusedIterator for StratifiedSampleTile {}
