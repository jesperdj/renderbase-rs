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

/// Windowed sinc filter.
#[derive(Clone, Debug)]
pub struct LanczosSincFilter {
    radius_x: f32,
    radius_y: f32,
    tau: f32,
}

// ===== LanczosSincFilter =====================================================================================================================================

impl LanczosSincFilter {
    pub fn new(radius_x: f32, radius_y: f32, tau: f32) -> LanczosSincFilter {
        LanczosSincFilter { radius_x, radius_y, tau }
    }

    pub fn with_defaults() -> LanczosSincFilter {
        LanczosSincFilter::new(4.0, 4.0, 3.0)
    }

    fn windowed_sinc(&self, v: f32, r: f32) -> f32 {
        let v = v.abs();
        if v > r {
            0.0
        } else {
            let lanczos = LanczosSincFilter::sinc(v / self.tau);
            LanczosSincFilter::sinc(v) * lanczos
        }
    }

    fn sinc(v: f32) -> f32 {
        let v = v.abs();
        if v < 1e-5 {
            1.0
        } else {
            let w = std::f32::consts::PI * v;
            f32::sin(w) / w
        }
    }
}

impl Filter for LanczosSincFilter {
    fn radius(&self) -> (f32, f32) {
        (self.radius_x, self.radius_y)
    }

    fn evaluate(&self, x: f32, y: f32) -> f32 {
        self.windowed_sinc(x, self.radius_x) * self.windowed_sinc(y, self.radius_y)
    }
}
