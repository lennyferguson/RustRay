/// Author: Stewart Charles
/// Rust RayTracer
/// Version 0.6
/// Date: 06/Oct/2016

#[macro_use]
extern crate lazy_static;
extern crate image;
extern crate nalgebra as na;
extern crate rand;
extern crate rayon;
extern crate time;

use na::{Norm, Vec3};
//use std::env;
//use std::str::FromStr;
use image::ImageBuffer;
use rand::distributions::{Range, Sample};
use rayon::prelude::*;
use std::{fs::File, process::Command, thread};
// use getopts::{optopt,optflag,getopts,OptGroup};

// Define various constants used throughout the program

const MAX_DEPTH: i32 = 10;
const NEAR: f32 = 1.0;
const EPSILON: f32 = 1.0 / 10000.0;

// Current version of program super-samples to reduce aliasing
// so the effective DIM of the final image will be DIM / 2
const DIM: i32 = 2000;
const HALFDIM: i32 = DIM / 2;
const T0: f32 = 0.0;
const T1: f32 = 100000.0;
const FLIP: usize = (HALFDIM - 1) as usize;

const UP: Vec3<f32> = Vec3 {
    x: 0.0f32,
    y: 1.0f32,
    z: 0.0f32,
};

// Define Materials
const BLUE: Material = Material {
    amb: Vec3 {
        x: 0.1,
        y: 0.1,
        z: 0.85,
    },
    reflect: 0.0,
};
const GREEN: Material = Material {
    amb: Vec3 {
        x: 0.1,
        y: 0.85,
        z: 0.1,
    },
    reflect: 0.675,
};
const RED: Material = Material {
    amb: Vec3 {
        x: 0.85,
        y: 0.1,
        z: 0.1,
    },
    reflect: 0.0,
};
const MIRROR: Material = Material {
    amb: Vec3 {
        x: 0.15,
        y: 0.15,
        z: 0.15,
    },
    reflect: 0.9,
};
const FLOOR_MAT: Material = Material {
    amb: Vec3 {
        x: 0.25,
        y: 0.56725,
        z: 0.20725,
    },
    reflect: 0.085,
};
const BRASS: Material = Material {
    amb: Vec3 {
        x: 0.329412,
        y: 0.223529,
        z: 0.027451,
    },
    reflect: 0.0,
};

// This is the default background color
const BKG_COLOR: Vec3<f32> = Vec3 {
    x: 0.4f32,
    y: 0.698f32,
    z: 1.0f32,
};

const SPHERE_RADIUS :f32 = 1.0f32;
const SPHERE_LOCATION: Vec3<f32> = na::Vec3{x: 3.5, y: SPHERE_RADIUS, z: 3.5};

// Define Vertices for various Surfaces
const FLOOR_VERTS: [Vec3<f32>; 4] = [
    Vec3 {
        x: -10.0,
        y: 0.0,
        z: -10.0,
    },
    Vec3 {
        x: -10.0,
        y: 0.0,
        z: 10.0,
    },
    Vec3 {
        x: 10.0,
        y: 0.0,
        z: 10.0,
    },
    Vec3 {
        x: 10.0,
        y: 0.0,
        z: -10.0,
    },
];

const CUBE: [Vec3<f32>; 8] = [
    Vec3 {
        x: 1.0,
        y: 0.0,
        z: 1.5,
    }, // 0
    Vec3 {
        x: 1.0,
        y: 0.0,
        z: 0.5,
    }, // 1
    Vec3 {
        x: 2.0,
        y: 0.0,
        z: 0.5,
    }, // 2
    Vec3 {
        x: 2.0,
        y: 0.0,
        z: 1.5,
    }, // 3
    Vec3 {
        x: 1.0,
        y: 1.0,
        z: 1.5,
    }, // 4
    Vec3 {
        x: 1.0,
        y: 1.0,
        z: 0.5,
    }, // 5
    Vec3 {
        x: 2.0,
        y: 1.0,
        z: 0.5,
    }, // 6
    Vec3 {
        x: 2.0,
        y: 1.0,
        z: 1.5,
    },
]; // 7

