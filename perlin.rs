use crate::vec3::{Point3, Vec3};
use rand::Rng;

#[derive(Debug)]
pub struct Perlin {
    point_count: usize,
    //randfloat: Vec<f64>,
    randvec: Vec<Vec3>,
    perm_x: Vec<usize>,
    perm_y: Vec<usize>,
    perm_z: Vec<usize>,
}

impl Perlin {
    const POINT_COUNT: usize = 256;

    pub fn new() -> Self {
        //let mut randfloat = Vec::with_capacity(Self::POINT_COUNT);
        let mut randvec = Vec::with_capacity(Self::POINT_COUNT);
        let mut rng = rand::thread_rng();
        
        for _ in 0..Self::POINT_COUNT {
            //randfloat.push(rng.gen_range(0.0..1.0));
            randvec.push(Vec3::random_vec3_in_range(-1.0, 1.0).unit_vector());
        }

        let perm_x = Self::generate_perm();
        let perm_y = Self::generate_perm();
        let perm_z = Self::generate_perm();

        Self {
            point_count: Self::POINT_COUNT,
            //randfloat,
            randvec,
            perm_x,
            perm_y,
            perm_z,
        }
    }

    pub fn noise(&self, p: &Point3) -> f64 {
        let mut u = p.x() - p.x().floor();
        let mut v = p.y() - p.y().floor();
        let mut w = p.z() - p.z().floor();
        
        //u = u * u * (3.0 - 2.0 * u);
        //v = v * v * (3.0 - 2.0 * v);
        //w = w * w * (3.0 - 2.0 * w);

        let i = p.x().floor() as i32;
        let j = p.y().floor() as i32;
        let k = p.z().floor() as i32;

        //let mut c = [[[0.0; 2]; 2]; 2];
        let mut c = [[[Vec3::default(); 2]; 2]; 2];

        for di in 0..2 {
            for dj in 0..2 {
                for dk in 0..2 {
                    let idx_x = ((i + di) & 255) as usize;
                    let idx_y = ((j + dj) & 255) as usize;
                    let idx_z = ((k + dk) & 255) as usize;
                    
                    //c[di as usize][dj as usize][dk as usize] = self.randfloat[
                    c[di as usize][dj as usize][dk as usize] = self.randvec[
                        self.perm_x[idx_x] ^ 
                        self.perm_y[idx_y] ^ 
                        self.perm_z[idx_z]
                    ];
                }
            }
        }

        //Self::trilinear_interp(c, u, v, w)
        Self::perlin_interp(c, u, v, w)
    }

    fn generate_perm() -> Vec<usize> {
        let mut p = Vec::with_capacity(Self::POINT_COUNT);
        
        for i in 0..Self::POINT_COUNT {
            p.push(i);
        }

        Self::permute(&mut p);
        p
    }

    fn permute(p: &mut [usize]) {
        let mut rng = rand::thread_rng();
        
        for i in (1..p.len()).rev() {
            let target = rng.gen_range(0..=i);
            p.swap(i, target);
        }
    }

    fn trilinear_interp(c: [[[f64; 2]; 2]; 2], u: f64, v: f64, w: f64) -> f64 {
        let mut accum = 0.0;

        for i in 0..2 {
            for j in 0..2 {
                for k in 0..2 {
                    accum += (i as f64 * u + (1 - i) as f64 * (1.0 - u))
                           * (j as f64 * v + (1 - j) as f64 * (1.0 - v))
                           * (k as f64 * w + (1 - k) as f64 * (1.0 - w))
                           * c[i][j][k];
                }
            }
        }

        accum
    }
    
    fn perlin_interp(c: [[[Vec3; 2]; 2]; 2], u: f64, v: f64, w: f64) -> f64 {
        let mut uu = u * u * (3.0 - 2.0 * u);
        let mut vv = v * v * (3.0 - 2.0 * v);
        let mut ww = w * w * (3.0 - 2.0 * w);
        let mut accum = 0.0;
        
        for i in 0..2 {
            for j in 0..2 {
                for k in 0..2 {
                    let weight_v: Vec3 = Vec3::new(u - i as f64, v - j as f64, w - k as f64);
                    accum += (i as f64 * uu + (1 - i) as f64 * (1.0 - uu))
                           * (j as f64 * vv + (1 - j) as f64 * (1.0 - vv))
                           * (k as f64 * ww + (1 - k) as f64 * (1.0 - ww))
                           * c[i][j][k].dot(weight_v);
                }
            }
        }
        
        accum
    }
    
    pub fn turb(&self, p: &Point3, depth: i32) -> f64 {
        let mut accum = 0.0;
        let mut temp_p = *p;
        let mut weight = 1.0;
        
        for _ in 0..depth {
            accum += weight * self.noise(&temp_p);
            weight *= 0.5;
            temp_p = temp_p * 2.0;
        }
        
        accum.abs()
    }
}
