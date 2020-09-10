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

use crate::rectangle::Rectangle;

#[derive(Clone)]
pub struct Raster<T: Copy + Default> {
    rectangle: Rectangle,
    elements: Vec<T>,
}

// ===== Raster ================================================================================================================================================

impl<T: Copy + Default> Raster<T> {
    pub fn new(rectangle: Rectangle) -> Raster<T> {
        let size = rectangle.size();
        let mut elements = Vec::with_capacity(size);
        elements.resize_with(size, T::default);

        Raster { rectangle, elements }
    }

    pub fn rectangle(&self) -> &Rectangle {
        &self.rectangle
    }

    pub fn get(&self, x: u32, y: u32) -> T {
        let index = self.rectangle.linear_index(x, y);
        self.elements[index]
    }

    pub fn get_mut(&mut self, x: u32, y: u32) -> &mut T {
        let index = self.rectangle.linear_index(x, y);
        &mut self.elements[index]
    }

    pub fn set(&mut self, x: u32, y: u32, value: T) {
        let index = self.rectangle.linear_index(x, y);
        self.elements[index] = value;
    }

    pub fn merge<U: Copy + Default, F: FnMut(T, U) -> T>(&mut self, other: &Raster<U>, mut merge_fn: F) {
        if let Some(intersection) = self.rectangle.intersection(other.rectangle()) {
            for (x, y) in intersection.index_iter() {
                self.set(x, y, merge_fn(self.get(x, y), other.get(x, y)));
            }
        }
    }

    pub fn map<U: Copy + Default, F: FnMut(T) -> U>(&mut self, mut map_fn: F) -> Raster<U> {
        let rectangle = self.rectangle.clone();

        let mut elements = Vec::with_capacity(self.elements.capacity());
        for &e in &self.elements {
            elements.push(map_fn(e));
        }

        Raster { rectangle, elements }
    }
}