//const LIGHT_POS:Vec3<f32> = Vec3{x:-25f32,y:40.0f32,z:-25f32};
const LIGHT_RADIUS: f32 = 10f32;
const SHADOW_SAMPLES: i32 = 50;

// Initialize Values that are not compile time constants that are still
// static relative to the scene / renderer
lazy_static! {
    // -----Setup Surfaces-----
    static ref SURFACES:Vec<Box<Surface>> = vec!(
        // Add Snowman
        Sphere::boxed(Vec3::new(0.0,0.5,3.0), 1.0, BLUE),
        Sphere::boxed(Vec3::new(0.0,1.85,3.0), 0.75, GREEN),
        Sphere::boxed(Vec3::new(0.0,2.65,3.0), 0.50, RED),

        // Add Mirror Sphere
        Sphere::boxed(SPHERE_LOCATION, SPHERE_RADIUS, MIRROR),

        // Add floor with pattern value set to true
        Triangle::boxed(FLOOR_VERTS[0], FLOOR_VERTS[1], FLOOR_VERTS[2], FLOOR_MAT, true),
        Triangle::boxed(FLOOR_VERTS[0], FLOOR_VERTS[2], FLOOR_VERTS[3], FLOOR_MAT, true),

        // Add brass cube triangles
        Triangle::boxed(CUBE[0],CUBE[5],CUBE[1], BRASS, false),
        Triangle::boxed(CUBE[0],CUBE[4],CUBE[5], BRASS, false),
        Triangle::boxed(CUBE[0],CUBE[4],CUBE[3], BRASS, false),
        Triangle::boxed(CUBE[4],CUBE[7],CUBE[3], BRASS, false),
        Triangle::boxed(CUBE[4],CUBE[7],CUBE[5], BRASS, false),
        Triangle::boxed(CUBE[5],CUBE[7],CUBE[6], BRASS, false),
        Triangle::boxed(CUBE[5],CUBE[2],CUBE[6], BRASS, false),
        Triangle::boxed(CUBE[5],CUBE[1],CUBE[2], BRASS, false),
        Triangle::boxed(CUBE[6],CUBE[7],CUBE[3], BRASS, false),
        Triangle::boxed(CUBE[6],CUBE[3],CUBE[2], BRASS, false)
    );
}

fn main() {
    /*
    let args: Vec<String> = env::args().collect();
    let mut unwrap:Vec<f32> = Vec::new();
    let mut error = false;
    for x in 1..args.len() {
        let parse = f32::from_str(&args[x]);
        match parse {
            Ok(num) => unwrap.push(num),
            Err(e) => {
                println!("Error Parsing Arg[{}] = '{}' , Err = '{}'", x , args[x], e );
                error = true; },
        }
        3 => {
            eye = Vec3::new(args[0],args[1],args[2]);
        }
        _ => { /* Use Default EYE and LOOK parameters */  }
    }
    */

    /*
    Render sets up Piston Window. Calls RayTracer
    Render function. Piston App Render merely renders
    the Image array generated by the Ray Tracer.
     */

    // TODO: Parameterize animation properties from command line args
    // Retrieve EYE and LOOKAT positions from commandline args
    // if they exist. Otherwise, default to initial values
    let look = SPHERE_LOCATION;
    let light = Vec3::new(-10f32, 15f32, -35.5f32);
    let d = SPHERE_RADIUS + 0.75f32;
    let h = SPHERE_LOCATION.y - 0.25f32;
    let time = 300;

    let max = 2.0f32 * std::f32::consts::PI;
    let init = Vec3::new(look.x, h, look.z);
    for dt in 0..time {
        let theta = (dt as f32) * (max / (time as f32));
        let x = theta.sin() * d;
        let z = theta.cos() * d;
        let eye = init + Vec3::new(x, 0f32, z);
        // Save the image buffer to a file
        let ref mut fout = File::create(format!("render{:04}.png", dt)).unwrap();
        render(eye, look, light).save(fout, image::PNG).unwrap();
    }

    // Export .png images to an animation [mp4 or mkv]
    let ffmpeg_shell = "ffmpeg -framerate 60 -pattern_type glob -i '*.png' -c:v libx264 -preset veryslow -crf 0 movie.mkv";
    let make_movie = Command::new("sh")
        .arg("-c")
        .arg(ffmpeg_shell)
        .output()
        .expect("failed to execute process");
    make_movie.stdout;
}

