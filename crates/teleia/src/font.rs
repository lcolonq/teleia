use std::collections::HashMap;

use crate::{context, mesh, shader, state, texture};
use glow::HasContext;

pub struct Bitmap {
    pub char_width: i32,
    pub char_height: i32,
    pub font_width: i32,
    pub font_height: i32,
    pub shader: shader::Shader,
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
        let shader = shader::Shader::new_nolib(
            &ctx,
            include_str!("assets/shaders/bitmap/vert.glsl"),
            include_str!("assets/shaders/bitmap/frag.glsl"),
        );
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
            ctx.gl.vertex_attrib_pointer_f32(mesh::ATTRIB_COLOR, 3, glow::FLOAT, false, 0, 0);
            ctx.gl.enable_vertex_attrib_array(mesh::ATTRIB_COLOR);
            let index_buf = ctx.gl.create_buffer().expect("failed to create buffer object");
            ctx.gl.bind_buffer(glow::ELEMENT_ARRAY_BUFFER, Some(index_buf));
            Self {
                char_width, char_height,
                font_width, font_height,
                shader,
                font,
                vao,
                vertex_buf,
                texcoords_buf,
                colors_buf,
                index_buf,
            }
        }
    }

    pub fn new(ctx: &context::Context) -> Self {
        Self::from_image(ctx, 7, 9, 112, 54, include_bytes!("assets/fonts/simple.png"))
    }

    pub fn render_text_helper(&self, ctx: &context::Context, st: &state::State, pos: &glam::Vec2, text: &str, color: &[glam::Vec3]) {
        let mut cur = glam::Vec2::new(0.0, 0.0);
        let mut vertices = Vec::new();
        let mut texcoords = Vec::new();
        let mut colors = Vec::new();
        let mut indices = Vec::new();
        let cwidth = self.char_width as f32 / self.font_width as f32;
        let cheight = self.char_height as f32 / self.font_height as f32;
        let row_len = self.font_width as u32 / self.char_width as u32;
        for (i, c) in text.chars().enumerate() {
            if c == '\n' {
                cur.x = 0.0;
                cur.y -= self.char_height as f32;
            } else {
                let idx = vertices.len() as u32;
                vertices.push(cur);
                vertices.push(cur + glam::Vec2::new(self.char_width as f32, 0.0));
                vertices.push(cur + glam::Vec2::new(self.char_width as f32, self.char_height as f32));
                vertices.push(cur + glam::Vec2::new(0.0, self.char_height as f32));
                let cidx = c as u32 - ' ' as u32;
                let col = cidx % row_len;
                let row = cidx / row_len;
                let tcbase = glam::Vec2::new(col as f32 * cwidth, row as f32 * cheight);
                texcoords.push(tcbase + glam::Vec2::new(0.0, cheight));
                texcoords.push(tcbase + glam::Vec2::new(cwidth, cheight));
                texcoords.push(tcbase + glam::Vec2::new(cwidth, 0.0));
                texcoords.push(tcbase);
                let c = if let Some(c) = color.get(if color.len() == 0 { 0 } else { i % color.len() }) {
                    *c
                } else {
                    glam::Vec3::new(1.0, 1.0, 1.0)
                };
                colors.push(c); colors.push(c); colors.push(c); colors.push(c);
                indices.push(idx + 0); indices.push(idx + 1); indices.push(idx + 2);
                indices.push(idx + 0); indices.push(idx + 3); indices.push(idx + 2);
                cur.x += self.char_width as f32; 
            }
        }
        let index_bytes: Vec<u8> = indices.iter().flat_map(|x| x.to_ne_bytes()).collect();
        self.shader.bind(ctx);
        self.font.bind(ctx);
        let scale = glam::Vec2::new(2.0 / st.render_dims.x, 2.0 / st.render_dims.y);
        let offset = glam::Vec2::new(
            -st.render_dims.x / 2.0,
            st.render_dims.y / 2.0 - self.char_height as f32,
        );
        let npos = (glam::Vec2::new(pos.x, -pos.y) + offset) * scale;
        self.shader.set_mat4(
            ctx, "transform",
            &glam::Mat4::from_scale_rotation_translation(
                glam::Vec3::new(scale.x, scale.y, 1.0),
                glam::Quat::IDENTITY,
                glam::Vec3::new(npos.x, npos.y, 0.0),
            ),
        );
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
                    colors.len() * std::mem::size_of::<f32>() * 3,
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

    pub fn render_text(&self, ctx: &context::Context, st: &state::State, pos: &glam::Vec2, text: &str) {
        self.render_text_helper(ctx, st, pos, text, &[]);
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
    pub vao: glow::VertexArray,
    pub vertex_buf: glow::Buffer,
    pub texcoords_buf: glow::Buffer,
    pub colors_buf: glow::Buffer,
    pub index_buf: glow::Buffer,
}

impl TrueType {
    pub fn new(ctx: &context::Context, size: f32, data: &[u8]) -> Self {
        let shader = shader::Shader::new_nolib(
            &ctx,
            include_str!("assets/shaders/truetype/vert.glsl"),
            include_str!("assets/shaders/truetype/frag.glsl"),
        );
        let font = fontdue::Font::from_bytes(data, fontdue::FontSettings::default())
            .expect("failed to load font");
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
            ctx.gl.vertex_attrib_pointer_f32(mesh::ATTRIB_COLOR, 3, glow::FLOAT, false, 0, 0);
            ctx.gl.enable_vertex_attrib_array(mesh::ATTRIB_COLOR);
            let index_buf = ctx.gl.create_buffer().expect("failed to create buffer object");
            ctx.gl.bind_buffer(glow::ELEMENT_ARRAY_BUFFER, Some(index_buf));
            Self {
                shader, font, atlas, atlaswidth, cellwidth, cellheight, info,
                vao, vertex_buf, texcoords_buf, colors_buf, index_buf,
            }
        }
    }

    pub fn render_text_helper(&self, ctx: &context::Context, st: &state::State, pos: &glam::Vec2, spacing: &glam::Vec2, text: &str, color: &[glam::Vec3]) {
        let mut cur = glam::Vec2::new(0.0, 0.0);
        let mut vertices = Vec::new();
        let mut texcoords = Vec::new();
        let mut colors = Vec::new();
        let mut indices = Vec::new();
        let cellwidth = self.cellwidth as f32;
        let cellheight = self.cellheight as f32;
        let cwidth = cellwidth / self.atlaswidth as f32;
        let cheight = 1.0;
        for (i, c) in text.chars().enumerate() {
            if c == '\n' {
                cur.x = 0.0;
                cur.y -= spacing.y;
            } else {
                let idx = vertices.len() as u32;
                if let Some(off) = self.info.get(&c) {
                    vertices.push(cur);
                    vertices.push(cur + glam::Vec2::new(cellwidth, 0.0));
                    vertices.push(cur + glam::Vec2::new(cellwidth, cellheight));
                    vertices.push(cur + glam::Vec2::new(0.0, cellheight));
                    let tcbase = glam::Vec2::new(off.pos as f32 / self.atlaswidth as f32, 0.0);
                    texcoords.push(tcbase + glam::Vec2::new(0.0, cheight));
                    texcoords.push(tcbase + glam::Vec2::new(cwidth, cheight));
                    texcoords.push(tcbase + glam::Vec2::new(cwidth, 0.0));
                    texcoords.push(tcbase);
                    let c = if let Some(c) = color.get(i) {
                        *c
                    } else {
                        glam::Vec3::new(1.0, 1.0, 1.0)
                    };
                    colors.push(c); colors.push(c); colors.push(c); colors.push(c);
                    indices.push(idx + 0); indices.push(idx + 1); indices.push(idx + 2);
                    indices.push(idx + 0); indices.push(idx + 3); indices.push(idx + 2);
                }
                cur.x += spacing.x;
            }
        }
        let index_bytes: Vec<u8> = indices.iter().flat_map(|x| x.to_ne_bytes()).collect();
        let scale = glam::Vec2::new(2.0 / st.render_dims.x, 2.0 / st.render_dims.y);
        let offset = glam::Vec2::new(
            -st.render_dims.x / 2.0,
            st.render_dims.y / 2.0 - cellheight as f32,
        );
        let npos = (glam::Vec2::new(pos.x, -pos.y) + offset) * scale;
        self.shader.bind(ctx);
        self.shader.set_mat4(
            ctx, "transform",
            &glam::Mat4::from_scale_rotation_translation(
                glam::Vec3::new(scale.x, scale.y, 1.0),
                glam::Quat::IDENTITY,
                glam::Vec3::new(npos.x, npos.y, 0.0),
            ),
        );
        unsafe {
            ctx.gl.active_texture(glow::TEXTURE0);
            ctx.gl.bind_texture(glow::TEXTURE_2D, Some(self.atlas.tex));
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
                    colors.len() * std::mem::size_of::<f32>() * 3,
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
}
