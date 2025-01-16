use std::collections::HashMap;

use crate::{context, texture, shader};
use glow::HasContext;

pub const CHAR_WIDTH: i32 = 7;
pub const CHAR_HEIGHT: i32 = 9;
pub const FONT_WIDTH: i32 = 112;
pub const FONT_HEIGHT: i32 = 54;

pub struct Bitmap {
    pub shader: shader::Shader,
    pub font: texture::Texture,
}

impl Bitmap {
    pub fn new(ctx: &context::Context) -> Self {
        let shader = shader::Shader::new_nolib(
            &ctx,
            include_str!("assets/shaders/bitmap/vert.glsl"),
            include_str!("assets/shaders/bitmap/frag.glsl"),
        );
        let font = texture::Texture::new(ctx, include_bytes!("assets/fonts/simple.png"));
        Self {
            shader,
            font,
        }
    }

    pub fn render_text_helper(&self, ctx: &context::Context, pos: &glam::Vec2, text: &str, color: &glam::Vec3) {
        let mut width = 0;
        let mut linewidth = 0;
        let mut height = CHAR_HEIGHT;
        for c in text.chars() {
            if c == '\n' {
                width = width.max(linewidth);
                linewidth = 0;
                height += CHAR_HEIGHT;
            } else {
                linewidth += CHAR_WIDTH; 
            }
        }
        width = width.max(linewidth);

        self.shader.bind(ctx);
        let len = text.len().min(256);
        self.shader.set_i32(ctx, "text_length", len as _);
        let textvals: Vec<i32> = text.as_bytes().into_iter().take(len).map(|b| {
            *b as i32
        }).collect();
        self.shader.set_i32_array(ctx, "text[0]", &textvals);
        self.shader.set_i32(ctx, "char_width", CHAR_WIDTH as _);
        self.shader.set_i32(ctx, "char_height", CHAR_HEIGHT as _);
        self.shader.set_i32(ctx, "font_width", FONT_WIDTH as _);
        self.shader.set_i32(ctx, "font_height", FONT_HEIGHT as _);
        self.shader.set_i32(ctx, "text_width", width as _);
        self.shader.set_i32(ctx, "text_height", height as _);
        self.shader.set_vec3(ctx, "text_color", color as _);
        self.shader.set_mat4(
            ctx, "view",
            &glam::Mat4::from_scale(
                glam::Vec3::new(
                    2.0 / context::RENDER_WIDTH,
                    2.0 / context::RENDER_HEIGHT,
                    1.0,
                ),
            ),
        );
        let halfwidth = width as f32 / 2.0;
        let halfheight = height as f32 / 2.0;
        self.shader.set_mat4(
            ctx, "position",
            &glam::Mat4::from_scale_rotation_translation(
                glam::Vec3::new(halfwidth, halfheight, 1.0),
                glam::Quat::IDENTITY,
                glam::Vec3::new(
                    -context::RENDER_WIDTH / 2.0 + pos.x + halfwidth,
                    context::RENDER_HEIGHT / 2.0 - pos.y - halfheight,
                    0.0,
                ),
            )
        );
        self.font.bind(ctx);
        ctx.render_no_geometry();
    }

    pub fn render_text(&self, ctx: &context::Context, pos: &glam::Vec2, text: &str) {
        self.render_text_helper(ctx, pos, text, &glam::Vec3::new(1.0, 1.0, 1.0));
    }
}

pub struct AtlasInfo {
    pub pos: usize,
}

pub struct TrueType {
    pub shader: shader::Shader,
    pub font: fontdue::Font,
    pub atlas: texture::Texture,
    pub atlaswidth: usize,
    pub cellwidth: usize,
    pub cellheight: usize,
    pub info: HashMap<char, AtlasInfo>,
}

