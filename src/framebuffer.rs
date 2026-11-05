use raylib::prelude::*;

// current_color sirve para pintar varios pixeles a la vez del mismo color
pub struct Framebuffer {
    pub width: u32,
    pub height: u32,
    pub color_buffer: Image,
    emissive_buffer: Image,
    pub background_color: Color,
    pub current_color: Color,
    depth_buffer: Vec<f32>,
}

/*
 * Para dibujar:
 *  - Escuchar input (usuario)
 *  - Limpiar frame anterior
 *  - Preparar variables / actualizarlas
 *  - Pintar al buffer
 *  - Pintar el buffer a la ventana (swap/flip buffers)
 *  - Esperar (vsync)
 * */
impl Framebuffer {
    pub fn new(width: u32, height: u32) -> Self {
        let color_buffer = Image::gen_image_color(width as i32, height as i32, Color::BLACK);
        let emissive_buffer = Image::gen_image_color(width as i32, height as i32, Color::BLANK);
        Framebuffer {
            width,
            height,
            color_buffer,
            emissive_buffer,
            background_color: Color::BLACK,
            current_color: Color::WHITE,
            depth_buffer: vec![f32::INFINITY; (width * height) as usize],
        }
    }

    pub fn clear(&mut self) {
        self.color_buffer =
            Image::gen_image_color(self.width as i32, self.height as i32, self.background_color);
        self.emissive_buffer =
            Image::gen_image_color(self.width as i32, self.height as i32, Color::BLANK);
        self.depth_buffer.fill(f32::INFINITY);
    }

    pub fn set_pixel(&mut self, x: u32, y: u32) {
        if x < self.width && y < self.height {
            self.color_buffer
                .draw_pixel(x as i32, y as i32, self.current_color);
        }
    }

    pub fn set_pixel_color(&mut self, x: u32, y: u32, color: Color) {
        if x < self.width && y < self.height {
            self.color_buffer.draw_pixel(x as i32, y as i32, color);
        }
    }

    /// Writes a shaded fragment only when it is closer than the fragment that
    /// is already at this pixel.
    pub fn set_fragment(&mut self, x: i32, y: i32, depth: f32, color: Color) {
        if x < 0 || y < 0 || x >= self.width as i32 || y >= self.height as i32 {
            return;
        }

        let index = y as usize * self.width as usize + x as usize;
        if depth < self.depth_buffer[index] {
            self.depth_buffer[index] = depth;
            self.color_buffer.draw_pixel(x, y, color);
            self.emissive_buffer.draw_pixel(x, y, color);
        }
    }

    pub fn apply_bloom(&mut self) {
        let original = self.emissive_buffer.clone();
        let mut wide_glow = original.clone();
        let mut near_glow = original.clone();
        wide_glow.blur_gaussian(31);
        near_glow.blur_gaussian(11);

        let bounds = Rectangle::new(0.0, 0.0, self.width as f32, self.height as f32);
        self.color_buffer.clear_background(self.background_color);
        self.color_buffer
            .draw(&wide_glow, bounds, bounds, Color::new(45, 105, 255, 150));
        self.color_buffer
            .draw(&near_glow, bounds, bounds, Color::new(105, 190, 255, 185));
        self.color_buffer
            .draw(&original, bounds, bounds, Color::WHITE);
    }

    pub fn set_background_color(&mut self, color: Color) {
        self.background_color = color;
    }

    pub fn set_current_color(&mut self, color: Color) {
        self.current_color = color;
    }

    pub fn render_to_file(&self, file_path: &str) {
        self.color_buffer.export_image(file_path);
    }

    pub fn swap_buffers(
        &mut self,
        window: &mut RaylibHandle,
        raylib_thread: &RaylibThread,
        active_shader: &str,
    ) {
        if let Ok(texture) = window.load_texture_from_image(raylib_thread, &self.color_buffer) {
            let win_height = window.get_render_height();
            let win_width = window.get_render_width();

            let mut renderer = window.begin_drawing(raylib_thread);

            // escalar pixeles si framebuffer es más pequeño que la ventana
            let source = Rectangle::new(0.0, 0.0, self.width as f32, self.height as f32);
            let dest = Rectangle::new(0.0, 0.0, win_width as f32, win_height as f32);

            /*
             * parámetros de pro:
             * la textura
             * source_rect: parte de la textura que usa (crop)
             * dest_rect: tamaño a lo que se escalará el source_rect
             * origin: pivote para posicionar/rotar dest_rect
             * rotation
             * tint: color que se le multiplica la imagen
             */
            renderer.draw_texture_pro(
                &texture,
                source,
                dest,
                Vector2::new(0.0, 0.0),
                0.0,
                Color::WHITE,
            );

            let label = format!("Switch shaders (1-3): {active_shader}");
            renderer.draw_text(&label, 10, win_height - 30, 20, Color::RAYWHITE);
        }
    }
}
