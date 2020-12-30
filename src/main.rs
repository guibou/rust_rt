extern crate rand;

use rand::Rng;
use std::fs::File;
use std::io::BufWriter;
use std::io::Write;

#[derive(Debug, PartialEq, Clone)]
pub struct Vec3 {
    x: f32,
    y: f32,
    z: f32,
}

#[derive(Debug, PartialEq)]
pub enum Material {
    Diffuse,
    Mirror,
    Glass,
}

impl Vec3 {
    pub fn new(x: f32, y: f32, z: f32) -> Vec3 {
        Vec3 { x: x, y: y, z: z }
    }

    pub fn add(&self, other: &Vec3) -> Vec3 {
        Vec3::new(self.x + other.x, self.y + other.y, self.z + other.z)
    }

    pub fn sub(&self, other: &Vec3) -> Vec3 {
        Vec3::new(self.x - other.x, self.y - other.y, self.z - other.z)
    }

    pub fn mul(&self, other: &Vec3) -> Vec3 {
        Vec3::new(self.x * other.x, self.y * other.y, self.z * other.z)
    }

    pub fn mulf(&self, f: f32) -> Vec3 {
        Vec3::new(self.x * f, self.y * f, self.z * f)
    }

    pub fn dot(&self, other: &Vec3) -> f32 {
        let m = self.mul(other);
        m.x + m.y + m.z
    }

    pub fn length2(&self) -> f32 {
        self.x * self.x + self.y * self.y + self.z * self.z
    }

    pub fn normalize(&self) -> Vec3 {
        self.mulf(1.0 / (self.length2().sqrt()))
    }
}

pub fn reflect(i: &Vec3, n: &Vec3) -> Vec3 {
    n.mulf(-i.dot(&n) * 2.0).add(&i)
}

#[derive(Debug)]
pub struct Ray {
    origin: Vec3,
    direction: Vec3,
}

impl Ray {
    pub fn get_p(&self, t: f32) -> Vec3 {
        self.origin.add(&self.direction.mulf(t))
    }
}

#[derive(Debug, PartialEq)]
pub struct Intersect<'a> {
    t: f32,
    sphere: &'a Sphere,
}

#[derive(Debug, PartialEq)]
pub struct Sphere {
    radius: f32,
    center: Vec3,
    emission: Vec3,
    color: Vec3,
    material: Material,
}

pub fn intersect<'a>(sphere: &'a Sphere, ray: &Ray) -> Option<Intersect<'a>> {
    let a = ray.direction.length2();
    let b = -2.0 * ray.direction.dot(&sphere.center.sub(&ray.origin));
    let c = (sphere.center.sub(&ray.origin)).length2() - sphere.radius * sphere.radius;

    let det = b * b - 4.0 * a * c;

    if det < 0. {
        None
    } else {
        let det_sqrt = det.sqrt();
        let t = (-b - det_sqrt) / (2.0 * a);

        if t >= 0. {
            Some(Intersect {
                t: t,
                sphere: sphere,
            })
        } else {
            let t2 = (-b + det_sqrt) / (2.0 * a);
            if t2 >= 0. {
                Some(Intersect {
                    t: t2,
                    sphere: sphere,
                })
            } else {
                None
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn test_basic_ops() {
        assert_eq!(
            Vec3::new(1.0, 2.0, 3.0).add(Vec3::new(4.0, 5.0, 6.0)),
            Vec3::new(5.0, 7.0, 9.0)
        );
        assert_eq!(
            Vec3::new(1.0, 2.0, 3.0).sub(Vec3::new(4.0, 5.0, 6.0)),
            Vec3::new(-3.0, -3.0, -3.0)
        );
        assert_eq!(
            Vec3::new(1.0, 2.0, 3.0).mul(Vec3::new(4.0, 5.0, 6.0)),
            Vec3::new(4.0, 10.0, 18.0)
        );
        assert_eq!(
            Vec3::new(1.0, 2.0, 3.0).mulf(10.0),
            Vec3::new(10.0, 20.0, 30.0)
        );
    }

    #[test]
    pub fn test_dot() {
        assert_eq!(Vec3::new(1.0, 2.0, 3.0).dot(Vec3::new(4.0, 5.0, 6.0)), 32.0);
    }

    #[test]
    pub fn test_length() {
        assert_eq!(Vec3::new(3.0, 2.0, 1.0).length2(), 14.0)
    }

    #[test]
    pub fn test_sphere_it_front() {
        let r = Ray {
            origin: Vec3::new(2.0, 0.0, 0.0),
            direction: Vec3::new(1.0, 0.0, 0.0),
        };
        let sphere = Sphere {
            center: Vec3::new(10.0, 0.0, 0.0),
            radius: 3.0,
        };

        assert_eq!(intersect(&sphere, &r), Some(Intersect::new(5.0)));
    }

    #[test]
    pub fn test_sphere_it_in() {
        let r = Ray {
            origin: Vec3::new(10.0, 0.0, 0.0),
            direction: Vec3::new(1.0, 0.0, 0.0),
        };
        let sphere = Sphere {
            center: Vec3::new(10.0, 0.0, 0.0),
            radius: 3.0,
        };

        assert_eq!(intersect(&sphere, &r), Some(Intersect::new(3.0)));
    }

    #[test]
    pub fn test_sphere_it_out() {
        let r = Ray {
            origin: Vec3::new(15.0, 0.0, 0.0),
            direction: Vec3::new(1.0, 0.0, 0.0),
        };
        let sphere = Sphere {
            center: Vec3::new(10.0, 0.0, 0.0),
            radius: 3.0,
        };

        assert_eq!(intersect(&sphere, &r), None);
    }
}

pub fn intersect_scene<'a>(scene: &'a Scene, ray: &Ray) -> Option<Intersect<'a>> {
    let mut res = None;
    for sphere in &scene.spheres {
        let new_it = intersect(&sphere, ray);

        res = match new_it {
            None => res,
            Some(Intersect { t, sphere: _sphere }) => match res {
                None => new_it,
                Some(Intersect {
                    t: t2,
                    sphere: _sphere2,
                }) => {
                    if t < t2 {
                        new_it
                    } else {
                        res
                    }
                }
            },
        }
    }

    res
}

