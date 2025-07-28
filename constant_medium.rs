use crate::{aabb::Aabb, color::Color, hittable::{HitRecord, Hittable, SharedHittable}, material::{Isotropic, SharedMaterial}, ray::Ray, texture::SharedTexture, interval::Interval, vec3::{Vec3, Point3}};
use rand::random;
use std::sync::Arc;

#[derive(Debug)]
pub struct ConstantMedium {
    boundary: SharedHittable,
    neg_inv_density: f64,
    phase_function: SharedMaterial,
}

impl ConstantMedium {
    pub fn new(boundary: SharedHittable, density: f64, texture: SharedTexture) -> Self {
        Self {
            boundary,
            neg_inv_density: -1.0 / density,
            phase_function: Arc::new(Isotropic::new(texture)),
        }
    }
    
    pub fn from_color(boundary: SharedHittable, density: f64, albedo: Color) -> Self {
        Self {
            boundary,
            neg_inv_density: -1.0 / density,
            phase_function: Arc::new(Isotropic::from_color(albedo)),
        }
    }
}

impl Hittable for ConstantMedium {
    fn hit(&self, r: &Ray, ray_t: Interval) -> Option<HitRecord> {
        let mut rec1 = match self.boundary.hit(r, Interval::UNIVERSE) {
            Some(rec) => rec,
            None => return None,
        };
        
        let mut rec2 = match self.boundary.hit(r, Interval::with_bounds(rec1.t + 0.0001, f64::INFINITY)) {
            Some(rec) => rec,
            None => return None,
        };
        
        rec1.t = rec1.t.max(ray_t.min);
        rec2.t = rec2.t.min(ray_t.max);
        
        if rec1.t >= rec2.t {
            return None;
        }
        
        if rec1.t < 0.0 {
            rec1.t = 0.0;
        }
        
        let ray_length = r.direction().length();
        let distance_inside_boundary = (rec2.t - rec1.t) * ray_length;
        let hit_distance = self.neg_inv_density * random::<f64>().ln();
        
        if hit_distance > distance_inside_boundary {
            return None;
        }
        
        let t = rec1.t + hit_distance / ray_length;
        let p = r.at(t);
        
        Some(HitRecord {
            p, 
            normal: Point3::new(1.0, 0.0, 0.0),
            mat: self.phase_function.clone(),
            t,
            u: 0.0,
            v: 0.0,
            front_face: true,
        })
    } 
    
    fn bounding_box(&self) -> Aabb {
        self.boundary.bounding_box()
    } 
}
