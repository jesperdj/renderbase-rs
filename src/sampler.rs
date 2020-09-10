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

pub use stratified::*;

use crate::rectangle::Rectangle;

mod stratified;

#[derive(Clone, PartialEq, Debug)]
pub struct PixelSample {
    pub pixel_x: u32,
    pub pixel_y: u32,
    pub pixel_offset_x: f32,
    pub pixel_offset_y: f32,
}

pub trait Sampler {
    type Tile: SampleTile + Send + Sync;
    type TileIter: Iterator<Item=Self::Tile>;

    fn rectangle(&self) -> &Rectangle;

    fn tiles(&self, tile_count_x: u32, tile_count_y: u32) -> Self::TileIter;
}

pub trait SampleTile: Iterator<Item=PixelSample> {
    fn rectangle(&self) -> &Rectangle;
}

// ===== PixelSample ===========================================================================================================================================

impl PixelSample {
    pub fn new(pixel_x: u32, pixel_y: u32, pixel_offset_x: f32, pixel_offset_y: f32) -> PixelSample {
        PixelSample { pixel_x, pixel_y, pixel_offset_x, pixel_offset_y }
    }
}
