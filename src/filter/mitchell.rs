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
#[derive(Clone, PartialEq, Debug)]
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

    pub fn with_defaults() -> MitchellFilter {
        MitchellFilter::new(2.0, 2.0, 1.0 / 3.0, 1.0 / 3.0)
    }

    fn mitchell(&self, v: f32) -> f32 {
        let x = 2.0 * v.abs();
        if x <= 1.0 {
            self.p1[0] + self.p1[1] * x + self.p1[2] * x * x + self.p1[3] * x * x * x
        } else {
            self.p2[0] + self.p2[1] * x + self.p2[2] * x * x + self.p2[3] * x * x * x
        }
    }
}

impl Filter for MitchellFilter {
    fn radius(&self) -> (f32, f32) {
        (self.radius_x, self.radius_y)
    }

    fn evaluate(&self, x: f32, y: f32) -> f32 {
        self.mitchell(x / self.radius_x) * self.mitchell(y / self.radius_y)
    }
}
