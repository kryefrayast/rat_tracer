use std::sync::Arc;
use rand::Rng;
use crate::{
    aabb::{Aabb, Axis},
    hittable::{Hittable, HitRecord, SharedHittable},
    ray::Ray,
    interval::Interval,
    hittable_list::HittableList
};

#[derive(Debug)]
pub struct BvhNode {
    left: SharedHittable,
    right: SharedHittable,
    bbox: Aabb,
}

impl BvhNode {
    const PADDING_DELTA: f64 = 0.0001;
    
    pub fn new(objects: &mut [SharedHittable]) -> Self {
        assert!(!objects.is_empty(), "Cannot build BVH from empty list");
        
        let mut bbox = objects[0].bounding_box();
        for obj in objects.iter().skip(1) {
            bbox = Aabb::merge(&bbox, &obj.bounding_box());
        }
        
        let axis = bbox.longest_axis();
        
        match objects.len() {
            1 => {
                let obj = objects[0].clone();
                Self::create_node(obj.clone(), obj)
            }
            2 => {
                if Self::box_compare(&objects[0], &objects[1], axis) {
                    Self::create_node(objects[0].clone(), objects[1].clone())
                } else {
                    Self::create_node(objects[1].clone(), objects[0].clone())
                }
            }
            _ => {
                objects.sort_by(|a, b| {
                    let a_axis = a.bounding_box().axis(axis as usize).min;
                    let b_axis = b.bounding_box().axis(axis as usize).min;
                    a_axis.partial_cmp(&b_axis).unwrap()
                });
                
                let mid = objects.len() / 2;
                let (left_objects, right_objects) = objects.split_at_mut(mid);
                let left = Arc::new(Self::new(left_objects));
                let right = Arc::new(Self::new(right_objects));
                
                Self {
                    left: left.clone(), 
                    right: right.clone(),
                    bbox: Aabb::merge(&left.bounding_box(), &right.bounding_box()),
                }
            }
        }
    }
    
    fn create_node(left: SharedHittable, right: SharedHittable) -> Self {
        let bbox = Aabb::merge(&left.bounding_box(), &right.bounding_box());
        Self {left, right, bbox}
    }
    
    fn box_compare(a: &SharedHittable, b: &SharedHittable, axis: Axis) -> bool {
        a.bounding_box().axis(axis as usize).min < b.bounding_box().axis(axis as usize).min
    }
}

impl Hittable for BvhNode {
    fn hit(&self, r: &Ray, mut ray_t: Interval) -> Option<HitRecord> {
        if !self.bbox.hit(r, ray_t) {
            return None;
        }
        
        let hit_left = self.left.hit(r, ray_t);
        let hit_right = match &hit_left {
            Some(hit) => self.right.hit(r, Interval::with_bounds(ray_t.min, hit.t)),
            None => self.right.hit(r, ray_t),
        };
        
        hit_right.or(hit_left)
    }
    
    fn bounding_box(&self) -> Aabb {
        self.bbox
    }
}
