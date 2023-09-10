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

use std::cmp::min;
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
    rectangle: Rectangle,

    index_x: u32,
    index_y: u32,
}

#[derive(Clone, Eq, PartialEq, Debug)]
pub struct RectangleTileIterator {
    rectangle: Rectangle,

    tile_count_x: u32,
    tile_count_y: u32,

    tile_index_x: u32,
    tile_index_y: u32,

    tile_left: u32,
    tile_top: u32,
}

// ===== Rectangle =============================================================================================================================================

impl Rectangle {
    #[inline]
    pub fn new(left: u32, top: u32, right: u32, bottom: u32) -> Rectangle {
        debug_assert!(left <= right, "left must be less than or equal to right but {} > {}", left, right);
        debug_assert!(top <= bottom, "top must be less than or equal to bottom but {} > {}", top, bottom);

        Rectangle { left, top, right, bottom }
    }

    #[inline]
    pub fn width(&self) -> u32 {
        self.right - self.left
    }

    #[inline]
    pub fn height(&self) -> u32 {
        self.bottom - self.top
    }

    #[inline]
    pub fn size(&self) -> usize {
        self.width() as usize * self.height() as usize
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        (self.left == self.right) || (self.top == self.bottom)
    }

    #[inline]
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

    #[inline]
    pub fn overlaps(&self, other: &Rectangle) -> bool {
        self.left < other.right && self.top < other.bottom && self.right > other.left && self.bottom > other.top
    }

    #[inline]
    pub fn index_iter(&self) -> RectangleIndexIterator {
        RectangleIndexIterator::new(self.clone())
    }

    #[inline]
    pub fn tile_iter(&self, tile_count_x: u32, tile_count_y: u32) -> RectangleTileIterator {
        RectangleTileIterator::new(self.clone(), tile_count_x, tile_count_y)
    }

    #[inline]
    pub fn linear_index(&self, x: u32, y: u32) -> usize {
        debug_assert!(x >= self.left && x < self.right, "invalid x index: {} (valid range is {}..{})", x, self.left, self.right);
        debug_assert!(y >= self.top && y < self.bottom, "invalid y index: {} (valid range is {}..{})", y, self.top, self.bottom);

        (y - self.top) as usize * self.width() as usize + (x - self.left) as usize
    }
}

// ===== RectangleIndexIterator ================================================================================================================================

impl RectangleIndexIterator {
    fn new(rectangle: Rectangle) -> RectangleIndexIterator {
        let index_x = rectangle.left;
        let index_y = if rectangle.right > rectangle.left { rectangle.top } else { rectangle.bottom };

        RectangleIndexIterator { rectangle, index_x, index_y }
    }
}

impl Iterator for RectangleIndexIterator {
    type Item = (u32, u32);

    fn next(&mut self) -> Option<(u32, u32)> {
        if self.index_y < self.rectangle.bottom {
            let indices = (self.index_x, self.index_y);

            // Advance indices
            self.index_x += 1;
            if self.index_x >= self.rectangle.right {
                self.index_x = self.rectangle.left;
                self.index_y += 1;
            }

            Some(indices)
        } else {
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        if self.index_y < self.rectangle.bottom {
            let remaining_y = (self.rectangle.bottom - self.index_y) as usize;
            let remaining = remaining_y + (self.rectangle.right - self.index_x) as usize;
            (remaining, Some(remaining))
        } else {
            (0, Some(0))
        }
    }
}

impl ExactSizeIterator for RectangleIndexIterator {}

impl FusedIterator for RectangleIndexIterator {}

// ===== RectangleTileIterator =================================================================================================================================

impl RectangleTileIterator {
    #[inline]
    fn new(rectangle: Rectangle, tile_count_x: u32, tile_count_y: u32) -> RectangleTileIterator {
        debug_assert!(tile_count_x > 0, "tile_count_x must be greater than zero but {} < 0", tile_count_x);
        debug_assert!(tile_count_y > 0, "tile_count_y must be greater than zero but {} < 0", tile_count_y);

        // Tile counts must not be greater than the width and height of the rectangle
        let tile_count_x = min(tile_count_x, rectangle.width());
        let tile_count_y = min(tile_count_y, rectangle.height());

        let (tile_left, tile_top) = (rectangle.left, rectangle.top);

        RectangleTileIterator { rectangle, tile_count_x, tile_count_y, tile_index_x: 0, tile_index_y: 0, tile_left, tile_top }
    }
}

impl Iterator for RectangleTileIterator {
    type Item = Rectangle;

