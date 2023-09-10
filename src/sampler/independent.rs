// Copyright 2023 Jesper de Jong
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
pub struct IndependentSampler {
    rectangle: Rectangle,
    samples_per_pixel: u32,
}

#[derive(Clone, Debug)]
pub struct IndependentSamplerTileIterator {
    rect_iter: RectangleTileIterator,
    samples_per_pixel: u32,
}

#[derive(Clone, Debug)]
pub struct IndependentSamplerTile {
    tile_rect: Rectangle,
    tile_rect_iter: RectangleIndexIterator,
    samples_per_pixel: u32,

    pixel_sample_count: u32,
    pixel_x: u32,
    pixel_y: u32,

    rng: Xoshiro128Plus,
}

// ===== IndependentSampler ====================================================================================================================================

impl IndependentSampler {
    #[inline]
    pub fn new(rectangle: &Rectangle, samples_per_pixel: u32) -> IndependentSampler {
        IndependentSampler { rectangle: rectangle.clone(), samples_per_pixel }
    }
}

impl Sampler for IndependentSampler {
    type Tile = IndependentSamplerTile;
    type TileIter = IndependentSamplerTileIterator;

    #[inline]
    fn rectangle(&self) -> &Rectangle {
        &self.rectangle
    }

    #[inline]
    fn tiles(&self, tile_count_x: u32, tile_count_y: u32) -> IndependentSamplerTileIterator {
        IndependentSamplerTileIterator::new(self.rectangle(), self.samples_per_pixel, tile_count_x, tile_count_y)
    }
}

// ===== IndependentSamplerTileIterator ========================================================================================================================

impl IndependentSamplerTileIterator {
    #[inline]
    fn new(sampler_rect: &Rectangle, samples_per_pixel: u32, tile_count_x: u32, tile_count_y: u32) -> IndependentSamplerTileIterator {
        IndependentSamplerTileIterator { rect_iter: sampler_rect.tile_iter(tile_count_x, tile_count_y), samples_per_pixel }
    }
}

impl Iterator for IndependentSamplerTileIterator {
    type Item = IndependentSamplerTile;

    fn next(&mut self) -> Option<IndependentSamplerTile> {
        self.rect_iter.next().map(|tile| {
            IndependentSamplerTile::new(tile, self.samples_per_pixel)
        })
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.rect_iter.size_hint()
    }
}

impl ExactSizeIterator for IndependentSamplerTileIterator {}

impl FusedIterator for IndependentSamplerTileIterator {}

// ===== IndependentSamplerTile ================================================================================================================================

impl IndependentSamplerTile {
    fn new(tile_rect: Rectangle, samples_per_pixel: u32) -> IndependentSamplerTile {
        let tile_rect_iter = tile_rect.index_iter();
        let (pixel_x, pixel_y) = (tile_rect.left, tile_rect.top);

        IndependentSamplerTile {
            tile_rect,
            tile_rect_iter,
            samples_per_pixel,

            pixel_sample_count: samples_per_pixel, // So that the first time, we advance to the first pixel
            pixel_x,
            pixel_y,

            rng: Xoshiro128Plus::from_entropy(),
        }
    }
}

impl SamplerTile for IndependentSamplerTile {
    #[inline]
    fn rectangle(&self) -> &Rectangle {
        &self.tile_rect
    }
}

impl Iterator for IndependentSamplerTile {
    type Item = PixelSample;

    fn next(&mut self) -> Option<PixelSample> {
        if self.pixel_sample_count >= self.samples_per_pixel {
            if let Some((px, py)) = self.tile_rect_iter.next() {
                // Advance to the next pixel in the tile
                self.pixel_sample_count = 0;
                self.pixel_x = px;
                self.pixel_y = py;
            } else {
                // No more pixels
                return None;
            }
        }

        // Generate the next sample for the current pixel
        self.pixel_sample_count += 1;
        let (sample_offset_x, sample_offset_y) = self.rng.gen();
        Some(PixelSample::new(self.pixel_x, self.pixel_y, sample_offset_x, sample_offset_y))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let (pixels_remaining, _) = self.tile_rect_iter.size_hint();
        let remaining = pixels_remaining * self.samples_per_pixel as usize + (self.samples_per_pixel - self.pixel_sample_count) as usize;

        (remaining, Some(remaining))
    }
}

impl ExactSizeIterator for IndependentSamplerTile {}

impl FusedIterator for IndependentSamplerTile {}

// ===== Tests =================================================================================================================================================

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn independent_sampler() {
        let rect = Rectangle::new(10, 20, 22, 30);
        let sampler = IndependentSampler::new(&rect, 2);

        let mut tile_count = 0;
        for tile in sampler.tiles(3, 2) {
            // println!("***** tile: {:?}", tile.tile_rect);
            tile_count += 1;

            let mut sample_count = 0;
            for sample in tile {
                // println!("  --- sample: {:?}", sample);
                sample_count += 1;
            }

            // Total rect size is 12 * 10, 2 samples per pixel, divided by 6 tiles
            assert_eq!(sample_count, 12 * 10 * 2 / 6, "wrong number of samples in tile");
        }

        assert_eq!(tile_count, 6, "wrong number of tiles");
    }
}
