extern crate rand;

use rand::Rng;
use vec3::Vec3;

pub struct Sample2D
{
    u: f32, v: f32
}

pub struct Sample<T>
{
    pub pdf: f32,
    pub value: T
}

pub fn thread_sample_2d() -> Sample2D
{
    let mut rng = rand::thread_rng();
    Sample2D { u: rng.gen(), v: rng.gen() }
}

// Most formulas are taken from https://people.cs.kuleuven.be/~philip.dutre/GI/TotalCompendium.pdf

// Formula 35: sampling proportional to cosinus weighted on hemisphere
pub fn sample_cosinus_hemisphere(Sample2D{u, v}: &Sample2D) -> Sample<Vec3>
{
    let phi = 2.0 * std::f32::consts::PI * u;
    let sqrt_v = v.sqrt();
    let theta = sqrt_v.acos();
    let sqrt_1_minus_v = (1.0 - v).sqrt();
    Sample{pdf: theta.cos() / std::f32::consts::PI,
	   value: Vec3::new(phi.cos() * sqrt_1_minus_v,
		     phi.sin() * sqrt_1_minus_v,
		     sqrt_v)}
}

// Basis rotation, based on: http://jcgt.org/published/0006/01/01/ Building an Orthonormal Basis, Revisited
pub fn branchless_onb(n : &Vec3) -> (Vec3, Vec3)
{
    let sign = (1.0 as f32).copysign(n.z);
    let a = -1.0 / (sign + n.z);
    let b = n.x * n.y * a;

    (Vec3::new(1.0 + sign * n.x * n.x * a, sign * b, -sign * n.x),
     Vec3::new(b, sign + n.y * n.y * a, -n.y)
     )
}

pub fn flip_normal(a : &Vec3, n : &Vec3) -> Vec3
{
    if n.dot(a) > 0.0
    {
	n.mulf(-1.0)
    }
    else
    {
	// perform a copy?
	n.mulf(1.0)
    }
}
