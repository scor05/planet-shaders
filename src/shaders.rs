use crate::algorithms::triangle::Fragment;
use raylib::prelude::Color;
use std::collections::HashMap;

pub struct Uniforms {
    /// Seconds elapsed since the program started.
    pub time: f32,
}

pub type FragmentShader = fn(&Fragment, &Uniforms) -> Color;

pub const STAR_SHADER: &str = "Star";
pub const ROCKY_PLANET_SHADER: &str = "Planet";
pub const GAS_GIANT_SHADER: &str = "Gas giant";

pub fn registry() -> HashMap<String, FragmentShader> {
    HashMap::from([
        (STAR_SHADER.to_owned(), star as FragmentShader),
        (
            ROCKY_PLANET_SHADER.to_owned(),
            rocky_planet as FragmentShader,
        ),
        (GAS_GIANT_SHADER.to_owned(), gas_giant as FragmentShader),
    ])
}

fn star(fragment: &Fragment, uniforms: &Uniforms) -> Color {
    // Keep the uniform wired into the placeholder until the animated shader is added.
    let _ = uniforms.time;
    fragment.color
}

fn rocky_planet(fragment: &Fragment, _uniforms: &Uniforms) -> Color {
    fragment.color
}

fn gas_giant(fragment: &Fragment, _uniforms: &Uniforms) -> Color {
    fragment.color
}
