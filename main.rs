use std::io::{self, Write};
use std::sync::{Arc, Mutex};
use rand::Rng;

mod rtweekend;
mod vec3 ; 
mod color;
mod ray;
mod hittable;
mod sphere;
mod hittable_list;
mod camera;
mod interval;
mod material;

use crate::camera::Camera;
use crate::hittable_list::HittableList;
use crate::sphere::Sphere;
use crate::vec3::{Point3, Vec3};
use crate::material::{Material, Lambertian, Metal, Dielectric};
use crate::color::Color;
use crate::rtweekend::{PI, random_double, random_double_in_range};

fn main() -> io::Result<()> {
    type SharedMaterial = Arc<dyn Material + Send + Sync>;
    type SharedSphere = Arc<Sphere>;

    let mut world = HittableList::new();
    
    //let material_ground = Arc::new(Lambertian::new(Color::new(0.8, 0.8, 0.0)));
    //let material_center = Arc::new(Lambertian::new(Color::new(0.1, 0.2, 0.5)));
    //let material_left = Arc::new(Dielectric::new(1.50));
    //let material_bubble = Arc::new(Dielectric::new(1.00 / 1.50));
    //let material_right = Arc::new(Metal::new(Color::new(0.8, 0.6, 0.2), 1.0));
    
    //world.add(Arc::new(Sphere::new(Point3::new(0.0, -100.5, -1.0), 100.0, material_ground,)));
    //world.add(Arc::new(Sphere::new(Point3::new(0.0, 0.0, -1.2), 0.5, material_center,)));
    //world.add(Arc::new(Sphere::new(Point3::new(-1.0, 0.0, -1.0), 0.5, material_left,)));
    //world.add(Arc::new(Sphere::new(Point3::new(-1.0, 0.0, -1.0), 0.4, material_bubble,)));
    //world.add(Arc::new(Sphere::new(Point3::new(1.0, 0.0, -1.0), 0.5, material_right,)));
    
    //let r = (PI / 4.0).cos();
    
    //let material_left = Arc::new(Lambertian::new(Color::new(0.0, 0.0, 1.0)));
    //let material_right = Arc::new(Lambertian::new(Color::new(1.0, 0.0, 0.0)));
    
    //world.add(Arc::new(Sphere::new(Point3::new(-r, 0.0, -1.0), r, material_left)));
    //world.add(Arc::new(Sphere::new(Point3::new(r, 0.0, -1.0), r, material_right)));
    
    let ground_material: SharedMaterial = Arc::new(Lambertian::new(Color::new(0.5, 0.5, 0.5)));
    world.add(Arc::new(Sphere::new(Point3::new(0.0, -1000.0, 0.0), 1000.0, ground_material)));
    
    let mut rng = rand::thread_rng();
    for a in -11..11 {
        for b in -11..11 {
            let choose_mat = random_double();
            let center = Point3::new(a as f64 + 0.9 * random_double(), 0.2, b as f64 + 0.9 * random_double());
            
            if (center - Point3::new(4.0, 0.2, 0.0)).length() > 0.9 {
                if choose_mat < 0.8 {
                    let albedo = Color::random() * Color::random();
                    let sphere_material: SharedMaterial = Arc::new(Lambertian::new(albedo));
                    world.add(Arc::new(Sphere::new(center, 0.2, sphere_material)));
                } else if choose_mat < 0.95 {
                    let albedo = Color::random_range(0.5, 1.0);
                    let fuzz = random_double_in_range(0.0, 0.5);
                    let sphere_material: SharedMaterial = Arc::new(Metal::new(albedo, fuzz));
                    world.add(Arc::new(Sphere::new(center, 0.2, sphere_material)));
                } else {
                    let sphere_material: SharedMaterial = Arc::new(Dielectric::new(1.5));
                    world.add(Arc::new(Sphere::new(center, 0.2, sphere_material)));
                }
            }
        }
    }
    
    let material1: SharedMaterial = Arc::new(Dielectric::new(1.5));
    world.add(Arc::new(Sphere::new(Point3::new(0.0, 1.0, 0.0), 1.0, material1)));
    
    let material2: SharedMaterial = Arc::new(Lambertian::new(Color::new(0.4, 0.2, 0.1)));
    world.add(Arc::new(Sphere::new(Point3::new(-4.0, 1.0, 0.0), 1.0, material2)));
    
    let material3: SharedMaterial = Arc::new(Metal::new(Color::new(0.7, 0.6, 0.5), 0.0));
    world.add(Arc::new(Sphere::new(Point3::new(4.0, 1.0, 0.0), 1.0, material3)));
    
    let cam = Camera::new(
        16.0 / 9.0, 
        1200, 
        500, 
        50, 
        20.0,
        Point3::new(13.0, 2.0, 3.0),
        Point3::new(0.0, 0.0, 0.0),
        Vec3::new(0.0, 1.0, 0.0),
        0.6,
        10.0,
    );
    let mut output_buffer = Vec::new();
    cam.render(&world, &mut output_buffer)?;
    io::stdout().write_all(&output_buffer)?;
    Ok(())
}
