/// Author: Stewart Charles
/// Rust RayTracer
/// Version 0.5
/// Date: 09/Jan/2016

extern crate nalgebra as na;
extern crate piston_window;
extern crate time;
//extern crate getopts;

use na::{Vec3,Norm};
use std::env;
use std::str::FromStr;
use std::thread;
use std::sync::{Arc};
use piston_window::*;
//use getopts::{optopt,optflag,getopts,OptGroup};

const MAX_DEPTH:i32 = 5;
const NEAR:f32 = 1.2;
const EPSILON:f32 = 1.0 / 10000.0;
const DIM:i32 = 800;
const T0:f32 = 0.0;
const T1:f32 = 100000.0;

const BKG_COLOR:Vec3<f32> = Vec3{x:0.4f32,y:0.698f32,z:1.0f32};
const UP:Vec3<f32> = Vec3{x:0.0f32,y:1.0f32,z:0.0f32};
const LIGHT_POS:Vec3<f32> = Vec3{x:25.0f32,y:25.0f32,z:-10.0f32};

/* Type Definition of Arc container of Vector of Boxes containing Surface Sructs.
   This type is used throughout the program and is quite verbose, so we will redefine
   it with a more succint name. */
type Surfaces = Arc<Vec<Box<Surface>>>;

fn main() {
    /*
    Render sets up Piston Window. Calls RayTracer
    Render function. Piston App Render merely renders
    the Image array generated by the Ray Tracer.
     */

    // Retrieve EYE and LOOKAT positions from commandline args
    // if they exist. Otherwise, default to initial values
    let mut eye = Vec3::new(0.0f32,2.5f32,-1.0f32);
    let mut look = Vec3::new(1.0f32, 1.0f32, 3.0f32);

    let args:Vec<String> = env::args().collect();

    //Begining work setting up getopts argument inputs
    /*
    let opts = [
    optopt("e", "eye","Sets the Camera Origin (i.e. the Eye)", "EYE"),
    optopt("a", "at", "Sets the position of what the camera looks AT", "AT"),
    optopt("d", "dim", "Set the Output Window X & Y dim. Allows range of [100:4000]", "DIM"),
    optopt("t", "thread", "Set # of Child Threads. Allows range of [1 : 16]", "THREAD"),
    optopt("h", "help", "Print RustRay opts", "HELP"),
    ];*/

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
    }
    if !error {
        match unwrap.len() {
            6 => {
                eye = Vec3::new(unwrap[0], unwrap[1], unwrap[2]);
                look = Vec3::new(unwrap[3], unwrap[4], unwrap[5]);
            }
            3 => {
                eye = Vec3::new(unwrap[0], unwrap[1], unwrap[2]);
            }
            _ => { /* Use Default EYE and LOOK parameters */  }
        }
    }
    render(eye, look);
}