pub struct Light {
    position: Vec3,
    emission: Vec3,
}

pub struct Scene {
    spheres: Vec<Sphere>,
    lights: Vec<Light>,
}

impl Scene {
    pub fn new(spheres: Vec<Sphere>, lights: Vec<Light>) -> Scene {
        Scene {
            spheres: spheres,
            lights: lights,
        }
    }
}

pub fn compute_indirect_lighting(scene: &Scene, sphere: &Sphere, p: &Vec3, depth: u32) -> Vec3 {
    let mut rng = rand::thread_rng();
    let normal_surface_norm = p.sub(&sphere.center).normalize();
    // That's not how you generate an uniform direction...
    let new_direction = Vec3::new(rng.gen(), rng.gen(), rng.gen())
        .mulf(2.0)
        .sub(&Vec3::new(1.0, 1.0, 1.0))
        .normalize();

    // rejection sampling, that's not how you are supposed to do that!
    let dot = normal_surface_norm.dot(&new_direction);

    // TODO; compute same side, not crappy dot
    if dot > 0.0 {
        compute_indirect_lighting(scene, sphere, p, depth)
    } else {
        let r = Ray {
            origin: p.add(&new_direction.mulf(0.01)),
            direction: new_direction,
        };

        sphere
            .color
            .mulf(dot.abs() * 2.0)
            .mul(&radiance(scene, &r, depth + 1))
    }
}

pub fn compute_direct_lighting(scene: &Scene, sphere: &Sphere, light: &Light, p: &Vec3) -> Vec3 {
    let light_p = &light.position;
    let sphere_to_light = light_p.sub(&p);
    let d2 = sphere_to_light.length2();
    let d = d2.sqrt();
    let sphere_to_light_norm = sphere_to_light.mulf(1.0 / d);
    let normal_surface_norm = p.sub(&sphere.center).normalize();
    let abs_dot = normal_surface_norm.dot(&sphere_to_light_norm).abs();
    let r = Ray {
        origin: p.add(&sphere_to_light_norm.mulf(0.01)),
        direction: sphere_to_light_norm,
    };
    let it = intersect_scene(scene, &r);

    let occludded = match it {
        None => false,
        Some(Intersect { t, sphere: _sphere }) => t < d,
    };

    if occludded {
        Vec3::new(0.0, 0.0, 0.0)
    } else {
        sphere
            .color
            .mulf(abs_dot / (3.14159 * d2))
            .mul(&light.emission)
    }
}

pub fn radiance(scene: &Scene, ray: &Ray, depth: u32) -> Vec3 {
    if depth > 3 {
        Vec3::new(0.0, 0.0, 0.0)
    } else {
        let it = intersect_scene(&scene, ray);

        match it {
            None => Vec3::new(0.0, 0.0, 0.0), // black if no it
            Some(Intersect { t, sphere }) => {
                let p = ray.get_p(t);
                match sphere.material {
                    Material::Diffuse => {
                        // There is only one light, that's easier
                        compute_direct_lighting(&scene, &sphere, &scene.lights[0], &p)
                            .add(&compute_indirect_lighting(&scene, &sphere, &p, depth))
                    }
                    Material::Mirror => {
                        let normal = p.sub(&sphere.center).normalize();
                        let dir = reflect(&ray.direction, &normal);
                        let r = Ray {
                            origin: p.add(&dir.mulf(0.01)),
                            direction: dir,
                        };

                        radiance(scene, &r, depth + 1)
                    }
                    Material::Glass => {
                        let normal = p.sub(&sphere.center).normalize();
                        let dir = reflect(&ray.direction, &normal);
                        let r = Ray {
                            origin: p.add(&dir.mulf(0.01)),
                            direction: dir,
                        };

                        radiance(scene, &r, depth + 1)
                    }
                }
            }
        }
    }
}

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

