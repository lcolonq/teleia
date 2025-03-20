use std::collections::HashMap;
use serde::Deserialize;
use glow::HasContext;

use crate::{context, erm, mesh, shader, texture, Erm};

#[derive(Debug)]
pub enum Err {
    LayerIndexOutOfBounds,
    LayerDataTooSmall,
    GIDNotFound(u32),
    AssetNotFound(String),
    GL(String),
}
impl std::error::Error for Err {}
impl std::fmt::Display for Err {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::LayerIndexOutOfBounds => write!(f, "layer index out of bounds"),
            Self::LayerDataTooSmall => write!(f, "layer data too small for dimensions"),
            Self::GIDNotFound(gid) => write!(f, "GID not found: {}", gid),
            Self::AssetNotFound(ass) => write!(f, "asset not found: {}", ass),
            Self::GL(msg) => write!(f, "GL error: {msg:}"),
        }
    }
}

#[derive(Debug, Deserialize)]
pub enum LayerType {
    #[serde(rename = "tilelayer")]
    Tile,
    #[serde(rename = "imagelayer")]
    Image,
    #[serde(rename = "objectgroup")]
    ObjectGroup,
    #[serde(rename = "group")]
    Group,
}

#[derive(Debug, Deserialize)]
pub struct Layer {
    name: String,
    id: i32,
    #[serde(rename = "type")] ty: LayerType,
    width: i32, height: i32,
    x: i32, y: i32,
    opacity: f32,
    visible: bool,
    data: Vec<u32>,
}

#[derive(Debug, Deserialize)]
pub struct LevelTileset {
    firstgid: i32,
    source: String,
}

#[derive(Debug, Deserialize)]
pub struct Level {
    width: i32, height: i32,
    tilewidth: i32, tileheight: i32,
    layers: Vec<Layer>,
    tilesets: Vec<LevelTileset>,
}
impl Level {
    pub fn new(bytes: &str) -> Erm<Self> {
        Ok(serde_json::from_str(bytes)?)
    }
}

#[derive(Debug, Deserialize)]
pub struct Tileset {
    name: String,
    imagewidth: i32, imageheight: i32,
    tilewidth: i32, tileheight: i32,
    margin: i32, spacing: i32,
}
impl Tileset {
    pub fn new(bytes: &str) -> Erm<Self> {
        Ok(serde_json::from_str(bytes)?)
    }
}

// TODO
pub enum Flip {
    None,
}

pub struct Asset {
    tileset: Tileset,
    texture: texture::Texture,
}
pub struct Assets {
    entries: HashMap<String, Asset>,
}
impl Assets {
    pub fn new() -> Self { Self { entries: HashMap::new() } }
    pub fn load(&mut self, ctx: &context::Context, nm: &str, ts: &str, img: &[u8]) -> Erm<()> {
        let ass = Asset {
            tileset: Tileset::new(ts)?,
            texture: texture::Texture::new(ctx, img),
        };
        if self.entries.insert(nm.to_string(), ass).is_some() {
            log::warn!("duplicate tileset entry named: {}", nm);
        }
        Ok(())
    }
    pub fn lookup_gid(&self, level: &Level, gid: u32) -> Erm<(i32, &Asset, Flip)> {
        let offset = (gid & 0x0fffffff) as i32;
        for lts in level.tilesets.iter().rev() {
            if lts.firstgid <= offset {
                return Ok((
                    offset - lts.firstgid,
                    self.entries.get(&lts.source).ok_or(Err::AssetNotFound(lts.source.clone()))?,
                    Flip::None
                ))
            }
        }
        return erm(Err::GIDNotFound(gid));
    }
}