fn render(eye:Vec3<f32>, look:Vec3<f32>) {
    /* Begin Render Loop for Image
       Init Vec containing Surfaces
       Surface is a trait, which means that we must Box the
       Structs that impl Surface to properly store them */

    // Setup Materials
    let blue = Material{amb:Vec3::new(0.0,0.0,1.0), reflect:0.0};
    let green = Material{amb:Vec3::new(0.0,1.0,0.0), reflect:0.35};
    let red = Material{amb:Vec3::new(1.0,0.0,0.0), reflect:0.0};
    let mirror = Material{amb:Vec3::new(0.15,0.15,0.15), reflect:0.9};
    let floor_mat = Material{amb:Vec3::new(0.25,0.56725, 0.20725), reflect:0.085};
    let brass = Material{amb:Vec3::new(0.329412, 0.223529, 0.027451), reflect:0.0 };

    // Setup Verts
    let floor_verts:[Vec3<f32>;4] = [
        Vec3::new(-10.0,0.0,-10.0),
        Vec3::new(-10.0,0.0,10.0),
        Vec3::new(10.0,0.0,10.0),
        Vec3::new(10.0,0.0,-10.0)
    ];

    let cube:[Vec3<f32>;8] = [
        Vec3::new(1.0,0.0,1.5), // 0
        Vec3::new(1.0,0.0,0.5), // 1
        Vec3::new(2.0,0.0,0.5), // 2
        Vec3::new(2.0,0.0,1.5), // 3
        Vec3::new(1.0,1.0,1.5), // 4
        Vec3::new(1.0,1.0,0.5), // 5
        Vec3::new(2.0,1.0,0.5), // 6
        Vec3::new(2.0,1.0,1.5)  // 7
    ];
    
    // -----Setup Surfaces-----

    let surfaces:Vec<Box<Surface>> = vec!(
        // Add Snowman
        Sphere::boxed(Vec3::new(0.0,0.5,3.0), 1.0, blue),
        Sphere::boxed(Vec3::new(0.0,1.85,3.0), 0.75, green),
        Sphere::boxed(Vec3::new(0.0,2.65,3.0), 0.55, red),

        // Add Mirror Sphere
        Sphere::boxed(Vec3::new(3.5,1.0,3.5),1.0, mirror),

        // Add floor with pattern value set to true
        Triangle::boxed(floor_verts[0], floor_verts[1], floor_verts[2], floor_mat, true),
        Triangle::boxed(floor_verts[0], floor_verts[2], floor_verts[3], floor_mat, true),

        // Add brass cube triangles
        Triangle::boxed(cube[0],cube[5],cube[1], brass, false),
        Triangle::boxed(cube[0],cube[4],cube[5], brass, false),
        Triangle::boxed(cube[0],cube[4],cube[3], brass, false),
        Triangle::boxed(cube[4],cube[7],cube[3], brass, false),
        Triangle::boxed(cube[4],cube[7],cube[5], brass, false),
        Triangle::boxed(cube[5],cube[7],cube[6], brass, false),
        Triangle::boxed(cube[5],cube[2],cube[6], brass, false),
        Triangle::boxed(cube[5],cube[1],cube[2], brass, false),
        Triangle::boxed(cube[6],cube[7],cube[3], brass, false),
        Triangle::boxed(cube[6],cube[3],cube[2], brass, false)
    );

    // Create Arc Shared Ref to Surface Vec
    let s_copy = Arc::new(surfaces);
    let s_a = s_copy.clone();
    let s_b = s_copy.clone();
    let s_c = s_copy.clone();
    let s_d = s_copy.clone();

    /* U,V,W basis Vecs are now captured from the environment of the calling
       thread and 'moved' to the closure 'calculate_viewray' */
    let eye_at = (eye - look).normalize();
    let u = na::cross(&eye_at, &UP).normalize();
    let v = na::cross(&u, &eye_at).normalize();
    let w = na::cross(&u, &v).normalize();

    let img_dim = 2.0 / (DIM as f32);

    let calculate_viewray = move |x, y| {
        let us = -1.0 + img_dim * ((x as f32) + 0.5);
        let vs = -1.0 + img_dim * ((y as f32) + 0.5);

        let mut temp = u * us;
        let mut s = eye + temp;
        temp = v * vs;
        s = s + temp;
        temp = w * NEAR;
        s = s + temp;
        Ray{src:eye, dir:(s - eye).normalize()} // -> Return Ray
    };

    let lambda = Arc::new(calculate_viewray);
    let lambda_a = lambda.clone();
    let lambda_b = lambda.clone();
    let lambda_c = lambda.clone();
    let lambda_d = lambda.clone();

    let mut start = time::precise_time_s();

    // Run Threads that operate on disjoint image Quads
    let a_thread = thread::spawn(move || {
        thread_render(s_a, lambda_a, 0, (DIM / 2) as i32, 0, (DIM / 2) as i32)
    });

    let b_thread = thread::spawn(move || {
        thread_render(s_b, lambda_b, (DIM / 2) as i32, DIM, 0, (DIM / 2) as i32)
    });

    let c_thread = thread::spawn(move || {
        thread_render(s_c, lambda_c, 0, (DIM / 2) as i32, (DIM / 2) as i32, DIM + 1)
    });

    let d_thread = thread::spawn(move || {
        thread_render(s_d, lambda_d, (DIM / 2) as i32, DIM, (DIM / 2) as i32, DIM + 1)
    });

    // Join Threads before displaying Image
    let quads = vec!(
        a_thread.join().unwrap(),
        b_thread.join().unwrap(),
        c_thread.join().unwrap(),
        d_thread.join().unwrap() );

    let mut end = time::precise_time_s() - start;
    println!("Rendering Time: {} Seconds", end);

    start = time::precise_time_s();

    // Setup Piston Window and display image
    let window:PistonWindow = WindowSettings::new(
        "RustRay @Author:Stewart Charles",
        [DIM as u32,DIM as u32])
        .exit_on_esc(true).build().unwrap();
    
    let mut first = true;

    for e in window {
        e.draw_2d(|c,g| {
            clear([1.0; 4], g);
            for quad in quads.iter() {
                let mut index = 0;
                for y in quad.ymin .. quad.ymax {
                    for x in quad.xmin .. quad.xmax {
                        let color = quad.img[index];
                        index += 1;
                        rectangle([color.x, color.y, color.z, 1.0],
                                  [x as f64, (DIM - y) as f64, 1.0, 1.0],
                                  c.transform, g);
                    }
                }
            }
            if first {
                end = time::precise_time_s() - start;
                println!("Piston Window Draw Time: {} Seconds", end);
                first = false;
            }
        });
    }
}