pub fn main() {
    let scene = Scene::new(
        vec![
            Sphere {
                radius: 1000.0,
                center: Vec3::new(1000.0 + 1.0, 40.8, 81.6),
                emission: Vec3::new(0.0, 0.0, 0.0),
                color: (Vec3::new(0.75, 0.25, 0.25)),
                material: Material::Diffuse,
            }, // Left
            Sphere {
                radius: 1000.0,
                center: Vec3::new(-1000.0 + 99.0, 40.8, 81.6),
                emission: Vec3::new(0.0, 0.0, 0.0),
                color: Vec3::new(0.25, 0.25, 0.75),
                material: Material::Diffuse,
            }, // Right
            Sphere {
                radius: 1000.0,
                center: Vec3::new(50.0, 40.8, 1000.0),
                emission: Vec3::new(0.0, 0.0, 0.0),
                color: Vec3::new(0.75, 0.75, 0.75),
                material: Material::Diffuse,
            }, // Back
            Sphere {
                radius: 1000.0,
                center: Vec3::new(50.0, 40.8, -1000.0 + 170.0),
                emission: Vec3::new(0.0, 0.0, 0.0),
                color: Vec3::new(0.0, 0.0, 0.0),
                material: Material::Diffuse,
            }, // Front
            Sphere {
                radius: 1000.0,
                center: Vec3::new(50.0, 1000.0, 81.6),
                emission: Vec3::new(0.0, 0.0, 0.0),
                color: Vec3::new(0.75, 0.75, 0.75),
                material: Material::Diffuse,
            }, // Bottom
            Sphere {
                radius: 1000.0,
                center: Vec3::new(50.0, -1000.0 + 81.6, 81.6),
                emission: Vec3::new(0.0, 0.0, 0.0),
                color: Vec3::new(0.75, 0.75, 0.75),
                material: Material::Diffuse,
            }, // Top
            Sphere {
                radius: 16.5,
                center: Vec3::new(27.0, 16.5, 47.0),
                emission: Vec3::new(0.0, 0.0, 0.0),
                color: Vec3::new(0.99, 0.0, 0.99),
                material: Material::Mirror,
            }, // Mirror
            Sphere {
                radius: 16.5,
                center: Vec3::new(73.0, 16.5, 78.0),
                emission: Vec3::new(0.0, 0.0, 0.0),
                color: Vec3::new(0.0, 0.99, 0.99),
                material: Material::Glass,
            }, // Glass

               //,Sphere {radius: 1000.0  ,center:(Vec3::new(50.0, (81.6-16.5), 81.6)),emission: ((Vec3::new(400.0, 400.0, 400.0)))   ,color:Vec3::new(0.0,0.0,0.0),material:  Material::Diffuse } // Light
        ],
        vec![Light {
            emission: Vec3::new(5000.0, 5000.0, 5000.0),
            position: Vec3::new(50.0, 81.6 - 16.4, 81.6),
        }],
    );

    let w = 768;
    let h = 768;

    let mut im = Image::new(w, h, Vec3::new(0., 0., 0.));

    for y in 0..h {
        for x in 0..w {
            let raster_x = 100. * ((x as f32) / (w as f32) - 0.5);
            let raster_x2 = 1.3 * raster_x;
            let raster_y = 100. * (((h - y) as f32) / (h as f32) - 0.5);
            let raster_y2 = 1.3 * raster_y;

            let p0 = Vec3::new(raster_x, raster_y, 150.0);
            let p1 = Vec3::new(raster_x2, raster_y2, 0.0);
            let direction = (p1.sub(&p0)).normalize();

            let ray = Ray {
                origin: p0.add(&Vec3::new(50.0, 40.0, 0.0)),
                direction: direction,
            };

            let mut color_accum = Vec3::new(0.0, 0.0, 0.0);

            for _sample in 0..10 {
                let color = radiance(&scene, &ray, 0);
                color_accum = color_accum.add(&color);
            }

            im.set_pixel(x, y, color_accum.mulf(1.0 / 11.0));
        }
    }

    im.write("output.ppm");
}
