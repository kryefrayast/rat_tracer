use std::io::{self, Write};
use std::sync::{Arc, Mutex, atomic::{AtomicUsize, Ordering}};
use rand::Rng;
use std::time::Instant;

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
mod aabb;
mod bvh;
mod texture;
mod rtw_stb_image;
mod perlin;
mod quad;
mod constant_medium;
mod onb;
mod pdf;

use crate::camera::Camera;
use crate::hittable_list::HittableList;
use crate::sphere::Sphere;
use crate::vec3::{Point3, Vec3};
use crate::material::{Material, Lambertian, Metal, Dielectric, SharedMaterial};
use crate::color::Color;
use crate::rtweekend::{PI, random_double, random_double_in_range};
use crate::aabb::Aabb;
use crate::bvh::BvhNode;
use crate::texture::{SharedTexture, Texture, CheckerTexture};
use crate::texture::SolidColor;
use crate::texture::{ImageTexture, NoiseTexture};
use crate::quad::{Quad, box_bounds};
use crate::material::DiffuseLight;
use crate::hittable::{Hittable, RotateY, Translate, SharedHittable};
use crate::constant_medium::ConstantMedium;
use crate::onb::ONB;

fn bouncing_spheres() -> io::Result<()> {
    type SharedSphere = Arc<Sphere>;

    let start_time = Instant::now();
    //let mut world = HittableList::new();
    let mut objects: Vec<SharedHittable> = Vec::new();
    
    let checker_texture: SharedTexture = Arc::new(CheckerTexture::from_colors(0.32, Color::new(0.2, 0.3, 0.1), Color::new(0.9, 0.9, 0.9)));
    let ground_material: SharedMaterial = Arc::new(Lambertian::new(checker_texture));
    objects.push(Arc::new(Sphere::new_static(Point3::new(0.0, -1000.0, 0.0), 1000.0, ground_material)) as SharedHittable);
    
    let mut rng = rand::thread_rng();
    for a in -11..11 {
        for b in -11..11 {
            let choose_mat = random_double();
            let center = Point3::new(a as f64 + 0.9 * random_double(), 0.2, b as f64 + 0.9 * random_double());
            
            if (center - Point3::new(4.0, 0.2, 0.0)).length() > 0.9 {
                if choose_mat < 0.8 {
                    let albedo = Color::random() * Color::random();
                    let sphere_material: SharedMaterial = Arc::new(Lambertian::new(Arc::new(SolidColor::new(albedo))));
                    let center2 = center + Vec3::new(0.0, random_double_in_range(0.0, 0.5), 0.0);
                    objects.push(Arc::new(Sphere::new_moving(center, center2, 0.2, sphere_material))as SharedHittable);
                } else if choose_mat < 0.95 {
                    let albedo = Color::random_range(0.5, 1.0);
                    let fuzz = random_double_in_range(0.0, 0.5);
                    let sphere_material: SharedMaterial = Arc::new(Metal::new(albedo, fuzz));
                    objects.push(Arc::new(Sphere::new_static(center, 0.2, sphere_material))as SharedHittable);
                } else {
                    let sphere_material: SharedMaterial = Arc::new(Dielectric::new(1.5));
                    objects.push(Arc::new(Sphere::new_static(center, 0.2, sphere_material))as SharedHittable);
                }
            }
        }
    }
    
    let material1: SharedMaterial = Arc::new(Dielectric::new(1.5));
    objects.push(Arc::new(Sphere::new_static(Point3::new(0.0, 1.0, 0.0), 1.0, material1))as SharedHittable);
    
    let material2: SharedMaterial = Arc::new(Lambertian::new(Arc::new(SolidColor::new(Color::new(0.4, 0.2, 0.1)))));
    objects.push(Arc::new(Sphere::new_static(Point3::new(-4.0, 1.0, 0.0), 1.0, material2))as SharedHittable);
    
    let material3: SharedMaterial = Arc::new(Metal::new(Color::new(0.7, 0.6, 0.5), 0.0));
    objects.push(Arc::new(Sphere::new_static(Point3::new(4.0, 1.0, 0.0), 1.0, material3))as SharedHittable);
    
    let world_bvh = Arc::new(BvhNode::new(&mut objects));

    let cam = Camera::new(
        16.0 / 9.0, 
        1200, 
        500, 
        50, 
        Color::new(0.70, 0.80, 1.00),
        20.0,
        Point3::new(13.0, 2.0, 3.0),
        Point3::new(0.0, 0.0, 0.0),
        Vec3::new(0.0, 1.0, 0.0),
        0.6,
        10.0,
    );
    
    let pixels_processed = Arc::new(AtomicUsize::new(0));
    let total_pixels = cam.image_width * cam.image_height();
    
    eprintln!("\nStart.");
    
    let mut output_buffer = Vec::new();
    //cam.render(&*world_bvh, &mut output_buffer)?;
    
    let render_time = start_time.elapsed();
    eprintln!("\nMission Acomplishment.");
    eprintln!("\nTime: {:.2?}", render_time);
    
    io::stdout().write_all(&output_buffer)?;
    Ok(())
}