/* This function will be used by a thread to Generate a section of the
   image being drawn. */
fn thread_render<F:Fn(i32,i32)->Ray>(surfaces:Surfaces, viewray_lambda:Arc<F>,
    xmin:i32, xmax:i32, ymin:i32, ymax:i32) -> ImageQuad {

    let mut image = ImageQuad::new(xmin, xmax, ymin, ymax);

    // Iterate through the the pixels in our Image Plane
    for y in ymin .. ymax {
        for x in xmin .. xmax {
            /* Generate the View Ray for 'this' pixel.
               Makes use of UVW basis vecs captured in the 'closure'
               of the lambda function No need for 'unsafe'
               static mut 'global' variables */
            let view_ray = viewray_lambda(x,y);

            // For each Surface, test for intersection with View Ray
            // Track surface nearest to Viewer with near_t scalar
            let mut near_surf:Option<&Box<Surface>> = None;
            let mut near_t = T1;

            for s in surfaces.iter() {
                let test = s.hit(&view_ray);
                match test {
                    Some(t) => {
                        if t < near_t {
                            near_t = t;
                            near_surf = Some(s);
                        }
                    }
                    None => { }
                }
            }
            
            // If we have a surface at this point, it is a surface that is nearest to the Eye
            // and can now be used to calculate a color. Otherwise, we push the bkg_color
            image.img.push(
                match near_surf {
                    Some(surf) => {
                        surf.calculate_color(&view_ray,&surfaces, near_t, MAX_DEPTH)
                    }
                    None => { BKG_COLOR }
                });
        }
    }
    image
}

/// For the given point, calculates if the point is shaded
/// and returns true if in shadow, false otherwise.
/// Requires access the Vec containing the scenes Surfaces.
fn shadow(point:Vec3<f32>, surfaces:&Surfaces)-> bool {
    let light_dir = (LIGHT_POS - point).normalize();
    let light_ray = Ray{src:point, dir:light_dir};
    for s in surfaces.iter() {
        let test = s.hit(&light_ray);
        match test {
            Some(_) => {
                return true; /* Exit immediately if surface occludes ray to light source */
            }
            None => { }
        }
    }
    false
}

/// Casts a Reflection ray from the 'Point' in a direction that is calculated from the incoming
/// view_dir and surface normal. If the maximum depth has been reached in computing rays, returns
/// the background color for the scene.
fn reflect(point:Vec3<f32>, view_dir:Vec3<f32>, normal:Vec3<f32>,
    depth:i32, surfaces:&Surfaces) -> Vec3<f32> {

    if depth == 0 { return BKG_COLOR; }

    // Calculate Reflection Ray
    let dot_n = 2.0 * na::dot(&view_dir, &normal);
    let dir = normal * dot_n;
    let ray = Ray{src:point, dir:(view_dir - dir).normalize()};

    // Rest of function is largely similar to intersection test
    // in thread_render()
    let mut near_surf:Option<&Box<Surface>> = None;
    let mut current = T1;
    for surf in surfaces.iter() {
        let test = surf.hit(&ray);
        match test {
            Some(t) => {
                if t < current {
                    current = t;
                    near_surf = Some(surf);
                }
            }
            None => { }
        }
    }
    match near_surf {
        Some(surf) => {
            surf.calculate_color(&ray,surfaces,current,depth)
        }
        None => { BKG_COLOR }
    }
}