fn calculate_viewray(x: i32, y: i32, view_ray: ViewRay) -> Ray {
    let us = -1.0 + view_ray.img_dim * ((x as f32) + 0.5);
    let vs = -1.0 + view_ray.img_dim * ((y as f32) + 0.5);
    let mut s = view_ray.eye + view_ray.u * us;
    s = s + view_ray.v * vs;
    s = s + view_ray.w * NEAR;
    Ray {
        src: view_ray.eye,
        dir: (s - view_ray.eye).normalize(),
    } // -> Return Ray
}

fn render(eye: Vec3<f32>, look: Vec3<f32>, light_pos: Vec3<f32>) -> image::DynamicImage {
    /* Begin Render Loop for Image
    Init Vec containing Surfaces
    Surface is a trait, which means that we must Box the
    Structs that impl Surface to properly store them */

    /* U,V,W basis Vecs are now captured from the environment of the calling
    thread and 'moved' to the closure 'calculate_viewray' */
    let eye_at = (eye - look).normalize();
    let u = na::cross(&eye_at, &UP).normalize();
    let v = na::cross(&u, &eye_at).normalize();
    let w = na::cross(&u, &v).normalize();

    let img_dim = 2.0 / (DIM as f32);

    let viewray_data = ViewRay {
        img_dim: img_dim,
        eye: eye,
        u: u,
        v: v,
        w: w,
    };

    let start = time::precise_time_s();

    // Run Threads that operate on disjoint image Quads
    let a_thread = thread::spawn(move || {
        thread_render(
            light_pos,
            viewray_data,
            0,
            HALFDIM as i32,
            0,
            HALFDIM as i32,
        )
    });

    let b_thread = thread::spawn(move || {
        thread_render(
            light_pos,
            viewray_data,
            HALFDIM as i32,
            DIM,
            0,
            HALFDIM as i32,
        )
    });

    let c_thread = thread::spawn(move || {
        thread_render(
            light_pos,
            viewray_data,
            0,
            HALFDIM as i32,
            HALFDIM as i32,
            DIM + 1,
        )
    });

    let d_thread = thread::spawn(move || {
        thread_render(
            light_pos,
            viewray_data,
            HALFDIM as i32,
            DIM,
            HALFDIM as i32,
            DIM + 1,
        )
    });

    // Join Threads before displaying Image
    let quads = vec![
        a_thread.join().unwrap(),
        b_thread.join().unwrap(),
        c_thread.join().unwrap(),
        d_thread.join().unwrap(),
    ];

    let end = time::precise_time_s() - start;
    println!("Rendering Time: {} Seconds", end);

    // Combine each of the image quadrants into a single image while also scaling the image to half size.
    let mut ans = vec![vec![Vec3::new(0f32, 0f32, 0f32); HALFDIM as usize]; HALFDIM as usize];
    for quad in quads.iter() {
        let avg_color = move |index| {
            let mut avg = Vec3::new(0.0, 0.0, 0.0);
            avg = avg + quad.img[index];
            avg = avg + quad.img[index + 1];
            avg = avg + quad.img[index + HALFDIM as usize];
            avg = avg + quad.img[index + HALFDIM as usize + 1];
            avg = avg / 4.0;
            avg
        };
        let mut index = 0;
        // We wish to map pixels from the expanded image space
        // to a reduced pixel space so we iterate over the
        // reduced pixel space and collect samples from the expanded space
        for y in (quad.ymin / 2)..(quad.ymax / 2) {
            for x in (quad.xmin / 2)..(quad.xmax / 2) {
                ans[x as usize][y as usize] = avg_color(index);
                index += 2;
            }
            // Translate index by the offset of the row size in the expanded space
            index += HALFDIM as usize;
        }
    }
    // Create an image buffer from the pixel vector
    let buf = ImageBuffer::from_fn(HALFDIM as u32, HALFDIM as u32, |x, y| {
        let color = ans[x as usize][FLIP - y as usize];
        image::Rgb([
            (color.x * 255f32) as u8,
            (color.y * 255f32) as u8,
            (color.z * 255f32) as u8,
        ])
    });
    image::ImageRgb8(buf)
}

