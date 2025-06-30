use crate::vec3::Vec3;
use std::fmt;
use std::io::{self, Write};
use std::ops::{Add, Mul};
use crate::interval::Interval;
use rand::Rng;

#[derive(Debug, Clone, Copy)]
pub struct Color(pub Vec3);

impl Color {
    pub fn new(r: f64, g: f64, b: f64) -> Self {
        Color(Vec3::new(r, g, b))
    }
    pub fn r(&self) -> f64 {self.0.x()}
    pub fn g(&self) -> f64 {self.0.y()}
    pub fn b(&self) -> f64 {self.0.z()}
    
    pub fn random() -> Self {
        let mut rng = rand::thread_rng();
        Color::new(rng.gen_range(0.0..1.0), rng.gen_range(0.0..1.0), rng.gen_range(0.0..1.0))
    }
    
    pub fn random_range(min: f64, max: f64) -> Self {
        let mut rng = rand::thread_rng();
        Color::new(rng.gen_range(min..max), rng.gen_range(min..max), rng.gen_range(min..max))
    }
}

fn linear_to_gamma(linear_component: f64) -> f64 {
    if linear_component > 0.0 {
        linear_component.sqrt()
    } else {
        0.0
    }
}

pub fn write_color(out: &mut impl Write, pixel_color: Color) ->  io::Result<()>  {
    static INTENSITY: Interval = Interval {
        min: 0.000, 
        max: 0.999,
    };
    
    let r = linear_to_gamma(pixel_color.r());
    let g = linear_to_gamma(pixel_color.g());
    let b = linear_to_gamma(pixel_color.b());
    
    let rbyte = (255.999 * INTENSITY.clamp(r)) as u8;
    let gbyte = (255.999 * INTENSITY.clamp(g)) as u8;
    let bbyte = (255.999 * INTENSITY.clamp(b)) as u8;
    
    writeln!(out, "{} {} {}", rbyte, gbyte, bbyte)
}

impl fmt::Display for Color {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        static INTENSITY: Interval = Interval {
        min: 0.000, 
        max: 0.999,
        };
    
        let rbyte = (255.999 * INTENSITY.clamp(self.r())) as u8;
        let gbyte = (255.999 * INTENSITY.clamp(self.g())) as u8;
        let bbyte = (255.999 * INTENSITY.clamp(self.b())) as u8;
        
        write!(f, "{} {} {}", rbyte, gbyte, bbyte)
    }
}

impl Add for Color {
    type Output = Self;
    fn add(self, other: Self) -> Self {
        Color(self.0 + other.0)
    }
}

impl Mul<f64> for Color {
    type Output = Self;
    fn mul(self, t: f64) -> Self{
        Color(self.0 * t)
    }
}

impl Mul<Color> for f64 {
    type Output = Color;
    fn mul(self, color: Color) -> Color {
        color * self
    }
}

impl Mul for Color {
    type Output = Self;
    fn mul(self, other: Self) -> Self {
        Color(self.0 * other.0)
    }
}