// Setup Data Structures used by the Ray Tracer

/// Material currently only consists of the ambient term in the
/// BDRF model, and the reflection coeff. Can be modified to include
/// the Specular, Diffuse, and Shininess terms if necessary.
/// (calculate_color function must be modified to use terms if so)
#[derive(Copy,Clone)]
struct Material {
    amb:Vec3<f32>,
    reflect:f32,
}

/// Structure that wraps an ImageQuad that is rended concurrently by one of 4 Threads.
/// Contains all necessary data to be able to properly place the final pixel in the window.
#[derive(Clone)]
struct ImageQuad {
    xmin:i32,
    xmax:i32,
    ymin:i32,
    ymax:i32,
    img:Vec<Vec3<f32>>
}

impl ImageQuad {
    fn new(_xmin:i32, _xmax:i32, _ymin:i32, _ymax:i32)->ImageQuad {
        ImageQuad{xmin:_xmin, xmax:_xmax, ymin:_ymin, ymax:_ymax, img:Vec::new()}
    }
}

/// Simple Container for a Ray
#[derive(Copy,Clone)]
struct Ray {
    src:Vec3<f32>,
    dir:Vec3<f32>,
}

/// Trait for Surface type that can calculate a Ray Surface intersection
/// and also calculates the Color for the point intersected on the Surface.
trait Surface: Sync + Send {
    fn hit(&self, ray:&Ray)->Option<f32>;
    fn calculate_color(&self, ray:&Ray,
                       surfaces:&Surfaces,t:f32, depth:i32)->Vec3<f32>;
}

#[derive(Copy,Clone)]
struct Sphere {
    center:Vec3<f32>,
    radius:f32,
    material:Material,
}

impl Sphere {
    fn new(c:Vec3<f32>,r:f32,mat:Material)->Sphere {
        Sphere{center:c, radius:r, material:mat}
    }

    fn quadratic(&self, a:f32, b:f32, disc:f32)-> Option<f32> {
        let p = (-b + disc.sqrt() ) / (2.0 * a);
        let q = (-b - disc.sqrt() ) / (2.0 * a);
        self.nearest(p,q)
    }

    fn nearest(&self,p:f32, q:f32) -> Option<f32> {
        let p_bound = p > T0 && p < T1;
        let q_bound = q > T0 && q < T1;
        if p_bound && q_bound {
            if p > q { Some(q) }
            else { Some(p) }
        }
        else if p_bound { Some(p) }
        else if q_bound { Some(q) }
        else { None }
    }
    fn boxed(c:Vec3<f32>, r:f32, mat:Material)-> Box<Sphere> {
        Box::new(Sphere::new(c,r,mat))
    }
}

impl Surface for Sphere {
    /* Solving for 't' for Ray: src + dir * t
       s.t. Ray intersects Sphere.
       Sphere is intersected by ray if t is real
       Returns Some(t) only if bounded by T0 && T1 */
    fn hit(&self, ray:&Ray)->Option<f32> {
        let e_minus_c = ray.src - self.center;
        let a = na::dot(&ray.dir, &ray.dir);
        let b = 2.0 * na::dot(&ray.dir, &e_minus_c);
        let c = na::dot(&e_minus_c,&e_minus_c) - self.radius * self.radius;
        let disc = (b * b) - (4.0 * a * c);
        if disc < 0.0 { None }
        else { self.quadratic(a,b,disc) }
    }

    fn calculate_color(&self, ray:&Ray, surfaces:&Surfaces,
        t:f32, depth:i32) -> Vec3<f32> {

        if depth == 0 { return self.material.amb; }
        let dir_ammt = ray.dir * (t - EPSILON);
        let point = ray.src + dir_ammt;
        let normal = (point - self.center).normalize();
        let in_shadow = shadow(point, &surfaces);
        let mut mat = self.material.amb;

        // Compute Diffuse Component of BRDF
        let light_dir = (LIGHT_POS - point).normalize();
        let mut max = largest_of(na::dot(&normal, &light_dir));
        mat = mat + Vec3::new(0.35,0.35,0.35) * max;

        // Add Specular Contribution of BRDF
        // Compute Halfway Vector
        let negative_dir = ray.dir * -1.0;
        let h = light_dir + negative_dir;
        max = largest_of(na::dot(&normal, &h));
        max.powf(1.5);  
        mat = mat + Vec3::new(0.35f32,0.35f32,0.35f32) * max;

        // Apply Shadow if necessary
        if in_shadow { mat = mat * 0.2; }
        else { mat = mat * 0.5 ; }

        // Cast Secondary Ray if Reflective index > 0.0
        if self.material.reflect > 0.0  {
            mix(mat, reflect(point, ray.dir, normal, depth - 1, surfaces),
            self.material.reflect)
        }
        else { mat }
    }
 }