/* This function will be used by a thread to Generate a section of the
image being drawn. */
fn thread_render(
    light_pos: Vec3<f32>,
    viewray_data: ViewRay,
    xmin: i32,
    xmax: i32,
    ymin: i32,
    ymax: i32,
) -> ImageQuad {
    let mut image = ImageQuad::new(xmin, xmax, ymin, ymax);
    // Iterate through the the pixels in our Image Plane
    let mut v = Vec::new();
    for y in ymin..ymax {
        for x in xmin..xmax {
            v.push((x, y));
        }
    }
    image.img = v
        .par_iter()
        .map(|&(x, y)| {
            /* Generate the View Ray for 'this' pixel.
            Makes use of UVW basis vecs captured in the 'closure'
            of the lambda function No need for 'unsafe'
            static mut 'global' variables */
            let view_ray = calculate_viewray(x, y, viewray_data);

            // For each Surface, test for intersection with View Ray
            // Track surface nearest to Viewer with near_t scalar
            let mut near_surf: Option<&Box<Surface>> = None;
            let mut near_t = T1;

            for s in SURFACES.iter() {
                if let Some(t) = s.hit(&view_ray) {
                    if t < near_t {
                        near_t = t;
                        near_surf = Some(s);
                    }
                }
            }
            match near_surf {
                Some(surf) => surf.calculate_color(&view_ray, light_pos, near_t, MAX_DEPTH),
                None => BKG_COLOR,
            }
        })
        .collect();
    image
}

/// For the given point, calculates if the point is shaded
/// and returns true if in shadow, false otherwise.
/// Requires access the Vec containing the scenes Surfaces.
fn shadow(point: Vec3<f32>, light_pos: Vec3<f32>) -> f32 {
    let mut count = 0;
    let mut rng = rand::thread_rng();
    let mut range = Range::new(-LIGHT_RADIUS, LIGHT_RADIUS);
    for _ in 0..SHADOW_SAMPLES {
        let light_loc = light_pos
            + Vec3::new(
                range.sample(&mut rng),
                range.sample(&mut rng),
                range.sample(&mut rng),
            );
        let light_dir = (light_loc - point).normalize();
        let light_ray = Ray {
            src: point,
            dir: light_dir,
        };
        for s in SURFACES.iter() {
            if let Some(_) = s.hit(&light_ray) {
                count += 1;
                break;
            }
        }
    }
    (count as f32) / (SHADOW_SAMPLES as f32)
}

/// Casts a Reflection ray from the 'Point' in a direction that is calculated from the incoming
/// view_dir and surface normal. If the maximum depth has been reached in computing rays, returns
/// the background color for the scene.
fn reflect(
    point: Vec3<f32>,
    view_dir: Vec3<f32>,
    normal: Vec3<f32>,
    light_pos: Vec3<f32>,
    depth: i32,
) -> Vec3<f32> {
    if depth == 0 {
        return BKG_COLOR;
    }

    // Calculate Reflection Ray
    let dot_n = 2.0 * na::dot(&view_dir, &normal);
    let dir = normal * dot_n;
    let ray = Ray {
        src: point,
        dir: (view_dir - dir).normalize(),
    };

    // Rest of function is largely similar to intersection test
    // in thread_render()
    let mut near_surf: Option<&Box<Surface>> = None;
    let mut current = T1;
    for surf in SURFACES.iter() {
        if let Some(t) = surf.hit(&ray) {
            if t < current {
                current = t;
                near_surf = Some(surf);
            }
        }
    }
    match near_surf {
        Some(surf) => surf.calculate_color(&ray, light_pos, current, depth),
        None => BKG_COLOR,
    }
}

