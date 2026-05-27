use glow::HasContext;

use crate::context;

pub const ATTRIB_VERTEX: u32 = 0;
pub const ATTRIB_NORMAL: u32 = 1;
pub const ATTRIB_TEXCOORD: u32 = 2;
pub const ATTRIB_JOINT: u32 = 3;
pub const ATTRIB_WEIGHT: u32 = 4;
pub const ATTRIB_COLOR: u32 = 5;

type Buffer = <glow::Context as glow::HasContext>::Buffer;

pub struct Mesh {
    pub vao: glow::VertexArray,
    pub vbo_vertex: Buffer,
    pub vbo_index: Buffer,
    pub vbo_normal: Option<Buffer>,
    pub vbo_texcoord: Option<Buffer>,
    pub mode: u32, // glow::TRIANGLES, etc.
    pub index_count: usize,
    pub index_type: u32, // glow::BYTE, glow::FLOAT, etc.
    pub index_offset: i32,
}

impl Mesh {
    pub fn new_empty(ctx: &context::Context, normals: bool, texcoords: bool) -> Self {
        unsafe {
            let vao = ctx.gl.create_vertex_array().expect("failed to initialize vao");
            ctx.gl.bind_vertex_array(Some(vao));

            let vbo_vertex = ctx.gl.create_buffer().expect("failed to create buffer object");
            ctx.gl.bind_buffer(glow::ARRAY_BUFFER, Some(vbo_vertex));
            ctx.gl.vertex_attrib_pointer_f32(ATTRIB_VERTEX, 3, glow::FLOAT, false, 0, 0);
            ctx.gl.enable_vertex_attrib_array(ATTRIB_VERTEX);

            let vbo_index = ctx.gl.create_buffer().expect("failed to create buffer object");
            ctx.gl.bind_buffer(glow::ELEMENT_ARRAY_BUFFER, Some(vbo_index));

            let vbo_normal = if normals {
                let vbo_normal = ctx.gl.create_buffer().expect("failed to create buffer object");
                ctx.gl.bind_buffer(glow::ARRAY_BUFFER, Some(vbo_normal));
                ctx.gl.vertex_attrib_pointer_f32(ATTRIB_NORMAL, 3, glow::FLOAT, false, 0, 0);
                ctx.gl.enable_vertex_attrib_array(ATTRIB_NORMAL);
                Some(vbo_normal)
            } else { None };

            let vbo_texcoord = if texcoords {
                let vbo_texcoord = ctx.gl.create_buffer().expect("failed to create buffer object");
                ctx.gl.bind_buffer(glow::ARRAY_BUFFER, Some(vbo_texcoord));
                ctx.gl.vertex_attrib_pointer_f32(ATTRIB_TEXCOORD, 2, glow::FLOAT, false, 0, 0);
                ctx.gl.enable_vertex_attrib_array(ATTRIB_TEXCOORD);
                Some(vbo_texcoord)
            } else { None };

            Self {
                vao,
                vbo_vertex,
                vbo_index,
                vbo_normal,
                vbo_texcoord,
                mode: glow::TRIANGLES,
                index_count: 0,
                index_type: glow::UNSIGNED_INT,
                index_offset: 0,
            }
        }
    }
    pub fn upload(
        &mut self,
        ctx: &context::Context,
        vertices: &[f32],
        indices: &[u32],
        snormals: Option<&[f32]>,
        stexcoords: Option<&[f32]>,
    ) {
        unsafe {
            ctx.gl.bind_buffer(glow::ARRAY_BUFFER, Some(self.vbo_vertex));
            ctx.gl.buffer_data_u8_slice(
                glow::ARRAY_BUFFER,
                std::slice::from_raw_parts(
                    vertices.as_ptr() as _,
                    std::mem::size_of_val(vertices),
                ),
                glow::STATIC_DRAW,
            );
            ctx.gl.bind_buffer(glow::ELEMENT_ARRAY_BUFFER, Some(self.vbo_index));
            ctx.gl.buffer_data_u8_slice(
                glow::ELEMENT_ARRAY_BUFFER,
                std::slice::from_raw_parts(
                    indices.as_ptr() as _,
                    std::mem::size_of_val(indices),
                ),
                glow::STATIC_DRAW,
            );

            if let Some(vbo_normal) = self.vbo_normal && let Some(normals) = snormals {
                ctx.gl.bind_buffer(glow::ARRAY_BUFFER, Some(vbo_normal));
                ctx.gl.buffer_data_u8_slice(
                    glow::ARRAY_BUFFER,
                    std::slice::from_raw_parts(
                        normals.as_ptr() as _,
                        std::mem::size_of_val(normals),
                    ),
                    glow::STATIC_DRAW,
                );
                Some(vbo_normal)
            } else { None };

            if let Some(vbo_texcoord) = self.vbo_texcoord && let Some(texcoords) = stexcoords {
                ctx.gl.bind_buffer(glow::ARRAY_BUFFER, Some(vbo_texcoord));
                ctx.gl.buffer_data_u8_slice(
                    glow::ARRAY_BUFFER,
                    std::slice::from_raw_parts(
                        texcoords.as_ptr() as _,
                        std::mem::size_of_val(texcoords),
                    ),
                    glow::STATIC_DRAW,
                );
                Some(vbo_texcoord)
            } else { None };
            self.index_count = indices.len();
        }
    }

    pub fn build(
        ctx: &context::Context,
        vertices: &[f32],
        indices: &[u32],
        snormals: Option<&[f32]>,
        stexcoords: Option<&[f32]>,
    ) -> Self {
        let mut ret = Self::new_empty(ctx, snormals.is_some(), stexcoords.is_some());
        ret.upload(ctx, vertices, indices, snormals, stexcoords);
        ret
    }

    pub fn upload_obj(&mut self, ctx: &context::Context, mut bytes: &[u8]) {
        let lopts = tobj::LoadOptions {
            triangulate: true,
            single_index: true,
            ..Default::default()
        };
        let (meshes, _materials) = tobj::load_obj_buf(
            &mut bytes,
            &lopts,
            |_| Err(tobj::LoadError::GenericFailure)
        ).expect("failed to load mesh");
        let mesh = meshes.into_iter().next()
            .expect("failed to load mesh")
            .mesh;
        self.upload(
            ctx,
            &mesh.positions,
            &mesh.indices,
            Some(&mesh.normals),
            Some(&mesh.texcoords),
        )
    }

    pub fn from_obj(ctx: &context::Context, mut bytes: &[u8]) -> Self {
        let lopts = tobj::LoadOptions {
            triangulate: true,
            single_index: true,
            ..Default::default()
        };
        let (meshes, _materials) = tobj::load_obj_buf(
            &mut bytes,
            &lopts,
            |_| Err(tobj::LoadError::GenericFailure)
        ).expect("failed to load mesh");
        let mesh = meshes.into_iter().next()
            .expect("failed to load mesh")
            .mesh;
        Self::build(
            ctx,
            &mesh.positions,
            &mesh.indices,
            Some(&mesh.normals),
            Some(&mesh.texcoords),
        )
    }

    pub fn render(&self, ctx: &context::Context) {
        unsafe {
            ctx.gl.bind_vertex_array(Some(self.vao));
            ctx.gl.draw_elements(self.mode, self.index_count as _, self.index_type, self.index_offset);
        }
    }

    pub fn render_instanced(&self, ctx: &context::Context, count: u64) {
        unsafe {
            ctx.gl.bind_vertex_array(Some(self.vao));
            ctx.gl.draw_elements_instanced(self.mode, self.index_count as _, self.index_type, self.index_offset, count as _);
        }
    }
}