fn checkered_spheres() -> io::Result<()> {
    let start_time = Instant::now();
    
    type SharedSphere = Arc<Sphere>;

    //let mut world = HittableList::new();
    let mut objects: Vec<SharedHittable> = Vec::new();
    
    let checker = Arc::new(CheckerTexture::new(0.32, Arc::new(SolidColor::new(Color::new(0.2, 0.3, 0.1))), Arc::new(SolidColor::new(Color::new(0.9, 0.9, 0.9)))));
    let checker_material = Arc::new(Lambertian::new(checker.clone()));
    objects.push(Arc::new(Sphere::new_static(Point3::new(0.0, -10.0, 0.0), 10.0, checker_material.clone()))as SharedHittable);
    objects.push(Arc::new(Sphere::new_static(Point3::new(0.0, 10.0, 0.0), 10.0, checker_material))as SharedHittable);
    
    let cam = Camera::new(
        16.0 / 9.0,
        400, 
        100,
        50, 
        Color::new(0.70, 0.80, 1.00),
        20.0,
        Point3::new(13.0, 2.0, 3.0),
        Point3::new(0.0, 0.0, 0.0),
        Vec3::new(0.0, 1.0, 0.0),
        0.0, 
        10.0
    );
    
    let world_bvh = Arc::new(BvhNode::new(&mut objects));
    
    let pixels_processed = Arc::new(AtomicUsize::new(0));
    let total_pixels = cam.image_width * cam.image_height();
    
    eprintln!("\nStart.");
    
    let mut output_buffer = Vec::new();
    //cam.render(&*world_bvh, &mut output_buffer)?;
    
    let render_time = start_time.elapsed();
    eprintln!("\nMission Acomplishment.");
    eprintln!("\nTime: {:.2?}", render_time);
    
    io::stdout().write_all(&output_buffer)?;
    
    Ok(())
}

fn earth() -> io::Result<()> {
    let start_time = Instant::now();
    
  
    let earth_texture: SharedTexture = Arc::new(ImageTexture::new("earthmap.jpg"));
    let earth_surface: SharedMaterial = Arc::new(Lambertian::new(earth_texture));
    

    let globe = Arc::new(Sphere::new_static(Point3::new(0.0, 0.0, 0.0), 2.0, earth_surface));
    
    let mut objects: Vec<SharedHittable> = Vec::new();
    objects.push(globe as SharedHittable);
    let world_bvh = Arc::new(BvhNode::new(&mut objects));
    
    let cam = Camera::new(
        16.0 / 9.0,
        400, 
        100,
        50,
        Color::new(0.70, 0.80, 1.00),
        20.0,
        Point3::new(0.0, 0.0, 12.0),
        Point3::new(0.0, 0.0, 0.0),
        Vec3::new(0.0, 1.0, 0.0),
        0.0, 
        10.0,
    );
    
    let mut output_buffer = Vec::new();
    //cam.render(&*world_bvh, &mut output_buffer)?;
    
    let render_time = start_time.elapsed();
    eprintln!("\nRendering completed.");
    eprintln!("Time: {:.2?}", render_time);
    
    io::stdout().write_all(&output_buffer)?;
    Ok(())
}