/// Mix function is used to blend between two colors
fn mix(color_a:Vec3<f32>, color_b:Vec3<f32>, alpha:f32) -> Vec3<f32> {
    let mut answer:Vec3<f32> = Vec3::new(0.0,0.0,0.0);
    for i in 0..3 {
        answer[i] = (1.0 - alpha) * color_a[i] + alpha * color_b[i];
    }
    answer
}

fn largest_of(num:f32) -> f32 {
    return if num < 0.0 { 0.0 as f32 } else { num as f32 }
}

#[derive(Copy,Clone)]
struct Triangle {
    a:Vec3<f32>,
    b:Vec3<f32>,
    c:Vec3<f32>,
    normal:Vec3<f32>,
    material:Material,
    pattern:bool,
}

impl Triangle {
    fn new(_a:Vec3<f32>, _b:Vec3<f32>, _c:Vec3<f32>, mat:Material, p:bool)->Triangle {
        let a_b = _a - _b;
        let a_c = _a - _c;
        let n = na::cross(&a_b, &a_c).normalize();
        Triangle{a:_a, b:_b, c:_c, normal:n, material:mat, pattern:p}
 }
    fn boxed(_a:Vec3<f32>, _b:Vec3<f32>, _c:Vec3<f32>, mat:Material, p:bool)->Box<Triangle> {
        Box::new(Triangle::new(_a,_b,_c,mat,p))
    }
}

impl Surface for Triangle {
    fn hit(&self, ray:&Ray)->Option<f32> {
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
        if beta < 0.0 || beta > (1.0 - gamma) { None }
        else { Some(t) }
    }

    /// Calculates Ray intersection of Triangle by utilizing
    /// Shirley's Ray Intersection formula that defines plane
    /// of points A,B,C in Triangle and tests if Barycentric coords
    /// are restricted to Triangle. Solve for Barycentric coords
    /// using Cramers rule of M * [Beta, Gamma, t] = [A - Src]
    /// to solve for [Beta,Gamma,t]
    fn calculate_color(&self, ray:&Ray, surfaces:&Surfaces,
        t:f32, depth:i32) -> Vec3<f32> {

        if depth == 0 { return self.material.amb; }
        let dir_ammt = ray.dir * (t - EPSILON);
        let point = ray.src + dir_ammt;
        let normal = self.normal;
        let in_shadow = shadow(point, &surfaces);
        let mut mat = self.material.amb;

        // Set Checkerboard pattern
        if self.pattern {
            let tx = (point.x + 10.0) % 2.0;
            let ty = (point.z + 10.0) % 2.0;
            if tx < 1.0 && ty < 1.0 || tx > 1.0 && ty > 1.0 {
                mat = mat - Vec3::new(0.2,0.2,0.2);
            }
        }

        // Compute Diffuse Component of BRDF
        let light_dir = (LIGHT_POS - point).normalize();
        let mut max = largest_of(na::dot(&normal, &light_dir));
        mat = mat + Vec3::new(0.25,0.25,0.25) * max;

        // Add Specular Contribution of BRDF
        // Compute Halfway Vector
        let h = light_dir + ray.dir * -1.0;
        max = largest_of(na::dot(&normal, &h));
        max.powf(1.2);
        mat = mat + Vec3::new(0.3f32,0.3f32,0.3f32) * max;

        // Apply Shadow if necessary
        if in_shadow { mat = mat * 0.2; }
        else { mat = mat * 0.5 ; }

        // Cast Secondary Ray if Reflective index > 0.0
        if self.material.reflect > 0.0  {
            mix(mat, reflect(point, ray.dir,normal, depth - 1, surfaces),
            self.material.reflect)
        }
        else { mat }
    }
}
