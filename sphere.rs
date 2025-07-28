use std::sync::Arc;
use std::f64::consts::PI;
use crate::hittable::{Hittable, HitRecord};
use crate::material::{Material, SharedMaterial};
use crate::ray::Ray;
use crate::vec3::{Point3, Vec3, random_vec3};
use crate::interval::Interval;
use crate::aabb::Aabb;
use crate::onb::ONB;
use rand::{Rng, thread_rng};

#[derive(Debug, Clone)]
pub struct Sphere {
    center: Point3,
    moving_vec: Vec3,
    radius: f64,
    mat: SharedMaterial,
    bbox: Aabb,
}

impl Sphere {
    pub fn new_static(center: Point3, radius: f64, mat: Arc<dyn Material + Send + Sync>) -> Self {
        let radius = radius.max(0.0);
        let rvec = Vec3::new(radius, radius, radius);
        let bbox = Aabb::from_point(center - rvec, center + rvec);
        
        Self {
            center, 
            moving_vec: Vec3::default(),
            radius,
            mat, 
            bbox,
        }
    }
    
    pub fn new_moving(center1: Point3, center2: Point3, radius: f64, mat: Arc<dyn Material + Send + Sync>) -> Self {
        let radius = radius.max(0.0);
        let rvec = Vec3::new(radius, radius, radius);
        let bbox1 = Aabb::from_point(center1 - rvec, center1 + rvec);
        let bbox2 = Aabb::from_point(center2 - rvec, center2 + rvec);
        let bbox = bbox1.merge(&bbox2);
        
        Self {
            center: center1,
            moving_vec: center2 - center1,
            radius,
            mat,
            bbox,
        }
    }
    
    fn center_at_time(&self, time: f64) -> Point3 {
        if self.moving_vec.near_zero() {
            self.center
        } else {
            self.center + time * self.moving_vec
        }
    }

    fn get_sphere_uv(p: &Point3) -> (f64, f64) {
        let theta = (-p.y()).acos();
        let phi = (-p.z()).atan2(p.x()) + PI;
        
        let u = phi / (2.0 * PI);
        let v = theta / PI;
        
        (u, v)
    }
    
    fn random_to_sphere(radius: f64, distance_squared: f64) -> Vec3 {
        let mut rng = thread_rng();
        let r1 = rng.gen_range(0.0..1.0);
        let r2 = rng.gen_range(0.0..1.0);
        
        let z = 1.0 + r2 * ((1.0 - radius.powi(2) / distance_squared).sqrt() - 1.0);
        let phi = 2.0 * PI * r1;
        let x = f64::cos(phi) * (1.0 - z.powi(2)).sqrt();
        let y = f64::sin(phi) * (1.0 - z.powi(2)).sqrt();
        
        Vec3::new(x, y, z)
    }
} 

impl Hittable for Sphere {
    fn hit(&self, ray: &Ray, ray_t: Interval) -> Option<HitRecord> {
        let center = self.center_at_time(ray.time());
        let oc = center - ray.origin();
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
        let outward_normal = (p - center) / self.radius;
        
        let mut rec = HitRecord {
            p, 
            normal: Vec3::default(),
            mat: Arc::clone(&self.mat) as Arc<dyn Material + Send + Sync>,
            t: root,
            u: 0.0, 
            v: 0.0,
            front_face: false,
        };
        
        rec.set_face_normal(ray, outward_normal);
        
        let (u, v) = Self::get_sphere_uv(&outward_normal);
        rec.u = u;
        rec.v = v;
        
        Some(rec)
    }
    
    fn bounding_box(&self) -> Aabb {
        self.bbox
    }
    
    fn pdf_value(&self, origin: &Point3, direction: &Vec3) -> f64 {
        let r = Ray::new(*origin, *direction);
        if self.hit(&r, Interval::with_bounds(0.001, f64::INFINITY)).is_none() {
            return 0.0;
        }
        
        let dist_squared = (self.center_at_time(0.0) - *origin).length_squared();
        let cos_theta_max = (1.0 - self.radius.powi(2) / dist_squared).sqrt();
        let solid_angle = 2.0 * PI * (1.0 - cos_theta_max);
        
        1.0 / solid_angle
    }
    
    fn random(&self, origin: &Point3) -> Vec3 {
        let direction = self.center_at_time(0.0) - *origin;
        let distance_squared = direction.length_squared();
        let uvw = ONB::new(direction);
        uvw.transform(Self::random_to_sphere(self.radius, distance_squared))
    }
}