fn perlin_spheres() -> io::Result<()> {
    let start_time = Instant::now();
    
    //let mut world = HittableList::new();
    let mut objects: Vec<SharedHittable> = Vec::new();
    
    let pertext: SharedTexture = Arc::new(NoiseTexture::new(4.0));
    let mat: SharedMaterial = Arc::new(Lambertian::new(pertext));
    
    objects.push(Arc::new(Sphere::new_static(Point3::new(0.0, -1000.0, 0.0), 1000.0, mat.clone(), )) as SharedHittable);
    objects.push(Arc::new(Sphere::new_static(Point3::new(0.0, 2.0, 0.0), 2.0, mat, )) as SharedHittable);
    
    let world_bvh = Arc::new(BvhNode::new(&mut objects));
    
    let mut cam = Camera::new(
        16.0 / 9.0,
        400,
        100,
        50,
        Color::new(0.70, 0.80, 1.00),
        20.0,
        Point3::new(13.0, 2.0, 3.0),
        Point3::new(0.0, 0.0, 0.0),
        Vec3::new(0.0, 1.0, 0.0),
        0.0, 
        10.0,
    );
    
    let mut output_buffer = Vec::new();
    //cam.render(&*world_bvh, &mut output_buffer)?;
    
    let render_time = start_time.elapsed();
    eprintln!("\nRendering completed.");
    eprintln!("Time: {:.2?}", render_time);
    
    io::stdout().write_all(&output_buffer)?;
    Ok(())
}

fn quads() -> io::Result<()> {
    let start_time = Instant::now();
    
    //let mut world = HittableList::new();
    let mut objects: Vec<SharedHittable> = Vec::new();
    
    let left_red = Arc::new(Lambertian::new_from_color(Color::new(1.0, 0.2, 0.2)));
    let back_green = Arc::new(Lambertian::new_from_color(Color::new(0.2, 1.0, 0.2)));
    let right_blue = Arc::new(Lambertian::new_from_color(Color::new(0.2, 0.2, 1.0)));
    let upper_orange = Arc::new(Lambertian::new_from_color(Color::new(1.0, 0.5, 0.0)));
    let lower_teal = Arc::new(Lambertian::new_from_color(Color::new(0.2, 0.8, 0.8)));
    
    objects.push(Arc::new(Quad::new(Point3::new(-3.0, -2.0, 5.0), Vec3::new(0.0, 0.0, -4.0), Vec3::new(0.0, 4.0, 0.0), left_red,))as SharedHittable);
    objects.push(Arc::new(Quad::new(Point3::new(-2.0, -2.0, 0.0), Vec3::new(4.0, 0.0, 0.0), Vec3::new(0.0, 4.0, 0.0), back_green,))as SharedHittable);
    objects.push(Arc::new(Quad::new(Point3::new(3.0, -2.0, 1.0), Vec3::new(0.0, 0.0, 4.0), Vec3::new(0.0, 4.0, 0.0), right_blue,))as SharedHittable);
    objects.push(Arc::new(Quad::new(Point3::new(-2.0, 3.0, 1.0), Vec3::new(4.0, 0.0, 0.0), Vec3::new(0.0, 0.0, 4.0), upper_orange,))as SharedHittable);
    objects.push(Arc::new(Quad::new(Point3::new(-2.0, -3.0, 5.0), Vec3::new(4.0, 0.0, 0.0), Vec3::new(0.0, 0.0, -4.0), lower_teal,))as SharedHittable);
    
    let world_bvh = Arc::new(BvhNode::new(&mut objects));
    
    let mut cam = Camera::new(
        1.0,
        400,
        100,
        50,
        Color::new(0.70, 0.80, 1.00),
        80.0,
        Point3::new(0.0, 0.0, 9.0),
        Point3::new(0.0, 0.0, 0.0),
        Vec3::new(0.0, 1.0, 0.0),
        0.0, 
        10.0,
    );
    
    let mut output_buffer = Vec::new();
    //cam.render(&*world_bvh, &mut output_buffer)?;
    
    let render_time = start_time.elapsed();
    eprintln!("\nRendering completed.");
    eprintln!("Time: {:.2?}", render_time);
    
    io::stdout().write_all(&output_buffer)?;
    Ok(())
} 

