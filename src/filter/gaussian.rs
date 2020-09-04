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

#[derive(Clone, PartialEq, Debug)]
pub struct GaussianFilter {
    radius_x: f32,
    radius_y: f32,
    alpha: f32,
    exp_x: f32,
    exp_y: f32,
}

// ===== GaussianFilter ========================================================================================================================================

impl GaussianFilter {
    pub fn new(radius_x: f32, radius_y: f32, alpha: f32) -> GaussianFilter {
        let exp_x = f32::exp(-alpha * radius_x * radius_x);
        let exp_y = f32::exp(-alpha * radius_y * radius_y);

        GaussianFilter { radius_x, radius_y, alpha, exp_x, exp_y }
    }

    pub fn with_defaults() -> GaussianFilter {
        GaussianFilter::new(2.0, 2.0, 2.0)
    }

    fn gaussian(&self, d: f32, exp: f32) -> f32 {
        f32::max(0.0, f32::exp(-self.alpha * d * d) - exp)
    }
}

impl Filter for GaussianFilter {
    fn radius(&self) -> (f32, f32) {
        (self.radius_x, self.radius_y)
    }

    fn evaluate(&self, x: f32, y: f32) -> f32 {
        self.gaussian(x, self.exp_x) * self.gaussian(y, self.exp_y)
    }
}
