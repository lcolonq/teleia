use std::collections::HashMap;

use glow::HasContext;

use crate::{context, mesh};

const COMMON_VERT: &'static str = include_str!("assets/shaders/common/vert.glsl");
const COMMON_FRAG: &'static str = include_str!("assets/shaders/common/frag.glsl");

#[derive(Clone)]
pub struct Shader {
    pub program: glow::Program,
    pub uniforms: std::rc::Rc<HashMap<String, glow::UniformLocation>>
}

impl Shader {
    pub fn new_nolib(ctx: &context::Context, vsrc: &str, fsrc: &str) -> Self {
        unsafe {
            let program = ctx.gl.create_program()
                .expect("cannot create shader program");

            let vert = ctx.gl.create_shader(glow::VERTEX_SHADER)
                .expect("cannot create shader");
            ctx.gl.shader_source(vert, &vsrc);
            ctx.gl.compile_shader(vert);
            if !ctx.gl.get_shader_compile_status(vert) {
                panic!(
                    "failed to compile vertex shader:\n{}",
                    ctx.gl.get_shader_info_log(vert)
                );
            }
            ctx.gl.attach_shader(program, vert);

            let frag = ctx.gl.create_shader(glow::FRAGMENT_SHADER)
                .expect("cannot create shader");
            ctx.gl.shader_source(frag, &fsrc);
            ctx.gl.compile_shader(frag);
            if !ctx.gl.get_shader_compile_status(frag) {
                panic!(
                    "failed to compile fragment shader:\n{}",
                    ctx.gl.get_shader_info_log(frag)
                );
            }
            ctx.gl.attach_shader(program, frag);

            ctx.gl.bind_attrib_location(program, mesh::ATTRIB_VERTEX, "vertex");
            ctx.gl.bind_attrib_location(program, mesh::ATTRIB_NORMAL, "normal");
            ctx.gl.bind_attrib_location(program, mesh::ATTRIB_TEXCOORD, "texcoord");

            ctx.gl.link_program(program);
            if !ctx.gl.get_program_link_status(program) {
                panic!(
                    "failed to link shader program:\n{}",
                    ctx.gl.get_program_info_log(program),
                );
            }

            ctx.gl.detach_shader(program, vert);
            ctx.gl.delete_shader(vert);
            ctx.gl.detach_shader(program, frag);
            ctx.gl.delete_shader(frag);

            ctx.gl.use_program(Some(program));

            let mut uniforms = HashMap::new();
            for index in 0..ctx.gl.get_active_uniforms(program) {
                if let Some(active) = ctx.gl.get_active_uniform(program, index) {
                    if let Some(loc) = ctx.gl.get_uniform_location(program, &active.name) {
                        uniforms.insert(active.name, loc);
                    } else {
                        log::warn!("failed to get location for uniform: {}", active.name);
                    }
                } else {
                        log::warn!("failed to get active uniform for index: {}", index);
                }
            }

            Self {
                program,
                uniforms: std::rc::Rc::new(uniforms),
            }
        }
    }

    pub fn new(ctx: &context::Context, vsrcstr: &str, fsrcstr: &str) -> Self {
        let vsrc = format!("{}\n{}\n", COMMON_VERT, vsrcstr);
        let fsrc = format!("{}\n{}\n", COMMON_FRAG, fsrcstr);
        Self::new_nolib(ctx, &vsrc, &fsrc)
    }

    pub fn set_i32(&self, ctx: &context::Context, name: &str, val: i32) {
        if let Some(loc) = self.uniforms.get(name) {
            unsafe { ctx.gl.uniform_1_i32(Some(loc), val) }
        }
    }

    pub fn set_i32_array(&self, ctx: &context::Context, name: &str, val: &[i32]) {
        if let Some(loc) = self.uniforms.get(name) {
            unsafe {
                ctx.gl.uniform_1_i32_slice(Some(loc), val)
            }
        }
    }

    pub fn set_f32(&self, ctx: &context::Context, name: &str, val: f32) {
        if let Some(loc) = self.uniforms.get(name) {
            unsafe { ctx.gl.uniform_1_f32(Some(loc), val) }
        }
    }
    