// Setup Data Structures used by the Ray Tracer

/// Material currently only consists of the ambient term in the
/// BDRF model, and the reflection coeff. Can be modified to include
/// the Specular, Diffuse, and Shininess terms if necessary.
/// (calculate_color function must be modified to use terms if so)
#[derive(Copy, Clone)]
struct Material {
    amb: Vec3<f32>,
    reflect: f32,
}

/// Structure that wraps an ImageQuad that is rended concurrently by one of 4 Threads.
/// Contains all necessary data to be able to properly place the final pixel in the window.
#[derive(Clone)]
struct ImageQuad {
    xmin: i32,
    xmax: i32,
    ymin: i32,
    ymax: i32,
    img: Vec<Vec3<f32>>,
}

impl ImageQuad {
    fn new(_xmin: i32, _xmax: i32, _ymin: i32, _ymax: i32) -> ImageQuad {
        ImageQuad {
            xmin: _xmin,
            xmax: _xmax,
            ymin: _ymin,
            ymax: _ymax,
            img: Vec::new(),
        }
    }
}

/// Simple Container for a Ray
/// Composed of ray position as Vec3
/// and and direction as Vec3
#[derive(Copy, Clone)]
struct Ray {
    src: Vec3<f32>,
    dir: Vec3<f32>,
}

/// Trait for Surface type that can calculate a Ray Surface intersection
/// and also calculates the Color for the point intersected on the Surface.
trait Surface: Sync + Send {
    fn hit(&self, ray: &Ray) -> Option<f32>;
    fn calculate_color(&self, ray: &Ray, light_pos: Vec3<f32>, t: f32, depth: i32) -> Vec3<f32>;
}

/// Datatype that contains sufficient information to
/// calculate view ray from view plane
#[derive(Copy, Clone)]
struct ViewRay {
    img_dim: f32,
    eye: Vec3<f32>,
    u: Vec3<f32>,
    v: Vec3<f32>,
    w: Vec3<f32>,
}

/// Datatype for representing Sphere scene objects
/// Contains location, radius (squared) and material
#[derive(Copy, Clone)]
struct Sphere {
    center: Vec3<f32>,
    radius_sqr: f32,
    material: Material,
}

impl Sphere {
    fn new(c: Vec3<f32>, r: f32, mat: Material) -> Sphere {
        Sphere {
            center: c,
            radius_sqr: r * r,
            material: mat,
        }
    }

    fn quadratic(&self, a: f32, b: f32, disc: f32) -> Option<f32> {
        let p = (-b + disc.sqrt()) / (2.0 * a);
        let q = (-b - disc.sqrt()) / (2.0 * a);
        self.nearest(p, q)
    }

    fn nearest(&self, p: f32, q: f32) -> Option<f32> {
        let p_bound = p > T0 && p < T1;
        let q_bound = q > T0 && q < T1;
        if p_bound && q_bound {
            if p > q {
                Some(q)
            } else {
                Some(p)
            }
        } else if p_bound {
            Some(p)
        } else if q_bound {
            Some(q)
        } else {
            None
        }
    }

    fn boxed(c: Vec3<f32>, r: f32, mat: Material) -> Box<Sphere> {
        Box::new(Sphere::new(c, r, mat))
    }
}

impl Surface for Sphere {
    /* Solving for 't' for Ray: src + dir * t
    s.t. Ray intersects Sphere.
    Sphere is intersected by ray if t is real
    Returns Some(t) only if bounded by T0 && T1 */
    fn hit(&self, ray: &Ray) -> Option<f32> {
        let e_minus_c = ray.src - self.center;
        let a = na::dot(&ray.dir, &ray.dir);
        let b = 2.0 * na::dot(&ray.dir, &e_minus_c);
        let c = na::dot(&e_minus_c, &e_minus_c) - self.radius_sqr;
        let disc = (b * b) - (4.0 * a * c);
        if disc < 0.0 {
            None
        } else {
            self.quadratic(a, b, disc)
        }
    }

