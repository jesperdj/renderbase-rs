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
use std::time::Instant;

use crossbeam_channel::{Receiver, Sender};
use crossbeam_utils::thread;
use crossbeam_utils::thread::Scope;

use crate::filter::Filter;
use crate::raster::Raster;
use crate::rectangle::Rectangle;
use crate::sampler::{PixelSample, Sampler, SampleTile};

pub trait RenderFunction {
    type Value;

    fn evaluate(&self, sample: &PixelSample) -> Self::Value;
}

pub fn render<S, R, F>(sampler: &S, render_fn: &R, filter: &F) -> Raster<R::Value>
    where
        S: Sampler + Send + Sync,
        R: RenderFunction + Send + Sync,
        <R as RenderFunction>::Value: Copy + Default + Add<Output=R::Value> + AddAssign + Mul<f32, Output=R::Value> + Div<f32, Output=R::Value> + Send + Sync,
        F: Filter + Send + Sync,
{
    let output_rectangle = sampler.rectangle().clone();

    let worker_count = num_cpus::get();

    // Determine number of sample tiles based on number of workers and desired number of tiles per worker
    const TILES_PER_WORKER: usize = 24;
    let tile_count = worker_count * TILES_PER_WORKER;
    let aspect_ratio = output_rectangle.width() as f32 / output_rectangle.height() as f32;
    let tile_count_x = (tile_count as f32 * aspect_ratio).sqrt();
    let tile_count_y = tile_count_x / aspect_ratio;
    let (tile_count_x, tile_count_y) = (tile_count_x.round() as u32, tile_count_y.round() as u32);

    // Create channels
    const INPUT_CHANNEL_CAPACITY: usize = 2048;
    const OUTPUT_CHANNEL_CAPACITY: usize = 2048;
    let (input_snd, input_rcv) = crossbeam_channel::bounded(INPUT_CHANNEL_CAPACITY);
    let (output_snd, output_rcv) = crossbeam_channel::bounded(OUTPUT_CHANNEL_CAPACITY);

    thread::scope(|scope| {
        let start_time = Instant::now();

        // Start sample generator and worker threads
        start_sample_generator(scope, sampler, tile_count_x, tile_count_y, &input_snd);
        start_workers::<S, R, F>(scope, worker_count, &output_rectangle, render_fn, filter, &input_rcv, &output_snd);

        // Disconnect channels used by sample generator and worker threads from the main thread
        drop(input_snd);
        drop(input_rcv);
        drop(output_snd);

        // Receive rendered tile rasters from workers and aggregate into output raster
        log::info!("Aggregating results");
        let mut raster = Raster::new(output_rectangle.clone());
        for tile_raster in output_rcv {
            raster.merge(&tile_raster, |(raster_value, raster_weight): (R::Value, f32), (tile_value, tile_weight): (R::Value, f32)| {
                (raster_value + tile_value, raster_weight + tile_weight)
            });
        }

        // Convert weighted raster to final result
        log::info!("Converting raster");
        let raster = raster.map(|(value, weight): (R::Value, f32)| {
            if weight != 0.0 { value / weight } else { R::Value::default() }
        });

        let duration = Instant::now().duration_since(start_time).as_millis();
        log::info!("Rendering finished, run time: {} ms", duration);

        raster
    }).unwrap()
}

fn start_sample_generator<'a, S: Sampler + Send + Sync>(scope: &Scope<'a>, sampler: &'a S, tile_count_x: u32, tile_count_y: u32, sender: &Sender<S::Tile>) {
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

fn start_workers<'a, S, R, F>(scope: &Scope<'a>, worker_count: usize, output_rectangle: &'a Rectangle, render_fn: &'a R, filter: &'a F,
                              receiver: &Receiver<S::Tile>, sender: &Sender<Raster<(R::Value, f32)>>)
    where
        S: Sampler + Send + Sync,
        <S as Sampler>::Tile: 'a,
        R: RenderFunction + Send + Sync,
        <R as RenderFunction>::Value: Copy + Default + AddAssign + Mul<f32, Output=R::Value> + Send + Sync,
        F: Filter + Send + Sync,
{
    let (min_left, min_top) = (output_rectangle.left as f32, output_rectangle.top as f32);
    let (max_right, max_bottom) = (output_rectangle.right as f32, output_rectangle.bottom as f32);

    log::info!("Starting {} worker threads", worker_count);
    for id in 1..=worker_count {
        let receiver = receiver.clone();
        let sender = sender.clone();

        scope.spawn(move |_| {
            log::info!("[{:02}] Worker thread started", id);
            let start_time = Instant::now();

            let (radius_x, radius_y) = filter.radius();

            let mut tile_count = 0;
            let mut sample_count = 0usize;
            for tile in receiver {
                tile_count += 1;

                let tile_rect = tile.rectangle();

                // Create tile raster with border the size of the filter radius and clip by output rectangle
                let tile_left = f32::max((tile_rect.left as f32 - radius_x).round(), min_left) as u32;
                let tile_top = f32::max((tile_rect.top as f32 - radius_y).round(), min_top) as u32;
                let tile_right = f32::min((tile_rect.right as f32 + radius_x).round(), max_right) as u32;
                let tile_bottom = f32::min((tile_rect.bottom as f32 + radius_y).round(), max_bottom) as u32;

                let mut tile_raster = Raster::<(R::Value, f32)>::new(Rectangle::new(tile_left, tile_top, tile_right, tile_bottom));

                // For all samples in this tile, render and update the raster using the filter
                for sample in tile {
                    sample_count += 1;

                    // Evaluate render function
                    let value = render_fn.evaluate(&sample);

                    let (sample_x, sample_y) = sample.sample();

                    // Determine which pixels in the raster need to be updated
                    let start_px = f32::max((sample_x - radius_x).round(), tile_left as f32) as u32;
                    let start_py = f32::max((sample_y - radius_y).round(), tile_top as f32) as u32;
                    let end_px = f32::min((sample_x + radius_x).round(), tile_right as f32) as u32;
                    let end_py = f32::min((sample_y + radius_y).round(), tile_bottom as f32) as u32;

                    // Update the relevant pixels
                    for py in start_py..end_py {
                        for px in start_px..end_px {
                            // Evaluate filter at this pixel's center
                            let (pixel_center_x, pixel_center_y) = (px as f32 + 0.5, py as f32 + 0.5);
                            let weight = filter.evaluate(pixel_center_x - sample_x, pixel_center_y - sample_y);

                            // Update pixel with weighted value and weight
                            let element = tile_raster.get_mut(px, py);
                            element.0 += value * weight;
                            element.1 += weight;
                        }
                    }
                }

                sender.send(tile_raster).unwrap();
            }

            let duration = Instant::now().duration_since(start_time).as_millis();
            log::info!("[{:02}] Worker thread finished, processed {} tiles; {} samples, run time: {} ms", id, tile_count, sample_count, duration);
        });
    }
}
