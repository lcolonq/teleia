use crate::{context, state, shader, texture, mesh};

use bitflags::bitflags;

bitflags! {
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub struct UberFlags: u32 {
        const TEXTURE_COLOR       = 0b000000001;
        const TEXTURE_NORMAL      = 0b000000010;
        const FLIP_TEXTURE        = 0b000000100;
        const LIGHT_AMBIENT       = 0b000001000;
        const LIGHT_DIR           = 0b000010000;
        const LIGHT_POINT         = 0b000100000;
        const SPRITE              = 0b001000000;
        const EFFECTS             = 0b010000000;
        const YSKEW               = 0b100000000;
    }
}
impl UberFlags {
    fn prelude() -> String {
        let mut s = String::new();
        s += "#version 300 es\nprecision highp float;";
        for (nm, f) in Self::all().iter_names() {
            s += &format!("const int {} = {};\n", nm, f.bits());
        }
        s
    }
    fn set_flags(self, ctx: &context::Context, shader: &shader::Shader) {
        shader.set_i32(ctx, "flags", self.bits() as i32);
    }
}

pub trait Assets {
    type Shader: PartialEq + Eq + Clone + Copy;
    fn shader(&self, i: Self::Shader) -> &shader::Shader;
    type Texture: PartialEq + Eq + Clone + Copy;
    fn texture(&self, i: Self::Texture) -> &texture::Texture;
    type Mesh: PartialEq + Eq + Clone + Copy;
    fn mesh(&self, i: Self::Mesh) -> &mesh::Mesh;
    type Material: PartialEq + Eq + Clone + Copy;
    fn material(&self, i: Self::Material) -> &texture::Material;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ShaderMode { TwoDimension, ThreeDimension, ThreeDimensionOrth }
#[derive(Debug, Clone, Copy)]
enum BoundShader<A: Assets> { None, Uber(UberFlags, ShaderMode), Shader(A::Shader, ShaderMode) }
impl<A: Assets> PartialEq for BoundShader<A> {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::None, Self::None) => true,
            (Self::Uber(sf, sm), Self::Uber(of, om)) => sf == of && sm == om,
            (Self::Shader(ss, sm), Self::Shader(os, om)) => ss == os && sm == om,
            _ => false,
        }
    }
}
#[derive(Debug, Clone, Copy)]
enum BoundTexture<A: Assets> { None, Texture(A::Texture), Material(A::Material) }
impl<A: Assets> PartialEq for BoundTexture<A> {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::None, Self::None) => true,
            (Self::Texture(s), Self::Texture(o)) => s == o,
            (Self::Material(s), Self::Material(o)) => s == o,
            _ => false,
        }
    }
}
pub struct Renderer<A: Assets> {
    pub assets: A,
    shader_uber: shader::Shader,
    shader: BoundShader<A>,
    texture: BoundTexture<A>,
}
impl<A: Assets> Renderer<A> {
    pub fn new<F>(ctx: &context::Context, f: F) -> Self
    where F: FnOnce(&context::Context) -> A {
        let shader_uber = shader::Shader::new_nolib(ctx,
            &format!("{}{}", UberFlags::prelude(), include_str!("assets/shaders/uber/vert.glsl")),
            &format!("{}{}", UberFlags::prelude(), include_str!("assets/shaders/uber/frag.glsl")),
        );
        shader_uber.bind(ctx);
        shader_uber.set_i32(ctx, "texture_normal", 1);
        Self {
            assets: f(ctx),
            shader_uber,
            shader: BoundShader::None,
            texture: BoundTexture::None,
        }
    }
    pub fn font_char_width(&self, st: &state::State) -> f32 { st.font_default.char_width as f32 }
    pub fn font_char_height(&self, st: &state::State) -> f32 { st.font_default.char_height as f32 }
    pub fn bind_texture(&mut self, ctx: &context::Context, _st: &mut state::State, texture: A::Texture) {
        if self.texture != BoundTexture::Texture(texture) {
            self.assets.texture(texture).bind(ctx);
            self.texture = BoundTexture::Texture(texture);
        }
    }
    pub fn bind_material(&mut self, ctx: &context::Context, _st: &mut state::State, mat: A::Material) {
        if self.texture != BoundTexture::Material(mat) {
            self.assets.material(mat).bind(ctx);
            self.texture = BoundTexture::Material(mat);
        }
    }
    fn shader(&self) -> Option<(&shader::Shader, ShaderMode)> {
        match self.shader {
            BoundShader::Uber(_, sm) => Some((&self.shader_uber, sm)),
            BoundShader::Shader(s, sm) => Some((self.assets.shader(s), sm)),
            _ => None,
        }
    }
    fn bind_uber(&mut self,
        ctx: &context::Context, st: &mut state::State,
        flags: UberFlags, mode: ShaderMode,
    ) {
        if let BoundShader::Uber(f, sm) = self.shader {
            if f == flags && sm == mode {
                return;
            }
        }
        match mode {
            ShaderMode::TwoDimension => st.bind_2d(ctx, &self.shader_uber),
            ShaderMode::ThreeDimension => st.bind_3d(ctx, &self.shader_uber),
            ShaderMode::ThreeDimensionOrth => st.bind_3d_orth(ctx, &self.shader_uber),
        }
        flags.set_flags(ctx, &self.shader_uber);
        self.shader = BoundShader::Uber(flags, mode)
    }
    fn bind_shader(&mut self,
        ctx: &context::Context, st: &mut state::State,
        shader: A::Shader, mode: ShaderMode,
    ) {
        if let BoundShader::Shader(s, sm) = self.shader {
            if s == shader && sm == mode {
                return;
            }
        }
        match mode {
            ShaderMode::TwoDimension => st.bind_2d(ctx, &self.assets.shader(shader)),
            ShaderMode::ThreeDimension => st.bind_3d(ctx, &self.assets.shader(shader)),
            ShaderMode::ThreeDimensionOrth => st.bind_3d_orth(ctx, &self.assets.shader(shader)),
        }
        self.shader = BoundShader::Shader(shader, mode)
    }
    pub fn bind_uber_2d(&mut self, ctx: &context::Context, st: &mut state::State, flags: UberFlags) {
        self.bind_uber(ctx, st, flags, ShaderMode::TwoDimension);
    }
    pub fn bind_uber_3d(&mut self, ctx: &context::Context, st: &mut state::State, flags: UberFlags) {
        self.bind_uber(ctx, st, flags, ShaderMode::ThreeDimension);
    }
    pub fn bind_uber_3d_orth(&mut self, ctx: &context::Context, st: &mut state::State, flags: UberFlags) {
        self.bind_uber(ctx, st, flags, ShaderMode::ThreeDimensionOrth);
    }
    pub fn bind_shader_2d(&mut self, ctx: &context::Context, st: &mut state::State, shader: A::Shader) {
        self.bind_shader(ctx, st, shader, ShaderMode::TwoDimension);
    }
    pub fn bind_shader_3d(&mut self, ctx: &context::Context, st: &mut state::State, shader: A::Shader) {
        self.bind_shader(ctx, st, shader, ShaderMode::ThreeDimension);
    }
    pub fn render(&self, ctx: &context::Context, _st: &state::State, mesh: A::Mesh) {
        self.assets.mesh(mesh).render(ctx)
    }
    pub fn render_square(&self, ctx: &context::Context, st: &state::State) {
        st.mesh_square.render(ctx)
    }
    pub fn set_position_2d(&self, ctx: &context::Context, st: &state::State, pos: glam::Vec2, dims: glam::Vec2) {
        if let Some((s, sm)) = self.shader() {
            debug_assert!(sm == ShaderMode::TwoDimension, "attempted to set_position_2d in wrong mode");
            s.set_position_2d(ctx, st, &pos, &dims)
        }
    }
    pub fn set_position_3d(&self, ctx: &context::Context, st: &state::State, pos: glam::Mat4) {
        if let Some((s, sm)) = self.shader() {
            debug_assert!(sm == ShaderMode::ThreeDimension || sm == ShaderMode::ThreeDimensionOrth,
                "attempted to set_position_3d in wrong mode"
            );
            s.set_position_3d(ctx, st, &pos)
        }
    }
    pub fn set_i32(&self, ctx: &context::Context, _st: &state::State, nm: &str, val: i32) {
        if let Some((s, _)) = self.shader() { s.set_i32(ctx, nm, val) }
    }
    pub fn set_f32(&self, ctx: &context::Context, _st: &state::State, nm: &str, val: f32) {
        if let Some((s, _)) = self.shader() { s.set_f32(ctx, nm, val) }
    }
    pub fn set_vec2(&self, ctx: &context::Context, _st: &state::State, nm: &str, val: glam::Vec2) {
        if let Some((s, _)) = self.shader() { s.set_vec2(ctx, nm, &val) }
    }
    pub fn set_vec3(&self, ctx: &context::Context, _st: &state::State, nm: &str, val: glam::Vec3) {
        if let Some((s, _)) = self.shader() { s.set_vec3(ctx, nm, &val) }
    }
    pub fn set_vec4(&self, ctx: &context::Context, _st: &state::State, nm: &str, val: glam::Vec4) {
        if let Some((s, _)) = self.shader() { s.set_vec4(ctx, nm, &val) }
    }
    pub fn set_mat4(&self, ctx: &context::Context, _st: &state::State, nm: &str, val: glam::Mat4) {
        if let Some((s, _)) = self.shader() { s.set_mat4(ctx, nm, &val) }
    }
    pub fn set_texture_offset(&self, ctx: &context::Context, st: &state::State, inc: i32, x: i32, y: i32) {
        let count = inc as f32;
        let ratio = 1.0 / count;
        self.set_vec2(
            ctx, st, "sprite_dims",
            glam::Vec2::new(ratio, ratio),
        );
        self.set_vec2(
            ctx, st, "sprite_offset",
            glam::Vec2::new((x % inc) as f32 * ratio, (y % inc) as f32 * ratio),
        );
    }

