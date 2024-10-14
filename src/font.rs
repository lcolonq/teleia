use crate::{context, texture, shader};

pub const CHAR_WIDTH: i32 = 7;
pub const CHAR_HEIGHT: i32 = 9;
pub const FONT_WIDTH: i32 = 112;
pub const FONT_HEIGHT: i32 = 54;

pub struct Font {
    pub shader: shader::Shader,
    pub font: texture::Texture,
}

impl Font {
    pub fn new(ctx: &context::Context) -> Self {
        let shader = shader::Shader::new_nolib(
            &ctx,
            include_str!("assets/shaders/text/vert.glsl"),
            include_str!("assets/shaders/text/frag.glsl"),
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
