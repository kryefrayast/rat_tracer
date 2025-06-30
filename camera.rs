use crossbeam::thread;
use std::sync::{Arc, Mutex, Condvar, atomic::{AtomicUsize, Ordering}};
use std::io::{self, Write};
use rand::Rng;

use crate::hittable::Hittable;
use crate::ray::Ray;
use crate::vec3::{Point3, Vec3, random_on_hemisphere, random_unit_vector, cross, random_in_unit_disk};
use crate::color::{Color, write_color};
use crate::interval::Interval;
use crate::material::Material;

use std::f64::INFINITY;
use std::f64::consts::PI;

const HEIGHT_PARTITION: usize = 20; 
const WIDTH_PARTITION: usize = 20;
const THREAD_LIMIT: usize = 16;

pub struct Camera {
    pub aspect_ratio: f64,
    pub image_width: usize,
    pub samples_per_pixel: usize,
    pub max_depth: usize,
    pub vfov: f64,
    pub lookfrom: Point3,
    pub lookat: Point3,
    pub vup: Vec3,
    pub defocus_angle: f64, 
    pub focus_dist: f64,
    
    image_height: usize,
    pixel_samples_scale: f64,
    center: Point3,
    pixel00_loc: Point3,
    pixel_delta_u: Vec3,
    pixel_delta_v: Vec3,
    u: Vec3, 
    v: Vec3, 
    w: Vec3,
    defocus_disk_u: Vec3,
    defocus_disk_v: Vec3,
}

impl Camera {
    pub fn new(
        aspect_ratio: f64, 
        image_width: usize, 
        samples_per_pixel: usize, 
        max_depth: usize,
        vfov: f64,
        lookfrom: Point3, 
        lookat: Point3,
        vup: Vec3,
        defocus_angle: f64, 
        focus_dist: f64,
    ) -> Self {
        let mut camera = Self {
            aspect_ratio,
            image_width,
            samples_per_pixel,
            max_depth,
            vfov, 
            lookfrom, 
            lookat, 
            vup,
            defocus_angle,
            focus_dist,
            
            image_height: 0,
            pixel_samples_scale: 0.0,
            center: Point3::new(0.0, 0.0, 0.0),
            pixel00_loc: Point3::default(),
            pixel_delta_u: Vec3::default(),
            pixel_delta_v: Vec3::default(),
            u: Vec3::default(),
            v: Vec3::default(),
            w: Vec3::default(),
            defocus_disk_u: Vec3::default(),
            defocus_disk_v: Vec3::default(),
        };
        camera.initialize();
        camera
    }
    
    pub fn render<W: Write>(&self, world: &(dyn Hittable + Sync), output: &mut W) -> io::Result<()> {
        writeln!(output, "P3\n{} {}\n255", self.image_width, self.image_height)?;
        
        //for j in 0..self.image_height {
            //eprint!("\rScanlines remaining: {}", self.image_height - j);
            //io::stderr().flush()?;
            
            //for i in 0..self.image_width {
                //let mut pixel_color = Color::new(0.0, 0.0, 0.0);
                
                //for _ in 0..self.samples_per_pixel {
                    //let r = self.get_ray(i, j);
                    //pixel_color = pixel_color + self.ray_color(&r, self.max_depth, world);
                //}
                
                //write_color(output, pixel_color * self.pixel_samples_scale)?;
            //}
        //}
        
        let mut pixels = vec![Color::new(0.0, 0.0, 0.0); self.image_width * self.image_height];
        let pixels_mtx = Arc::new(Mutex::new(pixels));
        
        let chunk_height = (self.image_height + HEIGHT_PARTITION - 1) / HEIGHT_PARTITION;
        let chunk_width = (self.image_width + WIDTH_PARTITION - 1) / WIDTH_PARTITION;
        
        thread::scope(|s| {
            let thread_count = Arc::new(AtomicUsize::new(0));
            let thread_controller = Arc::new(Condvar::new());
            let world_arc = Arc::new(world);
            let camera_arc = Arc::new(self);
            
            for j in 0..HEIGHT_PARTITION {
                for i in 0..WIDTH_PARTITION {
                    let lock = Mutex::new(false);
                    while thread_count.load(Ordering::SeqCst) >= THREAD_LIMIT {
                        let _guard = thread_controller.wait(lock.lock().unwrap()).unwrap();
                    }
                    
                    let pixels_clone = Arc::clone(&pixels_mtx);
                    let world_clone = Arc::clone(&world_arc);
                    let camera_clone = Arc::clone(&camera_arc);
                    let thread_count_clone = Arc::clone(&thread_count);
                    let thread_controller_clone = Arc::clone(&thread_controller);
                    
                    thread_count_clone.fetch_add(1, Ordering::SeqCst);
                    
                    s.spawn(move |_| {
                        let x_min = i * chunk_width;
                        let x_max = ((i + 1) * chunk_width).min(camera_clone.image_width);
                        let y_min = j * chunk_height;
                        let y_max = ((j + 1) * chunk_height).min(camera_clone.image_height);
                        
                        let mut local_pixels = vec![Color::new(0.0, 0.0, 0.0); (x_max - x_min) * (y_max - y_min)];
                        
                        for y in y_min..y_max {
                            for x in x_min..x_max {
                                let mut pixel_color = Color::new(0.0, 0.0, 0.0);
                                
                                for _ in 0..camera_clone.samples_per_pixel {
                                    let r = camera_clone.get_ray(x, y);
                                    pixel_color = pixel_color + camera_clone.ray_color(&r, camera_clone.max_depth, &**world_clone);
                                }
                                
                                local_pixels[(y - y_min) * (x_max - x_min) + (x - x_min)] = pixel_color * camera_clone.pixel_samples_scale;
                            } 
                        }
                        
                        let mut pixels = pixels_clone.lock().unwrap();
                        for y in y_min..y_max {
                            for x in x_min..x_max {
                                pixels[y * camera_clone.image_width + x] = local_pixels[(y - y_min) * (x_max - x_min) + (x - x_min)];
                            }
                        }
                        
                        thread_count_clone.fetch_sub(1, Ordering::SeqCst);
                        thread_controller_clone.notify_one();
                    });
                }
            }
        }).unwrap();
        
        for color in pixels_mtx.lock().unwrap().iter() {
            write_color(output, *color)?;
        }
        
        eprintln!("\rDone.");
        Ok(())
    }
    
