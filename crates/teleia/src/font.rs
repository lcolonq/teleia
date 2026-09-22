use crate::{context, mesh, shader, state, texture, utils};
use glow::HasContext;

pub struct BitmapParams<'color> {
    pub color: &'color [glam::Vec4],
    pub scale: glam::Vec2,
    pub offset: glam::Vec2,
}
impl<'color> Default for BitmapParams<'color> {
    fn default() -> Self {
        Self {
            color: &[],
            scale: glam::Vec2::ONE,
            offset: glam::Vec2::ZERO,
        }
    }
}
pub struct Bitmap {
    pub char_width: i32,
    pub char_height: i32,
    pub font_width: i32,
    pub font_height: i32,
    pub font: texture::Texture,
    pub vao: glow::VertexArray,
    pub vertex_buf: glow::Buffer,
    pub texcoords_buf: glow::Buffer,
    pub colors_buf: glow::Buffer,
    pub index_buf: glow::Buffer,
}

impl Bitmap {
    pub fn from_image(
        ctx: &context::Context,
        char_width: i32, char_height: i32,
        font_width: i32, font_height: i32,
        data: &[u8],
    ) -> Self {
        let font = texture::Texture::new(ctx, data);
        unsafe {
            let vao = ctx.gl.create_vertex_array().expect("failed to initialize vao");
            ctx.gl.bind_vertex_array(Some(vao));
            let vertex_buf = ctx.gl.create_buffer().expect("failed to create buffer object");
            ctx.gl.bind_buffer(glow::ARRAY_BUFFER, Some(vertex_buf));
            ctx.gl.vertex_attrib_pointer_f32(mesh::ATTRIB_VERTEX, 2, glow::FLOAT, false, 0, 0);
            ctx.gl.enable_vertex_attrib_array(mesh::ATTRIB_VERTEX);
            let texcoords_buf = ctx.gl.create_buffer().expect("failed to create buffer object");
            ctx.gl.bind_buffer(glow::ARRAY_BUFFER, Some(texcoords_buf));
            ctx.gl.vertex_attrib_pointer_f32(mesh::ATTRIB_TEXCOORD, 2, glow::FLOAT, false, 0, 0);
            ctx.gl.enable_vertex_attrib_array(mesh::ATTRIB_TEXCOORD);
            let colors_buf = ctx.gl.create_buffer().expect("failed to create buffer object");
            ctx.gl.bind_buffer(glow::ARRAY_BUFFER, Some(colors_buf));
            ctx.gl.vertex_attrib_pointer_f32(mesh::ATTRIB_COLOR, 4, glow::FLOAT, false, 0, 0);
            ctx.gl.enable_vertex_attrib_array(mesh::ATTRIB_COLOR);
            let index_buf = ctx.gl.create_buffer().expect("failed to create buffer object");
            ctx.gl.bind_buffer(glow::ELEMENT_ARRAY_BUFFER, Some(index_buf));
            Self {
                char_width, char_height,
                font_width, font_height,
                font,
                vao,
                vertex_buf,
                texcoords_buf,
                colors_buf,
                index_buf,
            }
        }
    }

    pub fn default(ctx: &context::Context) -> Self {
        Self::from_image(ctx, 7, 9, 112, 54, include_bytes!("assets/fonts/default.png"))
    }

    pub fn small(ctx: &context::Context) -> Self {
        Self::from_image(ctx, 6, 7, 96, 42, include_bytes!("assets/fonts/small.png"))
    }

