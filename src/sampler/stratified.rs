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
use rand_xoshiro::Xoshiro256PlusPlus;

use crate::sampler::{Sampler, SampleTile};

/// Stratified sampler.
///
/// A stratified sampler generates evenly spaced samples in a grid (unless `jitter` is `true`, in which case the samples will not be evenly spaced).
#[derive(Clone, PartialEq, Debug)]
pub struct StratifiedSampler {
    sample_count_x: u32,
    sample_count_y: u32,

    scale_x: f32,
    scale_y: f32,

    jitter: bool,
}

/// Iterator over tiles produced by a stratified sampler.
#[derive(Clone, PartialEq, Debug)]
pub struct StratifiedSampleTileIterator {
    sample_count_x: u32,
    sample_count_y: u32,

    tile_start_x: u32,
    tile_start_y: u32,
    tile_width: u32,
    tile_height: u32,

    scale_x: f32,
    scale_y: f32,

    jitter: bool,
}

/// Implementation of `SampleTile` for a stratified sampler.
#[derive(Clone, Debug)]
pub struct StratifiedSampleTile {
    start_x: u32,
    start_y: u32,
    end_x: u32,
    end_y: u32,

    index_x: u32,
    index_y: u32,

    scale_x: f32,
    scale_y: f32,

    jitter: bool,
    rng: Xoshiro256PlusPlus,
}

// ===== StratifiedSampler =====================================================================================================================================

impl StratifiedSampler {
    /// Creates a new stratified sampler which will generate `sample_count_x` samples in the x direction and `sample_count_y` samples in the y direction.
    ///
    /// If `jitter` is `false`, the samples will be evenly spaced on the center points of the 'pixels' that are sampled.
    ///
    /// If `jitter` is `true`, the samples will be randomly placed in the square of each 'pixel' that is sampled. This can help reduce aliasing effects.
    pub fn new(sample_count_x: u32, sample_count_y: u32, jitter: bool) -> StratifiedSampler {
        assert!(sample_count_x > 0, "sample_count_x must be greater than zero");
        assert!(sample_count_y > 0, "sample_count_y must be greater than zero");

        StratifiedSampler {
            sample_count_x,
            sample_count_y,

            scale_x: 1.0 / sample_count_x as f32,
            scale_y: 1.0 / sample_count_y as f32,

            jitter,
        }
    }
}

impl Sampler<StratifiedSampleTile, StratifiedSampleTileIterator> for StratifiedSampler {
    fn tiles(&self, tile_count_x: u32, tile_count_y: u32) -> StratifiedSampleTileIterator {
        assert!(tile_count_x > 0, "tile_count_x must be greater than zero");
        assert!(tile_count_x <= self.sample_count_x,
                "tile_count_x must be less than or equal to sample_count_x, but {} > {}", tile_count_x, self.sample_count_x);
        assert!(tile_count_y > 0, "tile_count_y must be greater than zero");
        assert!(tile_count_y <= self.sample_count_y,
                "tile_count_y must be less than or equal to sample_count_y, but {} > {}", tile_count_y, self.sample_count_y);

        StratifiedSampleTileIterator::new(&self, tile_count_x, tile_count_y)
    }
}

// ===== StratifiedSampleTileIterator ==========================================================================================================================

impl StratifiedSampleTileIterator {
    fn new(sampler: &StratifiedSampler, tile_count_x: u32, tile_count_y: u32) -> StratifiedSampleTileIterator {
        let sample_count_x = sampler.sample_count_x;
        let sample_count_y = sampler.sample_count_y;

        let tile_width = (sample_count_x as f32 / tile_count_x as f32).round() as u32;
        let tile_height = (sample_count_y as f32 / tile_count_y as f32).round() as u32;

        let jitter = sampler.jitter;

        StratifiedSampleTileIterator {
            sample_count_x,
            sample_count_y,

            tile_start_x: 0,
            tile_start_y: 0,
            tile_width,
            tile_height,

            scale_x: sampler.scale_x,
            scale_y: sampler.scale_y,

            jitter,
        }
    }
}

impl Iterator for StratifiedSampleTileIterator {
    type Item = StratifiedSampleTile;

    fn next(&mut self) -> Option<StratifiedSampleTile> {
        if self.tile_start_y < self.sample_count_y {
            let tile_end_x = u32::min(self.tile_start_x + self.tile_width, self.sample_count_x);
            let tile_end_y = u32::min(self.tile_start_y + self.tile_height, self.sample_count_y);

            let tile = StratifiedSampleTile::new(self.tile_start_x, self.tile_start_y, tile_end_x, tile_end_y, self.scale_x, self.scale_y, self.jitter);

            self.tile_start_x = tile_end_x;
            if self.tile_start_x >= self.sample_count_x {
                self.tile_start_x = 0;
                self.tile_start_y = tile_end_y;
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
    fn new(start_x: u32, start_y: u32, end_x: u32, end_y: u32, scale_x: f32, scale_y: f32, jitter: bool) -> StratifiedSampleTile {
        StratifiedSampleTile {
            start_x,
            start_y,
            end_x,
            end_y,

            index_x: start_x,
            index_y: if start_x < end_x { start_y } else { end_y },

            scale_x,
            scale_y,

            jitter,
            rng: Xoshiro256PlusPlus::from_entropy(),
        }
    }
}

impl SampleTile for StratifiedSampleTile {
    fn range(&self) -> (f32, f32, f32, f32) {
        (
            self.start_x as f32 * self.scale_x,
            self.start_y as f32 * self.scale_y,
            self.end_x as f32 * self.scale_x,
            self.end_y as f32 * self.scale_y
        )
    }
}

impl Iterator for StratifiedSampleTile {
    type Item = (f32, f32);

    fn next(&mut self) -> Option<(f32, f32)> {
        if self.index_y < self.end_y {
            let (jitter_x, jitter_y) = if self.jitter { self.rng.gen() } else { (0.5, 0.5) };

            let sample_x = (self.index_x as f32 + jitter_x) * self.scale_x;
            let sample_y = (self.index_y as f32 + jitter_y) * self.scale_y;

            self.index_x += 1;
            if self.index_x >= self.end_x {
                self.index_x = self.start_x;
                self.index_y += 1;
            }

            Some((sample_x, sample_y))
        } else {
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining_y = (self.end_y - self.index_y) as usize;
        if remaining_y > 0 {
            let remaining = remaining_y * (self.end_x - self.start_x) as usize + (self.end_x - self.index_x) as usize;
            (remaining, Some(remaining))
        } else {
            (0, Some(0))
        }
    }
}

impl ExactSizeIterator for StratifiedSampleTile {}

impl FusedIterator for StratifiedSampleTile {}
