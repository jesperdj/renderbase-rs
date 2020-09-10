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

#[derive(Clone, Eq, PartialEq, Debug)]
pub struct Rectangle {
    pub left: u32,
    pub top: u32,
    pub right: u32,
    pub bottom: u32,
}

#[derive(Clone, Eq, PartialEq, Debug)]
pub struct RectangleIndexIterator {
    left: u32,
    right: u32,
    bottom: u32,
    x: u32,
    y: u32,
}

// ===== Rectangle =============================================================================================================================================

impl Rectangle {
    pub fn new(left: u32, top: u32, right: u32, bottom: u32) -> Rectangle {
        assert!(left <= right, "left must be less than or equal to right but {} > {}", left, right);
        assert!(top <= bottom, "top must be less than or equal to bottom but {} > {}", top, bottom);

        Rectangle { left, top, right, bottom }
    }

    pub fn width(&self) -> u32 {
        self.right - self.left
    }

    pub fn height(&self) -> u32 {
        self.bottom - self.top
    }

    pub fn size(&self) -> usize {
        self.width() as usize * self.height() as usize
    }

    pub fn is_empty(&self) -> bool {
        (self.left == self.right) || (self.top == self.bottom)
    }

    pub fn contains(&self, x: u32, y: u32) -> bool {
        x >= self.left && x < self.right && y >= self.top && y < self.bottom
    }

    pub fn union(&self, other: &Rectangle) -> Rectangle {
        let left = u32::min(self.left, other.left);
        let top = u32::min(self.top, other.top);
        let right = u32::max(self.right, other.right);
        let bottom = u32::max(self.bottom, other.bottom);

        Rectangle { left, top, right, bottom }
    }

    pub fn intersection(&self, other: &Rectangle) -> Option<Rectangle> {
        if self.overlaps(other) {
            let left = u32::max(self.left, other.left);
            let top = u32::max(self.top, other.top);
            let right = u32::min(self.right, other.right);
            let bottom = u32::min(self.bottom, other.bottom);

            Some(Rectangle { left, top, right, bottom })
        } else {
            None
        }
    }

    pub fn overlaps(&self, other: &Rectangle) -> bool {
        self.left < other.right && self.top < other.bottom && self.right > other.left && self.bottom > other.top
    }

    pub fn index_iter(&self) -> RectangleIndexIterator {
        RectangleIndexIterator::new(&self)
    }

    pub fn linear_index(&self, x: u32, y: u32) -> usize {
        assert!(x >= self.left && x < self.right, "invalid x index: {}", x);
        assert!(y >= self.top && y < self.bottom, "invalid y index: {}", y);

        (y - self.top) as usize * self.width() as usize + (x - self.left) as usize
    }
}

// ===== RectangleIndexIterator ================================================================================================================================

impl RectangleIndexIterator {
    fn new(rectangle: &Rectangle) -> RectangleIndexIterator {
        let left = rectangle.left;
        let right = rectangle.right;
        let bottom = rectangle.bottom;
        let x = rectangle.left;
        let y = if rectangle.right > rectangle.left { rectangle.top } else { rectangle.bottom };

        RectangleIndexIterator { left, right, bottom, x, y }
    }
}

impl Iterator for RectangleIndexIterator {
    type Item = (u32, u32);

    fn next(&mut self) -> Option<(u32, u32)> {
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
