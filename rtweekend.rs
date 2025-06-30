use std::f64::consts::PI as OTHERPI;
use std::sync::Arc;
use rand::{Rng, thread_rng};
use rand_distr::Uniform;

pub use crate::color::Color;
pub use crate::ray::Ray;
pub use crate::vec3::Vec3;
pub use crate::interval::Interval;

pub const INFINITY: f64 = f64::INFINITY;
pub const PI: f64 = OTHERPI;

pub fn degrees_to_radians(degrees: f64) -> f64 {
    degrees * PI / 180.0
}

pub type SharedHittable = Arc<dyn crate::hittable::Hittable>;

pub fn random_double() -> f64 {
    let mut rng = thread_rng();
    rng.gen_range(0.0..1.0)
}

pub fn random_double_in_range(min: f64, max: f64) -> f64 {
    let mut rng = thread_rng();
    rng.gen_range(min..max)
}
 
#[macro_export] 
macro_rules! make_shared {
    ($obj:expr) => {
        Arc::new($obj)
    };
}
