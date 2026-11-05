use crate::algorithms::triangle::Fragment;
use crate::procedural::{cellular, fbm, smoothstep};
use raylib::prelude::{Color, Vector3};
use std::collections::HashMap;

pub struct Uniforms {
    pub time: f32,
    pub light_position: Vector3,
}

pub type FragmentShader = fn(&Fragment, &Uniforms) -> Color;

pub const STAR_SHADER: &str = "Neutron star";
pub const ROCKY_PLANET_SHADER: &str = "Ocean planet";
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
    let position = normalize(fragment.position);
    let time = uniforms.time;

    let warp = fbm(
        position * 3.2 + Vector3::new(time * 0.05, -time * 0.03, time * 0.04),
        3,
        17,
    );
    let cellular_point =
        position * 8.0 + Vector3::new(warp * 1.7, warp * 0.8 + time * 0.12, -warp * 1.2);
    let cells = cellular(cellular_point, 91);
    let hot_cell_centers = 1.0 - smoothstep(0.12, 0.72, cells);

    let turbulence = fbm(
        position * 5.0 + Vector3::new(-time * 0.08, time * 0.06, time * 0.03),
        3,
        203,
    );
    let magnetic_patches = smoothstep(
        0.66,
        0.82,
        fbm(
            position * 2.4 + Vector3::new(time * 0.015, 0.0, -time * 0.01),
            3,
            811,
        ),
    );

    let base = mix(
        ColorF::rgb(0.008, 0.015, 0.16),
        ColorF::rgb(0.04, 0.38, 1.0),
        0.35 + turbulence * 0.55,
    );
    let granulated = combine(
        ColorF::rgb(0.28, 0.78, 1.0),
        ColorF::BLACK,
        ColorF::splat(hot_cell_centers * 0.62),
        base,
    );
    let flicker = 0.5 + 0.5 * (time * 1.7 + warp * 8.0).sin();
    let active_surface = combine(
        ColorF::rgb(0.72, 0.94, 1.0),
        ColorF::BLACK,
        ColorF::splat(turbulence * (0.08 + flicker * 0.08)),
        granulated,
    );
    let patched = mix(
        active_surface,
        ColorF::rgb(0.025, 0.008, 0.14),
        magnetic_patches * 0.72,
    );

    let flares = flare_field(position, time);
    let flare_blue = combine(
        ColorF::rgb(0.12, 0.66, 1.0),
        ColorF::BLACK,
        ColorF::splat(flares * 0.82),
        patched,
    );
    let flare_core = combine(
        ColorF::rgb(0.88, 0.98, 1.0),
        ColorF::BLACK,
        ColorF::splat(smoothstep(0.58, 0.92, flares)),
        flare_blue,
    );

    flare_core.to_color()
}

