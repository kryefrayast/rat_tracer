use crate::hittable::{Hittable, HitRecord, SharedHittable};
use crate::ray::Ray;
use std::sync::Arc;
use std::fmt;
use crate::interval::Interval;
use crate::aabb::Aabb;
use crate::vec3::{Point3, Vec3};
use crate::rtweekend::random_int;

#[derive(Debug, Default)]
pub struct HittableList {
    pub objects: Vec<SharedHittable>,
    pub bbox: Aabb,
}

impl HittableList {
    pub fn new() -> Self {
        Self {
            objects: Vec::new(),
            bbox: Aabb::default(),
        }
    }
    pub fn with_object(object: SharedHittable) -> Self {
        let mut list = Self::new();
        list.add(object);
        list
    }
    pub fn clear(&mut self) {
        self.bbox = Aabb::default();
        self.objects.clear();
    }
    pub fn add(&mut self, object: SharedHittable) {
        self.bbox = if self.objects.is_empty() {
            object.bounding_box()
        } else {
            Aabb::merge(&self.bbox, &object.bounding_box())
        };
        self.objects.push(object);
    }
    pub fn new_shared() -> Arc<Self> {
        Arc::new(Self::new())
    }
}

impl Hittable for HittableList {
    fn hit(&self, r: &Ray, ray_t: Interval) -> Option<HitRecord> {
        let mut closest_hit = None;
        let mut closest_so_far = ray_t.max;
        
        for object in &self.objects {
            if let Some(hit) = object.hit(r, Interval::with_bounds(ray_t.min, closest_so_far)) {
                closest_so_far = hit.t;
                closest_hit = Some(hit);
            }
        }
        
        closest_hit
    }
    
    fn bounding_box(&self) -> Aabb {
        self.bbox
    }
    
    fn pdf_value(&self, origin: &Point3, direction: &Vec3) -> f64 {
        let weight = 1.0 / self.objects.len() as f64;
        let mut sum = 0.0;
        
        for object in &self.objects {
            sum = sum + weight * object.pdf_value(origin, direction);
        }
        
        sum
    }
    
    fn random(&self, origin: &Point3) -> Vec3 {
        let int_size = self.objects.len() as i32;
        let index = random_int(0, int_size - 1) as usize;
        self.objects[index].random(origin)
    }
}