    fn calculate_color(&self, ray: &Ray, light_pos: Vec3<f32>, t: f32, depth: i32) -> Vec3<f32> {
        if depth == 0 {
            return self.material.amb;
        }
        let dir_ammt = ray.dir * (t - EPSILON);
        let point = ray.src + dir_ammt;
        let normal = (point - self.center).normalize();
        let in_shadow = shadow(point, light_pos);
        let mut mat = self.material.amb;

        // Compute Diffuse Component of BRDF
        let light_dir = (light_pos - point).normalize();
        let mut max = largest_of(na::dot(&normal, &light_dir));
        mat = mat + Vec3::new(0.35, 0.35, 0.35) * max;

        // Add Specular Contribution of BRDF
        // Compute Halfway Vector
        let negative_dir = ray.dir * -1.0;
        let h = light_dir + negative_dir;
        max = largest_of(na::dot(&normal, &h));
        max.powf(1.2);
        mat = mat + Vec3::new(0.35f32, 0.35f32, 0.35f32) * max;

        mat = mat * (1.0f32 - in_shadow) * 0.5;
        // Apply Shadow if necessary

        // Cast Secondary Ray if Reflective index > 0.0
        if self.material.reflect > 0.0 {
            mix(
                mat,
                reflect(point, ray.dir, normal, light_pos, depth - 1),
                self.material.reflect,
            )
        } else {
            mat
        }
    }
}

/// Datatype for triangle's in a scene. Contains information

#[derive(Copy, Clone)]
struct Triangle {
    a: Vec3<f32>,
    b: Vec3<f32>,
    c: Vec3<f32>,
    normal: Vec3<f32>,
    material: Material,
    pattern: bool,
}

impl Triangle {
    fn new(_a: Vec3<f32>, _b: Vec3<f32>, _c: Vec3<f32>, mat: Material, p: bool) -> Triangle {
        let a_b = _a - _b;
        let a_c = _a - _c;
        let n = na::cross(&a_b, &a_c).normalize();
        Triangle {
            a: _a,
            b: _b,
            c: _c,
            normal: n,
            material: mat,
            pattern: p,
        }
    }
    fn boxed(_a: Vec3<f32>, _b: Vec3<f32>, _c: Vec3<f32>, mat: Material, p: bool) -> Box<Triangle> {
        Box::new(Triangle::new(_a, _b, _c, mat, p))
    }
}

