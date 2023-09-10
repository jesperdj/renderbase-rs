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

use std::time::Instant;

use crate::filter::Filter;
use crate::raster::Raster;
use crate::renderer::{Renderer, RenderFunction};
use crate::sampler::Sampler;

pub struct SimpleRenderer {}

// ===== SimpleRenderer ========================================================================================================================================

impl SimpleRenderer {
    #[inline]
    pub fn new() -> SimpleRenderer {
        SimpleRenderer {}
    }
}

impl Renderer for SimpleRenderer {
    fn render<S: Sampler, R: RenderFunction, F: Filter>(&self, sampler: &S, render_fn: &R, filter: &F) -> Raster<R::Value> {
        let mut raster = Raster::<(R::Value, f32)>::new(sampler.rectangle().clone());

        log::info!("Start rendering");
        let start_time = Instant::now();

        for tile in sampler.tiles(1, 1) {
            for sample in tile {
                let value = render_fn.evaluate(&sample);

                let (pixel_x, pixel_y) = sample.pixel();
                let (sample_x, sample_y) = sample.sample();

                // Evaluate filter at this pixel's center
                let (pixel_center_x, pixel_center_y) = (pixel_x as f32 + 0.5, pixel_y as f32 + 0.5);
                let weight = filter.evaluate(pixel_center_x - sample_x, pixel_center_y - sample_y);

                // Update pixel with weighted value and weight
                let element = raster.get_mut(pixel_x, pixel_y);
                element.0 += value * weight;
                element.1 += weight;
            }
        }

        // Convert weighted raster to final result
        log::info!("Converting raster");
        let raster = raster.map(|(value, weight): (R::Value, f32)| { if weight != 0.0 { value / weight } else { R::Value::default() } });

        let duration = Instant::now().duration_since(start_time).as_millis();
        log::info!("Rendering finished, run time: {} ms", duration);

        raster
    }
}