    /// Common case: draw the given textured mesh in the world (units are world tiles)
    pub fn textured_mesh_world(&mut self,
        ctx: &context::Context, st: &mut state::State,
        shader: A::Shader,
        texture: A::Texture,
        mesh: A::Mesh,
        pos: glam::Mat4,
    ) {
        self.bind_shader_3d(ctx, st, shader);
        self.bind_texture(ctx, st, texture);
        self.set_position_3d(ctx, st, pos);
        self.render(ctx, st, mesh);
    }

    /// Common case: draw the given color in a rectangle on the screen (units are pixels, pos is top left)
    pub fn color_screen(&mut self,
        ctx: &context::Context, st: &mut state::State,
        color: glam::Vec4,
        pos: glam::Vec2,
        dims: glam::Vec2,
    ) {
        self.bind_uber_2d(ctx, st, UberFlags::empty());
        self.set_vec4(ctx, st, "color", color);
        self.set_position_2d(ctx, st, pos, dims);
        self.render_square(ctx, st);
    }

    /// Common case: draw the given texture on the screen (units are pixels, pos is top left)
    pub fn texture_screen(&mut self,

        ctx: &context::Context, st: &mut state::State,
        texture: A::Texture,
        pos: glam::Vec2,
        dims: glam::Vec2,
    ) {
        self.bind_uber_2d(ctx, st, UberFlags::TEXTURE_COLOR);
        self.bind_texture(ctx, st, texture);
        self.set_position_2d(ctx, st, pos, dims);
        self.render_square(ctx, st);
    }