    fn next(&mut self) -> Option<Rectangle> {
        if self.tile_index_y < self.tile_count_y {
            let tile_right = min(self.tile_left + (self.rectangle.right - self.tile_left) / (self.tile_count_x - self.tile_index_x), self.rectangle.right);
            let tile_bottom = min(self.tile_top + (self.rectangle.bottom - self.tile_top) / (self.tile_count_y - self.tile_index_y), self.rectangle.bottom);

            let tile = Rectangle::new(self.tile_left, self.tile_top, tile_right, tile_bottom);

            // Advance indices
            self.tile_index_x += 1;
            self.tile_left = tile_right;
            if self.tile_index_x >= self.tile_count_x {
                self.tile_index_x = 0;
                self.tile_index_y += 1;
                self.tile_top = tile_bottom;
                self.tile_left = self.rectangle.left;
            }

            Some(tile)
        } else {
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        if self.tile_index_y < self.tile_count_y {
            let remaining_y = (self.tile_count_y - self.tile_index_y) as usize;
            let remaining = remaining_y + (self.tile_count_x - self.tile_index_x) as usize;
            (remaining, Some(remaining))
        } else {
            (0, Some(0))
        }
    }
}

impl ExactSizeIterator for RectangleTileIterator {}

impl FusedIterator for RectangleTileIterator {}

// ===== Tests =================================================================================================================================================

#[cfg(test)]
mod test {
    use std::cmp::{max, min};

    use super::*;

    #[test]
    fn rectangle_new() {
        let rect = Rectangle::new(10, 20, 100, 220);
        assert_eq!(rect.left, 10);
        assert_eq!(rect.top, 20);
        assert_eq!(rect.right, 100);
        assert_eq!(rect.bottom, 220);
    }

    #[test]
    fn rectangle_width() {
        let rect = Rectangle::new(10, 20, 100, 220);
        assert_eq!(rect.width(), 90);
    }

    #[test]
    fn rectangle_height() {
        let rect = Rectangle::new(10, 20, 100, 220);
        assert_eq!(rect.height(), 200);
    }

    #[test]
    fn rectangle_size() {
        let rect = Rectangle::new(10, 20, 100, 220);
        assert_eq!(rect.size(), 90 * 200);
    }

    #[test]
    fn rectangle_is_empty() {
        assert!(!Rectangle::new(10, 20, 11, 220).is_empty());
        assert!(!Rectangle::new(10, 20, 100, 21).is_empty());
        assert!(Rectangle::new(10, 20, 10, 220).is_empty());
        assert!(Rectangle::new(10, 20, 100, 20).is_empty());
    }

    #[test]
    fn rectangle_contains() {
        let rect = Rectangle::new(10, 20, 100, 220);
        assert!(rect.contains(10, 20));
        assert!(rect.contains(99, 219));
        assert!(!rect.contains(9, 19));
        assert!(!rect.contains(100, 220));
    }

    #[test]
    fn rectangle_union() {
        let rect1 = Rectangle::new(10, 20, 100, 220);

        // rect2 is outside rect1 to the top left
        let rect2 = Rectangle::new(5, 10, 8, 12);
        assert_eq!(rect1.union(&rect2), Rectangle::new(5, 10, 100, 220));

        // rect2's top left is outside rect1's top left, rect2's right bottom is inside rect1
        let rect2 = Rectangle::new(5, 10, 50, 80);
        assert_eq!(rect1.union(&rect2), Rectangle::new(5, 10, 100, 220));

        // rect2's top left is outside rect1's top left, rect2's right bottom is outside rect1 to the right bottom
        let rect2 = Rectangle::new(5, 10, 120, 240);
        assert_eq!(rect1.union(&rect2), Rectangle::new(5, 10, 120, 240));

        // rect2's top left is inside rect1, rect2's right bottom is outside rect1 to the right bottom
        let rect2 = Rectangle::new(15, 25, 120, 240);
        assert_eq!(rect1.union(&rect2), Rectangle::new(10, 20, 120, 240));
    }

