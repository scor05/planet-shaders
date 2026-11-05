mod algorithms;
mod framebuffer;
mod line;
mod model;
mod shaders;

use algorithms::barycentric::FilledBarycentric;
use algorithms::scanline::FilledScanline;
use algorithms::triangle::{Triangle, TriangleMode, TrianglePipeline};
use algorithms::wireframe::WireFrame;
use framebuffer::Framebuffer;
use model::Model;
use raylib::prelude::*;
use shaders::{FragmentShader, GAS_GIANT_SHADER, ROCKY_PLANET_SHADER, STAR_SHADER, Uniforms};
use std::f32::consts::PI;
use std::time::Instant;

const MODEL_PATH: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/assets/obj/sphere.obj");
const MOVE_SPEED: f32 = 0.1;
const ROT_SPEED: f32 = PI / 100.0;
const PIXELS_PER_UNIT: f32 = 25.0;

fn matrix_mult(mat: Vec<Vector3>, vec: Vector3) -> Vector3 {
    let mut result = Vector3::zero();
    result.x += mat[0].x * vec.x + mat[0].y * vec.y + mat[0].z * vec.z;
    result.y += mat[1].x * vec.x + mat[1].y * vec.y + mat[1].z * vec.z;
    result.z += mat[2].x * vec.x + mat[2].y * vec.y + mat[2].z * vec.z;

    result
}

fn transform(
    vertex: Vector3,
    translation: Vector3,
    scale: f32,
    rotation: Vector3,
    center: Vector3,
) -> Vector3 {
    let mut new_vertex = vertex;
    new_vertex -= center;

    if rotation.x != 0.0 {
        let (sin_theta, cos_theta) = rotation.x.sin_cos();
        let rot_matrix: Vec<Vector3> = vec![
            Vector3::new(1.0, 0.0, 0.0),
            Vector3::new(0.0, cos_theta, -sin_theta),
            Vector3::new(0.0, sin_theta, cos_theta),
        ];

        new_vertex = matrix_mult(rot_matrix, new_vertex);
    }

    if rotation.y != 0.0 {
        let (sin_theta, cos_theta) = rotation.y.sin_cos();
        let rot_matrix: Vec<Vector3> = vec![
            Vector3::new(cos_theta, 0.0, sin_theta),
            Vector3::new(0.0, 1.0, 0.0),
            Vector3::new(-sin_theta, 0.0, cos_theta),
        ];
        new_vertex = matrix_mult(rot_matrix, new_vertex);
    }

    if rotation.z != 0.0 {
        let (sin_theta, cos_theta) = rotation.z.sin_cos();
        let rot_matrix: Vec<Vector3> = vec![
            Vector3::new(cos_theta, -sin_theta, 0.0),
            Vector3::new(sin_theta, cos_theta, 0.0),
            Vector3::new(0.0, 0.0, 1.0),
        ];

        new_vertex = matrix_mult(rot_matrix, new_vertex);
    }

    new_vertex *= scale;

    new_vertex += translation;
    new_vertex
}

fn render(
    framebuffer: &mut Framebuffer,
    translation: Vector3,
    scale: f32,
    rotation: Vector3,
    model: &Model,
    pipeline: &dyn TrianglePipeline,
    shader: FragmentShader,
    uniforms: &Uniforms,
) {
    for indices in &model.triangles {
        let mut projected = [Vector3::zero(); 3];

        for (destination, vertex_index) in projected.iter_mut().zip(indices) {
            let transformed = transform(
                model.vertices[*vertex_index],
                translation,
                scale,
                rotation,
                model.center,
            );

            *destination = Vector3::new(
                framebuffer.width as f32 * 0.5 + transformed.x * PIXELS_PER_UNIT,
                framebuffer.height as f32 * 0.5 - transformed.y * PIXELS_PER_UNIT,
                transformed.z,
            );
        }

        pipeline.render(
            framebuffer,
            &Triangle {
                vertices: projected,
                colors: [
                    model.colors[indices[0]],
                    model.colors[indices[1]],
                    model.colors[indices[2]],
                ],
            },
            shader,
            uniforms,
        );
    }
}

fn main() {
    let model = Model::load(MODEL_PATH)
        .unwrap_or_else(|error| panic!("Could not load {MODEL_PATH}: {error}"));

    let window_width = 800;
    let window_height = 600;

    let (mut window, raylib_thread) = raylib::init()
        .size(window_width, window_height)
        .title("OBJ Renderer")
        .log_level(TraceLogLevel::LOG_WARNING)
        .build();
    window.set_target_fps(60);

    let mut framebuffer = Framebuffer::new(window_width as u32, window_height as u32);

    let wireframe = WireFrame;
    let scanline = FilledScanline;
    let barycentric = FilledBarycentric;

    let mode = TriangleMode::FilledBarycentric;
    let pipeline: &dyn TrianglePipeline = match mode {
        TriangleMode::Wireframe => &wireframe,
        TriangleMode::FilledBarycentric => &barycentric,
        TriangleMode::FilledScanline => &scanline,
    };

    let mut translation = Vector3::zero();
    let mut rotation = Vector3::zero();
    let mut scale = 10.0_f32;
    let shaders = shaders::registry();
    let mut active_shader = STAR_SHADER.to_owned();
    let start_time = Instant::now();

    while !window.window_should_close() {
        if window.is_key_pressed(KeyboardKey::KEY_ONE) {
            active_shader = STAR_SHADER.to_owned();
        } else if window.is_key_pressed(KeyboardKey::KEY_TWO) {
            active_shader = ROCKY_PLANET_SHADER.to_owned();
        } else if window.is_key_pressed(KeyboardKey::KEY_THREE) {
            active_shader = GAS_GIANT_SHADER.to_owned();
        }

        if window.is_key_down(KeyboardKey::KEY_RIGHT) {
            translation.x += MOVE_SPEED;
        }
        if window.is_key_down(KeyboardKey::KEY_LEFT) {
            translation.x -= MOVE_SPEED;
        }
        if window.is_key_down(KeyboardKey::KEY_DOWN) {
            translation.y -= MOVE_SPEED;
        }
        if window.is_key_down(KeyboardKey::KEY_UP) {
            translation.y += MOVE_SPEED;
        }

        if window.is_key_pressed(KeyboardKey::KEY_Q) {
            scale += 0.1;
        }
        if window.is_key_pressed(KeyboardKey::KEY_E) {
            scale = (scale - 0.1).max(0.5);
        }

        if window.is_key_down(KeyboardKey::KEY_A) {
            rotation.y -= ROT_SPEED;
        }
        if window.is_key_down(KeyboardKey::KEY_D) {
            rotation.y += ROT_SPEED;
        }

        if window.is_key_down(KeyboardKey::KEY_W) {
            rotation.x += ROT_SPEED;
        }
        if window.is_key_down(KeyboardKey::KEY_S) {
            rotation.x -= ROT_SPEED;
        }

        if window.is_key_down(KeyboardKey::KEY_Z) {
            rotation.z -= ROT_SPEED;
        }
        if window.is_key_down(KeyboardKey::KEY_X) {
            rotation.z += ROT_SPEED;
        }

        framebuffer.clear();

        let shader = *shaders
            .get(&active_shader)
            .expect("active shader must exist in the shader registry");
        let uniforms = Uniforms {
            time: start_time.elapsed().as_secs_f32(),
        };

        render(
            &mut framebuffer,
            translation,
            scale,
            rotation,
            &model,
            pipeline,
            shader,
            &uniforms,
        );

        framebuffer.swap_buffers(&mut window, &raylib_thread, &active_shader);
    }
}
