use crate::interval::Interval;
use crate::ray::Ray;
use crate::vec3::{Point3, Vec3};

#[derive(Debug, Clone, Copy)]
pub enum Axis {X = 0, Y = 1, Z = 2}

#[derive(Debug, Clone, Copy, PartialEq)] 
pub struct Aabb {
    pub x: Interval,
    pub y: Interval,
    pub z: Interval,
}

impl Aabb {
    const PADDING_DELTA: f64 = 0.0001;

    pub fn new() -> Self {
        Self {
            x: Interval::EMPTY,
            y: Interval::EMPTY,
            z: Interval::EMPTY,
        }
    }
    
    pub fn from_interval(x: Interval, y: Interval, z: Interval) -> Self {
        let mut aabb = Self {x, y, z};
        aabb.pad();
        aabb
    }
    
    fn pad(&mut self) -> &mut Self {
        if self.x.size() < Self::PADDING_DELTA {
            self.x = self.x.expand(Self::PADDING_DELTA);
        }
        if self.y.size() < Self::PADDING_DELTA {
            self.y = self.y.expand(Self::PADDING_DELTA);
        }
        if self.z.size() < Self::PADDING_DELTA {
            self.z = self.z.expand(Self::PADDING_DELTA);
        }
        self
    }
    
    pub fn from_point(a: Point3, b: Point3) -> Self {
        let x = Interval::with_bounds(a.x().min(b.x()), a.x().max(b.x()));
        let y = Interval::with_bounds(a.y().min(b.y()), a.y().max(b.y()));
        let z = Interval::with_bounds(a.z().min(b.z()), a.z().max(b.z()));
        Self::from_interval(x, y, z)
    }
    
    pub fn axis(&self, n: usize) -> &Interval {
        match n {
            0 => &self.x,
            1 => &self.y,
            _ => &self.z,
        }
    }
    
    pub fn longest_axis(&self) -> Axis {
        let x_size = self.x.size();
        let y_size = self.y.size();
        let z_size = self.z.size();

        if x_size > y_size && x_size > z_size {
            Axis::X
        } else if y_size > z_size {
            Axis::Y
        } else {
            Axis::Z
        }
    }
    
    pub fn hit(&self, r: &Ray, mut ray_t: Interval) -> bool {
        for axis in 0..3 {
            let inv_d = 1.0 / r.direction()[axis];
            let orig = r.origin()[axis];
            
            let mut t0 = (self.axis(axis).min - orig) * inv_d;
            let mut t1 = (self.axis(axis).max - orig) * inv_d;
            
            if inv_d < 0.0 {
                std::mem::swap(&mut t0, &mut t1);
            }
            
            ray_t.min = ray_t.min.max(t0);
            ray_t.max = ray_t.max.min(t1);
            
            if ray_t.max <= ray_t.min {
                return false;
            }
        } 
        true
    }
    
    pub fn merge(&self, other: &Self) -> Self {
        Self {
            x: self.x.merge(&other.x),
            y: self.y.merge(&other.y),
            z: self.z.merge(&other.z),
        }
    }
    
    pub fn expand(&self, delta: f64) -> Self {
        Self {
            x: self.x.expand(delta),
            y: self.y.expand(delta),
            z: self.z.expand(delta),
        }
    }

    pub const EMPTY: Self = Self {
        x: Interval::EMPTY,
        y: Interval::EMPTY,
        z: Interval::EMPTY,
    };
    
    pub const UNIVERSE: Self = Self {
        x: Interval::UNIVERSE,
        y: Interval::UNIVERSE,
        z: Interval::UNIVERSE,
    };
}

impl Default for Aabb {
    fn default() -> Self {
        Self::EMPTY
    }
}

impl std::ops::Add<Vec3> for Aabb {
    type Output = Self;
    
    fn add(self, offset: Vec3) -> Self {
        Self {
            x: self.x + offset.x(),
            y: self.y + offset.y(),
            z: self.z + offset.z(),
        }
    }
}

impl std::ops::Add<Aabb> for Vec3 {
    type Output = Aabb;
    
    fn add(self, bbox: Aabb) -> Aabb {
        bbox + self
    }
}

impl std::ops::AddAssign<Vec3> for Aabb {
    fn add_assign(&mut self, offset: Vec3) {
        *self = Self {
            x: self.x + offset.x(),
            y: self.y + offset.y(),
            z: self.z + offset.z(),
        };
    } 
}
