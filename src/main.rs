extern crate sdl2;

use std::time::{Instant};

use sdl2::rect::{Point};
use sdl2::pixels::Color;
use sdl2::event::Event;
use sdl2::mouse::{MouseButton};
use sdl2::keyboard::Keycode;

use draw::{Model};


const WIDTH: u32 = 800;
const HEIGHT: u32 = 600;

pub fn main() {
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem.window("rust-sdl2 demo", WIDTH, HEIGHT)
        .position_centered()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().build().unwrap();

    canvas.set_draw_color(Color::RGB(0, 0, 0));
    canvas.clear();
    let mut x_min: f32 = 0.;
    let mut x_max: f32 = 0.;
    let mut y_min: f32 = 0.;
    let mut y_max: f32 = 0.;
    let model = Model::new("african_head.obj");
    let faces = model.faces;
    for face in &faces {
        for j in 0..3 {
            let v0 = &model.vertices[face[j] as usize];
            let v1 = &model.vertices[face[(j + 1) % 3] as usize];
            if x_min > v0.x {
                x_min = v0.x
            }
            if x_min > v1.x {
                x_min = v1.x
            }
            if y_min > v0.y {
                y_min = v0.y
            }
            if y_min > v1.y {
                y_min = v1.y
            }

            if x_max < v0.x {
                x_max = v0.x
            }
            if x_max < v1.x {
                x_max = v1.x
            }
            if y_max < v0.y {
                y_max = v0.y
            }
            if y_max < v1.y {
                y_max = v1.y
            }
        }
    }

    let mut scale: f32;
    if (x_max - x_min).abs() < (y_max - y_min).abs() {
        scale = HEIGHT as f32 / (y_max - y_min).abs();
    } else {
        scale = WIDTH as f32 / (x_max - x_min).abs();
    }
    let mut offset_y = HEIGHT  as f32 - (HEIGHT  as f32 - y_max);
    let mut offset_x = WIDTH  as f32 - (WIDTH  as f32 - (x_max - x_min));
    let def_scale = scale;
    let mut def_offset_x = offset_x;
    let mut def_offset_y = offset_y;
    // let mut current_x: i32 = 0;
    // let mut current_y: i32 = 0;
    let mut current_right_state = false;

    canvas.present();
    let mut event_pump = sdl_context.event_pump().unwrap();
    'running: loop {
        canvas.set_draw_color(Color::RGB(0, 0, 0));
        canvas.clear();
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } |
                Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                    break 'running;
                }
                Event::MouseWheel { y, .. } => {
                    if y > 0 {
                        scale += 5.;
                    } else {
                        scale -= 5.;
                    }
                    if scale < 0. {
                        scale = 0.
                    }
                    offset_x = def_offset_x / (scale/def_scale);
                    offset_y = def_offset_y / (scale/def_scale);
                }
                Event::MouseMotion { xrel, yrel, .. } => {
                    if current_right_state {
                        offset_y += (y_max - y_min) / 500. * yrel as f32;
                        offset_x += (x_max - x_min) / 500. * xrel as f32;
                        def_offset_y = offset_y;
                        def_offset_x = offset_x;
                    }
                }
                Event::MouseButtonDown {mouse_btn, ..} => {
                    if mouse_btn == MouseButton::Left {
                        current_right_state = true;
                    }
                }
                Event::MouseButtonUp{mouse_btn, ..} => {
                    if mouse_btn == MouseButton::Left {
                        current_right_state = false;
                    }

                }
                _ => {
                    println!("event: {:?}", event)
                }
            }
        }
        // The rest of the game loop goes here...
        //canvas.clear();
        canvas.set_draw_color(Color::RGB(255, 255, 255));
        let start = Instant::now();
        for face in &faces {
            for j in 0..3 {
                let v0 = &model.vertices[face[j] as usize];
                let v1 = &model.vertices[face[(j + 1) % 3] as usize];
                let x1 = ((v0.x - offset_x) * scale * -1.) as i32;
                let y1 = ((v0.y - offset_y) * scale * -1.) as i32;
                let x2 = ((v1.x - offset_x) * scale * -1.) as i32;
                let y2 = ((v1.y - offset_y) * scale * -1.) as i32;
                let l = draw::line(x1, y1, x2, y2);
                for coord in l {
                    match canvas.draw_point(Point::new(coord[0], coord[1])) {
                        Ok(_) => {}
                        Err(e) => {
                            println!("error: {}", e)
                        }
                    }
                }
            }
        }
        //println!("{:?}", Instant::now().duration_since(start));
        canvas.present();

        //::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
    }
}

