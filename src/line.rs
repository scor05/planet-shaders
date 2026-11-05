use raylib::prelude::Vector3;

use crate::framebuffer::Framebuffer;

pub fn line(framebuffer: &mut Framebuffer, start: Vector3, end: Vector3) {
    let mut x0 = start.x.round() as i32;
    let mut y0 = start.y.round() as i32;
    let x1 = end.x.round() as i32;
    let y1 = end.y.round() as i32;

    let dx = (x1 - x0).abs();
    let step_x = if x0 < x1 { 1 } else { -1 };
    let dy = -(y1 - y0).abs();
    let step_y = if y0 < y1 { 1 } else { -1 };
    let mut error = dx + dy;

    loop {
        if x0 >= 0 && y0 >= 0 {
            framebuffer.set_pixel(x0 as u32, y0 as u32);
        }

        if x0 == x1 && y0 == y1 {
            break;
        }

        let doubled_error = 2 * error;
        if doubled_error >= dy {
            error += dy;
            x0 += step_x;
        }
        if doubled_error <= dx {
            error += dx;
            y0 += step_y;
        }
    }
}