fn simple_light() -> io::Result<()> {
    let start_time = Instant::now();
    
    //let mut world = HittableList::new();
    let mut objects: Vec<SharedHittable> = Vec::new();
    
    let pertext: SharedTexture = Arc::new(NoiseTexture::new(4.0));
    let mat: SharedMaterial = Arc::new(Lambertian::new(pertext.clone()));
    
    objects.push(Arc::new(Sphere::new_static(Point3::new(0.0, -1000.0, 0.0), 1000.0, mat.clone(),))as SharedHittable);
    objects.push(Arc::new(Sphere::new_static(Point3::new(0.0, 2.0, 0.0), 2.0, mat,))as SharedHittable);

    let difflight = Arc::new(DiffuseLight::from_color(Color::new(4.0, 4.0, 4.0)));
    objects.push(Arc::new(Sphere::new_static(Point3::new(0.0, 7.0, 0.0), 2.0, difflight.clone(),))as SharedHittable);
    objects.push(Arc::new(Quad::new(Point3::new(3.0, 1.0, -2.0), Vec3::new(2.0, 0.0, 0.0), Vec3::new(0.0, 2.0, 0.0), difflight,))as SharedHittable);

    
    let world_bvh = Arc::new(BvhNode::new(&mut objects));
    
    let mut cam = Camera::new(
        16.0 / 9.0,
        400,
        100,
        50,
        Color::new(0.0, 0.0, 0.0),
        20.0,
        Point3::new(26.0, 3.0, 6.0),
        Point3::new(0.0, 2.0, 0.0),
        Vec3::new(0.0, 1.0, 0.0),
        0.0, 
        10.0,
    );
    
    let mut output_buffer = Vec::new();
    //cam.render(&*world_bvh, &mut output_buffer)?;
    
    let render_time = start_time.elapsed();
    eprintln!("\nRendering completed.");
    eprintln!("Time: {:.2?}", render_time);
    
    io::stdout().write_all(&output_buffer)?;
    Ok(())
}