    pub fn texture_screen_recolor(&mut self,
        ctx: &context::Context, st: &mut state::State,
        texture: A::Texture, hue: f32,
        pos: glam::Vec2,
        dims: glam::Vec2,
    ) {
        self.bind_uber_2d(ctx, st, UberFlags::TEXTURE_COLOR | UberFlags::EFFECTS);
        self.bind_texture(ctx, st, texture);
        self.set_f32(ctx, st, "effect_huescale", 0.0);
        self.set_f32(ctx, st, "effect_hueshift", hue);
        self.set_position_2d(ctx, st, pos, dims);
        self.render_square(ctx, st);
        self.set_f32(ctx, st, "effect_huescale", 1.0);
        self.set_f32(ctx, st, "effect_hueshift", 0.0);
    }

    /// Common case: text in the default font (units are pixels, pos is top left)
    pub fn text_screen(&mut self,
        ctx: &context::Context, st: &mut state::State,
        pos: glam::Vec2,
        s: &str,
    ) {
        // drawing text might bind the shader and texture
        self.shader = BoundShader::None; self.texture = BoundTexture::None;
        st.font_default.render_text(ctx, st, &pos, s);
    }

    /// Common case: text in the default font, with a color (units are pixels, pos is top left)
    pub fn text_colored_screen(&mut self,
        ctx: &context::Context, st: &mut state::State,
        pos: glam::Vec2,
        col: glam::Vec3,
        s: &str,
    ) {
        // drawing text might bind the shader and texture
        self.shader = BoundShader::None; self.texture = BoundTexture::None;
        st.font_default.render_text_helper(ctx, st, &pos, s, &[col]);
    }

    /// Common case: text in the default font (units are pixels, pos is center)
    pub fn text_centered_screen(&mut self,
        ctx: &context::Context, st: &mut state::State,
        pos: glam::Vec2,
        s: &str,
    ) {
        // drawing text might bind the shader and texture
        self.shader = BoundShader::None; self.texture = BoundTexture::None;
        let width = s.len() as f32 * st.font_default.char_width as f32;
        let height = st.font_default.char_height as f32;
        st.font_default.render_text(ctx, st,
            &(pos - glam::Vec2::new((width / 2.0).round(), (height / 2.0).round())),
            s
        );
    }
}