impl TrueType {
    pub fn new(ctx: &context::Context) -> Self {
        let shader = shader::Shader::new_nolib(
            &ctx,
            include_str!("assets/shaders/truetype/vert.glsl"),
            include_str!("assets/shaders/truetype/frag.glsl"),
        );
        let size = 20.0;
        let font = fontdue::Font::from_bytes(
            include_bytes!("assets/fonts/ComicNeue-Regular.ttf") as &[u8],
            fontdue::FontSettings::default(),
        ).expect("failed to load font");
        let mut chardata = HashMap::new();
        for ci in 0..128 {
            if let Some(c) = char::from_u32(ci) {
                if !c.is_ascii_graphic() { continue; }
                let res = font.rasterize(c, size);
                chardata.insert(c, res);
            }
        }
        let mut cellwidth = 0;
        let mut cellbase = 0;
        let mut cellextra = 0;
        for (_, (m, _)) in &chardata {
            if m.width > cellwidth { cellwidth = m.width }
            if m.height > cellbase { cellbase = m.height }
            let extra = (-m.ymin.min(0)) as usize;
            if extra > cellextra { cellextra = extra }
        }
        let mut cellheight = cellbase + cellextra;
        cellwidth = cellwidth.next_power_of_two();
        cellheight = cellheight.next_power_of_two();
        let atlaswidth = (chardata.len() * cellwidth).next_power_of_two();
        let mut info = HashMap::new();
        let mut atlas_bmp: Vec<u8> = vec![0; atlaswidth * cellheight];
        for (i, (c, (m, bmp))) in chardata.iter().enumerate() {
            let by = ((cellbase as i32) - (m.height as i32) - m.ymin) as usize;
            let bx = cellwidth * i;
            info.insert(*c, AtlasInfo {
                pos: cellwidth * i,
            });
            for x in 0..m.width {
                for y in 0..m.height {
                    atlas_bmp[bx + x + (by + y) * atlaswidth] = bmp[x + y * m.width];
                }
            }
        }
        let atlas = texture::Texture::new_empty(ctx);
        unsafe {
            ctx.gl.bind_texture(glow::TEXTURE_2D, Some(atlas.tex));
            ctx.gl.pixel_store_i32(glow::UNPACK_ALIGNMENT, 1);
            ctx.gl.tex_image_2d(
                glow::TEXTURE_2D,
                0,
                glow::R8 as i32,
                atlaswidth as i32,
                cellheight as i32,
                0,
                glow::RED,
                glow::UNSIGNED_BYTE,
                Some(&atlas_bmp),
            );
            ctx.gl.generate_mipmap(glow::TEXTURE_2D);
        }
        Self { shader, font, atlas, atlaswidth, cellwidth, cellheight, info, }
    }

    pub fn render_text(&self, ctx: &context::Context, pos: &glam::Vec2, text: &str) {
        self.shader.bind(ctx);
        unsafe {
            ctx.gl.active_texture(glow::TEXTURE0);
            ctx.gl.bind_texture(glow::TEXTURE_2D, Some(self.atlas.tex));
        }
        self.shader.set_mat4(
            ctx, "view",
            &glam::Mat4::from_scale(
                glam::Vec3::new(
                    2.0 / context::RENDER_WIDTH,
                    2.0 / context::RENDER_HEIGHT,
                    1.0,
                ),
            ),
        );
        let width = text.len() * self.cellwidth;
        self.shader.set_mat4(
            ctx, "position",
            &glam::Mat4::from_scale_rotation_translation(
                glam::Vec3::new(width as f32 / 2.0, self.cellheight as f32 / 2.0, 1.0),
                glam::Quat::IDENTITY,
                glam::Vec3::new(
                    -context::RENDER_WIDTH / 2.0 + pos.x + width as f32 / 2.0,
                    context::RENDER_HEIGHT / 2.0 - pos.y - self.cellheight as f32 / 2.0,
                    0.0,
                ),
            )
        );
        let len = text.len().min(256);
        let textvals: Vec<i32> = text.chars().take(len).map(|c| {
            if let Some(i) = self.info.get(&c) {
                i.pos as i32
            } else {
                0
            }
        }).collect();
        self.shader.set_i32_array(ctx, "text[0]", &textvals);
        self.shader.set_i32(ctx, "atlas_width", self.atlaswidth as i32);
        self.shader.set_i32(ctx, "cell_width", self.cellwidth as i32);
        self.shader.set_i32(ctx, "text_width", width as i32);
        ctx.render_no_geometry();
    }
}