fn cornell_box() -> io::Result<()> {
    let start_time = Instant::now();
    
    let mut world = HittableList::new();
    //let mut objects: Vec<SharedHittable> = Vec::new();
    
    let red = Arc::new(Lambertian::new_from_color(Color::new(0.65, 0.05, 0.05)));
    let white = Arc::new(Lambertian::new_from_color(Color::new(0.73, 0.73, 0.73)));
    let green = Arc::new(Lambertian::new_from_color(Color::new(0.12, 0.45, 0.15)));
    let light = Arc::new(DiffuseLight::from_color(Color::new(15.0, 15.0, 15.0)));
    
    //world.add(Arc::new(Quad::new(Point3::new(555.0, 0.0, 0.0), Vec3::new(0.0, 0.0, 555.0), Vec3::new(0.0, 555.0, 0.0), green.clone(),)));
    //world.add(Arc::new(Quad::new(Point3::new(0.0, 0.0, 555.0), Vec3::new(0.0, 0.0, -555.0), Vec3::new(0.0, 555.0, 0.0), red.clone(),)));
    //objects.push(Arc::new(Quad::new(Point3::new(213.0, 554.0, 227.0), Vec3::new(130.0, 0.0, 0.0), Vec3::new(0.0, 0.0, 105.0), light.clone(),))as SharedHittable);
    //world.add(Arc::new(Quad::new(Point3::new(0.0, 555.0, 0.0), Vec3::new(555.0, 0.0, 0.0), Vec3::new(0.0, 0.0, 555.0), white.clone(),)));
    //world.add(Arc::new(Quad::new(Point3::new(0.0, 0.0, 555.0), Vec3::new(555.0, 0.0, 0.0), Vec3::new(0.0, 0.0, -555.0), white.clone(),)));
    //world.add(Arc::new(Quad::new(Point3::new(555.0, 0.0, 555.0), Vec3::new(-555.0, 0.0, 0.0), Vec3::new(0.0, 555.0, 0.0), white.clone(),)));
    
    let mut world = HittableList::new();
    world.add(Arc::new(Quad::new(
        Point3::new(555.0, 0.0, 0.0),
        Vec3::new(0.0, 555.0, 0.0),
        Vec3::new(0.0, 0.0, 555.0),
        green,
    )));
    world.add(Arc::new(Quad::new(
        Point3::new(0.0, 0.0, 0.0),
        Vec3::new(0.0, 555.0, 0.0),
        Vec3::new(0.0, 0.0, 555.0),
        red,
    )));
    world.add(Arc::new(Quad::new(
        Point3::new(343.0, 554.0, 332.0),
        Vec3::new(-130.0, 0.0, 0.0),
        Vec3::new(0.0, 0.0, -105.0),
        light.clone(),
    )));
    world.add(Arc::new(Quad::new(
        Point3::new(0.0, 0.0, 0.0),
        Vec3::new(555.0, 0.0, 0.0),
        Vec3::new(0.0, 0.0, 555.0),
        white.clone(),
    )));
    world.add(Arc::new(Quad::new(
        Point3::new(555.0, 555.0, 555.0),
        Vec3::new(-555.0, 0.0, 0.0),
        Vec3::new(0.0, 0.0, -555.0),
        white.clone(),
    )));
    world.add(Arc::new(Quad::new(
        Point3::new(0.0, 0.0, 555.0),
        Vec3::new(555.0, 0.0, 0.0),
        Vec3::new(0.0, 555.0, 0.0),
        white.clone(),
    )));
    
    //world.add(box_bounds(Point3::new(130.0, 0.0, 65.0), Point3::new(295.0, 165.0, 230.0), white.clone()));
    //world.add(box_bounds(Point3::new(265.0, 0.0, 295.0), Point3::new(430.0, 330.0, 460.0), white.clone()));
    
    // let aluminum = Arc::new(Metal::new(Color::new(0.8, 0.85, 0.88), 0.0));
    // let mut box1 = box_bounds(Point3::new(0.0, 0.0, 0.0), Point3::new(165.0, 330.0, 165.0), aluminum.clone());
    // box1 = Arc::new(RotateY::new(box1, 15.0));
    // box1 = Arc::new(Translate::new(box1, Vec3::new(265.0, 0.0, 295.0)));
    // world.add(box1);
    
    // let mut box2 = box_bounds(Point3::new(0.0, 0.0, 0.0), Point3::new(165.0, 165.0, 165.0), white.clone());
    // box2 = Arc::new(RotateY::new(box2, -18.0));
    // box2 = Arc::new(Translate::new(box2, Vec3::new(130.0, 0.0, 65.0)));
    // world.add(box2);
    
    let mut box1 = box_bounds(Point3::new(0.0, 0.0, 0.0), Point3::new(165.0, 330.0, 165.0), white.clone());
    box1 = Arc::new(RotateY::new(box1, 15.0));
    box1 = Arc::new(Translate::new(box1, Vec3::new(265.0, 0.0, 295.0)));
    world.add(box1);
    
    let glass = Arc::new(Dielectric::new(1.5));
    world.add(Arc::new(Sphere::new_static(Point3::new(190.0, 90.0, 190.0), 90.0, glass.clone())));
    
    let empty_material = Arc::new(Lambertian::new_from_color(Color::new(0.0, 0.0, 0.0)));
    let mut lights = HittableList::new();
    lights.add(Arc::new(Quad::new(
        Point3::new(343.0, 554.0, 332.0),
        Vec3::new(-130.0, 0.0, 0.0),
        Vec3::new(0.0, 0.0, -105.0),
        empty_material.clone(),
    )));
    lights.add(Arc::new(Sphere::new_static(
        Point3::new(190.0, 90.0, 190.0),
        90.0,
        empty_material,
    )));

    //let lights: SharedHittable  = Arc::new(Quad::new(
        //Point3::new(343.0, 554.0, 332.0),
        //Vec3::new(-130.0, 0.0, 0.0),
        //Vec3::new(0.0, 0.0, -105.0),
        //empty_material.clone()
    //));
    
    //let world_bvh: SharedHittable = Arc::new(BvhNode::new(&mut objects));
    
    let mut cam = Camera::new(
        1.0, 
        600, 
        1000,
        50,
        Color::new(0.0, 0.0, 0.0),
        40.0,
        Point3::new(278.0, 278.0, -800.0),
        Point3::new(278.0, 278.0, 0.0),
        Vec3::new(0.0, 1.0, 0.0),
        0.0,
        10.0,
    );
    
    let pixels_processed = Arc::new(AtomicUsize::new(0));
    let total_pixels = cam.image_width * cam.image_height();
    
    eprintln!("\nStart.");
    
    let mut output_buffer = Vec::new();
    //cam.render(Arc::new(world), lights, &mut output_buffer)?;
    cam.render(Arc::new(world), Arc::new(lights), &mut output_buffer)?;
    
    let render_time = start_time.elapsed();
    eprintln!("\nRendering completed.");
    eprintln!("Time: {:.2?}", render_time);
    
    io::stdout().write_all(&output_buffer)?;
    Ok(())
}

