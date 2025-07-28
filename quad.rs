use crate::{aabb::Aabb, hittable::{HitRecord, Hittable, SharedHittable}, hittable_list::HittableList, material::SharedMaterial, ray::Ray, vec3::{Point3, Vec3, cross}, interval::Interval};
use std::sync::Arc;
use rand::{Rng, thread_rng};
use std::f64::INFINITY;

#[derive(Debug)]
pub struct Quad {
    q: Point3,
    u: Vec3,
    v: Vec3,
    w: Vec3,
    mat: SharedMaterial,
    bbox: Aabb,
    normal: Vec3,
    d: f64,
    area: f64,
}

impl Quad {
    pub fn new(q: Point3, u: Vec3, v: Vec3, mat: SharedMaterial) -> Self {
        let n = cross(u, v);
        let normal = n.unit_vector();
        let d = normal.dot(q);
        let w = n / n.dot(n);
        let area = n.length();
        
        let mut quad = Self {
            q, 
            u, 
            v, 
            w,
            mat, 
            bbox: Aabb::EMPTY,
            normal,
            d,
            area,
        };
        
        quad.set_bounding_box();
        quad
    }
    
    fn set_bounding_box(&mut self) {
        let bbox_diagonal1 = Aabb::from_point(self.q, self.q + self.u + self.v);
        let bbox_diagonal2 = Aabb::from_point(self.q + self.u, self.q + self.v);
        self.bbox = Aabb::merge(&bbox_diagonal1, &bbox_diagonal2);
    }
    
    fn is_interior(&self, a: f64, b: f64, rec: &mut HitRecord) -> bool {
        let unit_interval = Interval::with_bounds(0.0, 1.0);
        
        if !unit_interval.contains(a) || !unit_interval.contains(b) {
            return false;
        }
        
        rec.u = a;
        rec.v = b;
        
        true
    }
    
    pub fn corners(&self) -> [Point3; 4] {
        [
            self.q,
            self.q + self.u,
            self.q + self.v,
            self.q + self.u + self.v,
        ]
    }
    
    pub fn plane_equation(&self) -> (Vec3, f64) {
        (self.normal, self.d)
    }
}

impl Hittable for Quad {
    fn hit(&self, r: &Ray, ray_t: Interval) -> Option<HitRecord> {
        let denom = self.normal.dot(r.direction());
        
        if denom.abs() < 1e-8 {
            return None;
        }
        
        let t = (self.d - self.normal.dot(r.origin())) / denom;
        
        if !ray_t.contains(t) {
            return None;
        }
        
        let intersection = r.at(t);
        
        let planar_hitpt_vector = intersection - self.q;
        let alpha = self.w.dot(cross(planar_hitpt_vector, self.v));
        let beta = self.w.dot(cross(self.u, planar_hitpt_vector));
        
        let mut rec = HitRecord {
            p: intersection,
            normal: Vec3::default(),
            mat: self.mat.clone(),
            t,
            u: alpha,
            v: beta,
            front_face: false,
        };
        
        if !self.is_interior(alpha, beta, &mut rec) {
            return None;
        }
        
        rec.set_face_normal(r, self.normal);
        
        Some(rec)
    }
    
    fn bounding_box(&self) -> Aabb {
        self.bbox
    }
    
    fn pdf_value(&self, origin: &Point3, direction: &Vec3) -> f64 {
        let ray = Ray::new(*origin, *direction);
        if let Some(rec) = self.hit(&ray, Interval::with_bounds(0.001, INFINITY)) {
            let distance_squared = rec.t * rec.t * direction.length_squared();
            let cosine = direction.dot(rec.normal).abs() / direction.length();
            distance_squared / (cosine * self.area)
        } else {
            0.0
        }
    }
    
    fn random(&self, origin: &Point3) -> Vec3 {
        let mut rng = thread_rng();
        let p = self.q + (rng.gen_range(0.0..1.0) * self.u) + (rng.gen_range(0.0..1.0) * self.v);
        p - *origin
    }
}

pub type SharedQuad = Arc<Quad>;

pub fn box_bounds(a: Point3, b: Point3, mat: SharedMaterial) -> SharedHittable {
    let mut sides = HittableList::new();
    
    let min = Point3::new(a.x().min(b.x()), a.y().min(b.y()), a.z().min(b.z()));
    let max = Point3::new(a.x().max(b.x()), a.y().max(b.y()), a.z().max(b.z()));
    
    let dx = Vec3::new(max.x() - min.x(), 0.0, 0.0);
    let dy = Vec3::new(0.0, max.y() - min.y(), 0.0);
    let dz = Vec3::new(0.0, 0.0, max.z() - min.z());
    
    sides.add(Arc::new(Quad::new(Point3::new(min.x(), min.y(), max.z()), dx, dy, mat.clone(),)));
    sides.add(Arc::new(Quad::new(Point3::new(max.x(), min.y(), max.z()), -dz, dy, mat.clone(),)));
    sides.add(Arc::new(Quad::new(Point3::new(max.x(), min.y(), min.z()), -dx, dy, mat.clone(),)));
    sides.add(Arc::new(Quad::new(Point3::new(min.x(), min.y(), min.z()), dz, dy, mat.clone(),)));
    sides.add(Arc::new(Quad::new(Point3::new(min.x(), max.y(), max.z()), dx, -dz, mat.clone(),)));
    sides.add(Arc::new(Quad::new(Point3::new(min.x(), min.y(), min.z()), dx, dz, mat,)));

    ((Arc::new(sides)) as SharedHittable)
} 