    pub fn render_text_parameterized(&self,
        ctx: &context::Context, _st: &state::State,
        text: &str,
        params: BitmapParams,
    ) {
        let mut cur = params.offset;
        let mut vertices = Vec::new();
        let mut texcoords = Vec::new();
        let mut colors = Vec::new();
        let mut indices = Vec::new();
        let cwidth = self.char_width as f32 / self.font_width as f32;
        let cheight = self.char_height as f32 / self.font_height as f32;
        let sdims = glam::Vec2::new(2.0, 2.0) * params.scale;
        let row_len = self.font_width as u32 / self.char_width as u32;
        for (i, c) in text.chars().enumerate() {
            if c == '\n' {
                cur.x = 0.0;
                cur.y -= sdims.y;
            } else {
                let idx = vertices.len() as u32;
                vertices.push(cur);
                vertices.push(cur + glam::Vec2::new(sdims.x, 0.0));
                vertices.push(cur + glam::Vec2::new(sdims.x, sdims.y));
                vertices.push(cur + glam::Vec2::new(0.0, sdims.y));
                let cidx = c as u32 - ' ' as u32;
                let col = cidx % row_len;
                let row = cidx / row_len;
                let tcbase = glam::Vec2::new(col as f32 * cwidth, row as f32 * cheight);
                texcoords.push(tcbase + glam::Vec2::new(0.0, cheight));
                texcoords.push(tcbase + glam::Vec2::new(cwidth, cheight));
                texcoords.push(tcbase + glam::Vec2::new(cwidth, 0.0));
                texcoords.push(tcbase);
                let c = if let Some(c) = params.color.get(if params.color.is_empty() { 0 } else { i % params.color.len() }) {
                    *c
                } else {
                    glam::Vec4::new(1.0, 1.0, 1.0, 1.0)
                };
                colors.push(c); colors.push(c); colors.push(c); colors.push(c);
                indices.push(idx); indices.push(idx + 1); indices.push(idx + 2);
                indices.push(idx); indices.push(idx + 3); indices.push(idx + 2);
                cur.x += sdims.x; 
            }
        }
        let index_bytes: Vec<u8> = indices.iter().flat_map(|x| x.to_ne_bytes()).collect();
        self.font.bind(ctx);
        unsafe {
            ctx.gl.bind_vertex_array(Some(self.vao));
            ctx.gl.bind_buffer(glow::ARRAY_BUFFER, Some(self.vertex_buf));
            ctx.gl.buffer_data_u8_slice(
                glow::ARRAY_BUFFER,
                std::slice::from_raw_parts(
                    vertices.as_ptr() as _,
                    vertices.len() * std::mem::size_of::<f32>() * 2,
                ),
                glow::STATIC_DRAW,
            );
            ctx.gl.bind_buffer(glow::ARRAY_BUFFER, Some(self.texcoords_buf));
            ctx.gl.buffer_data_u8_slice(
                glow::ARRAY_BUFFER,
                std::slice::from_raw_parts(
                    texcoords.as_ptr() as _,
                    texcoords.len() * std::mem::size_of::<f32>() * 2,
                ),
                glow::STATIC_DRAW,
            );
            ctx.gl.bind_buffer(glow::ARRAY_BUFFER, Some(self.colors_buf));
            ctx.gl.buffer_data_u8_slice(
                glow::ARRAY_BUFFER,
                std::slice::from_raw_parts(
                    colors.as_ptr() as _,
                    colors.len() * std::mem::size_of::<f32>() * 4,
                ),
                glow::STATIC_DRAW,
            );
            ctx.gl.bind_buffer(glow::ELEMENT_ARRAY_BUFFER, Some(self.index_buf));
            ctx.gl.buffer_data_u8_slice(
                glow::ELEMENT_ARRAY_BUFFER,
                &index_bytes,
                glow::STATIC_DRAW,
            );
            ctx.gl.draw_elements(glow::TRIANGLES, indices.len() as _, glow::UNSIGNED_INT, 0);
        }
    }

    pub fn render_text_helper(&self,
        ctx: &context::Context, st: &state::State,
        text: &str, color: &[glam::Vec4]
    ) {
        self.render_text_parameterized(
            ctx, st,
            text,
            BitmapParams {
                color,
                scale: glam::Vec2::ONE,
                offset: glam::Vec2::ZERO,
            }
        )
    }

    pub fn render_text(&self,
        ctx: &context::Context, st: &state::State,
        text: &str
    ) {
        self.render_text_helper(ctx, st, text, &[]);
    }

    pub fn render_text_at(&self,
        ctx: &context::Context, st: &state::State,
        pos: glam::Vec2, text: &str,
        params: BitmapParams,
    ) {
        st.bind_2d(ctx, &st.shader_text_bitmap);
        let dims = glam::Vec2::new(self.char_width as f32, self.char_height as f32);
        let fpos = pos + glam::Vec2::new(-dims.x / 2.0, dims.y / 2.0);
        st.shader_text_bitmap.set_position_text_bitmap(ctx, st, self, &fpos);
        self.render_text_parameterized(ctx, st, text, params);
    }
}

