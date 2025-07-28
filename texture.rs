use crate::{color::Color, vec3::Point3, interval::Interval, rtw_stb_image::RtwImage};
use std::sync::Arc;
use std::path::Path;
use std::io;
use crate::perlin::Perlin;

pub trait Texture: Send + Sync + std::fmt::Debug {
    fn value(&self, u: f64, v: f64, p: &Point3) -> Color;
}

pub type SharedTexture = Arc<dyn Texture + Send + Sync>;

#[derive(Debug, Clone)]
pub struct SolidColor {
    albedo: Color,
}

impl SolidColor {
    pub fn new(albedo: Color) -> Self {
        Self {albedo}
    }
    
    pub fn from_rgb(red: f64, green: f64, blue: f64) -> Self {
        Self {
            albedo: Color::new(red, green, blue),
        }
    }
}

impl Texture for SolidColor {
    fn value(&self, _u: f64, _v: f64, _p: &Point3) -> Color {
        self.albedo
    }    
}

#[derive(Debug)]
pub struct CheckerTexture {
    inv_scale: f64,
    even: SharedTexture,
    odd: SharedTexture,
}

impl CheckerTexture {
    pub fn new(scale: f64, even: SharedTexture, odd: SharedTexture) -> Self {
        Self {
            inv_scale: 1.0 / scale,
            even,
            odd,
        }
    }
    
    pub fn from_colors(scale: f64, c1: Color, c2: Color) -> Self {
        Self::new(scale, Arc::new(SolidColor::new(c1)), Arc::new(SolidColor::new(c2)))
    }
}

impl Texture for CheckerTexture {
    fn value(&self, u: f64, v: f64, p: &Point3) -> Color {
        let x = (self.inv_scale * p.x()).floor() as i32;
        let y = (self.inv_scale * p.y()).floor() as i32;
        let z = (self.inv_scale * p.z()).floor() as i32;
        
        if (x + y + z) % 2 == 0 {
            self.even.value(u, v, p)
        } else {
            self.odd.value(u, v, p)
        }
    }
}

#[derive(Debug)]
pub struct ImageTexture {
    image: RtwImage,
}

impl ImageTexture {
    pub fn new(filename: &str) -> Self {
        Self {
            image: RtwImage::from_file(filename),
        }
    }
}

impl Texture for ImageTexture {
    fn value(&self, u: f64, v: f64, _p: &Point3) -> Color {
        if self.image.height() <= 0 {
            let u = u.clamp(0.0, 1.0);
            let v = v.clamp(0.0, 1.0);
            
            let lat = v * std::f64::consts::PI; 
            let lon = u * 2.0 * std::f64::consts::PI; 
            
            let land_mask = ((lat.sin() * 3.0).sin() * (lon.sin() * 5.0).sin()).abs();
            if land_mask > 0.3 {
                Color::new(0.3, 0.7, 0.2)
            } else {
                Color::new(0.1, 0.3, 0.8)
            }
        } else {
            let u = u.clamp(0.0, 1.0);
            let v = 1.0 - v.clamp(0.0, 1.0);  

            let i = ((u * self.image.width() as f64) as u32).min(self.image.width() as u32 - 1);
            let j = ((v * self.image.height() as f64) as u32).min(self.image.height() as u32 - 1);
            
            let pixel = self.image.pixel_data(i as i32, j as i32);
            
            let color_scale = 1.0 / 255.0;
            Color::new(
                color_scale * pixel[0] as f64, 
                color_scale * pixel[1] as f64,  
                color_scale * pixel[2] as f64,  
            )
        }
    }
}

#[derive(Debug)]
pub struct NoiseTexture {
    noise: Perlin,
    scale: f64,
}

impl NoiseTexture {
    pub fn new(scale: f64) -> Self {
        Self {
            noise: Perlin::new(),
            scale,
        }
    }
}

impl Texture for NoiseTexture {
    fn value(&self, _u: f64, _v: f64, p: &Point3) -> Color {
        Color::new(0.5, 0.5, 0.5) * (1.0 + (self.scale * p.z() + 10 as f64 * self.noise.turb(p, 7)).sin())
    }
}
