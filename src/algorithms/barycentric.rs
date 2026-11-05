use super::triangle::{Fragment, Triangle, TrianglePipeline};
use crate::framebuffer::Framebuffer;
use crate::shaders::{FragmentShader, Uniforms};
use raylib::prelude::Vector3;

pub struct FilledBarycentric;

impl FilledBarycentric {
    pub fn rasterize(&self, triangle: &Triangle) -> Vec<Fragment> {
        let [first, second, third] = triangle.vertices;
        let area = edge_function(first, second, third.x, third.y);

        if area.abs() <= f32::EPSILON {
            return Vec::new();
        }

        let min_x = first.x.min(second.x).min(third.x).floor() as i32;
        let max_x = first.x.max(second.x).max(third.x).ceil() as i32;
        let min_y = first.y.min(second.y).min(third.y).floor() as i32;
        let max_y = first.y.max(second.y).max(third.y).ceil() as i32;

        let capacity = (max_x - min_x).max(0) as usize * (max_y - min_y).max(0) as usize;
        let mut fragments = Vec::with_capacity(capacity);

        for y in min_y..max_y {
            for x in min_x..max_x {
                if let Some(fragment) = triangle.fragment_at(x, y) {
                    fragments.push(fragment);
                }
            }
        }

        fragments
    }
}

impl TrianglePipeline for FilledBarycentric {
    fn render(
        &self,
        framebuffer: &mut Framebuffer,
        triangle: &Triangle,
        shader: FragmentShader,
        uniforms: &Uniforms,
    ) {
        for fragment in self.rasterize(triangle) {
            framebuffer.set_fragment(
                fragment.x,
                fragment.y,
                fragment.depth,
                shader(&fragment, uniforms),
            );
        }
    }
}

fn edge_function(start: Vector3, end: Vector3, x: f32, y: f32) -> f32 {
    (end.x - start.x) * (y - start.y) - (end.y - start.y) * (x - start.x)
}
