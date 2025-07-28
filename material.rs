use std::fmt;
use std::f64::consts::PI;
use crate::color::Color;
use crate::hittable::HitRecord;
use crate::ray::Ray;
use crate::vec3::{Vec3, Point3, random_unit_vector, reflect, refract, random_on_hemisphere};
use std::sync::Arc;
use crate::texture::{SolidColor, Texture, SharedTexture};
use crate::onb::ONB;
use crate::vec3::random_cosine_direction;
use crate::pdf::{PDF, CosinePDF, SpherePDF};
use rand::random;

pub trait Material: fmt::Debug + Send + Sync {
    fn scatter(&self, r_in: &Ray, rec: &HitRecord) -> Option<ScatterRecord> {
        None
    }
    
    fn emitted(&self, r_in: &Ray, rec: &HitRecord, u: f64, v: f64, p: &Point3) -> Color {
        Color::new(0.0, 0.0, 0.0)
    }
    
    fn scattering_pdf(&self, _r_in: &Ray, _rec: &HitRecord, _scattered: &Ray) -> f64 {
        0.0
    }
} 

#[derive(Debug)]
pub struct Lambertian {
    albedo: SharedTexture,
}

impl Lambertian {
    pub fn new_from_color(albedo: Color) -> Self {
        Self {
            albedo: Arc::new(SolidColor::new(albedo)),
        }
    }
    
    pub fn new(albedo: SharedTexture) -> Self {
        Self {albedo}
    }
}

impl Material for Lambertian {
    fn scatter(&self, _r_in: &Ray, rec: &HitRecord) -> Option<ScatterRecord> {
        //let uvw = ONB::new(rec.normal);
        //let mut scatter_direction = uvw.transform(random_cosine_direction());
        
        // //if scatter_direction.near_zero() {
           // //scatter_direction = rec.normal;
        // //}
        
        //let scattered = Ray::with_time(rec.p, scatter_direction.unit_vector(), _r_in.time());
        //let attenuation = self.albedo.value(rec.u, rec.v, &rec.p);
        //let attenuation = self.albedo.value(rec.u, rec.v, &rec.p);
        //let pdf = uvw.w().dot(scattered.direction()) / PI;
        
        //Some((attenuation, scattered, pdf))
        let mut srec = ScatterRecord::new();
        srec.attenuation = self.albedo.value(rec.u, rec.v, &rec.p);
        srec.pdf_ptr = Some(Arc::new(CosinePDF::new(rec.normal)));
        srec.skip_pdf = false;
        Some(srec)
    }
    
    fn scattering_pdf(&self, _r_in: &Ray, rec: &HitRecord, scattered: &Ray) -> f64 {
        let cos_theta = rec.normal.dot(scattered.direction().unit_vector());
        if cos_theta < 0.0 {
            0.0
        } else {
            cos_theta / PI
        }
        //1.0 / (2.0 * PI)
    }
}

#[derive(Debug)] 
pub struct Metal {
    albedo: Color,
    fuzz: f64,
}

impl Metal {
    pub fn new(albedo: Color, fuzz: f64) -> Self {
        Self {albedo, fuzz: fuzz.min(1.0),}
    }
}

impl Material for Metal {
    fn scatter(&self, r_in: &Ray, rec: &HitRecord) -> Option<ScatterRecord> {
        let mut srec = ScatterRecord::new();
        
        let reflected = reflect(r_in.direction().unit_vector(), rec.normal);
        let direction = reflected.unit_vector() + (self.fuzz * random_unit_vector());
        //let scattered = Ray::with_time(rec.p, scattered_dir, r_in.time());
        
        //if scattered.direction().dot(rec.normal) > 0.0 {
            //Some((self.albedo, scattered, 1.0))
        //} else {
            //None
        //}
        
        srec.attenuation = self.albedo;
        srec.pdf_ptr = None;
        srec.skip_pdf = true;
        srec.skip_pdf_ray = Some(Ray::with_time(rec.p, direction, r_in.time()));
        
        Some(srec)
    }
}

#[derive(Debug)] 
pub struct Dielectric {
    refraction_index: f64,
}

