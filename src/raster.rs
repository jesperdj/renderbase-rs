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

    #[inline]
    pub fn rectangle(&self) -> &Rectangle {
        &self.rectangle
    }

    #[inline]
    pub fn get(&self, x: u32, y: u32) -> T {
        let index = self.rectangle.linear_index(x, y);
        self.elements[index]
    }

    #[inline]
    pub fn get_mut(&mut self, x: u32, y: u32) -> &mut T {
        let index = self.rectangle.linear_index(x, y);
        &mut self.elements[index]
    }

    #[inline]
    pub fn set(&mut self, x: u32, y: u32, value: T) {
        let index = self.rectangle.linear_index(x, y);
        self.elements[index] = value;
    }

    pub fn merge<U: Copy + Default, F: FnMut(T, U) -> T>(&mut self, other: &Raster<U>, mut merge_fn: F) {
        if let Some(intersection) = self.rectangle.intersection(other.rectangle()) {
            for (x, y) in intersection.index_iter() {
                let index = self.rectangle.linear_index(x, y);
                let element = &mut self.elements[index];
                *element = merge_fn(*element, other.get(x, y));
            }
        }
    }

    pub fn map<U: Copy + Default, F: FnMut(T) -> U>(&self, mut map_fn: F) -> Raster<U> {
        let rectangle = self.rectangle.clone();

        let mut elements = Vec::with_capacity(self.elements.capacity());
        for &element in &self.elements {
            elements.push(map_fn(element));
        }

        Raster { rectangle, elements }
    }
}

// ===== Tests =================================================================================================================================================

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn raster_rectangle() {
        let raster = Raster::<u8>::new(Rectangle::new(10, 20, 100, 220));
        assert_eq!(*raster.rectangle(), Rectangle::new(10, 20, 100, 220));
    }

    #[test]
    fn raster_get_set() {
        let mut raster = Raster::<u8>::new(Rectangle::new(10, 20, 100, 220));
        raster.set(12, 40, 64u8);
        assert_eq!(raster.get(12, 40), 64u8);
    }

    #[test]
    fn raster_get_mut() {
        let mut raster = Raster::<u8>::new(Rectangle::new(10, 20, 100, 220));
        *raster.get_mut(12, 40) = 64u8;
        assert_eq!(raster.get(12, 40), 64u8);
    }

    #[test]
    fn raster_merge() {
        let mut source = Raster::<i16>::new(Rectangle::new(0, 0, 50, 80));
        let mut v = 0i16;
        for (x, y) in source.rectangle.index_iter() {
            source.set(x, y, v);
            v += 1;
        }
        let source = source; // Reassign to make it immutable

        let mut target = Raster::<i16>::new(Rectangle::new(10, 20, 100, 220));
        target.merge(&source, |v1, v2| { v1 + v2 });

        todo!()
    }

    #[test]
    fn raster_map() {
        let mut source = Raster::<i16>::new(Rectangle::new(0, 0, 50, 80));
        let mut v = 0i16;
        for (x, y) in source.rectangle.index_iter() {
            source.set(x, y, v);
            v += 1;
        }
        let source = source; // Reassign to make it immutable

        let result = source.map(|v| { v + 1 });

        todo!()
    }
}
