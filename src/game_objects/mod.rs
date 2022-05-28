//! Contains objects which get used in `Scene` that don't contain
//! special functions for rendering

mod cube;
mod renderable_3d_object;
mod square;

pub use cube::Cube;
pub use renderable_3d_object::{Renderable3dObject, RenderableIn3d};
pub use square::Square;