    #[test]
    fn rectangle_intersection() {
        let rect1 = Rectangle::new(10, 20, 100, 220);

        // rect2 is outside rect1 to the top left
        let rect2 = Rectangle::new(5, 10, 8, 12);
        assert_eq!(rect1.intersection(&rect2), None);

        // rect2's top left is outside rect1's top left, rect2's right bottom is inside rect1
        let rect2 = Rectangle::new(5, 10, 50, 80);
        assert_eq!(rect1.intersection(&rect2), Some(Rectangle::new(10, 20, 50, 80)));

        // rect2's top left is outside rect1's top left, rect2's right bottom is outside rect1 to the right bottom
        let rect2 = Rectangle::new(5, 10, 120, 240);
        assert_eq!(rect1.intersection(&rect2), Some(Rectangle::new(10, 20, 100, 220)));

        // rect2's top left is inside rect1, rect2's right bottom is outside rect1 to the right bottom
        let rect2 = Rectangle::new(15, 25, 120, 240);
        assert_eq!(rect1.intersection(&rect2), Some(Rectangle::new(15, 25, 100, 220)));
    }

    #[test]
    fn rectangle_overlaps() {
        let rect1 = Rectangle::new(10, 20, 100, 220);

        // rect2 is outside rect1 to the top left
        let rect2 = Rectangle::new(5, 10, 8, 12);
        assert!(!rect1.overlaps(&rect2));

        // rect2's top left is outside rect1's top left, rect2's right bottom is inside rect1
        let rect2 = Rectangle::new(5, 10, 50, 80);
        assert!(rect1.overlaps(&rect2));

        // rect2's top left is outside rect1's top left, rect2's right bottom is outside rect1 to the right bottom
        let rect2 = Rectangle::new(5, 10, 120, 240);
        assert!(rect1.overlaps(&rect2));

        // rect2's top left is inside rect1, rect2's right bottom is outside rect1 to the right bottom
        let rect2 = Rectangle::new(15, 25, 120, 240);
        assert!(rect1.overlaps(&rect2));
    }

    #[test]
    fn rectangle_index_iter() {
        let rect = Rectangle::new(10, 20, 100, 220);

        let (mut min_x, mut max_x) = (u32::MAX, u32::MIN);
        let (mut min_y, mut max_y) = (u32::MAX, u32::MIN);
        let mut count = 0;

        for (x, y) in rect.index_iter() {
            (min_x, max_x) = (min(min_x, x), max(max_x, x));
            (min_y, max_y) = (min(min_y, y), max(max_y, y));
            count += 1;
        }

        assert_eq!((min_x, max_x), (10, 99));
        assert_eq!((min_y, max_y), (20, 219));
        assert_eq!(count, 90 * 200);
    }

    #[test]
    fn rectangle_tile_iter_horizontal() {
        for width in 8..122 {
            let rect = Rectangle::new(13, 0, 13 + width, 10);
            // println!("***** rect: {:?}", rect);

            let mut count = 0;
            let mut last_right = 13;

            for tile in rect.tile_iter(11, 1) {
                // println!("  --- tile: {:?}", tile);
                count += 1;

                assert!(!tile.is_empty(), "empty tile");

                assert_eq!(tile.left, last_right, "horizontal gap between tiles");
                last_right = tile.right;
            }

            let expected_count = min(width, 11); // not more than the width of the rectangle
            assert_eq!(count, expected_count, "incorrect horizontal tile count: {} != {}", count, expected_count);
        }
    }

    #[test]
    fn rectangle_tile_iter_example() {
        let rect = Rectangle::new(0, 0, 1920, 1080);
        println!("***** rect: {:?}", rect);

        for tile in rect.tile_iter(16, 9) {
            println!("  --- tile: {:?}; w={}, h={}", tile, tile.width(), tile.height());
        }
    }

    #[test]
    fn rectangle_linear_index() {
        let rect = Rectangle::new(10, 20, 100, 220);
        assert_eq!(rect.linear_index(10, 20), 0);
        assert_eq!(rect.linear_index(99, 20), 89);
        assert_eq!(rect.linear_index(10, 21), 90);
        assert_eq!(rect.linear_index(10, 22), 180);
        assert_eq!(rect.linear_index(99, 219), 90 * 200 - 1);
    }
}
