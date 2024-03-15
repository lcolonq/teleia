use std::collections::HashMap;

use crate::{context, framebuffer, shader, audio};

const DELTA_TIME: f64 = 1.0 / 60.0;

pub trait Game {
    fn initialize_audio(&self, ctx: &context::Context, st: &State, actx: &audio::Context) ->
        HashMap<String, audio::Audio>;
    fn finish_title(&mut self);
    fn update(&mut self, ctx: &context::Context, st: &mut State) -> Option<()>;
    fn render(&mut self, ctx: &context::Context, st: &mut State) -> Option<()>;
}

pub struct Keys {
    pub up: bool,
    pub down: bool,
    pub left: bool,
    pub right: bool,
    pub a: bool,
    pub b: bool,
    pub l: bool,
    pub r: bool,
    pub start: bool,
    pub select: bool,
}

impl Keys {
    pub fn new() -> Self {
        Self {
            up: false,
            down: false,
            left: false,
            right: false,
            a: false,
            b: false,
            l: false,
            r: false,
            start: false,
            select: false,
        }
    }
}

pub struct State {
    pub acc: f64,
    pub last: f64,
    pub tick: u64,

    pub keys: Keys,

    pub screen: framebuffer::Framebuffer,
    pub render_framebuffer: framebuffer::Framebuffer,
    pub shader_upscale: shader::Shader,
    pub audio: Option<audio::Assets>,

    pub projection: glam::Mat4,
    pub camera: (glam::Vec3, glam::Vec3, glam::Vec3),
    pub lighting: (glam::Vec3, glam::Vec3),

    pub log: Vec<(u64, String)>,
}

pub fn now(ctx: &context::Context) -> f64 {
    ctx.performance.now() / 1000.0
}

impl State {
    pub fn new(ctx: &context::Context) -> Self {
        let screen = framebuffer::Framebuffer::screen(ctx);
        let render_framebuffer = framebuffer::Framebuffer::new(
            ctx,
            &glam::Vec2::new(context::RENDER_WIDTH, context::RENDER_HEIGHT),
            &glam::Vec2::new(0.0, 0.0),
        );
        let shader_upscale = shader::Shader::new_nolib(
            ctx,
            include_str!("assets/shaders/scale/vert.glsl"),
            include_str!("assets/shaders/scale/frag.glsl"),
        );

        Self {
            acc: 0.0,
            last: now(ctx),
            tick: 0,
            keys: Keys::new(),
            screen,
            render_framebuffer,
            shader_upscale,
            audio: None,

            projection: glam::Mat4::perspective_lh(
                std::f32::consts::PI / 4.0,
                context::RENDER_WIDTH / context::RENDER_HEIGHT,
                0.1,
                400.0,
            ),
            camera: (glam::Vec3::new(0.0, 0.0, 0.0), glam::Vec3::new(0.0, 0.0, 1.0), glam::Vec3::new(0.0, 1.0, 0.0)),
            lighting: (
                glam::Vec3::new(1.0, 1.0, 1.0),
                glam::Vec3::new(1.0, -1.0, 1.0),
            ),

            log: Vec::new(),
        }
    }

    pub fn write_log(&mut self, e: &str) {
        log::info!("log: {}", e.to_owned());
        self.log.push((self.tick, e.to_owned()));
    }

    pub fn handle_resize(&mut self, ctx: &context::Context) {
        self.screen = framebuffer::Framebuffer::screen(ctx);
    }

    pub fn move_camera(
        &mut self,
        _ctx: &context::Context,
        pos: &glam::Vec3,
        dir: &glam::Vec3,
        up: &glam::Vec3,
    ) {
        self.camera = (pos.clone(), dir.clone(), up.clone());
    }

    pub fn set_lighting(
        &mut self,
        _ctx: &context::Context,
        color: &glam::Vec3,
        dir: &glam::Vec3,
    ) {
        self.lighting = (color.clone(), dir.clone());
    }

    pub fn view(&self) -> glam::Mat4 {
        glam::Mat4::look_to_lh(
            self.camera.0,
            self.camera.1,
            self.camera.2,
        )
    }

    pub fn bind_3d(&mut self, ctx: &context::Context, shader: &shader::Shader) {
        shader.bind(ctx);
        shader.set_mat4(&ctx, "projection", &self.projection);
        shader.set_mat4(ctx, "view", &self.view());
        shader.set_vec3(
            ctx, "light_ambient_color",
            &glam::Vec3::new(1.0, 1.0, 1.0));
        shader.set_vec3(
            ctx, "light_dir_color",
            &self.lighting.0,
        );
        shader.set_vec3(
            ctx, "light_dir",
            &self.lighting.1,
        );
    }

