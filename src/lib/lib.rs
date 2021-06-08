#[macro_use]
extern crate bitflags;

mod field_of_view;
mod input_buffer;
mod path_find;
mod run;
mod tilegrid;
pub mod util;
mod word_wrap;

pub use field_of_view::{field_of_view, FovIter, FovShape, ViewableField};
pub use input_buffer::{InputBuffer, InputEvent, KeyMods};
pub use path_find::{find_path, AStarIter, PathableMap};
pub use run::{run, RunControl, RunSettings};
pub use tilegrid::{Symbol, TileGrid, TileGridLayer, TileGridView, Tileset, TilesetInfo};
pub use word_wrap::word_wrap;

/// A trait for a map that has minimum and maximum coordinate bounds.
pub trait BoundedMap {
    /// `min_x`, `min_y`, `max_x`, `max_y`.  Note that the latter two are inclusive.
    fn bounds(&self) -> (i32, i32, i32, i32);
}
