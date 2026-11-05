use raylib::prelude::Vector3;

pub fn smoothstep(edge0: f32, edge1: f32, value: f32) -> f32 {
    let t = ((value - edge0) / (edge1 - edge0)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

pub fn value_noise(position: Vector3, seed: u32) -> f32 {
    let x0 = position.x.floor() as i32;
    let y0 = position.y.floor() as i32;
    let z0 = position.z.floor() as i32;
    let tx = fade(position.x - x0 as f32);
    let ty = fade(position.y - y0 as f32);
    let tz = fade(position.z - z0 as f32);

    let mut corners = [0.0; 8];
    let mut index = 0;
    for z in 0..=1 {
        for y in 0..=1 {
            for x in 0..=1 {
                corners[index] = hash_3d(x0 + x, y0 + y, z0 + z, seed);
                index += 1;
            }
        }
    }

    let x00 = lerp(corners[0], corners[1], tx);
    let x10 = lerp(corners[2], corners[3], tx);
    let x01 = lerp(corners[4], corners[5], tx);
    let x11 = lerp(corners[6], corners[7], tx);
    lerp(lerp(x00, x10, ty), lerp(x01, x11, ty), tz)
}

pub fn fbm(position: Vector3, octaves: u32, seed: u32) -> f32 {
    let mut point = position;
    let mut amplitude = 0.5;
    let mut total = 0.0;
    let mut normalization = 0.0;

    for octave in 0..octaves {
        total += value_noise(point, seed.wrapping_add(octave * 1013)) * amplitude;
        normalization += amplitude;
        point *= 2.03;
        amplitude *= 0.5;
    }

    total / normalization
}

pub fn cellular(position: Vector3, seed: u32) -> f32 {
    let cell_x = position.x.floor() as i32;
    let cell_y = position.y.floor() as i32;
    let cell_z = position.z.floor() as i32;
    let mut minimum_squared = f32::INFINITY;

    for z in -1..=1 {
        for y in -1..=1 {
            for x in -1..=1 {
                let grid_x = cell_x + x;
                let grid_y = cell_y + y;
                let grid_z = cell_z + z;
                let feature = Vector3::new(
                    grid_x as f32 + hash_3d(grid_x, grid_y, grid_z, seed),
                    grid_y as f32 + hash_3d(grid_x, grid_y, grid_z, seed ^ 0x68bc_21eb),
                    grid_z as f32 + hash_3d(grid_x, grid_y, grid_z, seed ^ 0x02e5_be93),
                );
                let delta = position - feature;
                let distance_squared = delta.x * delta.x + delta.y * delta.y + delta.z * delta.z;
                minimum_squared = minimum_squared.min(distance_squared);
            }
        }
    }

    minimum_squared.sqrt().clamp(0.0, 1.0)
}

fn fade(value: f32) -> f32 {
    value * value * (3.0 - 2.0 * value)
}

fn lerp(start: f32, end: f32, amount: f32) -> f32 {
    start + (end - start) * amount
}

fn hash_3d(x: i32, y: i32, z: i32, seed: u32) -> f32 {
    let mut hash = seed
        ^ (x as u32).wrapping_mul(0x8da6_b343)
        ^ (y as u32).wrapping_mul(0xd816_3841)
        ^ (z as u32).wrapping_mul(0xcb1a_b31f);
    hash ^= hash >> 16;
    hash = hash.wrapping_mul(0x7feb_352d);
    hash ^= hash >> 15;
    hash = hash.wrapping_mul(0x846c_a68b);
    hash ^= hash >> 16;
    hash as f32 / u32::MAX as f32
}