impl Surface for Triangle {
    /// Calculates Ray intersection of Triangle by utilizing
    /// Shirley's Ray Intersection formula that defines plane
    /// of points A,B,C in Triangle and tests if Barycentric coords
    /// are restricted to Triangle. Solve for Barycentric coords
    /// using Cramers rule of M * [Beta, Gamma, t] = [A - Src]
    /// to solve for [Beta,Gamma,t]
    fn hit(&self, ray: &Ray) -> Option<f32> {
        /*
        let edge1 = self.b - self.a;
        let edge2 = self.c - self.a;
        let p = na::cross(&ray.dir, &edge2);
        let det = na::dot(&edge1, &p);
        if det > -EPSILON && det < EPSILON {return None;}
        let inv_det = 1.0f32 / det;

        let tvec = ray.src - self.a;
        let u = na::dot(&tvec,&p) * inv_det;
        if u < 0.0f32 || u > 1.0f32 { return None; }

        let q = na::cross(&tvec,&edge1);
        let v = na::dot(&ray.dir, &q);
        if v < 0.0f32 || u + v > 1.0f32 { return None; }

        let t = na::dot(&edge2,&q) * inv_det;
        if t > 0.000001f32 {
            Some(t)
        } else { None }
        */

        // Why not create transformation matrix?
        // Init Matrix M vals
        let a = self.a.x - self.b.x;
        let b = self.a.x - self.c.x;
        let c = ray.dir.x;
        let d = self.a.y - self.b.y;
        let e = self.a.y - self.c.y;
        let f = ray.dir.y;
        let g = self.a.z - self.b.z;
        let h = self.a.z - self.c.z;
        let i = ray.dir.z;
        let j = self.a.x - ray.src.x;
        let k = self.a.y - ray.src.y;
        let l = self.a.z - ray.src.z;

        let dheg = d * h - e * g;
        let eihf = e * i - h * f;
        let kilf = k * i - l * f;
        let digf = d * i - f * g;
        let dlgk = d * l - g * k;
        let elhk = e * l - h * k;

        // Solve for Determinant of M
        //A * (E*I – H*F) – B*(D*I – G*F) + C*(D*H – E*G)
        let det_m = a * eihf - b * digf + c * dheg;

        // Solve for 't'
        // A * (E*L – H*K) – B*(D*L – G*K) + J*(D*H – E*G)
        let mut t = a * elhk - b * dlgk + j * dheg;
        t /= det_m;
        if t < T0 || t > T1 {
            return None;
        }

        // Solve for 'gamma'
        // A * (K * I – L * F) – J * (D * I – F * G) + C* (D*L – K * G)
        let mut gamma = a * kilf - j * digf + c * dlgk;
        gamma /= det_m;
        if gamma < 0.0 || gamma > 1.0 {
            return None;
        }

        // Solve for 'beta'
        // J * ( E * I – H * F) – B * (K * I – l * F) + C * (K*H – L*E)
        let mut beta = j * eihf - b * kilf - c * elhk;
        beta /= det_m;
        if beta < 0.0 || beta > (1.0 - gamma) {
            None
        } else {
            Some(t)
        }
    }

    fn calculate_color(&self, ray: &Ray, light_pos: Vec3<f32>, t: f32, depth: i32) -> Vec3<f32> {
        if depth == 0 {
            return self.material.amb;
        }
        let dir_ammt = ray.dir * (t - EPSILON);
        let point = ray.src + dir_ammt;
        let normal = self.normal;
        let in_shadow = shadow(point, light_pos);
        let mut mat = self.material.amb;

        // Set Checkerboard pattern
        if self.pattern {
            let tx = (point.x + 10.0) % 2.0;
            let ty = (point.z + 10.0) % 2.0;
            if tx < 1.0 && ty < 1.0 || tx > 1.0 && ty > 1.0 {
                mat = mat - Vec3::new(0.2, 0.2, 0.2);
            }
        }

        // Compute Diffuse Component of BRDF
        let light_dir = (light_pos - point).normalize();
        let mut max = largest_of(na::dot(&normal, &light_dir));
        mat = mat + Vec3::new(0.25, 0.25, 0.25) * max;

        // Add Specular Contribution of BRDF
        // Compute Halfway Vector
        let h = light_dir + ray.dir * -1.0;
        max = largest_of(na::dot(&normal, &h));
        max.powf(1.2);
        mat = mat + Vec3::new(0.3f32, 0.3f32, 0.3f32) * max;

        // Apply Shadow if necessary
        mat = mat * 0.5 * (1.0f32 - in_shadow);

        // Cast Secondary Ray if Reflective index > 0.0
        if self.material.reflect > 0.0 {
            mix(
                mat,
                reflect(point, ray.dir, normal, light_pos, depth - 1),
                self.material.reflect,
            )
        } else {
            mat
        }
    }
}

// --- Helper Functions ---
/// Mix function is used to blend between two colors
fn mix(color_a: Vec3<f32>, color_b: Vec3<f32>, alpha: f32) -> Vec3<f32> {
    Vec3::new(
        (1.0 - alpha) * color_a[0] + alpha * color_b[0],
        (1.0 - alpha) * color_a[1] + alpha * color_b[1],
        (1.0 - alpha) * color_a[2] + alpha * color_b[2],
    )
}

fn largest_of(num: f32) -> f32 {
    return if num < 0.0 { 0.0 as f32 } else { num as f32 };
}
