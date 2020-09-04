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
use crate::raster::{Raster, Rectangle};
use crate::sampler::{Sampler, SampleTile};

pub trait RenderFunction<V: Copy + Default> {
    fn evaluate(&self, x: f32, y: f32) -> V;
}

pub fn render<S, T, I, R, V, F>(sampler: &S, render_fn: &R, filter: &F, width: u32, height: u32) -> Raster<V>
    where
        S: Sampler<T, I> + Send + Sync,
        T: SampleTile + Send + Sync,
        I: Iterator<Item=T> + Send + Sync,
        R: RenderFunction<V> + Send + Sync,
        V: Copy + Default + Add<Output=V> + AddAssign + Mul<f32, Output=V> + Div<f32, Output=V> + Send + Sync,
        F: Filter + Send + Sync,
{
    let worker_count = num_cpus::get();

    // Determine number of sample tiles based on number of workers and desired number of tiles per worker
    const TILES_PER_WORKER: usize = 24;
    let tile_count = worker_count * TILES_PER_WORKER;
    let aspect_ratio = width as f32 / height as f32;
    let tile_count_x = (tile_count as f32 * aspect_ratio).sqrt();
    let tile_count_y = tile_count_x / aspect_ratio;
    let (tile_count_x, tile_count_y) = (tile_count_x.round() as u32, tile_count_y.round() as u32);

    // Create channels
    let (input_snd, input_rcv) = crossbeam_channel::bounded(tile_count);
    let (output_snd, output_rcv) = crossbeam_channel::bounded(tile_count);

    thread::scope(|scope| {
        let start_time = Instant::now();

        // Start worker and sample generator threads
        start_workers(scope, worker_count, render_fn, filter, width as f32, height as f32, &input_rcv, &output_snd);
        start_sample_generator(scope, tile_count_x, tile_count_y, sampler, &input_snd);

        // Disconnect channels used by sample generator and worker threads from the main thread
        drop(input_snd);
        drop(input_rcv);
        drop(output_snd);

        // Receive rendered tile rasters from workers and aggregate into output raster
        log::info!("Aggregating results");
        let mut raster = Raster::new(Rectangle::new(0, 0, width as i32, height as i32));
        for tile_raster in output_rcv {
            raster.merge(&tile_raster, |(raster_value, raster_weight): (V, f32), (tile_value, tile_weight): (V, f32)| {
                (raster_value + tile_value, raster_weight + tile_weight)
            });
        }

        // Convert weighted raster to final result
        log::info!("Converting raster");
        let raster = raster.map(|(value, weight): (V, f32)| {
            if weight != 0.0 { value / weight } else { V::default() }
        });

        let duration = Instant::now().duration_since(start_time).as_millis();
        log::info!("Rendering finished, run time: {} ms", duration);

        raster
    }).unwrap()
}

fn start_workers<'a, T, R, V, F>(scope: &Scope<'a>, worker_count: usize, render_fn: &'a R, filter: &'a F,
                                 raster_scale_x: f32, raster_scale_y: f32,
                                 receiver: &Receiver<T>, sender: &Sender<Raster<(V, f32)>>)
    where
        T: 'a + SampleTile + Send + Sync,
        R: RenderFunction<V> + Send + Sync,
        V: 'a + Copy + Default + AddAssign + Mul<f32, Output=V> + Send + Sync,
        F: Filter + Send + Sync,
{
    for id in 1..=worker_count {
        let receiver = receiver.clone();
        let sender = sender.clone();

        scope.spawn(move |_| {
            log::info!("[{:>02}] Worker thread started", id);
            let start_time = Instant::now();

            let (radius_x, radius_y) = filter.radius();

            let mut tile_count = 0;
            let mut sample_count = 0usize;
            for tile in receiver {
                tile_count += 1;

                let (tile_start_x, tile_start_y, tile_end_x, tile_end_y) = tile.range();

                // Create tile raster with border the size of the filter radius
                let tile_left = (tile_start_x * raster_scale_x - radius_x).round() as i32;
                let tile_top = (tile_start_y * raster_scale_y - radius_y).round() as i32;
                let tile_right = (tile_end_x * raster_scale_x + radius_x).round() as i32;
                let tile_bottom = (tile_end_y * raster_scale_y + radius_y).round() as i32;

                let mut tile_raster = Raster::<(V, f32)>::new(Rectangle::new(tile_left, tile_top, tile_right, tile_bottom));

                // For all samples in this tile, render and update the raster using the filter
                for (sample_x, sample_y) in tile {
                    sample_count += 1;

                    // Evaluate render function
                    let value = render_fn.evaluate(sample_x, sample_y);

                    // Determine which pixels in the raster need to be updated
                    let (sample_pixel_x, sample_pixel_y) = (sample_x * raster_scale_x, sample_y * raster_scale_y);
                    let (start_px, end_px) = ((sample_pixel_x - radius_x).round() as i32, (sample_pixel_x + radius_x).round() as i32);
                    let (start_py, end_py) = ((sample_pixel_y - radius_y).round() as i32, (sample_pixel_y + radius_y).round() as i32);

                    // Update the relevant pixels
                    for py in start_py..end_py {
                        for px in start_px..end_px {
                            // Evaluate filter at this pixel's center
                            let (pixel_center_x, pixel_center_y) = (px as f32 + 0.5, py as f32 + 0.5);
                            let weight = filter.evaluate(pixel_center_x - sample_pixel_x, pixel_center_y - sample_pixel_y);

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
            log::info!("[{:>02}] Worker thread finished, processed {} tiles; {} samples, run time: {} ms", id, tile_count, sample_count, duration);
        });
    }
}

fn start_sample_generator<'a, S, T, I>(scope: &Scope<'a>, tile_count_x: u32, tile_count_y: u32, sampler: &'a S, sender: &Sender<T>)
    where
        S: Sampler<T, I> + Send + Sync,
        T: 'a + SampleTile + Send + Sync,
        I: Iterator<Item=T> + Send + Sync,
{
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
