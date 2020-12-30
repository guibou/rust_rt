extern crate rand;

mod image;
mod sampling;
mod vec3;

use image::Image;
use vec3::Vec3;

#[derive(Debug, PartialEq)]
pub enum Material {
    Diffuse,
    Mirror,
    Glass,
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

pub fn compute_indirect_lighting(
    ray_dir: &Vec3,
    scene: &Scene,
    sphere: &Sphere,
    p: &Vec3,
    depth: u32,
) -> Vec3 {
    let normal_surface_norm = sampling::flip_normal(ray_dir, &p.sub(&sphere.center).normalize());

    let sampling::Sample {
        pdf: _pdf,
        value: cos_sampled_dir,
    } = sampling::sample_cosinus_hemisphere(&sampling::thread_sample_2d());
    let (b1, b2) = sampling::branchless_onb(&normal_surface_norm);
    let new_direction = b1
        .mulf(cos_sampled_dir.x)
        .add(&b2.mulf(cos_sampled_dir.y))
        .add(&normal_surface_norm.mulf(cos_sampled_dir.z));

    let r = Ray {
        origin: p.add(&new_direction.mulf(0.01)),
        direction: new_direction,
    };

    sphere.color.mul(&radiance(scene, &r, depth + 1))
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
                        compute_direct_lighting(&scene, &sphere, &scene.lights[0], &p).add(
                            &compute_indirect_lighting(&ray.direction, &scene, &sphere, &p, depth),
                        )
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

    let im = Image::new(w, h, |y: i32, x: i32| {
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

        let color = (0..10).fold(Vec3::new(0.0, 0.0, 0.0), |sum, _x| {
            sum.add(&radiance(&scene, &ray, 0))
        });
        color.mulf(1.0 / 10.0)
    });

    im.write("output.ppm");
}