fn cornell_smoke() -> io::Result<()> {
    let start_time = Instant::now();
    
    //let mut world = HittableList::new();
    let mut objects: Vec<SharedHittable> = Vec::new();
    
    let red = Arc::new(Lambertian::new(Arc::new(SolidColor::new(Color::new(0.65, 0.05, 0.05)))));
    let white = Arc::new(Lambertian::new(Arc::new(SolidColor::new(Color::new(0.73, 0.73, 0.73)))));
    let green = Arc::new(Lambertian::new(Arc::new(SolidColor::new(Color::new(0.12, 0.45, 0.15)))));
    let light = Arc::new(DiffuseLight::from_color(Color::new(7.0, 7.0, 7.0)));

    objects.push(Arc::new(Quad::new(
        Point3::new(555.0, 0.0, 0.0),
        Vec3::new(0.0, 555.0, 0.0),
        Vec3::new(0.0, 0.0, 555.0),
        green,
    ))as SharedHittable);

    objects.push(Arc::new(Quad::new(
        Point3::new(0.0, 0.0, 0.0),
        Vec3::new(0.0, 555.0, 0.0),
        Vec3::new(0.0, 0.0, 555.0),
        red,
    ))as SharedHittable);

    objects.push(Arc::new(Quad::new(
        Point3::new(113.0, 554.0, 127.0),
        Vec3::new(330.0, 0.0, 0.0),
        Vec3::new(0.0, 0.0, 305.0),
        light,
    ))as SharedHittable);

    objects.push(Arc::new(Quad::new(
        Point3::new(0.0, 555.0, 0.0),
        Vec3::new(555.0, 0.0, 0.0),
        Vec3::new(0.0, 0.0, 555.0),
        white.clone(),
    ))as SharedHittable);

    objects.push(Arc::new(Quad::new(
        Point3::new(0.0, 0.0, 0.0),
        Vec3::new(555.0, 0.0, 0.0),
        Vec3::new(0.0, 0.0, 555.0),
        white.clone(),
    ))as SharedHittable);

    objects.push(Arc::new(Quad::new(
        Point3::new(0.0, 0.0, 555.0),
        Vec3::new(555.0, 0.0, 0.0),
        Vec3::new(0.0, 555.0, 0.0),
        white.clone(),
    ))as SharedHittable);
    
    let box1 = {
        let bounds = box_bounds(
            Point3::new(0.0, 0.0, 0.0),
            Point3::new(165.0, 330.0, 165.0),
            white.clone(),
        );
        let rotated = Arc::new(RotateY::new(bounds, 15.0));
        Arc::new(Translate::new(rotated, Vec3::new(265.0, 0.0, 295.0)))
    };
    objects.push(Arc::new(ConstantMedium::from_color(
        box1,
        0.01,
        Color::new(0.0, 0.0, 0.0),
    ))as SharedHittable);

    let box2 = {
        let bounds = box_bounds(
            Point3::new(0.0, 0.0, 0.0),
            Point3::new(165.0, 165.0, 165.0),
            white.clone(),
        );
        let rotated = Arc::new(RotateY::new(bounds, -18.0));
        Arc::new(Translate::new(rotated, Vec3::new(130.0, 0.0, 65.0)))
    };
    objects.push(Arc::new(ConstantMedium::from_color(
        box2,
        0.01,
        Color::new(1.0, 1.0, 1.0),
    ))as SharedHittable);
    
    let mut cam = Camera::new(
        1.0, 
        600, 
        200,
        50,
        Color::new(0.0, 0.0, 0.0),
        40.0,
        Point3::new(278.0, 278.0, -800.0),
        Point3::new(278.0, 278.0, 0.0),
        Vec3::new(0.0, 1.0, 0.0),
        0.0,
        10.0,
    );
    
    let world_bvh = Arc::new(BvhNode::new(&mut objects));
    
    let pixels_processed = Arc::new(AtomicUsize::new(0));
    let total_pixels = cam.image_width * cam.image_height();
    
    eprintln!("\nStart.");
    
    let mut output_buffer = Vec::new();
    //cam.render(&*world_bvh, &mut output_buffer)?;
    
    let render_time = start_time.elapsed();
    eprintln!("\nRendering completed.");
    eprintln!("Time: {:.2?}", render_time);
    
    io::stdout().write_all(&output_buffer)?;
    Ok(())
}

