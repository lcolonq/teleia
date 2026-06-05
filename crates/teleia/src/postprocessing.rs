use crate::{context, framebuffer, shader, state, utils};

#[derive(Clone, Copy)]
pub struct Effect(usize);
pub struct Pipeline {
    pub effects: Vec<shader::Shader>,
    pub upscale: shader::Shader,
    pub stages: Vec<Effect>,
    pub framebuffer: framebuffer::Framebuffer,
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
        self.stages.push(effect);
    }
    /// Upscale the image in the state's rendering framebuffer to the screen,
    /// applying postprocessing effects in between.
    pub fn render(&self, ctx: &context::Context, st: &state::State) -> utils::Erm<()> {
        let mut src = &st.render_framebuffer;
        let mut dst = &self.framebuffer;
        for Effect(e) in self.stages.iter() {
            let s = &self.effects[*e];
            dst.bind(ctx);
            ctx.clear();
            s.bind(ctx);
            s.set_f32(ctx, "time", st.tick as f32 + (st.tick % 60) as f32 / 60.0);
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
