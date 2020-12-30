extern crate rayon;

use std::fs::File;
use std::io::BufWriter;
use std::io::Write;

use image::rayon::prelude::*;
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
    pub fn new<F>(width: i32, height: i32, f: F) -> Image
    where
        F: Fn(i32, i32) -> Vec3 + Sync,
    {
        Image {
            width: width,
            height: height,
            pixels: (0..(height * width))
                .into_par_iter()
                .map(|idx| {
                    let line = idx / height;
                    let col = idx % height;
                    f(line, col)
                })
                .collect(),
        }
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