    pub fn bind_2d(&mut self, ctx: &context::Context, shader: &shader::Shader) {
        shader.bind(ctx);
        shader.set_mat4(&ctx, "projection", &glam::Mat4::IDENTITY);
        shader.set_mat4(
            ctx, "view",
            &glam::Mat4::from_scale(
                glam::Vec3::new(
                    2.0 / context::RENDER_WIDTH,
                    2.0 / context::RENDER_HEIGHT,
                    1.0,
                ),
            ),
        );
    }

    pub fn mouse_pressed<G>(
        &mut self,
        ctx: &context::Context,
        _button: winit::event::MouseButton,
        game: &mut G
    ) where G: Game {
        log::info!("click");
        if self.audio.is_none() {
            self.audio = Some(audio::Assets::new(|actx| {
                game.initialize_audio(ctx, &self, actx)
            }));
            game.finish_title();
        }
    }

    pub fn mouse_released(
        &mut self,
        _ctx: &context::Context,
        _button: winit::event::MouseButton,
    ) {
    }

    pub fn key_pressed(
        &mut self,
        _ctx: &context::Context,
        key: winit::keyboard::KeyCode,
    ) {
        match key {
            winit::keyboard::KeyCode::KeyW => self.keys.up = true,
            winit::keyboard::KeyCode::KeyS => self.keys.down = true,
            winit::keyboard::KeyCode::KeyA => self.keys.left = true,
            winit::keyboard::KeyCode::KeyD => self.keys.right = true,
            winit::keyboard::KeyCode::Digit1 => self.keys.a = true,
            winit::keyboard::KeyCode::Digit2 => self.keys.b = true,
            winit::keyboard::KeyCode::KeyQ => self.keys.l = true,
            winit::keyboard::KeyCode::KeyE => self.keys.r = true,
            winit::keyboard::KeyCode::Tab => self.keys.start = true,
            winit::keyboard::KeyCode::Space => self.keys.select = true,
            _ => {},
        }
    }

    pub fn key_released(
        &mut self,
        _ctx: &context::Context,
        key: winit::keyboard::KeyCode,
    ) {
        match key {
            winit::keyboard::KeyCode::KeyW => self.keys.up = false,
            winit::keyboard::KeyCode::KeyS => self.keys.down = false,
            winit::keyboard::KeyCode::KeyA => self.keys.left = false,
            winit::keyboard::KeyCode::KeyD => self.keys.right = false,
            winit::keyboard::KeyCode::Digit1 => self.keys.a = false,
            winit::keyboard::KeyCode::Digit2 => self.keys.b = false,
            winit::keyboard::KeyCode::KeyQ => self.keys.l = false,
            winit::keyboard::KeyCode::KeyE => self.keys.r = false,
            winit::keyboard::KeyCode::Tab => self.keys.start = false,
            winit::keyboard::KeyCode::Space => self.keys.select = false,
            _ => {},
        }
    }

    pub fn run_update<G>(&mut self, ctx: &context::Context, game: &mut G) where G: Game {
        let now = now(ctx);
        let diff = now - self.last;
        self.acc += diff;
        self.last = now;

        // update, if enough time has accumulated since last update
        if self.acc >= DELTA_TIME {
            self.acc -= DELTA_TIME;
            self.tick += 1;
            game.update(ctx, self);

            // if a lot of time has elapsed (e.g. if window is unfocused and not
            // running update loop), prevent "death spiral"
            if self.acc >= DELTA_TIME { self.acc = 0.0 }
        }
    }

    pub fn run_render<G>(&mut self, ctx: &context::Context, game: &mut G) where G: Game {
        self.render_framebuffer.bind(&ctx);
        ctx.clear_color(glam::Vec4::new(0.1, 0.1, 0.1, 1.0));
        ctx.clear();

        game.render(ctx, self);

        self.screen.bind(&ctx);
        ctx.clear_color(glam::Vec4::new(0.0, 0.0, 0.0, 1.0));
        ctx.clear();
        self.shader_upscale.bind(&ctx);
        self.render_framebuffer.bind_texture(&ctx);
        ctx.render_no_geometry();
    }
}
