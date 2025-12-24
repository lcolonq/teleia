use glow::HasContext;
use image::EncodableLayout;

use crate::context;

pub struct Texture {
    pub tex: glow::Texture,
}

impl Texture {
    pub fn new_empty(ctx: &context::Context) -> Self {
        unsafe {
            let tex = ctx.gl.create_texture().expect("failed to create texture");
            ctx.gl.bind_texture(glow::TEXTURE_2D, Some(tex));
            ctx.gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_WRAP_S, glow::CLAMP_TO_EDGE as i32);
            ctx.gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_WRAP_T, glow::CLAMP_TO_EDGE as i32);
            ctx.gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_MIN_FILTER, glow::NEAREST as i32);
            ctx.gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_MAG_FILTER, glow::NEAREST as i32);
            Self {
                tex,
            }
        }
    }

    pub fn new(ctx: &context::Context, bytes: &[u8]) -> Self {
        let rgba = image::ImageReader::new(std::io::Cursor::new(bytes))
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

    pub fn upload_rgba8(&self, ctx: &context::Context, width: i32, height: i32, data: &[u8]) {
        unsafe {
            ctx.gl.bind_texture(glow::TEXTURE_2D, Some(self.tex));
            ctx.gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_WRAP_S, glow::CLAMP_TO_EDGE as i32);
            ctx.gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_WRAP_T, glow::CLAMP_TO_EDGE as i32);
            ctx.gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_MIN_FILTER, glow::NEAREST as i32);
            ctx.gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_MAG_FILTER, glow::NEAREST as i32);
            ctx.gl.tex_image_2d(
                glow::TEXTURE_2D,
                0,
                glow::RGBA as i32,
                width,
                height,
                0,
                glow::RGBA,
                glow::UNSIGNED_BYTE,
                Some(data),
            );
            ctx.gl.generate_mipmap(glow::TEXTURE_2D);
        }
    }

    pub fn set_anisotropic_filtering(&self, ctx: &context::Context) {
        unsafe {
            ctx.gl.bind_texture(glow::TEXTURE_2D, Some(self.tex));
            ctx.gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_WRAP_S, glow::REPEAT as i32);
            ctx.gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_WRAP_T, glow::REPEAT as i32);
            ctx.gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_MIN_FILTER, glow::LINEAR_MIPMAP_LINEAR as i32);
            ctx.gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_MAG_FILTER, glow::LINEAR as i32);
            ctx.gl.tex_parameter_f32(glow::TEXTURE_2D, glow::TEXTURE_MAX_ANISOTROPY_EXT, 4.0);
        }
    }

    pub fn bind(&self, ctx: &context::Context) {
        unsafe {
            ctx.gl.active_texture(glow::TEXTURE0);
            ctx.gl.bind_texture(glow::TEXTURE_2D, Some(self.tex));
        }
    }

    pub fn bind_initial(ctx: &context::Context) {
        unsafe {
            ctx.gl.active_texture(glow::TEXTURE0);
            ctx.gl.bind_texture(glow::TEXTURE_2D, None);
        }
    }

    pub fn bind_index(&self, ctx: &context::Context, idx: u32) {
        unsafe {
            ctx.gl.active_texture(glow::TEXTURE0 + idx);
            ctx.gl.bind_texture(glow::TEXTURE_2D, Some(self.tex));
        }
    }
}

pub struct Material {
    pub color: Texture,
    pub normal: Texture,
}
impl Material {
    pub fn new(ctx: &context::Context, color_bytes: &[u8], normal_bytes: &[u8]) -> Self {
        let color = Texture::new(ctx, color_bytes);
        let normal = Texture::new(ctx, normal_bytes);
        color.set_anisotropic_filtering(ctx);
        normal.set_anisotropic_filtering(ctx);
        Self { color, normal }
    }
    pub fn bind(&self, ctx: &context::Context) {
        self.color.bind(ctx);
        self.normal.bind_index(ctx, 1);
    }
}
