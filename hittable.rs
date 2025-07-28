use std::fmt;
use std::sync::Arc;
use std::f64::{INFINITY, NEG_INFINITY};

use crate::ray::Ray;
use crate::vec3::{Point3, Vec3};
use crate::interval::Interval;
use crate::material::Material;
use crate::aabb::Aabb;

#[derive(Debug, Clone)]
pub struct HitRecord {
    pub p: Point3,
    pub normal: Vec3,
    pub mat: Arc<dyn Material + Send + Sync>,
    pub t: f64,
    pub u: f64, 
    pub v: f64,
    pub front_face: bool,
}

impl HitRecord {
    pub fn set_face_normal(&mut self, r: &Ray, outward_normal: Vec3) {
        self.front_face = r.direction().dot(outward_normal) < 0.0;
        self.normal = if self.front_face {
            outward_normal
        } else {
            -outward_normal
        };
    }
}

pub trait Hittable: Send + Sync + fmt::Debug {
    fn hit(&self, ray: &Ray, ray_t: Interval) -> Option<HitRecord>;
    fn bounding_box(&self) -> Aabb;
    fn pdf_value(&self, _origin: &Point3, _direction: &Vec3) -> f64 {
        0.0
    }
    fn random(&self, _origin: &Point3) -> Vec3 {
        Vec3::new(1.0, 0.0, 0.0)
    }
}

pub type SharedHittable = Arc<dyn Hittable + Send + Sync>;

#[derive(Debug)]
pub struct Translate {
    object: SharedHittable,
    offset: Vec3,
    bbox: Aabb,
}

impl Translate {
    pub fn new(object: SharedHittable, offset: Vec3) -> Self {
        let bbox = object.bounding_box() + offset;
        Self {object, offset, bbox}
    }
}

impl Hittable for Translate {
    fn hit(&self, r: &Ray, ray_t: Interval) -> Option<HitRecord> {
        let offset_r = Ray::with_time(r.origin() - self.offset, r.direction(), r.time());
        
        let mut rec = self.object.hit(&offset_r, ray_t)?;
        
        rec.p += self.offset;
        Some(rec)
    } 
    
    fn bounding_box(&self) -> Aabb {
        self.bbox
    }
}

#[derive(Debug)] 
pub struct RotateY {
    object: SharedHittable,
    sin_theta: f64,
    cos_theta: f64,
    bbox: Aabb,
}

impl RotateY {
    pub fn new(object: SharedHittable, angle: f64) -> Self {
        let radians = angle.to_radians();
        let sin_theta = radians.sin();
        let cos_theta = radians.cos();
        let bbox = object.bounding_box();
        
        let mut min = Point3::new(INFINITY, INFINITY, INFINITY);
        let mut max = Point3::new(NEG_INFINITY, NEG_INFINITY, NEG_INFINITY);
        
        for i in 0..2 {
            for j in 0..2 {
                for k in 0..2 {
                    let x = if i == 0 {bbox.x.min} else {bbox.x.max};
                    let y = if j == 0 {bbox.y.min} else {bbox.y.max};
                    let z = if k == 0 {bbox.z.min} else {bbox.z.max};
                    
                    let new_x = cos_theta * x + sin_theta * z;
                    let new_z = -sin_theta * x + cos_theta * z;
                    
                    let test = Vec3::new(new_x, y, new_z);
                    
                    min = Point3::new(
                        min.x().min(test.x()),
                        min.y().min(test.y()),
                        min.z().min(test.z()),
                    );

                    max = Point3::new(
                        max.x().max(test.x()),
                        max.y().max(test.y()),
                        max.z().max(test.z()),
                    );
                }
            } 
        }
        
        let bbox = Aabb::from_point(min, max);
        
        Self {object, sin_theta, cos_theta, bbox}
    }
}

impl Hittable for RotateY {
    fn hit(&self, r: &Ray, ray_t: Interval) -> Option<HitRecord> {
        let origin = Point3::new(self.cos_theta * r.origin().x() - self.sin_theta * r.origin().z(), r.origin().y(), self.sin_theta * r.origin().x() + self.cos_theta * r.origin().z());
        
        let direction = Vec3::new(self.cos_theta * r.direction().x() - self.sin_theta * r.direction().z(), r.direction().y(), self.sin_theta * r.direction().x() + self.cos_theta * r.direction().z());
        
        let rotated_r = Ray::with_time(origin, direction, r.time());
        
        let mut rec = self.object.hit(&rotated_r, ray_t)?;
        
        rec.p = Point3::new(self.cos_theta * rec.p.x() + self.sin_theta * rec.p.z(), rec.p.y(), -self.sin_theta * rec.p.x() + self.cos_theta * rec.p.z());
        
        rec.normal = Vec3::new(self.cos_theta * rec.normal.x() + self.sin_theta * rec.normal.z(), rec.normal.y(), -self.sin_theta * rec.normal.x() + self.cos_theta * rec.normal.z());

        Some(rec)
    }
    
    fn bounding_box(&self) -> Aabb {
        self.bbox
    }
}
