use crate::{context, framebuffer, shader, state, utils};

const NUM_BINDINGS: usize = 5;

#[derive(Clone, Copy)]
pub struct Effect(usize);

pub enum Uniform {
    I32(i32),
    F32(f32),
    Vec2(glam::Vec2),
    Vec3(glam::Vec3),
    Vec4(glam::Vec4),
    Mat4(glam::Mat4),
}
struct Stage {
    effect: Effect,
    bindings: [Option<(&'static str, Uniform)>; NUM_BINDINGS],
}

pub struct Pipeline {
    effects: Vec<shader::Shader>,
    upscale: shader::Shader,
    stages: Vec<Stage>,
    framebuffer: framebuffer::Framebuffer,
}
impl Pipeline {
    pub fn new(ctx: &context::Context) -> Self {
        Self {
            effects: Vec::new(),
            upscale: shader::Shader::new_nolib(
                ctx,
                include_str!("assets/shaders/common/postprocessing_vert.glsl"),
                include_str!("assets/shaders/scale/frag.glsl"),
            ),
            stages: Vec::new(),
            framebuffer: framebuffer::Framebuffer::new(
                ctx,
                &glam::Vec2::new(ctx.render_width, ctx.render_height),
                &glam::Vec2::new(0.0, 0.0),
            ),
        }
    }
    /// Create a new postprocessing effect from a fragment shader.
    pub fn effect(&mut self, ctx: &context::Context, fsrc: &str) -> utils::Erm<Effect> {
        let shader = shader::Shader::new_nolib(ctx,
            include_str!("assets/shaders/common/postprocessing_vert.glsl"),
            fsrc
        );
        let idx = self.effects.len();
        self.effects.push(shader);
        Ok(Effect(idx))
    }
    /// Apply a postprocessing effect to the current frame.
    pub fn apply(&mut self, effect: Effect) {
        self.stages.push(Stage { effect, bindings: Default::default() });
    }
    /// Apply a postprocessing effect and set extra uniforms.
    pub fn apply_with_bindings<I>(&mut self, effect: Effect, bs: I)
    where I: IntoIterator<Item=(&'static str, Uniform)> {
        let mut bindings: [_; _] = Default::default();
        for (si, b) in bs.into_iter().enumerate() {
            if si < NUM_BINDINGS {
                bindings[si] = Some(b);
            } else {
                log::warn!("too many bindings on postprocessing effect!");
                break;
            }
        }
        self.stages.push(Stage { effect, bindings });
    }
    /// Upscale the image in the state's rendering framebuffer to the screen,
    /// applying postprocessing effects in between.
    pub fn render(&self, ctx: &context::Context, st: &state::State) -> utils::Erm<()> {
        let mut src = &st.render_framebuffer;
        let mut dst = &self.framebuffer;
        for Stage { effect: Effect(e), bindings } in self.stages.iter() {
            let s = &self.effects[*e];
            dst.bind(ctx);
            ctx.clear();
            s.bind(ctx);
            s.set_f32(ctx, "time", st.tick as f32 + (st.tick % 60) as f32 / 60.0);
            for (nm, val) in bindings.iter().flatten(){
                match val {
                    Uniform::I32(x) => s.set_i32(ctx, nm, *x),
                    Uniform::F32(x) => s.set_f32(ctx, nm, *x),
                    Uniform::Vec2(x) => s.set_vec2(ctx, nm, x),
                    Uniform::Vec3(x) => s.set_vec3(ctx, nm, x),
                    Uniform::Vec4(x) => s.set_vec4(ctx, nm, x),
                    Uniform::Mat4(x) => s.set_mat4(ctx, nm, x),
                }
            }
            src.bind_texture(ctx);
            ctx.render_no_geometry();
            std::mem::swap(&mut src, &mut dst);
        }
        st.screen.bind(ctx);
        ctx.clear_color(
            if ctx.options.contains(crate::Options::OVERLAY) {
                glam::Vec4::new(0.0, 0.0, 0.0, 0.0)
            } else {
                glam::Vec4::new(0.0, 0.0, 0.0, 1.0)
            }
        );
        ctx.clear();
        self.upscale.bind(ctx);
        src.bind_texture(ctx);
        ctx.render_no_geometry();
        Ok(())
    }
    /// Conclude the current frame, and clear all staged effects.
    pub fn finish(&mut self) {
        self.stages.clear();
    }
}