pub mod draw {
    use std::mem;
    use std::fmt;
    use std::fs::File;
    use std::io::{BufReader, BufRead};
    use std::path::Path;

    pub fn line_old(mut x1: i32, mut y1: i32, mut x2: i32, mut y2: i32) -> Vec<[i32; 2]> {
        if (x1 == x2) && (y1 == y2) {
            return vec!([x1, y1]);
        }
        let mut v = Vec::new();
        if y1 > y2 {
            mem::swap(&mut y1, &mut y2);
        }
        if x1 > x2 {
            mem::swap(&mut x1, &mut x2);
        }

        if x1 == x2 {
            for y in y1..y2 {
                v.push([x1, y]);
            }
            return v;
        }
        if y1 == y2 {
            for x in x1..x2 {
                v.push([x, y1]);
            }
            return v;
        }
        let total_steps = ((x1.pow(2) + x2.pow(2)) as f64).sqrt() as i32;
        for step in 0..total_steps {
            let k = step * 100 / total_steps;
            let y = (y2 - y1) * k / 100 as i32 + y1;
            let x = (x2 - x1) * k / 100 as i32 + x1;
            v.push([x, y]);
        }
        return v;
    }

    pub fn line(mut x0: i32, mut y0: i32, mut x1: i32, mut y1: i32) -> Vec<[i32; 2]> {
        let mut v = Vec::new();

        let mut steep = false;
        if (x0 - x1).abs() < (y0 - y1).abs() {
            mem::swap(&mut x0, &mut y0);
            mem::swap(&mut x1, &mut y1);
            steep = true;
        }
        if x0 > x1 {
            mem::swap(&mut x0, &mut x1);
            mem::swap(&mut y0, &mut y1);
        }
        let dx = x1 - x0;
        let dy = y1 - y0;
        let derror2 = dy.abs() * 2;
        let mut error2 = 0;
        let mut y = y0;
        for x in x0..x1 + 1 {
            if steep {
                v.push([y, x]);
            } else {
                v.push([x, y]);
            }
            error2 += derror2;

            if error2 > dx {
                y += if y1 > y0 { 1 } else { -1 };
                error2 -= dx * 2;
            }
        }
        return v;
    }

    pub struct Vector3D {
        pub x: f32,
        pub y: f32,
        pub z: f32,
    }

    impl Vector3D {
        pub fn new(x: f32, y: f32, z: f32) -> Vector3D {
            Vector3D {
                x,
                y,
                z,
            }
        }
    }

    impl fmt::Display for Vector3D {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "({},{},{})", self.x, self.y, self.z)
        }
    }

    pub struct Model {
        pub vertices: Vec<Vector3D>,
        pub faces: Vec<[i32; 3]>,
    }

    impl Model {
        pub fn new(file_path: &str) -> Model {
            let path = Path::new(file_path);
            let file = BufReader::new(File::open(&path).unwrap());
            let mut vertices = Vec::new();
            let mut faces = Vec::new();
            for line in file.lines() {
                let line = line.unwrap();
                if line.starts_with("v ") {
                    let words: Vec<&str> = line.split_whitespace().collect();
                    vertices.push(Vector3D::new(words[1].parse().unwrap(),
                                                words[2].parse().unwrap(),
                                                words[3].parse().unwrap()));
                } else if line.starts_with("f ") {
                    let mut face: [i32; 3] = [-1, -1, -1];
                    let words: Vec<&str> = line.split_whitespace().collect();
                    for i in 0..3 {
                        face[i] = words[i + 1].split("/").next().unwrap().parse().unwrap();
                        face[i] -= 1;
                    }
                    faces.push(face);
                }
            }
            Model {
                vertices: vertices,
                faces: faces,
            }
        }
    }
}

