# RenderBase: A framework for multi-threaded graphics rendering

This is a simple little framework for building multi-threaded graphics rendering applications, written in Rust.

Note: This is not meant for real-time graphics applications such as games and does not use the GPU.

## Main concepts

The main idea in this framework is that we want to generate an image by sampling points on an image plane, evaluating a render function at each sample point
and using a reconstruction filter to build up an image. We need the following components:

- A sampler that generates the samples on the image plane.
- A render function that computes a value for each sample.
- A raster that stores the output values in a rectangular grid.
- A filter that combines samples and output values into the raster.

RenderBase provides implementations of samplers, the raster and filters, and a trait that defines the render function. Applications that use RenderBase provide
implementations of the render function.

RenderBase also provides a `render()` function that executes the rendering process in a multi-threaded way.

The result of calling `render()` is a raster of values. The type of the values is determined by the render function. It's up to the client application to
convert this raster into the desired output, for example an image.

RenderBase was mainly inspired by ideas from the book [Physically Based Rendering: From Theory to Implementation](https://pbrt.org/), especially the chapter
about [sampling and reconstruction](http://www.pbr-book.org/3ed-2018/Sampling_and_Reconstruction.html).

## Source code organization

RenderBase contains four main modules (see [src/lib.rs](https://github.com/jesperdj/renderbase-rs/blob/master/src/lib.rs)):

- `raster` - struct `Raster` and related items.
- `sampler` - the interface for samplers: trait `Sampler` and related items.
- `filter` - the interface for filters: trait `Filter`.
- `renderer` - the interface for render functions: trait `RenderFunction` and the `render()` function which executes the rendering process.

Implementations of samplers and filters are available in submodules of `sampler` and `filter`.

There is currently only one sampler implementation: `StratifiedSampler`.

There are five different reconstruction filter implementations, which correspond to the filters in the book Physically Based Rendering:

- `BoxFilter` - simple and fast
- `TriangleFilter` - simple and slower
- `GaussianFilter` - uses a Gaussian function for reconstruction
- `MitchellFilter` - Mitchell-Netravali reconstruction filter
- `LanczosSincFilter` - very slow

I prefer to use the `BoxFilter` for quick preview rendering with 1 sample per pixel, and the `MitchellFilter` for high-quality rendering with many samples per
pixel.

## Examples

[mandelbrot-rs](https://github.com/jesperdj/mandelbrot-rs) is an example that shows how to render Mandelbrot and Julia fractals using RenderBase.
