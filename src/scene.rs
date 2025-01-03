use std::{collections::{HashMap, VecDeque}, mem::offset_of};

use glow::HasContext;
use image::EncodableLayout;

use crate::{context, mesh, shader, texture};

pub type Index = usize;

pub struct Primitive {
    pub mesh: mesh::Mesh,
    pub material: Index,
}

pub struct Object {
    pub primitives: Vec<Primitive>,
}

pub struct Material {
    pub base_color_factor: glam::Vec4,
    pub base_color_texture: Option<Index>,

    pub metallic_factor: f32,
    pub roughness_factor: f32,
    pub metallic_roughness_texture: Option<Index>,

    pub normal_texture: Option<Index>,

    pub occlusion_texture: Option<Index>,

    pub emissive_factor: glam::Vec3,
    pub emissive_texture: Option<Index>,
}

pub struct Skin {
    pub inverse_bind_matrices: Vec<glam::Mat4>,
    pub joints: Vec<Index>,
}

pub struct Node {
    pub children: Vec<Index>,
    pub object: Option<Index>,
    pub skin: Option<Index>,
    pub transform: glam::Mat4,
}

pub struct Scene {
    pub objects: Vec<Object>,
    pub textures: Vec<texture::Texture>,
    pub materials: Vec<Material>,
    pub skins: Vec<Skin>,
    pub nodes: Vec<Node>,
    pub nodes_by_name: HashMap<String, Index>,
    pub scene_nodes: Vec<Index>,
}

