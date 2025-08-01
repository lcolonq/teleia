use glow::HasContext;

use crate::Options;
use crate::context;

pub struct Framebuffer {
    pub tex: Option<glow::Texture>,
    pub fbo: Option<glow::Framebuffer>,
    pub dims: glam::Vec2,
    pub offsets: glam::Vec2,
}

impl Framebuffer {
    pub fn screen(ctx: &context::Context) -> Self {
        #[cfg(target_arch = "wasm32")]
        let (windoww, windowh): (f32, f32) = if !ctx.options.contains(Options::NORESIZE) {
            ctx.window.inner_size().into()
        } else {
            (ctx.render_width, ctx.render_height)
        };
        #[cfg(not(target_arch = "wasm32"))]
        let (windoww, windowh) = {
            let (w, h) = ctx.window.borrow().get_size();
            (w as f32, h as f32)
        };
        let ratio = ctx.compute_upscale(windoww as _, windowh as _) as f32;
        let upscalew = ctx.render_width * ratio;
        let upscaleh = ctx.render_height * ratio;
        let offsetx = (windoww - upscalew) / 2.0;
        let offsety = (windowh - upscaleh) / 2.0;
        log::info!("resize window: {:?}, upscale: {:?}, offset: {:?}", (windoww, windowh), (upscalew, upscaleh), (offsetx, offsety));
        Self {
            tex: None,
            fbo: None,
            dims: glam::Vec2::new(upscalew, upscaleh),
            offsets: glam::Vec2::new(offsetx, offsety),
        }
    }

    pub fn new(ctx: &context::Context, dims: &glam::Vec2, offsets: &glam::Vec2) -> Self {
        unsafe {
            let fbo = ctx.gl.create_framebuffer()
                .expect("failed to create framebuffer");
            ctx.gl.bind_framebuffer(glow::FRAMEBUFFER, Some(fbo));

            let depth_buffer = ctx.gl.create_renderbuffer()
                .expect("failed to create depth buffer");
            ctx.gl.bind_renderbuffer(glow::RENDERBUFFER, Some(depth_buffer));
            ctx.gl.renderbuffer_storage(glow::RENDERBUFFER, glow::DEPTH_COMPONENT32F, dims.x as _, dims.y as _);
            ctx.gl.framebuffer_renderbuffer(glow::FRAMEBUFFER, glow::DEPTH_ATTACHMENT, glow::RENDERBUFFER, Some(depth_buffer));

            let stencil_buffer = ctx.gl.create_renderbuffer()
                .expect("failed to create stencil buffer");
            ctx.gl.bind_renderbuffer(glow::RENDERBUFFER, Some(stencil_buffer));
            ctx.gl.renderbuffer_storage(glow::RENDERBUFFER, glow::DEPTH_STENCIL, dims.x as _, dims.y as _);
            ctx.gl.framebuffer_renderbuffer(glow::FRAMEBUFFER, glow::DEPTH_STENCIL_ATTACHMENT, glow::RENDERBUFFER, Some(stencil_buffer));

            let tex = ctx.gl.create_texture()
                .expect("failed to create framebuffer texture");
            ctx.gl.bind_texture(glow::TEXTURE_2D, Some(tex));
            ctx.gl.tex_image_2d(
                glow::TEXTURE_2D,
                0,
                glow::RGBA as _,
                dims.x as _,
                dims.y as _,
                0,
                glow::RGBA,
                glow::UNSIGNED_BYTE,
                None,
            );
            ctx.gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_WRAP_S, glow::CLAMP_TO_EDGE as _);
            ctx.gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_WRAP_T, glow::CLAMP_TO_EDGE as _);
            ctx.gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_MIN_FILTER, glow::NEAREST as _);
            ctx.gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_MAG_FILTER, glow::NEAREST as _);
            ctx.gl.framebuffer_texture_2d(glow::FRAMEBUFFER, glow::COLOR_ATTACHMENT0, glow::TEXTURE_2D, Some(tex), 0);
            ctx.gl.draw_buffer(glow::COLOR_ATTACHMENT0);

            let status = ctx.gl.check_framebuffer_status(glow::FRAMEBUFFER);
            if status != glow::FRAMEBUFFER_COMPLETE {
                panic!("error initializing framebuffer:\n{}", status);
            }

            Self {
                tex: Some(tex),
                fbo: Some(fbo),
                dims: dims.clone(),
                offsets: offsets.clone(),
            }
        }
    }

    pub fn bind_texture(&self, ctx: &context::Context) {
        unsafe {
            ctx.gl.active_texture(glow::TEXTURE0);
            ctx.gl.bind_texture(glow::TEXTURE_2D, self.tex);
        }
    }

    pub fn bind(&self, ctx: &context::Context) {
        unsafe {
            ctx.gl.bind_framebuffer(glow::FRAMEBUFFER, self.fbo);
            ctx.gl.viewport(
                self.offsets.x as _,
                self.offsets.y as _,
                self.dims.x as _,
                self.dims.y as _,
            );
        }
    }

    pub fn blit(&self, ctx: &context::Context, dest: &Self, pos: &glam::Vec2, scale: &glam::Vec2) {
        unsafe {
            ctx.gl.bind_framebuffer(glow::READ_FRAMEBUFFER, self.fbo);
            ctx.gl.bind_framebuffer(glow::DRAW_FRAMEBUFFER, dest.fbo);
            ctx.gl.blit_framebuffer(
                0, 0, self.dims.x as _, self.dims.y as _,
                pos.x as _, pos.y as _, (pos.x + scale.x) as _, (pos.y + scale.y) as _,
                glow::COLOR_BUFFER_BIT, glow::NEAREST
            );
        }
    }

    pub fn get_pixels(&self, ctx: &context::Context, buf: &mut [glam::Vec3]) {
        let w = self.dims.x as usize;
        let h = self.dims.y as usize;
        let tmp = vec![glam::Vec3::default(); w * h];
        unsafe {
            ctx.gl.bind_texture(glow::TEXTURE_2D, self.tex);
            ctx.gl.get_tex_image(
                glow::TEXTURE_2D, 0, glow::RGB, glow::FLOAT,
                glow::PixelPackData::Slice(
                    std::slice::from_raw_parts_mut(
                        tmp.as_ptr() as *mut u8,
                        tmp.len() * std::mem::size_of::<glam::Vec3>()
                    )
                ),
            );
        }
        for y in 0..h {
            for x in 0..w {
                buf[x + (h - 1 - y) * w] = tmp[x + y * w];
            }
        }
    }

    pub fn get_pixels_raw(&self, ctx: &context::Context, buf: &mut [u8]) {
        unsafe {
            ctx.gl.bind_texture(glow::TEXTURE_2D, self.tex);
            ctx.gl.get_tex_image(
                glow::TEXTURE_2D, 0, glow::RGBA, glow::UNSIGNED_BYTE,
                glow::PixelPackData::Slice(buf),
            );
        }
    }
}
