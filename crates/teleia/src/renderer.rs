use crate::{context, font, mesh, postprocessing, shader, state, texture};

use bitflags::bitflags;

bitflags! {
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub struct UberFlags: u32 {
        const VERTEX_COLOR        = 1 << 0;
        const TEXTURE_COLOR       = 1 << 1;
        const TEXTURE_NORMAL      = 1 << 2;
        const TEXTURE_FLIP        = 1 << 3;
        const LIGHT_AMBIENT       = 1 << 4;
        const LIGHT_DIR           = 1 << 5;
        const LIGHT_POINT         = 1 << 6;
        const RGB_ADD             = 1 << 7;
        const HUE                 = 1 << 8;
        const SPRITE              = 1 << 9;
        const YSKEW               = 1 << 10;
        const OPACITY             = 1 << 11;
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
    type Effect: PartialEq + Eq + Clone + Copy;
    fn effect(&self, i: Self::Effect) -> &postprocessing::Effect;
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

#[must_use]
pub struct ApplyPostprocessingEffect
<'s, 'r, A: Assets> {
    st: &'s mut state::State,
    renderer: &'r mut Renderer<A>,
    effect: A::Effect,
    bindings: [Option<(&'static str, postprocessing::Uniform)>; postprocessing::NUM_BINDINGS],
}
impl<'s, 'r, A: Assets> ApplyPostprocessingEffect<'s, 'r, A> {
    pub fn apply(self) {
        let eff = *self.renderer.assets.effect(self.effect);
        self.st.postprocessing.apply_with_bindings(eff, self.bindings.into_iter().flatten());
    }
    pub fn bind_uniform(mut self, nm: &'static str, val: postprocessing::Uniform) -> Self {
        if let Some(u) = self.bindings.iter_mut().find(|b| b.is_none()) {
            *u = Some((nm, val));
        } else {
            log::warn!("too many bindings on postprocessing effect in renderer!");
        }
        self
    }
    pub fn bind_i32(self, nm: &'static str, x: i32) -> Self {
        self.bind_uniform(nm, postprocessing::Uniform::I32(x))
    }
    pub fn bind_f32(self, nm: &'static str, x: f32) -> Self {
        self.bind_uniform(nm, postprocessing::Uniform::F32(x))
    }
    pub fn bind_vec2(self, nm: &'static str, x: glam::Vec2) -> Self {
        self.bind_uniform(nm, postprocessing::Uniform::Vec2(x))
    }
    pub fn bind_vec3(self, nm: &'static str, x: glam::Vec3) -> Self {
        self.bind_uniform(nm, postprocessing::Uniform::Vec3(x))
    }
    pub fn bind_vec4(self, nm: &'static str, x: glam::Vec4) -> Self {
        self.bind_uniform(nm, postprocessing::Uniform::Vec4(x))
    }
    pub fn bind_mat4(self, nm: &'static str, x: glam::Mat4) -> Self {
        self.bind_uniform(nm, postprocessing::Uniform::Mat4(x))
    }
}

#[must_use]
pub struct RenderMesh<'c, 's, 'r, A: Assets> {
    ctx: &'c context::Context,
    st: &'s mut state::State,
    renderer: &'r mut Renderer<A>,
    mesh: A::Mesh,
    pos: glam::Mat4,
    texture: Option<A::Texture>,
}
impl<'c, 's, 'r, A: Assets> RenderMesh<'c, 's, 'r, A> {
    pub fn render(self) {
        self.renderer.bind_uber_2d(self.ctx, self.st, UberFlags::TEXTURE_COLOR);
        self.renderer.set_position_3d(self.ctx, self.st, self.pos);
        if let Some(texture) = self.texture {
            self.renderer.bind_texture(self.ctx, self.st, texture);
        }
        self.renderer.render(self.ctx, self.st, self.mesh);
    }
}

#[must_use]
pub struct RenderTextureScreen<'c, 's, 'r, A: Assets> {
    ctx: &'c context::Context,
    st: &'s mut state::State,
    renderer: &'r mut Renderer<A>,
    texture: A::Texture,
    pos: glam::Vec2,
    dims: Option<glam::Vec2>,
    rot: Option<glam::Quat>,
    hue: Option<f32>,
}
impl<'c, 's, 'r, A: Assets> RenderTextureScreen<'c, 's, 'r, A> {
    pub fn render(self) {
        self.renderer.bind_uber_2d(self.ctx, self.st, UberFlags::TEXTURE_COLOR | UberFlags::TEXTURE_FLIP);
        self.renderer.bind_texture(self.ctx, self.st, self.texture);
        let dims = if let Some(dims) = self.dims { dims } else {
            let t = self.renderer.assets.texture(self.texture);
            glam::Vec2::new(t.width as f32, t.height as f32)
        };
        if let Some(rot) = self.rot {
            self.renderer.set_position_2d_rotate(self.ctx, self.st, self.pos, dims, rot);
        } else {
            self.renderer.set_position_2d(self.ctx, self.st, self.pos, dims);
        }
        self.renderer.set_vec2(self.ctx, self.st, "texture_flip", glam::Vec2::new(0.0, 1.0));
        if let Some(hue) = self.hue {
            self.renderer.set_f32(self.ctx, self.st, "hue_scale", 0.0);
            self.renderer.set_f32(self.ctx, self.st, "hue_shift", hue);
        }
        self.renderer.render_square(self.ctx, self.st);
    }
    pub fn dimensions(mut self, dims: glam::Vec2) -> Self { self.dims = Some(dims); self }
    pub fn rotation(mut self, rot: glam::Quat) -> Self { self.rot = Some(rot); self }
    pub fn hue(mut self, hue: f32) -> Self { self.hue = Some(hue); self }
}

#[must_use]
pub struct RenderTextScreen<'c, 's, 'r, 'str, 'f, A: Assets> {
    ctx: &'c context::Context,
    st: &'s mut state::State,
    renderer: &'r mut Renderer<A>,
    text: &'str str,
    pos: glam::Vec2,
    font: Option<&'f font::Bitmap>,
    centered: bool,
    col: Option<glam::Vec3>,
    scale: Option<glam::Vec2>,
    offset: Option<glam::Vec2>,
}
impl<'c, 's, 'r, 'str, 'f, A: Assets> RenderTextScreen<'c, 's, 'r, 'str, 'f, A> {
    pub fn render(self) {
        // drawing text might bind the texture
        self.renderer.texture = BoundTexture::None;
        self.renderer.bind_uber_2d(self.ctx, self.st, UberFlags::TEXTURE_COLOR | UberFlags::VERTEX_COLOR);
        let font = if let Some(font) = self.font { font } else { &self.st.font_default };
        let dims = glam::Vec2::new(font.char_width as f32, font.char_height as f32);
        let fpos = if self.centered {
            let width = self.text.len() as f32 * font.char_width as f32;
            self.pos + glam::Vec2::new(
                -dims.x / 2.0 - (width / 2.0).round(),
                font.char_height as f32 / 2.0
            )
        } else {
            self.pos + glam::Vec2::new(-dims.x / 2.0, dims.y / 2.0)
        };
        self.renderer.set_position_2d(self.ctx, self.st, fpos, dims);
        let color: &[glam::Vec3] = if let Some(col) = self.col { &[col] } else { &[] };
        font.render_text_parameterized(self.ctx, self.st, self.text, font::BitmapParams {
            color,
            scale: if let Some(scale) = self.scale { scale } else { glam::Vec2::ONE },
            offset: if let Some(off) = self.offset { off } else { glam::Vec2::ZERO },
        });
    }
    pub fn font(mut self, font: &'f font::Bitmap) -> Self { self.font = Some(font); self }
    pub fn centered(mut self) -> Self { self.centered = true; self }
    pub fn color(mut self, col: glam::Vec3) -> Self { self.col = Some(col); self }
    pub fn scale(mut self, scale: glam::Vec2) -> Self { self.scale = Some(scale); self }
    pub fn offset(mut self, offset: glam::Vec2) -> Self { self.offset = Some(offset); self }
}

pub struct Renderer<A: Assets> {
    pub assets: A,
    shader_uber: shader::Shader,
    shader: BoundShader<A>,
    texture: BoundTexture<A>,
}
impl<A: Assets> Renderer<A> {
    pub fn new<F>(ctx: &context::Context, st: &mut state::State, f: F) -> Self
    where F: FnOnce(&context::Context, &mut state::State) -> A {
        let shader_uber = shader::Shader::new_nolib(ctx,
            &format!("{}{}", UberFlags::prelude(), include_str!("assets/shaders/uber/vert.glsl")),
            &format!("{}{}", UberFlags::prelude(), include_str!("assets/shaders/uber/frag.glsl")),
        );
        shader_uber.bind(ctx);
        shader_uber.set_i32(ctx, "texture_normal", 1);
        Self {
            assets: f(ctx, st),
            shader_uber,
            shader: BoundShader::None,
            texture: BoundTexture::None,
        }
    }
    pub fn unbind_texture(&mut self, _ctx: &context::Context, _st: &mut state::State) {
        self.texture = BoundTexture::None;
    }
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
        if let BoundShader::Uber(f, sm) = self.shader && f == flags && sm == mode { return }
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
        if let BoundShader::Shader(s, sm) = self.shader && s == shader && sm == mode { return }
        match mode {
            ShaderMode::TwoDimension => st.bind_2d(ctx, self.assets.shader(shader)),
            ShaderMode::ThreeDimension => st.bind_3d(ctx, self.assets.shader(shader)),
            ShaderMode::ThreeDimensionOrth => st.bind_3d_orth(ctx, self.assets.shader(shader)),
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
    pub fn set_position_2d_mat(&self, ctx: &context::Context, st: &state::State, pos: glam::Mat4) {
        if let Some((s, sm)) = self.shader() {
            debug_assert!(sm == ShaderMode::TwoDimension, "attempted to set_position_2d in wrong mode");
            s.set_position_2d_mat(ctx, st, &pos);
        }
    }
    pub fn set_position_2d_rotate(&self, ctx: &context::Context, st: &state::State, pos: glam::Vec2, dims: glam::Vec2, rot: glam::Quat) {
        if let Some((s, sm)) = self.shader() {
            debug_assert!(sm == ShaderMode::TwoDimension, "attempted to set_position_2d in wrong mode");
            s.set_position_2d_helper(ctx, st, &pos, &dims, &rot)
        }
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
    pub fn set_texture_offset(&self,
        ctx: &context::Context, st: &state::State,
        xinc: i32, yinc: i32,
        x: i32, y: i32
    ) {
        let xcount = xinc as f32; let xratio = 1.0 / xcount;
        let ycount = yinc as f32; let yratio = 1.0 / ycount;
        self.set_vec2(
            ctx, st, "sprite_dims",
            glam::Vec2::new(xratio, yratio),
        );
        self.set_vec2(
            ctx, st, "sprite_offset",
            glam::Vec2::new((x % xinc) as f32 * xratio, (y % yinc) as f32 * yratio),
        );
    }

    pub fn begin_frame(&mut self,
        ctx: &context::Context, _st: &mut state::State,
        clear_color: glam::Vec4
    ) {
        self.texture = BoundTexture::None;
        self.shader = BoundShader::None;
        ctx.clear_color(clear_color);
        ctx.clear();
    }

    /// Enable the given postprocessing effect for the current frame
    pub fn postprocessing_effect<'s, 'r>(&'r mut self,
        _ctx: &context::Context, st: &'s mut state::State,
        effect: A::Effect,
    ) -> ApplyPostprocessingEffect<'s, 'r, A> {
        ApplyPostprocessingEffect {
            st, renderer: self,
            effect,
            bindings: [const { None }; _],
        }
    }

    /// Common case: draw the given textured mesh in the world (units are world coordinates)
    pub fn mesh_world<'c, 's, 'r>(&'r mut self,
        ctx: &'c context::Context, st: &'s mut state::State,
        pos: glam::Mat4,
        mesh: A::Mesh,
    ) -> RenderMesh<'c, 's, 'r, A> {
        RenderMesh {
            ctx, st, renderer: self,
            mesh,
            pos,
            texture: None,
        }
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
    pub fn texture_screen<'c, 's, 'r>(&'r mut self, ctx: &'c context::Context, st: &'s mut state::State,
        pos: glam::Vec2,
        texture: A::Texture,
    ) -> RenderTextureScreen<'c, 's, 'r, A> {
        RenderTextureScreen {
            ctx, st, renderer: self,
            texture, pos,
            dims: None,
            rot: None,
            hue: None,
        }
    }

    /// Common case: text in the default font (units are pixels, pos is top left)
    pub fn text_screen<'c, 's, 'r, 'str, 'f>(&'r mut self,
        ctx: &'c context::Context, st: &'s mut state::State,
        pos: glam::Vec2,
        text: &'str str,
    ) -> RenderTextScreen<'c, 's, 'r, 'str, 'f, A> {
        RenderTextScreen {
            ctx, st, renderer: self,
            text,
            pos,
            font: None,
            centered: false,
            col: None,
            scale: None,
            offset: None,
        }
    }
}
