use teleia::*;

use std::ops::Rem;

pub struct TestGame {
    font: font::Bitmap,
    tt: font::TrueType,
    // cube: mesh::Mesh,
    fox: scene::Scene,
    tex: texture::Texture,
    shader: shader::Shader,
}

impl TestGame {
    pub async fn new(ctx: &context::Context) -> Self {
        Self {
            font: font::Bitmap::new(ctx),
            tt: font::TrueType::new(ctx, 12.0, include_bytes!("assets/fonts/ComicNeue-Regular.ttf")),
            // cube: mesh::Mesh::from_obj(ctx, include_bytes!("assets/meshes/cube.obj")),
            fox: scene::Scene::from_gltf(ctx, include_bytes!("assets/scenes/fox.glb")),
            // fox: scene::Scene::from_gltf(ctx, include_bytes!("/home/llll/src/colonq/assets/lcolonq_flat.vrm")),
            tex: texture::Texture::new(ctx, include_bytes!("assets/textures/test.png")),
            shader: scene::Scene::load_default_shader(ctx),
        }
    }
}

impl state::Game for TestGame {
    fn update(&mut self, ctx: &context::Context, st: &mut state::State) -> Option<()> {
        st.move_camera(
            ctx,
            &glam::Vec3::new(0.0, 0.0, -1.0),
            &glam::Vec3::new(0.0, 0.0, 1.0),
            &glam::Vec3::new(0.0, 1.0, 0.0),
        );
        Some(())
    }
    fn render(&mut self, ctx: &context::Context, st: &mut state::State) -> Option<()> {
        // if let Some(n) = self.fox.nodes_by_name.get("J_Bip_C_Neck").and_then(|i| self.fox.nodes.get_mut(*i)) {
        //     n.transform *= glam::Mat4::from_rotation_z(0.05);
        // }
        ctx.clear();
        self.fox.reflect_animation("Run", (st.tick as f32 / 60.0).rem(3.0));
        st.bind_3d(ctx, &self.shader);
        self.shader.set_position_3d(
            ctx,
            &glam::Mat4::from_scale_rotation_translation(
                glam::Vec3::new(0.005, 0.005, 0.005),
                // glam::Vec3::new(1.0, 1.0, 1.0),
                glam::Quat::from_rotation_y(st.tick as f32 / 60.0),
                glam::Vec3::new(0.0, -0.2, 0.0),
            ),
        );
        self.tex.bind(ctx);
        self.fox.render(ctx, &self.shader);
        self.font.render_text(ctx, &glam::Vec2::new(0.0, 10.0), "he's all FIXED up");
        self.tt.render_text_helper(
            ctx, &glam::Vec2::new(10.0, 60.0), &glam::Vec2::new(20.0, 30.0),
            "tESTge",
            &[
                glam::Vec3::new(1.0, 0.0, 0.0),
                glam::Vec3::new(0.0, 1.0, 0.0),
            ],
        );
        Some(())
    }
}

#[tokio::main]
pub async fn main() {
    run("teleia test", 240, 160, Options::empty(), TestGame::new).await;
}
