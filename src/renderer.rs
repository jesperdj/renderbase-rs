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

use std::ops::{Add, AddAssign, Div, Mul};

pub use multithreaded::*;

use crate::filter::Filter;
use crate::raster::Raster;
use crate::sampler::{PixelSample, Sampler};

mod multithreaded;
mod simple;

pub trait RenderFunction: Send + Sync {
    type Value: Copy + Default + Add<Output=Self::Value> + AddAssign + Mul<f32, Output=Self::Value> + Div<f32, Output=Self::Value> + Send + Sync;

    fn evaluate(&self, sample: &PixelSample) -> Self::Value;
}

pub trait Renderer {
    fn render<S: Sampler, R: RenderFunction, F: Filter>(&self, sampler: &S, render_fn: &R, filter: &F) -> Raster<R::Value>;
}