impl Dielectric {
    pub fn new(refraction_index: f64) -> Self {
        Self {refraction_index}
    }
    
    fn reflectance(cosine: f64, ref_idx: f64) -> f64 {
        let r0 = ((1.0 - ref_idx) / (1.0 + ref_idx)).powi(2);
        r0 + (1.0 - r0) * (1.0 - cosine).powi(5)
    }
}

impl Material for Dielectric {
    fn scatter(&self, r_in: &Ray, rec: &HitRecord) -> Option<ScatterRecord> {
        let mut srec = ScatterRecord::new();
        
        srec.attenuation = Color::new(1.0, 1.0, 1.0);
        srec.pdf_ptr = None;
        srec.skip_pdf = true;
        
        let refraction_ratio = if rec.front_face {
            1.0 / self.refraction_index
        } else {
            self.refraction_index
        };
        
        let unit_direction = r_in.direction().unit_vector();
        let cos_theta = (-unit_direction).dot(rec.normal).min(1.0);
        let sin_theta = (1.0 - cos_theta * cos_theta).sqrt();
        
        let cannot_refract = refraction_ratio * sin_theta > 1.0;
        let direction = if cannot_refract || Self::reflectance(cos_theta, refraction_ratio) > rand::random::<f64>() {
            unit_direction.reflect(rec.normal)
        } else {
            refract(unit_direction, rec.normal, refraction_ratio)
        };
        
        //let scattered = Ray::with_time(rec.p, direction, r_in.time());
        //Some((attenuation, scattered, 1.0))
        
        srec.skip_pdf_ray = Some(Ray::with_time(rec.p, direction, r_in.time()));
        Some(srec)
    }
}

pub type SharedMaterial = Arc<dyn Material + Send + Sync>;

#[derive(Debug)]
pub struct DiffuseLight {
    texture: SharedTexture,
}

impl DiffuseLight {
    pub fn new(texture: SharedTexture) -> Self {
        Self {texture}
    }
    
    pub fn from_color(emit: Color) -> Self {
        Self {
            texture: Arc::new(SolidColor::new(emit)),
        }
    }
}

impl Material for DiffuseLight {
    fn emitted(&self, r_in: &Ray, rec: &HitRecord, u: f64, v: f64, p: &Point3) -> Color {
        self.texture.value(u, v, p)
    }
}

#[derive(Debug)]
pub struct Isotropic {
    texture: SharedTexture,
}

impl Isotropic {
    pub fn new(texture: SharedTexture) -> Self {
        Self {texture}
    }
    
    pub fn from_color(albedo: Color) -> Self {
        Self {
            texture: Arc::new(SolidColor::new(albedo)),
        }
    } 
}

impl Material for Isotropic {
    fn scatter(&self, r_in: &Ray, rec: &HitRecord) -> Option<ScatterRecord> {
        //let scattered = Ray::with_time(rec.p, random_unit_vector(), r_in.time());
        //let attenuation = self.texture.value(rec.u, rec.v, &rec.p);
        //let pdf = 1.0 / (4.0 * PI);
        //Some((attenuation, scattered, pdf))
        
        let mut srec = ScatterRecord::new();
        srec.attenuation = self.texture.value(rec.u, rec.v, &rec.p);
        srec.pdf_ptr = Some(Arc::new(SpherePDF::new()));
        srec.skip_pdf = false;
        Some(srec)
    }
    
    fn scattering_pdf(&self, _r_in: &Ray, _rec: &HitRecord, _scattered: &Ray) -> f64 {
        1.0 / (4.0 * PI)
    }
}

#[derive(Debug)]
pub struct ScatterRecord {
    pub attenuation: Color,
    pub pdf_ptr: Option<Arc<dyn PDF + Send + Sync>>,
    pub skip_pdf: bool,
    pub skip_pdf_ray: Option<Ray>,
}

impl ScatterRecord {
    pub fn new() -> Self {
        Self {
            attenuation: Color::new(0.0, 0.0, 0.0),
            pdf_ptr: None,
            skip_pdf: false,
            skip_pdf_ray: None,
        }
    }
}
