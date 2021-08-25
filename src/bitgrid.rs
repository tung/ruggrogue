use bitvec::prelude::*;
use serde::{Deserialize, Serialize};

/// A width-by-height-sized BitVec for convenient handling of a grid of boolean values.
#[derive(Deserialize, Serialize)]
pub struct BitGrid {
    width: i32,
    height: i32,
    #[serde(with = "crate::saveload::bit_vec")]
    bv: BitVec,
}

impl BitGrid {
    /// Create a new BitGrid with the given width and height.
    pub fn new(width: i32, height: i32) -> Self {
        assert!(width >= 0);
        assert!(height >= 0);

        Self {
            width,
            height,
            bv: bitvec![0; (width * height) as usize],
        }
    }

    #[inline]
    fn index(&self, x: i32, y: i32) -> usize {
        (y * self.width + x) as usize
    }

    /// Reset all elements to false.
    pub fn zero_out_bits(&mut self) {
        self.bv.set_elements(0);
    }

    /// Get the bool at the given x and y.
    ///
    /// Returns false if out of bounds.
    #[inline]
    pub fn get_bit(&self, x: i32, y: i32) -> bool {
        if x < 0 || y < 0 || x >= self.width || y >= self.height {
            false
        } else {
            self.bv[self.index(x, y)]
        }
    }

    /// Set the bool at the given x and y to value.
    ///
    /// Panics if out of bounds.
    #[inline]
    pub fn set_bit(&mut self, x: i32, y: i32, value: bool) {
        let index = self.index(x, y);
        self.bv.set(index, value);
    }

    /// Apply all true elements of this BitGrid onto another.
    ///
    /// Panics if any true bits of self would fall outside of the other grid, given the offset.
    pub fn apply_bits_onto(&self, other: &mut BitGrid, offset_x: i32, offset_y: i32) {
        for y in 0..self.height {
            for x in 0..self.width {
                if self.bv[self.index(x, y)] {
                    let other_index = other.index(x + offset_x, y + offset_y);
                    other.bv.set(other_index, true);
                }
            }
        }
    }
}
