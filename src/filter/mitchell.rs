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

use crate::filter::Filter;

/// Mitchell-Netravali filter.
#[derive(Clone, Debug)]
pub struct MitchellFilter {
    radius_x: f32,
    radius_y: f32,
    p1: [f32; 4],
    p2: [f32; 4],
}

// ===== MitchellFilter ========================================================================================================================================

impl MitchellFilter {
    pub fn new(radius_x: f32, radius_y: f32, b: f32, c: f32) -> MitchellFilter {
        let p1 = [1.0 - b / 3.0, 0.0, -3.0 + 2.0 * b + c, 2.0 - 1.5 * b - c];
        let p2 = [4.0 / 3.0 * b + 4.0 * c, -2.0 * b - 8.0 * c, b + 5.0 * c, -b / 6.0 - c];

        MitchellFilter { radius_x, radius_y, p1, p2 }
    }

    #[inline]
    pub fn with_defaults() -> MitchellFilter {
        MitchellFilter::new(2.0, 2.0, 1.0 / 3.0, 1.0 / 3.0)
    }

    #[inline]
    fn mitchell(&self, v: f32) -> f32 {
        let x = 2.0 * v.abs();
        if x <= 1.0 {
            self.p1[0] + self.p1[1] * x + self.p1[2] * x * x + self.p1[3] * x * x * x
        } else if x <= 2.0 {
            self.p2[0] + self.p2[1] * x + self.p2[2] * x * x + self.p2[3] * x * x * x
        } else {
            0.0
        }
    }
}

impl Filter for MitchellFilter {
    #[inline]
    fn radius(&self) -> (f32, f32) {
        (self.radius_x, self.radius_y)
    }

    #[inline]
    fn evaluate(&self, x: f32, y: f32) -> f32 {
        self.mitchell(x / self.radius_x) * self.mitchell(y / self.radius_y)
    }
}

// ===== Tests =================================================================================================================================================

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn mitchell_filter_new() {
        let filter = MitchellFilter::new(1.0, 2.0, 0.5, 0.75);
        assert_eq!(filter.radius_x, 1.0, "radius_x is incorrect");
        assert_eq!(filter.radius_y, 2.0, "radius_y is incorrect");
        assert_eq!(filter.p1, [0.8333333, 0.0, -1.25, 0.5], "p1 is incorrect");
        assert_eq!(filter.p2, [3.6666667, -7.0, 4.25, -0.8333333], "p2 is incorrect");
    }

    #[test]
    fn mitchell_filter_with_defaults() {
        let filter = MitchellFilter::with_defaults();
        assert_eq!(filter.radius_x, 2.0, "radius_x is incorrect");
        assert_eq!(filter.radius_y, 2.0, "radius_y is incorrect");
        assert_eq!(filter.p1, [0.8888889, 0.0, -1.9999999, 1.1666666], "p1 is incorrect");
        assert_eq!(filter.p2, [1.7777779, -3.3333335, 2.0, -0.3888889], "p2 is incorrect");
    }

    #[test]
    fn mitchell_filter_radius() {
        let filter = MitchellFilter::new(1.0, 2.0, 0.5, 0.75);
        assert_eq!(filter.radius(), (1.0, 2.0), "radius() is incorrect");
    }

    #[test]
    fn mitchell_filter_evaluate() {
        let filter = MitchellFilter::new(1.0, 2.0, 0.5, 0.75);
        assert_eq!(filter.evaluate(0.0, 0.0), 0.6944444);
    }

    #[test]
    fn mitchell_filter_evaluate_zero_outside_radius() {
        let filter = MitchellFilter::new(1.0, 2.0, 0.5, 0.75);
        assert_eq!(filter.evaluate(-1.001, 0.0), 0.0);
        assert_eq!(filter.evaluate(1.001, 0.0), 0.0);
        assert_eq!(filter.evaluate(0.0, -2.001), 0.0);
        assert_eq!(filter.evaluate(0.0, 2.001), 0.0);
    }

    #[test]
    fn mitchell_filter_is_debug() {
        let filter = MitchellFilter::with_defaults();
        println!("{:?}", filter);
    }
}