pub struct LayerRenderer {
    pub vao: glow::VertexArray,
    pub vertex_buf: glow::Buffer,
    pub texcoords_buf: glow::Buffer,
    pub index_buf: glow::Buffer,
    pub index_count: usize,
}
impl LayerRenderer {
    pub fn new(ctx: &context::Context) -> Erm<Self> {
        unsafe {
            let vao = ctx.gl.create_vertex_array().map_err(Err::GL)?;
            ctx.gl.bind_vertex_array(Some(vao));
            let vertex_buf = ctx.gl.create_buffer().map_err(Err::GL)?;
            ctx.gl.bind_buffer(glow::ARRAY_BUFFER, Some(vertex_buf));
            ctx.gl.vertex_attrib_pointer_f32(mesh::ATTRIB_VERTEX, 2, glow::FLOAT, false, 0, 0);
            ctx.gl.enable_vertex_attrib_array(mesh::ATTRIB_VERTEX);
            let texcoords_buf = ctx.gl.create_buffer().map_err(Err::GL)?;
            ctx.gl.bind_buffer(glow::ARRAY_BUFFER, Some(texcoords_buf));
            ctx.gl.vertex_attrib_pointer_f32(mesh::ATTRIB_TEXCOORD, 2, glow::FLOAT, false, 0, 0);
            ctx.gl.enable_vertex_attrib_array(mesh::ATTRIB_TEXCOORD);
            let index_buf = ctx.gl.create_buffer().map_err(Err::GL)?;
            ctx.gl.bind_buffer(glow::ELEMENT_ARRAY_BUFFER, Some(index_buf));
            Ok(Self {
                vao,
                vertex_buf,
                texcoords_buf,
                index_buf,
                index_count: 0,
            })
        }
    }
}
pub struct LevelRenderer {
    pub layers: Vec<LayerRenderer>,
    pub shader: shader::Shader,
}
impl LevelRenderer {
    pub fn new(ctx: &context::Context, level: &Level) -> Erm<Self> {
        let mut layers = Vec::new();
        for _ in level.layers.iter() {
            layers.push(LayerRenderer::new(ctx)?);
        }
        let shader = shader::Shader::new_nolib(
            &ctx,
            include_str!("../assets/shaders/tiled/vert.glsl"),
            include_str!("../assets/shaders/tiled/frag.glsl"),
        );
        Ok(Self {
            layers,
            shader,
        })
    }
    pub fn populate_layer(
        &mut self,
        ctx: &context::Context,
        assets: &Assets,
        level: &Level,
        lidx: usize
    ) -> Erm<()> {
        let lr = self.layers.get_mut(lidx).ok_or(Err::LayerIndexOutOfBounds)?;
        let layer = level.layers.get(lidx).ok_or(Err::LayerIndexOutOfBounds)?;
        let mut vertices = Vec::new();
        let mut texcoords = Vec::new();
        let mut indices = Vec::new();
        for y in 0..layer.height {
            for x in 0..layer.width {
                let idx = x as usize + (y * layer.width) as usize;
                let gid = *layer.data.get(idx).ok_or(Err::LayerDataTooSmall)?;
                if gid == 0 { continue; }
                let (lid, ass, _) = assets.lookup_gid(level, gid)?;
                let cols = ass.tileset.imagewidth / ass.tileset.tilewidth;
                let rows = ass.tileset.imageheight / ass.tileset.tileheight;
                let col = lid % cols;
                let row = lid / cols;
                let twidth = 1.0 / cols as f32;
                let theight = 1.0 / rows as f32;

                let i = vertices.len() as u32;
                let v = glam::Vec2::new(x as _, y as _);
                vertices.push(v);
                vertices.push(v + glam::Vec2::new(1.0, 0.0));
                vertices.push(v + glam::Vec2::new(1.0, 1.0));
                vertices.push(v + glam::Vec2::new(0.0, 1.0));
                let uvbase = glam::Vec2::new(col as f32 / cols as f32, row as f32 / rows as f32);
                texcoords.push(uvbase + glam::Vec2::new(0.0, theight));
                texcoords.push(uvbase + glam::Vec2::new(twidth, theight));
                texcoords.push(uvbase + glam::Vec2::new(twidth, 0.0));
                texcoords.push(uvbase);
                indices.push(i + 0); indices.push(i + 1); indices.push(i + 2);
                indices.push(i + 0); indices.push(i + 3); indices.push(i + 2);
            }
        }
        let index_bytes: Vec<u8> = indices.iter().flat_map(|x| x.to_ne_bytes()).collect();
        unsafe {
            ctx.gl.bind_vertex_array(Some(lr.vao));
            ctx.gl.bind_buffer(glow::ARRAY_BUFFER, Some(lr.vertex_buf));
            ctx.gl.buffer_data_u8_slice(
                glow::ARRAY_BUFFER,
                std::slice::from_raw_parts(
                    vertices.as_ptr() as _,
                    vertices.len() * std::mem::size_of::<f32>() * 2,
                ),
                glow::STATIC_DRAW,
            );
            ctx.gl.bind_buffer(glow::ARRAY_BUFFER, Some(lr.texcoords_buf));
            ctx.gl.buffer_data_u8_slice(
                glow::ARRAY_BUFFER,
                std::slice::from_raw_parts(
                    texcoords.as_ptr() as _,
                    texcoords.len() * std::mem::size_of::<f32>() * 2,
                ),
                glow::STATIC_DRAW,
            );
            ctx.gl.bind_buffer(glow::ELEMENT_ARRAY_BUFFER, Some(lr.index_buf));
            ctx.gl.buffer_data_u8_slice(
                glow::ELEMENT_ARRAY_BUFFER,
                &index_bytes,
                glow::STATIC_DRAW,
            );
            lr.index_count = indices.len();
        }
        Ok(())
    }
    pub fn render_layer(
        &self,
        ctx: &context::Context,
        assets: &Assets,
        level: &Level,
        lidx: usize,
    ) -> Erm<()> {
        let layer = level.layers.get(lidx).ok_or(Err::LayerIndexOutOfBounds)?;
        let lr = self.layers.get(lidx).ok_or(Err::LayerIndexOutOfBounds)?;
        // TODO: handle layers with multiple textures
        let gid = *layer.data.iter().find(|g| **g > 0).unwrap();
        let (_, ass, _) = assets.lookup_gid(level, gid)?;
        ass.texture.bind(ctx);
        unsafe {
            ctx.gl.bind_vertex_array(Some(lr.vao));
            ctx.gl.draw_elements(glow::TRIANGLES, lr.index_count as _, glow::UNSIGNED_INT, 0);
        }
        Ok(())
    }
    pub fn populate(
        &mut self,
        ctx: &context::Context,
        assets: &Assets,
        level: &Level,
    ) -> Erm<()> {
        for lidx in 0..level.layers.len() {
            self.populate_layer(ctx, assets, level, lidx)?;
        }
        Ok(())
    }
    pub fn render(
        &self,
        ctx: &context::Context,
        assets: &Assets,
        level: &Level,
    ) -> Erm<()> {
        self.shader.bind(ctx);
        let sx = 2.0 * level.tilewidth as f32 / ctx.render_width;
        let sy = 2.0 * level.tileheight as f32 / ctx.render_height;
        self.shader.set_mat4(
            ctx, "transform",
            &glam::Mat4::from_scale_rotation_translation(
                glam::Vec3::new(sx, -sy, 1.0),
                glam::Quat::IDENTITY,
                glam::Vec3::new(0.0, 0.0, 0.0),
            ),
        );
        for lidx in 0..level.layers.len() {
            self.render_layer(ctx, assets, level, lidx)?;
        }
        Ok(())
    }
}