fn final_scene(image_width: usize, samples_per_pixel: usize, max_depth: usize) -> io::Result<()> {
    let start_time = Instant::now();
    
    //let mut objects: Vec<SharedHittable> = Vec::new();
    
    let mut objects1: Vec<SharedHittable> = Vec::new();
    
    let ground = Arc::new(Lambertian::new_from_color(Color::new(0.48, 0.83, 0.53)));
    
    for i in 0..20 {
        for j in 0..20 {
            let w = 100.0;
            let x0 = -1000.0 + i as f64 * w;
            let z0 = -1000.0 + j as f64 * w;
            let y0 = 0.0;
            let x1 = x0 + w;
            let y1 = rand::random::<f64>() * 100.0 + 1.0;
            let z1 = z0 + w;
            
            objects1.push(box_bounds(Point3::new(x0, y0, z0), Point3::new(x1, y1, z1), ground.clone()) as SharedHittable);
        }
    }
    
    let mut objects2: Vec<SharedHittable> = Vec::new();
    
    objects2.push(Arc::new(BvhNode::new(&mut objects1)) as SharedHittable);
    
    let light = Arc::new(DiffuseLight::from_color(Color::new(7.0, 7.0, 7.0)));
    objects2.push(Arc::new(Quad::new(Point3::new(123.0, 554.0, 147.0), Vec3::new(300.0, 0.0, 0.0), Vec3::new(0.0, 0.0, 265.0), light)) as SharedHittable);
    
    let center1 = Point3::new(400.0, 400.0, 200.0);
    let center2 = center1 + Vec3::new(30.0, 0.0, 0.0);
    let sphere_material = Arc::new(Lambertian::new_from_color(Color::new(0.7, 0.3, 0.1)));
    objects2.push(Arc::new(Sphere::new_moving(center1, center2, 50.0, sphere_material)) as SharedHittable);
    objects2.push(Arc::new(Sphere::new_static(Point3::new(260.0, 150.0, 45.0), 50.0, Arc::new(Dielectric::new(1.5)))) as SharedHittable);
    objects2.push(Arc::new(Sphere::new_static(Point3::new(0.0, 150.0, 145.0), 50.0, Arc::new(Metal::new(Color::new(0.8, 0.8, 0.9), 1.0)), )) as SharedHittable);
    
    let boundary = Arc::new(Sphere::new_static(Point3::new(360.0, 150.0, 145.0), 70.0, Arc::new(Dielectric::new(1.5)),));
    objects2.push(boundary.clone() as SharedHittable);
    objects2.push(Arc::new(ConstantMedium::from_color(boundary, 0.2, Color::new(0.2, 0.4, 0.9), )) as SharedHittable);
    
    let boundary2 = Arc::new(Sphere::new_static(Point3::new(0.0, 0.0, 0.0), 5000.0, Arc::new(Dielectric::new(1.5)), ));
    objects2.push(Arc::new(ConstantMedium::from_color(boundary2, 0.0001, Color::new(1.0, 1.0, 1.0), )) as SharedHittable);
    
    let emat = Arc::new(Lambertian::new(Arc::new(ImageTexture::new("earthmap.jpg"))));
    objects2.push(Arc::new(Sphere::new_static(Point3::new(400.0, 200.0, 400.0), 100.0, emat, )) as SharedHittable);
    
    let pertext = Arc::new(NoiseTexture::new(0.2));
    objects2.push(Arc::new(Sphere::new_static(Point3::new(220.0, 280.0, 300.0), 80.0, Arc::new(Lambertian::new(pertext)), )) as SharedHittable);
    
    let mut objects3: Vec<SharedHittable> = Vec::new();
    let white = Arc::new(Lambertian::new_from_color(Color::new(0.73, 0.73, 0.73)));
    let ns = 1000;
    for _ in 0..ns {
        objects3.push(Arc::new(Sphere::new_static(Point3::random_vec3_in_range(0.0, 165.0), 10.0, white.clone())) as SharedHittable);
    }
    
    objects2.push(Arc::new(Translate::new(Arc::new(RotateY::new(Arc::new(BvhNode::new(&mut objects3)), 15.0, )), Vec3::new(-100.0, 270.0, 395.0), )) as SharedHittable);
    
    let mut cam = Camera::new(
        1.0, 
        image_width, 
        samples_per_pixel,
        max_depth,
        Color::new(0.0, 0.0, 0.0),
        40.0,
        Point3::new(478.0, 278.0, -600.0),
        Point3::new(278.0, 278.0, 0.0),
        Vec3::new(0.0, 1.0, 0.0),
        0.0,
        10.0,
    );
    
    let world_bvh = Arc::new(BvhNode::new(&mut objects2));
    
    let pixels_processed = Arc::new(AtomicUsize::new(0));
    let total_pixels = cam.image_width * cam.image_height();
    
    eprintln!("\nStart.");
    
    let mut output_buffer = Vec::new();
    //cam.render(&*world_bvh, &mut output_buffer)?;
    
    let render_time = start_time.elapsed();
    eprintln!("\nRendering completed.");
    eprintln!("Time: {:.2?}", render_time);
    
    io::stdout().write_all(&output_buffer)?;
    Ok(())
}

fn main() -> io::Result<()> {
    //final_scene(400, 250, 4)?;
    //final_scene(800, 10000, 40)?;
    //earth()?;
    cornell_box()?;
    Ok(())
}
