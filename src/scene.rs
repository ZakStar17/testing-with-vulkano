use crate::{
  game_objects::{Cube, Square},
};
use cgmath::Point3;

pub struct Scene {
  pub cubes: Vec<Cube>,
  pub squares: Vec<Square>,
}

impl Scene {
  pub fn load() -> Self {
    let cubes = vec![
      Cube::new(Point3::new(5.0, 1.0, 0.0), [0.0, 0.0, 1.0]),
      Cube::new(Point3::new(0.0, 0.0, 0.0), [0.0, 1.0, 0.0]),
    ];
    let squares = vec![Square::new()];

    Scene {
      cubes, squares
    }
  }

  pub fn get_cube_mut(&mut self, i: usize) -> &mut Cube {
    &mut self.cubes[i]
  }

  pub fn get_square_mut(&mut self, i: usize) -> &mut Square {
    &mut self.squares[i]
  }
}