    fn initialize(&mut self) {
        self.image_height = ((self.image_width as f64) / self.aspect_ratio) as usize;
        self.image_height = self.image_height.max(1);
        self.pixel_samples_scale = 1.0 / self.samples_per_pixel as f64;
        self.center = self.lookfrom;
        
        let theta = Self::degrees_to_radians(self.vfov);
        let h = (theta / 2.0).tan();
        let viewport_height = 2.0 * h * self.focus_dist;
        let viewport_width = viewport_height * (self.image_width as f64 / self.image_height as f64);
        
        self.w = (self.lookfrom - self.lookat).unit_vector();
        self.u = cross(self.vup, self.w).unit_vector();
        self.v = cross(self.w, self.u);
        
        let viewport_u = viewport_width * self.u;
        let viewport_v = viewport_height * (-self.v);
        
        self.pixel_delta_u = viewport_u / self.image_width as f64;
        self.pixel_delta_v = viewport_v / self.image_height as f64;
        
        let viewport_upper_left = self.center - self.focus_dist * self.w - viewport_u / 2.0 - viewport_v / 2.0;
        self.pixel00_loc = viewport_upper_left + 0.5 * (self.pixel_delta_u + self.pixel_delta_v);
        
        let defocus_radius = self.focus_dist * Self::degrees_to_radians(self.defocus_angle / 2.0).tan();
        self.defocus_disk_u = self.u * defocus_radius;
        self.defocus_disk_v = self.v * defocus_radius;
    }
    
    fn degrees_to_radians(degrees: f64) -> f64 {
        degrees * PI / 180.0
    }
    
    fn get_ray(&self, i: usize, j: usize) -> Ray {
        let offset = self.sample_square();
        let pixel_sample = self.pixel00_loc + ((i as f64 + offset.x()) * self.pixel_delta_u) + ((j as f64 + offset.y()) * self.pixel_delta_v);
        
        let ray_origin = if self.defocus_angle <= 0.0 {
            self.center
        } else {
            self.defocus_disk_sample()
        };
        let ray_direction = pixel_sample - ray_origin;
        
        Ray::new(ray_origin, ray_direction)
    }
    
    fn sample_square(&self) -> Vec3 {
        let mut rng = rand::thread_rng();
        Vec3::new(
        rng.gen_range(-0.5..0.5), 
        rng.gen_range(-0.5..0.5),
        0.0,
        )
    }
    
    fn defocus_disk_sample(&self) -> Point3 {
        let p = random_in_unit_disk();
        self.center + p.x() * self.defocus_disk_u + p.y() * self.defocus_disk_v
    }
    
    fn ray_color(&self, r: &Ray, depth: usize, world: &dyn Hittable) -> Color {
        if depth <= 0 {
            return Color::new(0.0, 0.0, 0.0);
        }
        
        if let Some(rec) = world.hit(r, Interval::with_bounds(0.001, INFINITY)) {
            if let Some((attenuation, scattered)) = rec.mat.scatter(r, &rec) {
                return attenuation * self.ray_color(&scattered, depth - 1, world);
            }
            return Color::new(0.0, 0.0, 0.0);
        }
        
        let unit_direction = r.direction().unit_vector();
        let a = 0.5 * (unit_direction.y() + 1.0);
        (1.0 - a) * Color::new(1.0, 1.0, 1.0) + a * Color::new(0.5, 0.7, 1.0)
    }
}
