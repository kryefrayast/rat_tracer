use crate::hittable::{Hittable, HitRecord, SharedHittable};
use crate::ray::Ray;
use std::sync::Arc;
use std::fmt;
use crate::interval::Interval;

#[derive(Debug, Default)]
pub struct HittableList {
    objects: Vec<SharedHittable>,
}

impl HittableList {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn with_object(object: SharedHittable) -> Self {
        let mut list = Self::new();
        list.add(object);
        list
    }
    pub fn clear(&mut self) {
        self.objects.clear();
    }
    pub fn add(&mut self, object: SharedHittable) {
        self.objects.push(object);
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
}

impl HittableList {
    pub fn new_shared() -> Arc<Self> {
        Arc::new(Self::new())
    }
}
