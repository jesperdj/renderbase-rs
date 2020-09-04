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

mod stratified;

/// Sampler interface.
///
/// A sampler must generate (x, y) samples in the range (0..1, 0..1), independent of the size and resolution of the output raster.
///
/// Samplers don't return the samples directly, but provide an implementation of the method `tiles()` which returns an iterator over sample tiles.
/// A sample tile provides samples over a subregion of the total range (0..1, 0..1). Sample tiles must not overlap, and all tiles produced by the sampler
/// must cover the entire range (0..1, 0..1).
///
/// Tiles are necessary to facilitate multi-threaded rendering. The `render()` function of module `renderer` will distribute sample tiles over a pool of worker
/// threads, so that multiple tiles can be processed in parallel.
pub trait Sampler<T: SampleTile, I: Iterator<Item=T>> {
    /// Returns an iterator over sample tiles.
    ///
    /// The parameters `tile_count_x` and `tile_count_y` specify the preferred number of tiles in the x and y directions. The actual number of tiles returned by
    /// the iterator does not need to be exactly `tile_count_x` times `tile_count_y` tiles.
    fn tiles(&self, tile_count_x: u32, tile_count_y: u32) -> I;
}

/// Sample tile.
///
/// A sample tile is an iterator of samples in a subrange of the total sample range (0..1, 0..1) of a sampler.
pub trait SampleTile: Iterator<Item=(f32, f32)> {
    /// Returns the range (start_x, start_y, end_x, end_y) in which this tile generates samples.
    ///
    /// The number of samples in a tile is determined by the sampler implementation.
    fn range(&self) -> (f32, f32, f32, f32);
}
