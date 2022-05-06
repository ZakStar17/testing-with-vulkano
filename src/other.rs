use cgmath::Point3;
use std::ops::Add;

pub fn add_points<T: Add<Output = T>>(a: Point3<T>, b: Point3<T>) -> Point3<T> {
  Point3 {
    x: a.x + b.x,
    y: a.y + b.y,
    z: a.z + b.z,
  }
}
