use image::{DynamicImage, ImageBuffer, Rgb, Rgb32FImage};
use image::io::Reader as ImageReader;
use std::env;
use std::path::{Path, PathBuf};

#[derive(Debug)]
pub struct RtwImage {
    bytes_per_pixel: usize,
    float_data: Vec<f32>,          
    byte_data: Vec<u8>,          
    width: i32,
    height: i32,
    bytes_per_scanline: usize,
}

impl RtwImage {
    pub fn new() -> Self {
        RtwImage {
            bytes_per_pixel: 3,
            float_data: Vec::new(),
            byte_data: Vec::new(),
            width: 0,
            height: 0,
            bytes_per_scanline: 0,
        }
    }

    pub fn from_file(image_filename: &str) -> Self {
        let mut img = RtwImage::new();
        let filename = Path::new(image_filename);
        
        eprintln!("Attempting to load image: {}", image_filename);
        
        if let Some(imagedir) = env::var_os("RTW_IMAGES") {
            let mut path = PathBuf::from(imagedir);
            path.push(filename);
            eprintln!("Trying RTW_IMAGES path: {:?}", path);
            if img.load(&path) {
                eprintln!("Successfully loaded from RTW_IMAGES path");
                return img;
            }
        }

        let paths_to_try = vec![
            filename.to_path_buf(),
            Path::new("images").join(filename),
            Path::new("../images").join(filename),
            Path::new("../../images").join(filename),
            Path::new("../../../images").join(filename),
            Path::new("../../../../images").join(filename),
            Path::new("../../../../../images").join(filename),
            Path::new("../../../../../../images").join(filename),
            // Also try absolute path if it looks like one
            PathBuf::from(format!("/home/logic/projects/raytracer/{}", image_filename)),
            PathBuf::from(format!("/home/logic/projects/raytracer/src/{}", image_filename)),
            PathBuf::from(format!("/home/logic/projects/raytracer/images/{}", image_filename)),
        ];

        for path in &paths_to_try {
            eprintln!("Trying path: {:?}", path);
            if path.exists() {
                eprintln!("Path exists, attempting to load...");
                if img.load(path) {
                    eprintln!("Successfully loaded from path: {:?}", path);
                    return img;
                }
            } else {
                eprintln!("Path does not exist: {:?}", path);
            }
        }

        eprintln!("ERROR: Could not load image file '{}' from any location", image_filename);
        eprintln!("Current working directory: {:?}", env::current_dir());
        img
    }

    pub fn load(&mut self, filename: &Path) -> bool {
        eprintln!("Attempting to load file: {:?}", filename);
        let img = match ImageReader::open(filename) {
            Ok(reader) => match reader.decode() {
                Ok(img) => {
                    eprintln!("Successfully decoded image: {}x{}", img.width(), img.height());
                    img
                },
                Err(e) => {
                    eprintln!("Image decoding error for {:?}: {}", filename, e);
                    return false;
                }
            },
            Err(e) => {
                eprintln!("File opening error for {:?}: {}", filename, e);
                return false;
            }
        };

        self.width = img.width() as i32;
        self.height = img.height() as i32;
        self.bytes_per_scanline = (self.width as usize) * self.bytes_per_pixel;

        let rgb32f_img = img.into_rgb32f();
        self.float_data = rgb32f_img.as_raw().to_vec();

        self.convert_to_bytes();
        eprintln!("Image loaded successfully: {}x{}", self.width, self.height);
        true
    }

    pub fn width(&self) -> i32 {
        if self.float_data.is_empty() { 0 } else { self.width }
    }

    pub fn height(&self) -> i32 {
        if self.float_data.is_empty() { 0 } else { self.height }
    }

    pub fn pixel_data(&self, x: i32, y: i32) -> &[u8] {
        static MAGENTA: [u8; 3] = [255, 0, 255];
        
        if self.byte_data.is_empty() {
            return &MAGENTA;
        }

        let x = self.clamp(x, 0, self.width - 1) as usize;
        let y = self.clamp(y, 0, self.height - 1) as usize;

        let start = y * self.bytes_per_scanline + x * self.bytes_per_pixel;
        &self.byte_data[start..start+3]
    }

    pub fn float_pixel_data(&self, x: i32, y: i32) -> Option<[f32; 3]> {
        if self.float_data.is_empty() {
            return None;
        }

        let x = self.clamp(x, 0, self.width - 1) as usize;
        let y = self.clamp(y, 0, self.height - 1) as usize;

        let idx = (y * self.width as usize + x) * 3;
        Some([
            self.float_data[idx],
            self.float_data[idx+1],
            self.float_data[idx+2]
        ])
    }

    fn clamp(&self, x: i32, low: i32, high: i32) -> i32 {
        x.clamp(low, high)
    }

    fn float_to_byte(value: f32) -> u8 {
        let linear = if value <= 0.04045 {
            value / 12.92
        } else {
            ((value + 0.055)/1.055).powf(2.4)
        };
        (linear.clamp(0.0, 1.0) * 255.0).round() as u8
    }

    fn convert_to_bytes(&mut self) {
        self.byte_data = self.float_data
            .chunks_exact(3)
            .flat_map(|rgb| {
                rgb.iter().map(|&v| Self::float_to_byte(v))
            })
            .collect();
    }
}
