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

use crate::rectangle::{Rectangle, RectangleIndexIterator, RectangleTileIterator};
use crate::sampler::{PixelSample, Sampler, SamplerTile};

#[derive(Clone, Debug)]
pub struct StratifiedSampler {
    rectangle: Rectangle,
    sqrt_samples_per_pixel: u32,
    jitter: bool,
}

#[derive(Clone, Debug)]
pub struct StratifiedSamplerTileIterator {
    rect_iter: RectangleTileIterator,
    sqrt_samples_per_pixel: u32,
    jitter: bool,
}

#[derive(Clone, Debug)]
pub struct StratifiedSamplerTile {
    tile_rect: Rectangle,
    tile_rect_iter: RectangleIndexIterator,
    sqrt_samples_per_pixel: u32,

    pixel_x: u32,
    pixel_y: u32,
    stratum_x: u32,
    stratum_y: u32,

    jitter: bool,
    rng: Xoshiro128Plus,
}

// ===== StratifiedSampler =====================================================================================================================================

impl StratifiedSampler {
    #[inline]
    pub fn new(rectangle: Rectangle, sqrt_samples_per_pixel: u32, jitter: bool) -> StratifiedSampler {
        StratifiedSampler { rectangle, sqrt_samples_per_pixel, jitter }
    }
}

impl Sampler for StratifiedSampler {
    type Tile = StratifiedSamplerTile;
    type TileIter = StratifiedSamplerTileIterator;

    #[inline]
    fn rectangle(&self) -> &Rectangle {
        &self.rectangle
    }

    #[inline]
    fn tiles(&self, tile_count_x: u32, tile_count_y: u32) -> StratifiedSamplerTileIterator {
        StratifiedSamplerTileIterator::new(&self.rectangle, self.sqrt_samples_per_pixel, tile_count_x, tile_count_y, self.jitter)
    }
}

// ===== StratifiedSamplerTileIterator =========================================================================================================================

impl StratifiedSamplerTileIterator {
    #[inline]
    fn new(sampler_rect: &Rectangle, sqrt_samples_per_pixel: u32, tile_count_x: u32, tile_count_y: u32, jitter: bool) -> StratifiedSamplerTileIterator {
        StratifiedSamplerTileIterator { rect_iter: sampler_rect.tile_iter(tile_count_x, tile_count_y), sqrt_samples_per_pixel, jitter }
    }
}

impl Iterator for StratifiedSamplerTileIterator {
    type Item = StratifiedSamplerTile;

    fn next(&mut self) -> Option<StratifiedSamplerTile> {
        self.rect_iter.next().map(|tile| {
            StratifiedSamplerTile::new(tile, self.sqrt_samples_per_pixel, self.jitter)
        })
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.rect_iter.size_hint()
    }
}

impl ExactSizeIterator for StratifiedSamplerTileIterator {}

impl FusedIterator for StratifiedSamplerTileIterator {}

// ===== StratifiedSamplerTile =================================================================================================================================

impl StratifiedSamplerTile {
    fn new(tile_rect: Rectangle, sqrt_samples_per_pixel: u32, jitter: bool) -> StratifiedSamplerTile {
        let tile_rect_iter = tile_rect.index_iter();
        let (pixel_x, pixel_y) = (tile_rect.left, tile_rect.top);

        StratifiedSamplerTile {
            tile_rect,
            tile_rect_iter,
            sqrt_samples_per_pixel,

            pixel_x,
            pixel_y,
            stratum_x: 0,
            stratum_y: sqrt_samples_per_pixel, // So that the first time, we advance to the first pixel

            jitter,
            rng: Xoshiro128Plus::from_entropy(),
        }
    }
}

impl SamplerTile for StratifiedSamplerTile {
    #[inline]
    fn rectangle(&self) -> &Rectangle {
        &self.tile_rect
    }
}

impl Iterator for StratifiedSamplerTile {
    type Item = PixelSample;

    fn next(&mut self) -> Option<PixelSample> {
        if self.stratum_y >= self.sqrt_samples_per_pixel {
            if let Some((px, py)) = self.tile_rect_iter.next() {
                // Advance to the next pixel in the tile
                self.pixel_x = px;
                self.pixel_y = py;
                self.stratum_x = 0;
                self.stratum_y = 0;
            } else {
                // No more pixels
                return None;
            }
        }

        // Generate the next sample for the current pixel
        let (jitter_x, jitter_y) = if self.jitter { self.rng.gen() } else { (0.5, 0.5) };
        let sample_offset_x = (self.stratum_x as f32 + jitter_x) / self.sqrt_samples_per_pixel as f32;
        let sample_offset_y = (self.stratum_y as f32 + jitter_y) / self.sqrt_samples_per_pixel as f32;

        self.stratum_x += 1;
        if self.stratum_x >= self.sqrt_samples_per_pixel {
            self.stratum_x = 0;
            self.stratum_y += 1;
        }

        Some(PixelSample::new(self.pixel_x, self.pixel_y, sample_offset_x, sample_offset_y))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let (pixels_remaining, _) = self.tile_rect_iter.size_hint();
        let samples_per_pixel = self.sqrt_samples_per_pixel * self.sqrt_samples_per_pixel;
        let pixel_sample_count = self.stratum_y * self.sqrt_samples_per_pixel + self.stratum_x;
        let remaining = pixels_remaining * samples_per_pixel as usize + (samples_per_pixel - pixel_sample_count) as usize;

        (remaining, Some(remaining))
    }
}

impl ExactSizeIterator for StratifiedSamplerTile {}

impl FusedIterator for StratifiedSamplerTile {}

// ===== Tests =================================================================================================================================================

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn stratified_sampler() {
        let rect = Rectangle::new(10, 20, 22, 30);
        let sampler = StratifiedSampler::new(rect, 2, true);

        let mut tile_count = 0;
        for tile in sampler.tiles(3, 2) {
            println!("***** tile: {:?}", tile.tile_rect);
            tile_count += 1;

            let mut sample_count = 0;
            for sample in tile {
                println!("  --- sample: {:?}", sample);
                sample_count += 1;
            }

            // Total rect size is 12 * 10, 4 samples per pixel, divided by 6 tiles
            assert_eq!(sample_count, 12 * 10 * 4 / 6, "wrong number of samples in tile");
        }

        assert_eq!(tile_count, 6, "wrong number of tiles");
    }
}
