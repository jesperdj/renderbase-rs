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

pub use gaussian::*;
pub use lanczos_sinc::*;
pub use mitchell::*;
pub use r#box::*;
pub use triangle::*;

mod r#box;
mod triangle;
mod gaussian;
mod mitchell;
mod lanczos_sinc;

/// Sampling reconstruction filter.
pub trait Filter: Send + Sync {
    /// Returns the radius of the filter in the x and y direction.
    fn radius(&self) -> (f32, f32);

    /// Evaluates the filter at the point (x, y).
    fn evaluate(&self, x: f32, y: f32) -> f32;
}
