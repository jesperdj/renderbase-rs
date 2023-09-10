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

/// Box filter.
#[derive(Clone, Debug)]
pub struct BoxFilter {
    radius_x: f32,
    radius_y: f32,
}

// ===== BoxFilter =============================================================================================================================================

impl BoxFilter {
    #[inline]
    pub fn new(radius_x: f32, radius_y: f32) -> BoxFilter {
        BoxFilter { radius_x, radius_y }
    }

    #[inline]
    pub fn with_defaults() -> BoxFilter {
        BoxFilter::new(0.5, 0.5)
    }
}

impl Filter for BoxFilter {
    #[inline]
    fn radius(&self) -> (f32, f32) {
        (self.radius_x, self.radius_y)
    }

    #[inline]
    fn evaluate(&self, x: f32, y: f32) -> f32 {
        if x.abs() <= self.radius_x && y.abs() <= self.radius_y { 1.0 } else { 0.0 }
    }
}

// ===== Tests =================================================================================================================================================

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn box_filter_new() {
        let filter = BoxFilter::new(1.0, 2.0);
        assert_eq!(filter.radius_x, 1.0, "radius_x is incorrect");
        assert_eq!(filter.radius_y, 2.0, "radius_y is incorrect");
    }

    #[test]
    fn box_filter_with_defaults() {
        let filter = BoxFilter::with_defaults();
        assert_eq!(filter.radius_x, 0.5, "radius_x is incorrect");
        assert_eq!(filter.radius_y, 0.5, "radius_y is incorrect");
    }

    #[test]
    fn box_filter_radius() {
        let filter = BoxFilter::new(1.0, 2.0);
        assert_eq!(filter.radius(), (1.0, 2.0), "radius() is incorrect");
    }

    #[test]
    fn box_filter_evaluate() {
        let filter = BoxFilter::new(1.0, 2.0);
        assert_eq!(filter.evaluate(0.0, 0.0), 1.0);
        assert_eq!(filter.evaluate(-1.0, 0.0), 1.0);
        assert_eq!(filter.evaluate(1.0, 0.0), 1.0);
        assert_eq!(filter.evaluate(0.0, -2.0), 1.0);
        assert_eq!(filter.evaluate(0.0, 2.0), 1.0);
    }

    #[test]
    fn box_filter_evaluate_zero_outside_radius() {
        let filter = BoxFilter::new(1.0, 2.0);
        assert_eq!(filter.evaluate(-1.001, 0.0), 0.0);
        assert_eq!(filter.evaluate(1.001, 0.0), 0.0);
        assert_eq!(filter.evaluate(0.0, -2.001), 0.0);
        assert_eq!(filter.evaluate(0.0, 2.001), 0.0);
    }

    #[test]
    fn box_filter_is_debug() {
        let filter = BoxFilter::with_defaults();
        println!("{:?}", filter);
    }
}
