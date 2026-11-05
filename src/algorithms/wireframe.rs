use super::triangle::{Triangle, TrianglePipeline};
use crate::framebuffer::Framebuffer;
use crate::line::line;
use crate::shaders::{FragmentShader, Uniforms};

pub struct WireFrame;

impl TrianglePipeline for WireFrame {
    fn render(
        &self,
        framebuffer: &mut Framebuffer,
        triangle: &Triangle,
        _shader: FragmentShader,
        _uniforms: &Uniforms,
    ) {
        let [first, second, third] = triangle.vertices;

        line(framebuffer, first, second);
        line(framebuffer, second, third);
        line(framebuffer, third, first);
    }
}