fn rocky_planet(fragment: &Fragment, uniforms: &Uniforms) -> Color {
    let position = normalize(fragment.position);
    let light_direction = normalize(uniforms.light_position - fragment.position);

    let continent_warp = fbm(position * 1.8, 3, 300);
    let terrain = fbm(
        position * 3.6 + Vector3::new(continent_warp * 1.3, -continent_warp * 0.7, continent_warp),
        4,
        431,
    );
    let detail = fbm(position * 13.0, 2, 991);
    let land_mask = smoothstep(0.49, 0.59, terrain);

    let ocean = mix(
        ColorF::rgb(0.008, 0.035, 0.13),
        ColorF::rgb(0.015, 0.22, 0.34),
        detail,
    );
    let moisture = fbm(position * 5.5 + Vector3::new(7.0, 2.0, -4.0), 3, 712);
    let vegetation = smoothstep(0.42, 0.68, moisture) * (1.0 - position.y.abs());
    let land = mix(
        ColorF::rgb(0.48, 0.27, 0.08),
        ColorF::rgb(0.035, 0.30, 0.10),
        vegetation,
    );
    let mut surface = mix(ocean, land, land_mask);

    let ice_mask = smoothstep(0.68, 0.84, position.y.abs() + (detail - 0.5) * 0.12);
    surface = mix(surface, ColorF::rgb(0.72, 0.88, 0.94), ice_mask);

    let cloud_point = rotate_y(position, uniforms.time * 0.045);
    let clouds = fbm(
        cloud_point * 7.5 + Vector3::new(uniforms.time * 0.025, 0.0, 0.0),
        3,
        1501,
    );
    let cloud_mask = smoothstep(0.60, 0.72, clouds);

    let shadow_clouds = fbm(
        (cloud_point - light_direction * 0.075) * 7.5
            + Vector3::new(uniforms.time * 0.025, 0.0, 0.0),
        3,
        1501,
    );
    let shadow_mask = smoothstep(0.60, 0.72, shadow_clouds) * 0.28;
    surface = combine(
        surface,
        ColorF::BLACK,
        ColorF::splat(1.0 - shadow_mask),
        ColorF::BLACK,
    );

    let diffuse = dot(position, light_direction).max(0.0);
    let illumination = 0.10 + diffuse * 0.90;
    surface = combine(
        surface,
        ColorF::BLACK,
        ColorF::splat(illumination),
        ColorF::rgb(0.0, 0.008, 0.018),
    );

    let reflected = reflect(-light_direction, position);
    let specular = dot(reflected, Vector3::new(0.0, 0.0, -1.0))
        .max(0.0)
        .powf(48.0)
        * (1.0 - land_mask)
        * diffuse;
    surface = combine(
        ColorF::rgb(0.68, 0.90, 1.0),
        ColorF::BLACK,
        ColorF::splat(specular * 0.8),
        surface,
    );

    let cloud_light = 0.28 + diffuse * 0.72;
    let clouds_color = ColorF::rgb(0.86, 0.91, 0.92) * cloud_light;
    mix(surface, clouds_color, cloud_mask * 0.82).to_color()
}

fn gas_giant(fragment: &Fragment, uniforms: &Uniforms) -> Color {
    let position = normalize(fragment.position);
    let flowing = rotate_y(position, uniforms.time * 0.025);
    let broad_noise = fbm(
        flowing * 4.0 + Vector3::new(uniforms.time * 0.018, 0.0, 0.0),
        3,
        2401,
    );

    let storm_x = position.x - 0.34;
    let storm_y = position.y + 0.14;
    let storm_distance = (storm_x * storm_x * 1.6 + storm_y * storm_y * 5.0).sqrt();
    let front_facing = 1.0 - smoothstep(-0.72, -0.15, position.z);
    let storm_influence = (1.0 - smoothstep(0.08, 0.48, storm_distance)) * front_facing;
    let storm_angle = storm_y.atan2(storm_x);

    let warped_latitude = position.y
        + (broad_noise - 0.5) * 0.13
        + storm_influence * (storm_angle * 2.0 + uniforms.time * 0.65).sin() * 0.10;
    let band_wave = (warped_latitude * 29.0 + broad_noise * 4.5).sin() * 0.5 + 0.5;
    let soft_band = smoothstep(0.12, 0.88, band_wave);

    let pale = ColorF::rgb(0.84, 0.64, 0.38);
    let rust = ColorF::rgb(0.43, 0.12, 0.055);
    let cream = ColorF::rgb(0.96, 0.82, 0.57);
    let base = mix(rust, pale, soft_band);
    let mut atmosphere = combine(
        cream,
        ColorF::BLACK,
        ColorF::splat((broad_noise - 0.35).max(0.0) * 0.34),
        base,
    );

    let storm_rings =
        (storm_distance * 46.0 - storm_angle * 3.0 + uniforms.time * 0.9).sin() * 0.5 + 0.5;
    let storm_color = mix(
        ColorF::rgb(0.55, 0.10, 0.035),
        ColorF::rgb(1.0, 0.65, 0.28),
        storm_rings,
    );
    atmosphere = mix(atmosphere, storm_color, storm_influence * 0.82);

    let wisps = fbm(flowing * 15.0, 2, 3217);
    atmosphere = combine(
        ColorF::rgb(0.90, 0.70, 0.45),
        ColorF::BLACK,
        ColorF::splat((wisps - 0.5).max(0.0) * 0.18),
        atmosphere,
    );

    let light_direction = normalize(uniforms.light_position - fragment.position);
    let diffuse = dot(position, light_direction).max(0.0);
    combine(
        atmosphere,
        ColorF::BLACK,
        ColorF::splat(0.13 + diffuse * 0.87),
        ColorF::rgb(0.008, 0.003, 0.012),
    )
    .to_color()
}

