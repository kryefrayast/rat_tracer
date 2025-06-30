use std::sync::Arc;
use crate::hittable::{Hittable, HitRecord};
use crate::material::Material;
use crate::ray::Ray;
use crate::vec3::{Point3, Vec3};
use crate::interval::Interval;

#[derive(Debug, Clone)]
pub struct Sphere {
    center: Point3,
    radius: f64,
    mat: Arc<dyn Material + Send + Sync>,
}

impl Sphere {
    pub fn new(center: Point3, radius: f64, mat: Arc<dyn Material + Send + Sync>) -> Self {
        Sphere {
            center, 
            radius: radius.max(0.0),
            mat, 
        }
    }
}

impl Hittable for Sphere {
    fn hit(&self, ray: &Ray, ray_t: Interval) -> Option<HitRecord> {
        let oc = self.center - ray.origin();
        let a = ray.direction().length_squared();
        let h = ray.direction().dot(oc);
        let c = oc.length_squared() - self.radius * self.radius;
        
        let discriminant = h * h - a * c;
        if discriminant < 0.0 {
            return None;
        }
        
        let sqrtd = discriminant.sqrt();
        
        let mut root = (h - sqrtd) / a;
        if !ray_t.surrounds(root) {
            root = (h + sqrtd) / a;
            if !ray_t.surrounds(root) {
                return None;
            }
        }
        
        let p = ray.at(root);
        let outward_normal = (p - self.center) / self.radius;
        
        let mut rec = HitRecord {
            p, 
            normal: Vec3::default(),
            mat: Arc::clone(&self.mat) as Arc<dyn Material + Send + Sync>,
            t: root,
            front_face: false,
        };
        
        rec.set_face_normal(ray, outward_normal);
        
        Some(rec)
    }
}
