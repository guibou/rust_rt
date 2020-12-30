use std::fs::File;
use std::io::BufWriter;
use std::io::Write;

use vec3::Vec3;

pub fn tonemap(x: f32) -> i32 {
    if x <= 0.0 {
        0
    } else if x >= 1.0 {
        255
    } else {
        // Gamma correction
        (x.powf(1.0 / 2.2) * 255.0) as i32
    }
}

pub struct Image {
    width: i32,
    height: i32,

    pixels: Vec<Vec3>,
}

impl Image {
    pub fn new(width: i32, height: i32, default: Vec3) -> Image {
        let mut v = Vec::new();

        v.resize((width * height) as usize, default);

        Image {
            width: width,
            height: height,
            pixels: v,
        }
    }

    pub fn set_pixel(&mut self, x: i32, y: i32, v: Vec3) {
        self.pixels[(x + y * self.width) as usize] = v
    }

    pub fn write(&self, filepath: &str) {
        let f = File::create(filepath).unwrap();
        let mut buf = BufWriter::new(f);
        write!(buf, "P3\n{:?} {:?}\n255\n", self.width, self.height).unwrap();

        for y in 0..self.height {
            for x in 0..self.width {
                let color = &self.pixels[(x + y * self.width) as usize];

                write!(
                    buf,
                    "{:?} {:?} {:?} ",
                    tonemap(color.x),
                    tonemap(color.y),
                    tonemap(color.z)
                )
                .unwrap();
            }
        }
    }
}
