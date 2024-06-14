use glow::HasContext;

use crate::context;

pub struct ShadowBuffer {
    pub fbo: glow::Framebuffer,
    pub depth_tex: glow::Texture,
    pub width: i32,
    pub height: i32,
}

impl ShadowBuffer {
    pub fn new(ctx: &context::Context, w: i32, h: i32) -> Self {
        unsafe {
            // generate and bind FBO
            let fbo = ctx.gl.create_framebuffer().expect("failed to create framebuffer");
            ctx.gl.bind_framebuffer(glow::FRAMEBUFFER, Some(fbo));

            // generate and attach depth buffer
            let depth_tex = ctx.gl.create_texture().expect("failed to create texture");
            ctx.gl.bind_texture(glow::TEXTURE_2D, Some(depth_tex));
            ctx.gl.tex_image_2d(
                glow::TEXTURE_2D,
                0,
                glow::DEPTH_COMPONENT as i32,
                w,
                h,
                0,
                glow::DEPTH_COMPONENT,
                glow::FLOAT,
                None,
            );
            ctx.gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_MIN_FILTER, glow::LINEAR as i32);
            ctx.gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_MAG_FILTER, glow::LINEAR as i32);
            ctx.gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_WRAP_S, glow::CLAMP_TO_BORDER as i32);
            ctx.gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_WRAP_T, glow::CLAMP_TO_BORDER as i32);
            ctx.gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_COMPARE_MODE, glow::COMPARE_REF_TO_TEXTURE as i32);
            ctx.gl.tex_parameter_f32_slice(glow::TEXTURE_2D, glow::TEXTURE_BORDER_COLOR, &[1.0, 1.0, 1.0, 1.0]);
            ctx.gl.framebuffer_texture_2d(glow::FRAMEBUFFER, glow::DEPTH_ATTACHMENT, glow::TEXTURE_2D, Some(depth_tex), 0);
            ctx.gl.draw_buffer(glow::NONE);
            ctx.gl.read_buffer(glow::NONE);

            let status = ctx.gl.check_framebuffer_status(glow::FRAMEBUFFER);
            if status != glow::FRAMEBUFFER_COMPLETE {
                panic!("error initializing framebuffer: {}", status);
            }
            Self {
                fbo,
                depth_tex,
                width: w,
                height: h,
            }
        }
    }

    pub fn bind(&self, ctx: &context::Context) {
        unsafe {
            ctx.gl.bind_framebuffer(glow::FRAMEBUFFER, Some(self.fbo));
            ctx.gl.viewport(0, 0, self.width as _, self.height as _);
            ctx.gl.clear(glow::DEPTH_BUFFER_BIT);

        }
    }
}

pub struct ShadowBuffer3D {
    pub fbo: glow::Framebuffer,
    pub depth_cubemap: glow::Texture,
    pub width: i32,
    pub height: i32,
}

impl ShadowBuffer3D {
    pub fn new(ctx: &context::Context, w: i32, h: i32) -> Self {
        unsafe {
            // generate and bind FBO
            let fbo = ctx.gl.create_framebuffer().expect("failed to create framebuffer");
            ctx.gl.bind_framebuffer(glow::FRAMEBUFFER, Some(fbo));

            // generate and attach depth buffer
            let depth_cubemap = ctx.gl.create_texture().expect("failed to create texture");
            ctx.gl.bind_texture(glow::TEXTURE_CUBE_MAP, Some(depth_cubemap));
            for i in 0..6 {
                ctx.gl.tex_image_2d(
                    glow::TEXTURE_CUBE_MAP_POSITIVE_X + i,
                    0,
                    glow::DEPTH_COMPONENT as i32,
                    w,
                    h,
                    0,
                    glow::DEPTH_COMPONENT,
                    glow::FLOAT,
                    None,
                );
                ctx.gl.tex_parameter_i32(glow::TEXTURE_CUBE_MAP, glow::TEXTURE_MIN_FILTER, glow::LINEAR as i32);
                ctx.gl.tex_parameter_i32(glow::TEXTURE_CUBE_MAP, glow::TEXTURE_MAG_FILTER, glow::LINEAR as i32);
                ctx.gl.tex_parameter_i32(glow::TEXTURE_CUBE_MAP, glow::TEXTURE_WRAP_S, glow::CLAMP_TO_EDGE as i32);
                ctx.gl.tex_parameter_i32(glow::TEXTURE_CUBE_MAP, glow::TEXTURE_WRAP_T, glow::CLAMP_TO_EDGE as i32);
                ctx.gl.tex_parameter_i32(glow::TEXTURE_CUBE_MAP, glow::TEXTURE_WRAP_R, glow::CLAMP_TO_EDGE as i32);
            }

            ctx.gl.framebuffer_texture(glow::FRAMEBUFFER, glow::DEPTH_ATTACHMENT, Some(depth_cubemap), 0);
            ctx.gl.draw_buffer(glow::NONE);
            ctx.gl.read_buffer(glow::NONE);

            let status = ctx.gl.check_framebuffer_status(glow::FRAMEBUFFER);
            if status != glow::FRAMEBUFFER_COMPLETE {
                panic!("error initializing framebuffer: {}", status);
            }
            Self {
                fbo,
                depth_cubemap,
                width: w,
                height: h,
            }
        }
    }

    pub fn bind(&self, ctx: &context::Context) {
        unsafe {
            ctx.gl.bind_framebuffer(glow::FRAMEBUFFER, Some(self.fbo));
            ctx.gl.viewport(0, 0, self.width as _, self.height as _);
            ctx.gl.clear(glow::DEPTH_BUFFER_BIT);

        }
    }
}
