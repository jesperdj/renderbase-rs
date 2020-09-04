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

pub trait Sampler<T: SampleTile, I: Iterator<Item=T>> {
    fn tiles(&self, tile_count_x: u32, tile_count_y: u32) -> I;
}

pub trait SampleTile: Iterator<Item=(f32, f32)> {
    fn range(&self) -> (f32, f32, f32, f32);
}
