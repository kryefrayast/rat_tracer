use std::f64::consts::PI;
use crate::onb::ONB;
use crate::vec3::{Vec3, Point3, random_unit_vector, random_cosine_direction};
use crate::hittable::{Hittable, SharedHittable};
use std::sync::Arc;
use rand::random;

pub trait PDF: std::fmt::Debug + Send + Sync {
    fn value(&self, direction: Vec3) -> f64;
    fn generate(&self) -> Vec3;
}

#[derive(Debug)]
pub struct SpherePDF;

impl SpherePDF {
    pub fn new() -> Self {
        Self
    }
}

impl PDF for SpherePDF {
    fn value(&self, _direction: Vec3) -> f64 {
        1.0 / (4.0 * PI)
    }
    
    fn generate(&self) -> Vec3 {
        random_unit_vector()
    }
}

#[derive(Debug)]
pub struct CosinePDF {
    uvw: ONB,
}

impl CosinePDF {
    pub fn new(w: Vec3) -> Self {
        Self {
            uvw: ONB::new(w),
        }
    }
}

impl PDF for CosinePDF {
    fn value(&self, direction: Vec3) -> f64 {
        let cosine_theta = direction.unit_vector().dot(self.uvw.w());
        f64::max(0.0, cosine_theta / PI)
    }
    
    fn generate(&self) -> Vec3 {
        self.uvw.transform(random_cosine_direction())
    }
}

#[derive(Debug)]
pub struct HittablePDF {
    objects: SharedHittable,
    origin: Point3,
}

impl HittablePDF {
    pub fn new(objects: SharedHittable, origin: Point3) -> Self {
        Self {objects, origin}
    }
}

impl PDF for HittablePDF {
    fn value(&self, direction: Vec3) -> f64 {
        self.objects.pdf_value(&self.origin, &direction)
    }
    
    fn generate(&self) -> Vec3 {
        self.objects.random(&self.origin)
    }
}

#[derive(Debug)]
pub struct MixturePDF {
    p: [Arc<dyn PDF + Send + Sync>; 2],
}

impl MixturePDF {
    pub fn new(p0: Arc<dyn PDF + Send + Sync>, p1: Arc<dyn PDF + Send + Sync>) -> Self {
        Self {
            p: [p0, p1],
        }
    }
}

impl PDF for MixturePDF {
    fn value(&self, direction: Vec3) -> f64 {
        0.5 * self.p[0].value(direction) + 0.5 * self.p[1].value(direction)
    }
    
    fn generate(&self) -> Vec3 {
        if random::<f64>() < 0.5 {
            self.p[0].generate()
        } else {
            self.p[1].generate()
        }
    }
}
