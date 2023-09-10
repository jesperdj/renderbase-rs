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

/// Windowed sinc filter.
#[derive(Clone, Debug)]
pub struct LanczosSincFilter {
    radius_x: f32,
    radius_y: f32,
    tau: f32,
}

// ===== LanczosSincFilter =====================================================================================================================================

impl LanczosSincFilter {
    #[inline]
    pub fn new(radius_x: f32, radius_y: f32, tau: f32) -> LanczosSincFilter {
        LanczosSincFilter { radius_x, radius_y, tau }
    }

    #[inline]
    pub fn with_defaults() -> LanczosSincFilter {
        LanczosSincFilter::new(4.0, 4.0, 3.0)
    }

    #[inline]
    fn windowed_sinc(&self, v: f32, r: f32) -> f32 {
        let v = v.abs();
        if v <= r { LanczosSincFilter::sinc(v) * LanczosSincFilter::sinc(v / self.tau) } else { 0.0 }
    }

    #[inline]
    fn sinc(v: f32) -> f32 {
        let w = PI * v.abs();
        if w >= 1e-5 { f32::sin(w) / w } else { 1.0 }
    }
}

impl Filter for LanczosSincFilter {
    #[inline]
    fn radius(&self) -> (f32, f32) {
        (self.radius_x, self.radius_y)
    }

    #[inline]
    fn evaluate(&self, x: f32, y: f32) -> f32 {
        self.windowed_sinc(x, self.radius_x) * self.windowed_sinc(y, self.radius_y)
    }
}

// ===== Tests =================================================================================================================================================

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn lanczos_sinc_filter_new() {
        let filter = LanczosSincFilter::new(1.0, 2.0, 2.5);
        assert_eq!(filter.radius_x, 1.0, "radius_x is incorrect");
        assert_eq!(filter.radius_y, 2.0, "radius_y is incorrect");
        assert_eq!(filter.tau, 2.5, "tau is incorrect");
    }

    #[test]
    fn lanczos_sinc_filter_with_defaults() {
        let filter = LanczosSincFilter::with_defaults();
        assert_eq!(filter.radius_x, 4.0, "radius_x is incorrect");
        assert_eq!(filter.radius_y, 4.0, "radius_y is incorrect");
        assert_eq!(filter.tau, 3.0, "tau is incorrect");
    }

    #[test]
    fn lanczos_sinc_filter_radius() {
        let filter = LanczosSincFilter::new(1.0, 2.0, 2.5);
        assert_eq!(filter.radius(), (1.0, 2.0), "radius() is incorrect");
    }

    #[test]
    fn lanczos_sinc_filter_evaluate() {
        let filter = LanczosSincFilter::new(1.0, 2.0, 2.5);
        assert_eq!(filter.evaluate(0.0, 0.0), 1.0);
        assert_eq!(filter.evaluate(0.1, 0.2), 0.9081256);
    }

    #[test]
    fn lanczos_sinc_filter_evaluate_zero_outside_radius() {
        let filter = LanczosSincFilter::new(1.0, 2.0, 2.5);
        assert_eq!(filter.evaluate(-1.001, 0.0), 0.0);
        assert_eq!(filter.evaluate(1.001, 0.0), 0.0);
        assert_eq!(filter.evaluate(0.0, -2.001), 0.0);
        assert_eq!(filter.evaluate(0.0, 2.001), 0.0);
    }

    #[test]
    fn lanczos_sinc_filter_is_debug() {
        let filter = LanczosSincFilter::with_defaults();
        println!("{:?}", filter);
    }
}