    pub fn set_vec2(&self, ctx: &context::Context, name: &str, val: &glam::Vec2) {
        if let Some(loc) = self.uniforms.get(name) {
            unsafe {
                ctx.gl.uniform_2_f32(
                    Some(loc),
                    val.x,
                    val.y,
                );
            }
        }
    }

    pub fn set_vec2_array(&self, ctx: &context::Context, name: &str, val: &[glam::Vec2]) {
        if let Some(loc) = self.uniforms.get(name) {
            let vs: Vec<f32> = val.iter().flat_map(|v| [v.x, v.y]).collect();
            unsafe {
                ctx.gl.uniform_2_f32_slice(
                    Some(loc),
                    &vs,
                );
            }
        }
    }

    pub fn set_vec3(&self, ctx: &context::Context, name: &str, val: &glam::Vec3) {
        if let Some(loc) = self.uniforms.get(name) {
            unsafe {
                ctx.gl.uniform_3_f32(
                    Some(loc),
                    val.x,
                    val.y,
                    val.z,
                );
            }
        }
    }

    pub fn set_vec3_array(&self, ctx: &context::Context, name: &str, val: &[glam::Vec3]) {
        if let Some(loc) = self.uniforms.get(name) {
            let vs: Vec<f32> = val.iter().flat_map(|v| [v.x, v.y, v.z]).collect();
            unsafe {
                ctx.gl.uniform_3_f32_slice(
                    Some(loc),
                    &vs,
                );
            }
        }
    }

    pub fn set_vec4(&self, ctx: &context::Context, name: &str, val: &glam::Vec4) {
        if let Some(loc) = self.uniforms.get(name) {
            unsafe {
                ctx.gl.uniform_4_f32(
                    Some(loc),
                    val.x,
                    val.y,
                    val.z,
                    val.w,
                );
            }
        }
    }

    pub fn set_mat4(&self, ctx: &context::Context, name: &str, val: &glam::Mat4) {
        if let Some(loc) = self.uniforms.get(name) {
            unsafe {
                ctx.gl.uniform_matrix_4_f32_slice(
                    Some(loc),
                    false,
                    &val.to_cols_array(),
                );
            }
        }
    }

    pub fn set_mat4_array(&self, ctx: &context::Context, name: &str, val: &[glam::Mat4]) {
        if let Some(loc) = self.uniforms.get(name) {
            let vs: Vec<f32> = val.iter().flat_map(|m| m.to_cols_array()).collect();
            unsafe {
                ctx.gl.uniform_matrix_4_f32_slice(
                    Some(loc),
                    false,
                    &vs,
                );
            }
        }
    }

    pub fn set_position_3d(&self, ctx: &context::Context, position: &glam::Mat4) {
        self.set_mat4(&ctx, "position", &position);
        self.set_mat4(&ctx, "normal_matrix", &position.inverse().transpose());
    }

    pub fn set_position_2d_helper(&self, ctx: &context::Context, pos: &glam::Vec2, dims: &glam::Vec2, rot: &glam::Quat) {
        let halfwidth = dims.x / 2.0;
        let halfheight = dims.y / 2.0;
        self.set_mat4(
            &ctx, "position",
            &glam::Mat4::from_scale_rotation_translation(
                glam::Vec3::new(halfwidth, halfheight, 1.0),
                rot.clone(),
                glam::Vec3::new(
                    -ctx.render_width / 2.0 + pos.x + halfwidth,
                    ctx.render_height / 2.0 - pos.y - halfheight,
                    0.0,
                ),
            )
        );
    }

    pub fn set_position_2d(&self, ctx: &context::Context, pos: &glam::Vec2, dims: &glam::Vec2) {
        self.set_position_2d_helper(ctx, pos, dims, &glam::Quat::IDENTITY)
    }

    pub fn set_texture_offset(&self, ctx: &context::Context, inc: i32, x: i32, y: i32) {
        let count = inc as f32;
        let ratio = 1.0 / count;
        self.set_vec3(
            ctx, "texture_offset",
            &glam::Vec3::new((x % inc) as f32 * ratio, (y % inc) as f32 * ratio, count)
        );
    }

    pub fn bind(&self, ctx: &context::Context) {
        unsafe {
            ctx.gl.use_program(Some(self.program));
        }
    }
}
