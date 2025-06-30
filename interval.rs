use std::f64::{INFINITY, NEG_INFINITY};

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
    
    pub const EMPTY: Self = Self {
        min: INFINITY,
        max: NEG_INFINITY,
    };
    
    pub const UNIVERSE: Self = Self {
        min: NEG_INFINITY,
        max: INFINITY,
    };
}

impl Default for Interval {
    fn default() -> Self {
        Self::new()
    }
}
