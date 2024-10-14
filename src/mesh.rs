use glow::HasContext;

use crate::context;

pub const ATTRIB_VERTEX: u32 = 0;
pub const ATTRIB_NORMAL: u32 = 1;
pub const ATTRIB_TEXCOORD: u32 = 2;

pub struct Mesh {
    pub vao: glow::VertexArray,
    pub index_count: usize,
}

impl Mesh {
    pub fn build(
        ctx: &context::Context,
        vertices: &Vec<f32>,
        indices: &Vec<u32>,
        snormals: &Option<Vec<f32>>,
        stexcoords: &Option<Vec<f32>>,
    ) -> Self {
        unsafe {
            let vao = ctx.gl.create_vertex_array().expect("failed to initialize vao");
            ctx.gl.bind_vertex_array(Some(vao));

            let vertices_vbo = ctx.gl.create_buffer().expect("failed to initialize vbo");
            ctx.gl.bind_buffer(glow::ARRAY_BUFFER, Some(vertices_vbo));
            ctx.gl.buffer_data_u8_slice(
                glow::ARRAY_BUFFER,
                std::slice::from_raw_parts(
                    vertices.as_ptr() as _,
                    vertices.len() * std::mem::size_of::<f32>(),
                ),
                glow::STATIC_DRAW,
            );
            ctx.gl.vertex_attrib_pointer_f32(ATTRIB_VERTEX, 3, glow::FLOAT, false, 0, 0);
            ctx.gl.enable_vertex_attrib_array(ATTRIB_VERTEX);

            let indices_vbo = ctx.gl.create_buffer().expect("failed to initialize vbo");
            ctx.gl.bind_buffer(glow::ELEMENT_ARRAY_BUFFER, Some(indices_vbo));
            ctx.gl.buffer_data_u8_slice(
                glow::ELEMENT_ARRAY_BUFFER,
                std::slice::from_raw_parts(
                    indices.as_ptr() as _,
                    indices.len() * std::mem::size_of::<f32>(),
                ),
                glow::STATIC_DRAW,
            );

            if let Some(normals) = snormals {
                let normals_vbo = ctx.gl.create_buffer().expect("failed to initialize vbo");
                ctx.gl.bind_buffer(glow::ARRAY_BUFFER, Some(normals_vbo));
                ctx.gl.buffer_data_u8_slice(
                    glow::ARRAY_BUFFER,
                    std::slice::from_raw_parts(
                        normals.as_ptr() as _,
                        normals.len() * std::mem::size_of::<f32>(),
                    ),
                    glow::STATIC_DRAW,
                );
                ctx.gl.vertex_attrib_pointer_f32(ATTRIB_NORMAL, 3, glow::FLOAT, false, 0, 0);
                ctx.gl.enable_vertex_attrib_array(ATTRIB_NORMAL);
            }

            if let Some(texcoords) = stexcoords {
                let texcoords_vbo = ctx.gl.create_buffer().expect("failed to initialize vbo");
                ctx.gl.bind_buffer(glow::ARRAY_BUFFER, Some(texcoords_vbo));
                ctx.gl.buffer_data_u8_slice(
                    glow::ARRAY_BUFFER,
                    std::slice::from_raw_parts(
                        texcoords.as_ptr() as _,
                        texcoords.len() * std::mem::size_of::<f32>(),
                    ),
                    glow::STATIC_DRAW,
                );
                ctx.gl.vertex_attrib_pointer_f32(ATTRIB_TEXCOORD, 2, glow::FLOAT, false, 0, 0);
                ctx.gl.enable_vertex_attrib_array(ATTRIB_TEXCOORD);
            }

            Self {
                vao,
                index_count: indices.len(),
            }
        }
    }

    pub fn new(ctx: &context::Context, mut bytes: &[u8]) -> Self {
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
            &Some(mesh.normals),
            &Some(mesh.texcoords),
        )
    }

    pub fn render(&self, ctx: &context::Context) {
        unsafe {
            ctx.gl.bind_vertex_array(Some(self.vao));
            ctx.gl.draw_elements(glow::TRIANGLES, self.index_count as _, glow::UNSIGNED_INT, 0);
        }
    }

    pub fn render_instanced(&self, ctx: &context::Context, count: u64) {
        unsafe {
            ctx.gl.bind_vertex_array(Some(self.vao));
            ctx.gl.draw_elements_instanced(glow::TRIANGLES, self.index_count as _, glow::UNSIGNED_INT, 0, count as _);
        }
    }
}