struct GlyphOutliner {
    pos: (f32, f32),
    data: Vec<((f32, f32), (f32, f32), (f32, f32))>,
}
impl GlyphOutliner {
    fn new() -> Self {
        Self {
            pos: (0.0, 0.0),
            data: Vec::new(),
        }
    }
    fn add_curve(&mut self, p1: (f32, f32), p2: (f32, f32), p3: (f32, f32)) {
        self.data.push(((p1.0, p1.1), (p2.0, p2.1), (p3.0, p3.1)),);
    }
}
impl ttf_parser::OutlineBuilder for GlyphOutliner {
    fn move_to(&mut self, x: f32, y: f32) {
        self.pos = (x, y);
    }
    fn line_to(&mut self, x: f32, y: f32) {
        self.add_curve(self.pos, (x, y), (x, y));
        self.pos = (x, y);
    }
    fn quad_to(&mut self, x1: f32, y1: f32, x: f32, y: f32) {
        self.add_curve(self.pos, (x1, y1), (x, y));
        self.pos = (x, y);
    }
    fn curve_to(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, x: f32, y: f32) {
        panic!("cubic");
        // self.add_curve(self.pos, ((x1 + x2) / 2.0, (y1 + y2) / 2.0), (x, y));
        // self.pos = (x, y);
    }
    fn close(&mut self) {}
}

pub struct Truetype {
    shader: shader::Shader,
    tex_bezier: texture::Texture,
    face: ttf_parser::Face<'static>,
}
impl Truetype {
    pub fn new(ctx: &context::Context, bs: &[u8]) -> utils::Erm<Self> {
        let shader = shader::Shader::new(ctx,
            include_str!("assets/shaders/slug/vert.glsl"),
            include_str!("assets/shaders/slug/frag.glsl"),
        );
        let mut v: Vec<u8> = Vec::new();
        v.extend_from_slice(bs);
        let data = Box::leak(Box::new(v));
        let tex_bezier = texture::Texture::new_empty(ctx);
        let face = ttf_parser::Face::parse(&*data, 0)?;
        Ok(Self {
            shader,
            tex_bezier,
            face,
        })
    }
    pub fn render_glyph(&self,
        ctx: &context::Context, st: &mut state::State,
        pos: glam::Vec2, sz: f32,
        char: char
    ) -> utils::Erm<()> {
        let mut outliner = GlyphOutliner::new();
        let g = self.face.glyph_index(char).unwrap();
        let bound = self.face.outline_glyph(g, &mut outliner).unwrap();
        log::info!("glyph {:?} bound: {:?}", g, bound);
        let width = (bound.x_max - bound.x_min) as f32;
        let height = (bound.y_max - bound.y_min) as f32;
        let curves_len = outliner.data.len() as i32;
        let mut bytes = Vec::new();
        let scale_point = |bound: &ttf_parser::Rect, p: (f32, f32)| {
            ((p.0 - bound.x_min as f32) / width - 0.5,
            (p.1 - bound.y_min as f32) / height - 0.5)
        };
        fn extend_with_point(bytes: &mut Vec<u8>, p: (f32, f32)) {
            bytes.extend_from_slice(&p.0.to_ne_bytes());
            bytes.extend_from_slice(&p.1.to_ne_bytes());
        }
        for (p1, p2, p3) in outliner.data {
            extend_with_point(&mut bytes, scale_point(&bound, p1));
            extend_with_point(&mut bytes, scale_point(&bound, p2));
            extend_with_point(&mut bytes, scale_point(&bound, p3));
            extend_with_point(&mut bytes, (0.0, 0.0));
        }
        bytes.resize(4096 * 4 * 4, 0);
        self.tex_bezier.bind(ctx);
        unsafe {
            ctx.gl.tex_image_2d(
                glow::TEXTURE_2D,
                0,
                glow::RGBA32F as i32,
                4096,
                1,
                0,
                glow::RGBA,
                glow::FLOAT,
                Some(&bytes),
            );
        }
        st.bind_2d(ctx, &self.shader);
        self.shader.set_position_2d(ctx, st,
            &pos,
            &glam::Vec2::new(sz, height * sz / width)
        );
        self.shader.set_i32(ctx, "curves_len", curves_len);
        st.mesh_square.render(ctx);
        Ok(())
    }
}
