use crate::{
  game_objects::{Cube, Renderable3dObject, Square},
  GENERATE_CUBES,
};
use cgmath::{Euler, Point3, Rad};
use rand::Rng;
use std::f32::consts::PI;

/// Contains game objects not directly related to rendering
pub struct Scene {
  pub cubes: Vec<Cube>,
  pub squares: Vec<Square>,
  pub objects_changed: bool,
  pub total_object_count: usize,
}

impl Scene {
  pub fn load() -> Self {
    let cubes = if GENERATE_CUBES == None {
      vec![
        Cube::new(Point3::new(5.0, 1.0, 0.0)),
        Cube::new(Point3::new(2.0, 0.0, 0.0)),
      ]
    } else {
      Self::get_random_cubes()
    };
    let squares = vec![Square::new()];

    let total_object_count = cubes.len() + squares.len();

    Scene {
      cubes,
      squares,
      objects_changed: true,
      total_object_count,
    }
  }

  fn get_random_cubes() -> Vec<Cube> {
    let gen_length = if let Some(value) = GENERATE_CUBES {
      value
    } else {
      panic!()
    };

    println!("Generating cubes...");
    let mut rng = rand::thread_rng();
    let mut cubes: Vec<Cube> = Vec::with_capacity(gen_length * gen_length * gen_length);
    for i in 0..gen_length {
      for j in 0..gen_length {
        for k in 0..gen_length {
          cubes.push(Cube::from_full(Renderable3dObject::from_full(
            Point3::new(
              i as f32 - 0.5 + rng.gen::<f32>(),
              j as f32 - 0.5 + rng.gen::<f32>(),
              k as f32 - 0.5 + rng.gen::<f32>(),
            ),
            Euler::new(
              Rad(rng.gen_range(-PI..PI)),
              Rad(rng.gen_range(-PI..PI)),
              Rad(rng.gen_range(-PI..PI)),
            ),
            0.15,
          )))
        }
      }
    }

    cubes
  }

  pub fn get_cube_mut(&mut self, i: usize) -> &mut Cube {
    self.objects_changed = true;
    &mut self.cubes[i]
  }

  pub fn get_square_mut(&mut self, i: usize) -> &mut Square {
    self.objects_changed = true;
    &mut self.squares[i]
  }
}
