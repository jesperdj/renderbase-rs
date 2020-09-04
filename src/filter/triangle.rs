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
pub struct TriangleFilter {
    radius_x: f32,
    radius_y: f32,
}

// ===== TriangleFilter ========================================================================================================================================

impl TriangleFilter {
    pub fn new(radius_x: f32, radius_y: f32) -> TriangleFilter {
        TriangleFilter { radius_x, radius_y }
    }

    pub fn with_defaults() -> TriangleFilter {
        TriangleFilter::new(2.0, 2.0)
    }
}

impl Filter for TriangleFilter {
    fn radius(&self) -> (f32, f32) {
        (self.radius_x, self.radius_y)
    }

    fn evaluate(&self, x: f32, y: f32) -> f32 {
        f32::max(0.0, self.radius_x - x.abs()) * f32::max(0.0, self.radius_y - y.abs())
    }
}
