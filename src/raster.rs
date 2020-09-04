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

use std::iter::FusedIterator;

/// Rectangle that determines the range of indices of a raster.
///
/// Note that left, top are inclusive and right, bottom are exclusive, so that width == right - left and height == bottom - top.
#[derive(Clone, Eq, PartialEq, Debug)]
pub struct Rectangle {
    pub left: i32,
    pub top: i32,
    pub right: i32,
    pub bottom: i32,
}

/// Iterator over rectangle indices.
pub struct RectangleIndexIterator {
    left: i32,
    right: i32,
    bottom: i32,

    x: i32,
    y: i32,
}

/// Raster that stores elements in a rectangular grid.
pub struct Raster<T: Copy + Default> {
    rectangle: Rectangle,
    elements: Vec<T>,
}

// ===== Rectangle =============================================================================================================================================

impl Rectangle {
    /// Creates a new rectangle with the specified top, left, right and bottom limits.
    pub fn new(left: i32, top: i32, right: i32, bottom: i32) -> Rectangle {
        assert!(left <= right, "left must be less than or equal to right but {} > {}", left, right);
        assert!(top <= bottom, "top must be less than or equal to bottom but {} > {}", top, bottom);

        Rectangle { left, top, right, bottom }
    }

    /// Returns the width of this rectangle.
    pub fn width(&self) -> u32 {
        (self.right - self.left) as u32
    }

    /// Returns the height of this rectangle.
    pub fn height(&self) -> u32 {
        (self.bottom - self.top) as u32
    }

    /// Returns the size (number of index pairs) of this rectangle.
    pub fn size(&self) -> usize {
        self.width() as usize * self.height() as usize
    }

    /// Returns a rectangle which is the union of this and another rectangle.
    pub fn union(&self, other: &Rectangle) -> Rectangle {
        let left = i32::min(self.left, other.left);
        let top = i32::min(self.top, other.top);
        let right = i32::max(self.right, other.right);
        let bottom = i32::max(self.bottom, other.bottom);

        Rectangle::new(left, top, right, bottom)
    }

    /// Returns an `Option` with the intersection of this rectangle and another rectangle.
    ///
    /// If this rectangle and the other rectangle do not overlap, `None` is returned.
    pub fn intersection(&self, other: &Rectangle) -> Option<Rectangle> {
        if self.overlaps(other) {
            let left = i32::max(self.left, other.left);
            let top = i32::max(self.top, other.top);
            let right = i32::min(self.right, other.right);
            let bottom = i32::min(self.bottom, other.bottom);

            Some(Rectangle::new(left, top, right, bottom))
        } else {
            None
        }
    }

    /// Checks if this and another rectangle overlap.
    pub fn overlaps(&self, other: &Rectangle) -> bool {
        self.left < other.right && self.top < other.bottom && self.right > other.left && self.bottom > other.top
    }

    /// Returns an iterator over the indices in this rectangle.
    ///
    /// This makes it convenient to iterate over the indices of a raster with a single `for` loop, instead of nested loops:
    ///
    /// ```
    /// for (x, y) in raster.rectangle().index_iter() {
    ///     let value = raster.get(x, y);
    ///     // ...
    /// }
    /// ```
    pub fn index_iter(&self) -> RectangleIndexIterator {
        RectangleIndexIterator::new(&self)
    }

    /// Converts a pair of (x, y) indices into a linear index.
    pub fn linear_index(&self, x: i32, y: i32) -> usize {
        assert!(x >= self.left && x < self.right, "invalid x index: {}", x);
        assert!(y >= self.top && y < self.bottom, "invalid y index: {}", y);

        (y - self.top) as usize * self.width() as usize + (x - self.left) as usize
    }
}

// ===== RectangleIndexIterator ================================================================================================================================

impl RectangleIndexIterator {
    fn new(rectangle: &Rectangle) -> RectangleIndexIterator {
        RectangleIndexIterator {
            left: rectangle.left,
            right: rectangle.right,
            bottom: rectangle.bottom,

            x: rectangle.left,
            y: if rectangle.right > rectangle.left { rectangle.top } else { rectangle.bottom },
        }
    }
}

impl Iterator for RectangleIndexIterator {
    type Item = (i32, i32);

    fn next(&mut self) -> Option<(i32, i32)> {
        if self.y < self.bottom {
            let indices = (self.x, self.y);

            self.x += 1;
            if self.x >= self.right {
                self.x = self.left;
                self.y += 1;
            }

            Some(indices)
        } else {
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        if self.y < self.bottom {
            let remaining_y = (self.bottom - self.y) as usize;
            let remaining = remaining_y + (self.right - self.x) as usize;
            (remaining, Some(remaining))
        } else {
            (0, Some(0))
        }
    }
}

impl ExactSizeIterator for RectangleIndexIterator {}

impl FusedIterator for RectangleIndexIterator {}

// ===== Raster ================================================================================================================================================

impl<T: Copy + Default> Raster<T> {
    /// Creates a new raster.
    pub fn new(rectangle: Rectangle) -> Raster<T> {
        let size = rectangle.size();
        let mut elements = Vec::with_capacity(size);
        elements.resize_with(size, T::default);

        Raster::new_impl(rectangle, elements)
    }

    fn new_impl(rectangle: Rectangle, elements: Vec<T>) -> Raster<T> {
        Raster { rectangle, elements }
    }

    /// Returns the rectangle with valid indices for this raster.
    pub fn rectangle(&self) -> &Rectangle {
        &self.rectangle
    }

    /// Returns the element at index (x, y).
    pub fn get(&self, x: i32, y: i32) -> T {
        let index = self.rectangle.linear_index(x, y);
        self.elements[index]
    }

    /// Returns a mutable reference to the element at index (x, y).
    pub fn get_mut(&mut self, x: i32, y: i32) -> &mut T {
        let index = self.rectangle.linear_index(x, y);
        &mut self.elements[index]
    }

    /// Sets the element at index (x, y) to the specified value.
    pub fn set(&mut self, x: i32, y: i32, value: T) {
        let index = self.rectangle.linear_index(x, y);
        self.elements[index] = value;
    }

    /// Merges another raster into this raster.
    ///
    /// The merge function determines how to combine the elements of this raster and the other raster.
    pub fn merge<U: Copy + Default, F: FnMut(T, U) -> T>(&mut self, other: &Raster<U>, mut merge_fn: F) {
        if let Some(intersection) = self.rectangle.intersection(other.rectangle()) {
            for (x, y) in intersection.index_iter() {
                self.set(x, y, merge_fn(self.get(x, y), other.get(x, y)));
            }
        }
    }

    /// Converts this raster into another raster.
    ///
    /// The map function determines how to convert the elements of this raster into the elements of the result raster.
    pub fn map<U: Copy + Default, F: FnMut(T) -> U>(&mut self, mut map_fn: F) -> Raster<U> {
        let mut new_elements = Vec::with_capacity(self.elements.capacity());
        for &e in &self.elements {
            new_elements.push(map_fn(e));
        }

        Raster::new_impl(self.rectangle.clone(), new_elements)
    }
}
