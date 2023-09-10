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

use std::time::Instant;

use crossbeam_channel::{Receiver, Sender};
use crossbeam_utils::thread;
use crossbeam_utils::thread::Scope;

use crate::filter::Filter;
use crate::raster::Raster;
use crate::rectangle::Rectangle;
use crate::renderer::{Renderer, RenderFunction};
use crate::sampler::{Sampler, SamplerTile};

pub struct MultiThreadedRenderer {
    worker_count: usize,
    tiles_per_worker: usize,
}

// ===== MultiThreadedRenderer =================================================================================================================================

impl MultiThreadedRenderer {
    const DEFAULT_TILES_PER_WORKER: usize = 24;

    pub fn new(worker_count: usize, tiles_per_worker: usize) -> MultiThreadedRenderer {
        MultiThreadedRenderer { worker_count, tiles_per_worker }
    }

    pub fn with_defaults() -> MultiThreadedRenderer {
        MultiThreadedRenderer::new(num_cpus::get(), MultiThreadedRenderer::DEFAULT_TILES_PER_WORKER)
    }

    fn start_sample_generator<'a, S: Sampler>(&self, scope: &Scope<'a>, sampler: &'a S, tile_count_x: u32, tile_count_y: u32, sender: &Sender<S::Tile>) {
        let sender = sender.clone();

        scope.spawn(move |_| {
            log::info!("Sample generator thread started");
            let start_time = Instant::now();

            let mut tile_count = 0;
            for tile in sampler.tiles(tile_count_x, tile_count_y) {
                tile_count += 1;
                sender.send(tile).unwrap();
            }

            let duration = Instant::now().duration_since(start_time).as_millis();
            log::info!("Sample generator thread finished, generated {} tiles, run time: {} ms", tile_count, duration);
        });
    }

    fn start_workers<'a, S: Sampler, R: RenderFunction, F: Filter>(
        &self, scope: &Scope<'a>, output_rectangle: &'a Rectangle, render_fn: &'a R, filter: &'a F,
        receiver: &Receiver<S::Tile>, sender: &Sender<Raster<(R::Value, f32)>>)
        where
            <S as Sampler>::Tile: 'a
    {
        let (min_left, min_top) = (output_rectangle.left as f32, output_rectangle.top as f32);
        let (max_right, max_bottom) = (output_rectangle.right as f32, output_rectangle.bottom as f32);

        log::info!("Starting {} worker threads", self.worker_count);
        for id in 1..=self.worker_count {
            let receiver = receiver.clone();
            let sender = sender.clone();

            scope.spawn(move |_| {
                log::info!("[{:02}] Worker thread started", id);
                let start_time = Instant::now();

                let mut tile_count = 0;
                let mut sample_count = 0usize;
                for tile in receiver {
                    tile_count += 1;

                    let mut tile_raster = Raster::<(R::Value, f32)>::new(tile.rectangle().clone());

                    // For all samples in this tile, render and update the raster using the filter
                    for sample in tile {
                        sample_count += 1;

                        // Evaluate render function
                        let value = render_fn.evaluate(&sample);

                        let (pixel_x, pixel_y) = sample.pixel();
                        let (sample_x, sample_y) = sample.sample();

                        // Evaluate filter at this pixel's center
                        let (pixel_center_x, pixel_center_y) = (pixel_x as f32 + 0.5, pixel_y as f32 + 0.5);
                        let weight = filter.evaluate(pixel_center_x - sample_x, pixel_center_y - sample_y);

                        // Update pixel with weighted value and weight
                        let element = tile_raster.get_mut(pixel_x, pixel_y);
                        element.0 += value * weight;
                        element.1 += weight;
                    }

                    sender.send(tile_raster).unwrap();
                }

                let duration = Instant::now().duration_since(start_time).as_millis();
                log::info!("[{:02}] Worker thread finished, processed {} tiles; {} samples, run time: {} ms", id, tile_count, sample_count, duration);
            });
        }
    }
}

impl Renderer for MultiThreadedRenderer {
    fn render<S: Sampler, R: RenderFunction, F: Filter>(&self, sampler: &S, render_fn: &R, filter: &F) -> Raster<R::Value> {
        // Create channels
        const INPUT_CHANNEL_CAPACITY: usize = 2048;
        const OUTPUT_CHANNEL_CAPACITY: usize = 2048;
        let (input_snd, input_rcv) = crossbeam_channel::bounded(INPUT_CHANNEL_CAPACITY);
        let (output_snd, output_rcv) = crossbeam_channel::bounded(OUTPUT_CHANNEL_CAPACITY);

        thread::scope(|scope| {
            let start_time = Instant::now();

            let tile_count = self.worker_count * self.tiles_per_worker;
            let tile_count_dim = (tile_count as f32).sqrt().round() as u32;

            // Start sample generator and worker threads
            self.start_sample_generator(scope, sampler, tile_count_dim, tile_count_dim, &input_snd);
            self.start_workers::<S, R, F>(scope, &sampler.rectangle(), render_fn, filter, &input_rcv, &output_snd);

            // Disconnect channels used by sample generator and worker threads from the main thread
            drop(input_snd);
            drop(input_rcv);
            drop(output_snd);

            // Receive rendered tile rasters from workers and aggregate into output raster
            log::info!("Aggregating results");
            let mut raster = Raster::new(sampler.rectangle().clone());
            for tile_raster in output_rcv {
                raster.merge(&tile_raster, |(raster_value, raster_weight): (R::Value, f32), (tile_value, tile_weight): (R::Value, f32)| {
                    (raster_value + tile_value, raster_weight + tile_weight)
                });
            }

            // Convert weighted raster to final result
            log::info!("Converting raster");
            let raster = raster.map(|(value, weight): (R::Value, f32)| { if weight != 0.0 { value / weight } else { R::Value::default() } });

            let duration = Instant::now().duration_since(start_time).as_millis();
            log::info!("Rendering finished, run time: {} ms", duration);

            raster
        }).unwrap()
    }
}