fn flare_field(position: Vector3, time: f32) -> f32 {
    let longitude = position.z.atan2(position.x);
    let latitude = position.y.clamp(-1.0, 1.0).asin();
    let flare_data = [
        (-1.15_f32, 0.24_f32, 0.00_f32, 4103_u32),
        (-2.05_f32, -0.20_f32, 0.37_f32, 5417_u32),
        (-0.52_f32, 0.02_f32, 0.71_f32, 6857_u32),
    ];
    let mut field: f32 = 0.0;

    for (center_longitude, center_latitude, offset, seed) in flare_data {
        let cycle = (time * 0.105 + offset).fract();
        let formation = smoothstep(0.0, 0.16, cycle);
        let decay = 1.0 - smoothstep(0.66, 1.0, cycle);
        let envelope = formation * decay;
        let radius = 0.10 + cycle * 0.72;

        let longitude_delta =
            wrap_angle(longitude - center_longitude) * center_latitude.cos() * 0.82;
        let latitude_delta = (latitude - center_latitude) * 1.65;
        let ellipse = (longitude_delta * longitude_delta + latitude_delta * latitude_delta).sqrt();
        let breakup = (fbm(
            position * 14.0 + Vector3::new(offset * 9.0, time * 0.09, -offset * 5.0),
            2,
            seed,
        ) - 0.5)
            * 0.15;
        let loop_line = 1.0 - smoothstep(0.025, 0.105, (ellipse + breakup - radius).abs());
        let upper_arc = smoothstep(-0.16, 0.10, latitude_delta + radius * 0.32);
        let footpoint = 1.0 - smoothstep(0.0, 0.11, ellipse);

        field = field.max((loop_line * upper_arc + footpoint * 0.75) * envelope);
    }

    field.clamp(0.0, 1.0)
}

fn wrap_angle(angle: f32) -> f32 {
    let tau = std::f32::consts::TAU;
    (angle + std::f32::consts::PI).rem_euclid(tau) - std::f32::consts::PI
}

#[derive(Clone, Copy)]
struct ColorF {
    r: f32,
    g: f32,
    b: f32,
}

impl ColorF {
    const BLACK: Self = Self::rgb(0.0, 0.0, 0.0);

    const fn rgb(r: f32, g: f32, b: f32) -> Self {
        Self { r, g, b }
    }

    const fn splat(value: f32) -> Self {
        Self::rgb(value, value, value)
    }

    fn to_color(self) -> Color {
        Color::new(
            (self.r.clamp(0.0, 1.0) * 255.0) as u8,
            (self.g.clamp(0.0, 1.0) * 255.0) as u8,
            (self.b.clamp(0.0, 1.0) * 255.0) as u8,
            255,
        )
    }
}

impl std::ops::Mul<f32> for ColorF {
    type Output = Self;

    fn mul(self, factor: f32) -> Self::Output {
        Self::rgb(self.r * factor, self.g * factor, self.b * factor)
    }
}

/// PS2-style fixed-function color equation: (A - B) * C + D.
fn combine(a: ColorF, b: ColorF, c: ColorF, d: ColorF) -> ColorF {
    ColorF::rgb(
        ((a.r - b.r) * c.r + d.r).clamp(0.0, 1.0),
        ((a.g - b.g) * c.g + d.g).clamp(0.0, 1.0),
        ((a.b - b.b) * c.b + d.b).clamp(0.0, 1.0),
    )
}

fn mix(start: ColorF, end: ColorF, amount: f32) -> ColorF {
    combine(end, start, ColorF::splat(amount), start)
}

fn normalize(vector: Vector3) -> Vector3 {
    let length_squared = dot(vector, vector);
    if length_squared <= f32::EPSILON {
        Vector3::zero()
    } else {
        vector * length_squared.sqrt().recip()
    }
}

fn dot(first: Vector3, second: Vector3) -> f32 {
    first.x * second.x + first.y * second.y + first.z * second.z
}

fn reflect(direction: Vector3, normal: Vector3) -> Vector3 {
    direction - normal * (2.0 * dot(direction, normal))
}

fn rotate_y(point: Vector3, angle: f32) -> Vector3 {
    let (sin, cos) = angle.sin_cos();
    Vector3::new(
        point.x * cos + point.z * sin,
        point.y,
        -point.x * sin + point.z * cos,
    )
}
