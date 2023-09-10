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

pub use independent::*;
pub use stratified::*;

use crate::rectangle::Rectangle;

mod independent;
mod stratified;

#[derive(Clone, PartialEq, Debug)]
pub struct PixelSample {
    pixel_x: u32,
    pixel_y: u32,
    sample_offset_x: f32,
    sample_offset_y: f32,
}

pub trait Sampler: Send + Sync {
    type Tile: SamplerTile;
    type TileIter: Iterator<Item=Self::Tile>;

    fn rectangle(&self) -> &Rectangle;

    fn tiles(&self, tile_count_x: u32, tile_count_y: u32) -> Self::TileIter;
}

pub trait SamplerTile: Iterator<Item=PixelSample> + Send + Sync {
    fn rectangle(&self) -> &Rectangle;
}

// ===== PixelSample ===========================================================================================================================================

impl PixelSample {
    #[inline]
    pub fn new(pixel_x: u32, pixel_y: u32, sample_offset_x: f32, sample_offset_y: f32) -> PixelSample {
        PixelSample { pixel_x, pixel_y, sample_offset_x, sample_offset_y }
    }

    #[inline]
    pub fn pixel(&self) -> (u32, u32) {
        (self.pixel_x, self.pixel_y)
    }

    #[inline]
    pub fn sample_offset(&self) -> (f32, f32) {
        (self.sample_offset_x, self.sample_offset_y)
    }

    #[inline]
    pub fn sample(&self) -> (f32, f32) {
        (self.pixel_x as f32 + self.sample_offset_x, self.pixel_y as f32 + self.sample_offset_y)
    }
}

// ===== Tests =================================================================================================================================================

#[cfg(test)]
mod test {
    use super::*;

// TODO: Write tests.
}
