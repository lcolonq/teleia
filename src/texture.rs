use glow::HasContext;
use image::EncodableLayout;

use crate::context;

pub struct Texture {
    pub tex: glow::Texture,
}

impl Texture {
    pub fn new(ctx: &context::Context, bytes: &[u8]) -> Self {
        let rgba = image::io::Reader::new(std::io::Cursor::new(bytes))
            .with_guessed_format()
            .expect("failed to guess image format")
            .decode()
            .expect("failed to decode image")
            .into_rgba8();
        let pixels = rgba.as_bytes();
        unsafe {
            let tex = ctx.gl.create_texture().expect("failed to create texture");
            ctx.gl.bind_texture(glow::TEXTURE_2D, Some(tex));
            ctx.gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_WRAP_S, glow::CLAMP_TO_EDGE as i32);
            ctx.gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_WRAP_T, glow::CLAMP_TO_EDGE as i32);
            ctx.gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_MIN_FILTER, glow::NEAREST as i32);
            ctx.gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_MAG_FILTER, glow::NEAREST as i32);
            ctx.gl.tex_image_2d(
                glow::TEXTURE_2D,
                0,
                glow::RGBA as i32,
                rgba.width() as i32,
                rgba.height() as i32,
                0,
                glow::RGBA,
                glow::UNSIGNED_BYTE,
                Some(pixels),
            );
            ctx.gl.generate_mipmap(glow::TEXTURE_2D);

            Self {
                tex,
            }
        }
    }

    pub fn bind(&self, ctx: &context::Context) {
        unsafe {
            ctx.gl.active_texture(glow::TEXTURE0);
            ctx.gl.bind_texture(glow::TEXTURE_2D, Some(self.tex));
        }
    }
}
