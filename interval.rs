use std::f64::{INFINITY, NEG_INFINITY};
use std::ops::Add;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Interval {
    pub min: f64,
    pub max: f64,
}

impl Interval {
    pub fn new() -> Self {
        Self {
            min: INFINITY,
            max: NEG_INFINITY,
        }
    }
    
    pub fn with_bounds(min: f64, max: f64) -> Self {
        Self {
            min, 
            max,
        }
    }
    
    pub fn size(&self) -> f64 {
        self.max - self.min
    }
    
    pub fn contains(&self, x: f64) -> bool {
        self.min <= x && x <= self.max
    } 
    
    pub fn surrounds(&self, x: f64) -> bool {
        self.min < x && x < self.max
    }
    
    pub fn clamp(&self, x: f64) -> f64 {
        if x < self.min {
            self.min
        } else if x > self.max {
            self.max
        } else {
            x
        }
    }
    
    pub fn expand(&self, delta: f64) -> Self {
        let padding = delta / 2.0;
        Self::with_bounds(self.min - padding, self.max + padding)
    }
    
    pub fn merge(&self, other: &Self) -> Self {
        Self {
            min: self.min.min(other.min),
            max: self.max.max(other.max),
        }
    }
    
    pub const EMPTY: Self = Self {
        min: INFINITY,
        max: NEG_INFINITY,
    };
    
    pub const UNIVERSE: Self = Self {
        min: NEG_INFINITY,
        max: INFINITY,
    };
    
    pub fn with_displacement(&self, displacement: f64) -> Self {
        Self {
            min: self.min + displacement,
            max: self.max + displacement,
        }
    }
}

impl Default for Interval {
    fn default() -> Self {
        Self::new()
    }
}

impl Add<f64> for Interval {
    type Output = Self;
    
    fn add(self, displacement: f64) -> Self::Output {
        self.with_displacement(displacement)
    }
}

impl Add<Interval> for f64 {
    type Output = Interval;
    
    fn add(self, interval: Interval) -> Self::Output {
        interval.with_displacement(self)
    }
}
