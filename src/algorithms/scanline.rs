use super::triangle::{COVERAGE_EPSILON, Fragment, Triangle, TrianglePipeline};
use crate::framebuffer::Framebuffer;
use crate::shaders::{FragmentShader, Uniforms};
use raylib::prelude::Vector3;

pub struct FilledScanline;

impl FilledScanline {
    pub fn rasterize(&self, triangle: &Triangle) -> Vec<Fragment> {
        let [first, second, third] = triangle.vertices;

        if signed_area(first, second, third).abs() <= f32::EPSILON {
            return Vec::new();
        }

        let min_y = first.y.min(second.y).min(third.y).floor() as i32;
        let max_y = first.y.max(second.y).max(third.y).ceil() as i32;
        let mut fragments = Vec::new();

        for y in min_y..max_y {
            let sample_y = y as f32 + 0.5;
            let mut intersections = Vec::with_capacity(6);

            add_edge_intersections(&mut intersections, first, second, sample_y);
            add_edge_intersections(&mut intersections, second, third, sample_y);
            add_edge_intersections(&mut intersections, third, first, sample_y);

            if intersections.is_empty() {
                continue;
            }

            intersections.sort_by(f32::total_cmp);
            let left = intersections[0];
            let right = intersections[intersections.len() - 1];

            let first_x = (left - 0.5 - COVERAGE_EPSILON).ceil() as i32;
            let last_x = (right - 0.5 + COVERAGE_EPSILON).floor() as i32;

            for x in first_x..=last_x {
                if let Some(fragment) = triangle.fragment_at(x, y) {
                    fragments.push(fragment);
                }
            }
        }

        fragments
    }
}

impl TrianglePipeline for FilledScanline {
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

fn add_edge_intersections(
    intersections: &mut Vec<f32>,
    start: Vector3,
    end: Vector3,
    sample_y: f32,
) {
    let delta_y = end.y - start.y;

    if delta_y.abs() <= f32::EPSILON {
        if (sample_y - start.y).abs() <= COVERAGE_EPSILON {
            intersections.push(start.x);
            intersections.push(end.x);
        }
        return;
    }

    let minimum_y = start.y.min(end.y);
    let maximum_y = start.y.max(end.y);
    if sample_y < minimum_y - COVERAGE_EPSILON || sample_y > maximum_y + COVERAGE_EPSILON {
        return;
    }

    let interpolation = (sample_y - start.y) / delta_y;
    intersections.push(start.x + interpolation * (end.x - start.x));
}

fn signed_area(first: Vector3, second: Vector3, third: Vector3) -> f32 {
    (second.x - first.x) * (third.y - first.y) - (second.y - first.y) * (third.x - first.x)
}
