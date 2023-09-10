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

use std::f32::consts::PI;

use crate::filter::Filter;

/// Gaussian filter.
#[derive(Clone, Debug)]
pub struct GaussianFilter {
    radius_x: f32,
    radius_y: f32,
    two_sigma_sq: f32,
    scale: f32,
    exp_x: f32,
    exp_y: f32,
}

// ===== GaussianFilter ========================================================================================================================================

impl GaussianFilter {
    pub fn new(radius_x: f32, radius_y: f32, sigma: f32) -> GaussianFilter {
        let two_sigma_sq = 2.0 * sigma * sigma;
        let scale = f32::sqrt(PI * two_sigma_sq).recip();
        let exp_x = scale * f32::exp(-(radius_x * radius_x) / two_sigma_sq);
        let exp_y = scale * f32::exp(-(radius_y * radius_y) / two_sigma_sq);

        GaussianFilter { radius_x, radius_y, two_sigma_sq, scale, exp_x, exp_y }
    }

    #[inline]
    pub fn with_defaults() -> GaussianFilter {
        GaussianFilter::new(1.5, 1.5, 0.5)
    }

    #[inline]
    fn gaussian(&self, v: f32) -> f32 {
        self.scale * f32::exp(-(v * v) / self.two_sigma_sq)
    }
}

impl Filter for GaussianFilter {
    #[inline]
    fn radius(&self) -> (f32, f32) {
        (self.radius_x, self.radius_y)
    }

    #[inline]
    fn evaluate(&self, x: f32, y: f32) -> f32 {
        f32::max(0.0, self.gaussian(x) - self.exp_x) * f32::max(0.0, self.gaussian(y) - self.exp_y)
    }
}

// ===== Tests =================================================================================================================================================

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn gaussian_filter_new() {
        let filter = GaussianFilter::new(1.0, 2.0, 1.5);
        assert_eq!(filter.radius_x, 1.0, "radius_x is incorrect");
        assert_eq!(filter.radius_y, 2.0, "radius_y is incorrect");
        assert_eq!(filter.two_sigma_sq, 2.0 * 1.5 * 1.5, "two_sigma_sq is incorrect");
        assert_eq!(filter.scale, f32::sqrt(PI * filter.two_sigma_sq).recip(), "scale is incorrect");
        assert_eq!(filter.exp_x, filter.scale * f32::exp(-1.0 / filter.two_sigma_sq), "exp_x is incorrect");
        assert_eq!(filter.exp_y, filter.scale * f32::exp(-4.0 / filter.two_sigma_sq), "exp_y is incorrect");
    }

    #[test]
    fn gaussian_filter_with_defaults() {
        let filter = GaussianFilter::with_defaults();
        assert_eq!(filter.radius_x, 1.5, "radius_x is incorrect");
        assert_eq!(filter.radius_y, 1.5, "radius_y is incorrect");
        assert_eq!(filter.two_sigma_sq, 2.0 * 0.5 * 0.5, "two_sigma_sq is incorrect");
    }

    #[test]
    fn gaussian_filter_radius() {
        let filter = GaussianFilter::new(1.0, 2.0, 1.5);
        assert_eq!(filter.radius(), (1.0, 2.0), "radius() is incorrect");
    }

    #[test]
    fn gaussian_filter_evaluate() {
        let filter = GaussianFilter::new(1.0, 2.0, 1.5);
        assert_eq!(filter.evaluate(0.0, 0.0), (filter.scale - filter.exp_x) * (filter.scale - filter.exp_y));
    }

    #[test]
    fn gaussian_filter_evaluate_zero_outside_radius() {
        let filter = GaussianFilter::new(1.0, 2.0, 1.5);
        assert_eq!(filter.evaluate(-1.001, 0.0), 0.0);
        assert_eq!(filter.evaluate(1.001, 0.0), 0.0);
        assert_eq!(filter.evaluate(0.0, -2.001), 0.0);
        assert_eq!(filter.evaluate(0.0, 2.001), 0.0);
    }

    #[test]
    fn gaussian_filter_is_debug() {
        let filter = GaussianFilter::with_defaults();
        println!("{:?}", filter);
    }
}