impl Scene {
    pub fn from_gltf(ctx: &context::Context, bytes: &[u8]) -> Self {
        let (gltf, buffers, images) = gltf::import_slice(bytes).expect("failed to parse GLTF");
        let get_buffer_data = |b: gltf::Buffer| {
            buffers.get(b.index()).map(|gltf::buffer::Data(bytes)| bytes.as_slice())
        };
        let objects = gltf.meshes().map(|m| {
            let primitives = m.primitives().filter_map(|p| {
                let mode = match p.mode() {
                    gltf::mesh::Mode::Points => glow::POINTS,
                    gltf::mesh::Mode::Lines => glow::LINES,
                    gltf::mesh::Mode::LineLoop => glow::LINE_LOOP,
                    gltf::mesh::Mode::LineStrip => glow::LINE_STRIP,
                    gltf::mesh::Mode::Triangles => glow::TRIANGLES,
                    gltf::mesh::Mode::TriangleStrip => glow::TRIANGLE_STRIP,
                    gltf::mesh::Mode::TriangleFan => glow::TRIANGLE_FAN,
                };
                unsafe {
                    let vao = ctx.gl.create_vertex_array().expect("failed to initialize vao");
                    ctx.gl.bind_vertex_array(Some(vao));

                    // in the past, I've been lazy and just uploaded whole buffers to the GPU.
                    // this is certainly not the right thing to do in general.
                    // perhaps I am misunderstanding, but it feels like GLTF makes it pretty difficult to do
                    // that in general, on account of things like sparse accessors.
                    // instead, we'll use the gltf crate's handy "reader" abstraction to iterate over all of the
                    // data in the buffers, assemble it ourselves, and then upload that.
                    let reader = p.reader(get_buffer_data);

                    // on to the actual vertex data.
                    // this is the layout of a single vertex in the buffer we send to the GPU.
                    struct Vertex {
                        pos: glam::Vec3,
                        normal: glam::Vec3,
                        texcoord: glam::Vec2,
                        joints: glam::Vec4,
                        weights: glam::Vec4,
                    }

                    // vertices always have positions
                    let mut vertices = Vec::new();
                    for pos in reader.read_positions().expect("primitive has no positions") {
                        vertices.push(Vertex {
                            pos: glam::Vec3::from_array(pos),
                            normal: glam::Vec3::default(),
                            texcoord: glam::Vec2::default(),
                            joints: glam::Vec4::default(),
                            weights: glam::Vec4::default(),
                        });
                    }

                    // if we find indices, use those. otherwise generate indices
                    let indices: Vec<u32> = if let Some(ri) = reader.read_indices() {
                        ri.into_u32().collect()
                    } else {
                        vertices.iter().enumerate().map(|(i, _)| i as u32).collect()
                    };
                    let indices_bytes: Vec<u8> = indices.iter().flat_map(|x| x.to_ne_bytes()).collect();
                    let indices_buf = ctx.gl.create_buffer().expect("failed to create index buffer object");
                    ctx.gl.bind_buffer(glow::ELEMENT_ARRAY_BUFFER, Some(indices_buf));
                    ctx.gl.buffer_data_u8_slice(
                        glow::ELEMENT_ARRAY_BUFFER,
                        &indices_bytes,
                        glow::STATIC_DRAW,
                    );

                    // optionally, we might have some other vertex attributes too
                    if let Some(iter) = reader.read_normals() {
                        for (i, n) in iter.enumerate() {
                            vertices[i].normal = glam::Vec3::from_array(n)
                        }
                    }
                    if let Some(iter) = reader.read_tex_coords(0) {
                        for (i, uv) in iter.into_f32().enumerate() {
                            vertices[i].texcoord = glam::Vec2::from_array(uv)
                        }
                    }
                    if let Some(iter) = reader.read_joints(0) {
                        for (i, j) in iter.into_u16().enumerate() {
                            vertices[i].joints = glam::Vec4::from_slice(&j.into_iter().map(|x| x as f32).collect::<Vec<f32>>())
                        }
                    }
                    if let Some(iter) = reader.read_weights(0) {
                        for (i, w) in iter.into_f32().enumerate() {
                            vertices[i].weights = glam::Vec4::from_array(w)
                        }
                    }

                    let vertex_size = std::mem::size_of::<Vertex>() as i32;
                    let vertices_buf = ctx.gl.create_buffer().expect("failed to create buffer object");
                    ctx.gl.bind_buffer(glow::ARRAY_BUFFER, Some(vertices_buf));
                    ctx.gl.buffer_data_u8_slice(
                        glow::ARRAY_BUFFER,
                        std::slice::from_raw_parts(
                            vertices.as_ptr() as _,
                            vertices.len() * (vertex_size as usize),
                        ),
                        glow::STATIC_DRAW,
                    );
                    ctx.gl.enable_vertex_attrib_array(mesh::ATTRIB_VERTEX);
                    ctx.gl.vertex_attrib_pointer_f32(mesh::ATTRIB_VERTEX, 3, glow::FLOAT, false, vertex_size, offset_of!(Vertex, pos) as _);
                    ctx.gl.enable_vertex_attrib_array(mesh::ATTRIB_NORMAL);
                    ctx.gl.vertex_attrib_pointer_f32(mesh::ATTRIB_NORMAL, 3, glow::FLOAT, false, vertex_size, offset_of!(Vertex, normal) as _);
                    ctx.gl.enable_vertex_attrib_array(mesh::ATTRIB_TEXCOORD);
                    ctx.gl.vertex_attrib_pointer_f32(mesh::ATTRIB_TEXCOORD, 2, glow::FLOAT, false, vertex_size, offset_of!(Vertex, texcoord) as _);
                    ctx.gl.enable_vertex_attrib_array(mesh::ATTRIB_JOINT);
                    ctx.gl.vertex_attrib_pointer_f32(mesh::ATTRIB_JOINT, 4, glow::FLOAT, false, vertex_size, offset_of!(Vertex, joints) as _);
                    ctx.gl.enable_vertex_attrib_array(mesh::ATTRIB_WEIGHT);
                    ctx.gl.vertex_attrib_pointer_f32(mesh::ATTRIB_WEIGHT, 4, glow::FLOAT, false, vertex_size, offset_of!(Vertex, weights) as _);


                    Some(Primitive {
                        mesh: mesh::Mesh {
                            vao,
                            mode,
                            index_count: indices.len(),
                            index_type: glow::UNSIGNED_INT,
                            index_offset: 0,
                        },
                        material: p.material().index().unwrap(),
                    })
                }
            }).collect();
            Object {
                primitives,
            }
        }).collect();
        let textures: Vec<texture::Texture> = images.into_iter().map(|bi| {
            unsafe {
                let i = bi.image.into_rgba8();
                let tex = ctx.gl.create_texture().expect("failed to create texture");
                ctx.gl.bind_texture(glow::TEXTURE_2D, Some(tex));
                ctx.gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_WRAP_S, glow::CLAMP_TO_EDGE as i32);
                ctx.gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_WRAP_T, glow::CLAMP_TO_EDGE as i32);
                ctx.gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_MIN_FILTER, glow::NEAREST as i32);
                ctx.gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_MAG_FILTER, glow::NEAREST as i32);
                ctx.gl.tex_image_2d(
                    glow::TEXTURE_2D,
                    0,
                    glow::RGBA as i32,
                    i.width() as i32,
                    i.height() as i32,
                    0,
                    glow::RGBA,
                    glow::UNSIGNED_BYTE,
                    Some(&i.as_bytes()),
                );
                ctx.gl.generate_mipmap(glow::TEXTURE_2D);
                texture::Texture { tex }
            }
        }).collect();
        let materials: Vec<Material> = gltf.materials().map(|m| {
            let pbr = m.pbr_metallic_roughness();
            let [bcr, bcg, bcb, bca] = pbr.base_color_factor();
            let [emx, emy, emz] = m.emissive_factor();
            Material {
                base_color_factor: glam::Vec4::new(bcr, bcg, bcb, bca),
                base_color_texture: pbr.base_color_texture().map(|tex| tex.texture().source().index()),
                metallic_factor: pbr.metallic_factor(),
                roughness_factor: pbr.roughness_factor(),
                metallic_roughness_texture: pbr.metallic_roughness_texture().map(|tex| tex.texture().source().index()),
                normal_texture: m.normal_texture().map(|tex| tex.texture().source().index()),
                occlusion_texture: m.occlusion_texture().map(|tex| tex.texture().source().index()),
                emissive_factor: glam::Vec3::new(emx, emy, emz),
                emissive_texture: m.emissive_texture().map(|tex| tex.texture().source().index()),
            }
        }).collect();

        let skins = gltf.skins().map(|s| {
            let ibm = s.reader(get_buffer_data).read_inverse_bind_matrices()
                .expect("missing read inverse bind matrices")
                .map(|m| glam::Mat4::from_cols_array_2d(&m)).collect();
            Skin {
                inverse_bind_matrices: ibm,
                joints: s.joints().map(|j| j.index()).collect(),
            }
        }).collect();

        let nodes = gltf.nodes().map(|n| {
            Node {
                children: n.children().map(|c| c.index()).collect(),
                object: n.mesh().map(|m| m.index()),
                skin: n.skin().map(|s| s.index()),
                transform: glam::Mat4::from_cols_array_2d(&n.transform().matrix()),
            }
        }).collect();

        let mut nodes_by_name = HashMap::new();
        for n in gltf.nodes() {
            if let Some(nm) = n.name() {
                nodes_by_name.insert(nm.to_owned(), n.index());
            }
        }

        let scene_nodes = gltf.default_scene().unwrap().nodes().map(|n| n.index()).collect();

        Self {
            objects,
            textures,
            materials,
            skins,
            nodes,
            nodes_by_name,
            scene_nodes,
        }
    }

    pub fn compute_joint_matrices(&self, skin: &Skin) -> Vec<glam::Mat4> {
        let mut q: VecDeque<(Index, glam::Mat4)> = VecDeque::new();
        q.push_back((skin.joints[0], glam::Mat4::IDENTITY));
        let mut transforms = vec![glam::Mat4::IDENTITY; self.nodes.len()];
        while let Some((ni, m)) = q.pop_front() {
            let n = &self.nodes[ni];
            transforms[ni] = m.mul_mat4(&n.transform);
            for ci in &n.children {
                q.push_back((*ci, transforms[ni]));
            }
        }
        let mut ret = vec![glam::Mat4::IDENTITY; skin.joints.len()];
        for (idx, ni) in skin.joints.iter().enumerate() {
            ret[idx] = transforms[*ni].mul_mat4(&skin.inverse_bind_matrices[idx]);
        }
        ret
    }

    fn render_node(&self, ctx: &context::Context, shader: &shader::Shader, n: &Node) {
        if let Some(o) = n.object.and_then(|i| self.objects.get(i)) {
            if let Some(s) = n.skin.and_then(|i| self.skins.get(i)) {
                let jms = self.compute_joint_matrices(s); 
                shader.set_mat4_array(ctx, "joint_matrices[0]", &jms);
            }
            for p in &o.primitives {
                if let Some(tex) = self.materials.get(p.material)
                    .and_then(|m| m.base_color_texture)
                    .and_then(|t| self.textures.get(t)) {
                        tex.bind(ctx);
                    }
                p.mesh.render(ctx);
            }
        }
    }

    pub fn render(&self, ctx: &context::Context, shader: &shader::Shader) {
        let mut q: VecDeque<Index> = VecDeque::new();
        for sn in &self.scene_nodes {
            q.push_back(*sn);
        }
        while let Some(ni) = q.pop_front() {
            let n = &self.nodes[ni];
            self.render_node(ctx, shader, n);
            for ci in &n.children {
                q.push_back(*ci);
            }
        }
    }
}
