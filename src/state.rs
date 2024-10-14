#![allow(dead_code, unused_variables)]
use std::collections::HashMap;
use bimap::BiHashMap;
use enum_map::{enum_map, Enum, EnumMap};

use crate::{context, framebuffer, shader, audio};

const DELTA_TIME: f64 = 1.0 / 60.0;

pub struct WinitWaker {}

impl WinitWaker {
    fn new() -> Self { Self {} }
}
impl std::task::Wake for WinitWaker {
    fn wake(self: std::sync::Arc<Self>) {}
}

pub trait Game {
    fn initialize_audio(&self, ctx: &context::Context, st: &State, actx: &audio::Context) ->
        HashMap<String, audio::Audio>
    {
        HashMap::new()
    }
    fn finish_title(&mut self, st: &mut State) {}
    fn mouse_move(&mut self, ctx: &context::Context, st: &mut State, x: i32, y: i32) {}
    fn mouse_press(&mut self, ctx: &context::Context, st: &mut State) {}
    fn request_return(&mut self, ctx: &context::Context, st: &mut State, res: reqwest::Response) {}
    fn update(&mut self, ctx: &context::Context, st: &mut State) -> Option<()> { Some(()) }
    fn render(&mut self, ctx: &context::Context, st: &mut State) -> Option<()> { Some(()) }
}

#[derive(Debug, Enum, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Key {
    Up, Down, Left, Right,
    A, B, L, R,
    Start, Select,
}
pub const KEYS: [Key; 10] = [
    Key::Up, Key::Down, Key::Left, Key::Right,
    Key::A, Key::B, Key::L, Key::R,
    Key::Start, Key::Select,
];
pub struct Keys {
    pub pressed: EnumMap<Key, bool>,
    pub new: EnumMap<Key, bool>,
}
impl Keys {
    pub fn new() -> Self {
        Self {
            pressed: enum_map! {
                Key::Up => false, Key::Down => false, Key::Left => false, Key::Right => false,
                Key::A => false, Key::B => false, Key::L => false, Key::R => false,
                Key::Start => false, Key::Select => false,
            },
            new: enum_map! {
                Key::Up => false, Key::Down => false, Key::Left => false, Key::Right => false,
                Key::A => false, Key::B => false, Key::L => false, Key::R => false,
                Key::Start => false, Key::Select => false,
            },
        }
    }
    pub fn up(&self) -> bool { self.pressed[Key::Up] }
    pub fn down(&self) -> bool { self.pressed[Key::Down] }
    pub fn left(&self) -> bool { self.pressed[Key::Left] }
    pub fn right(&self) -> bool { self.pressed[Key::Right] }
    pub fn a(&self) -> bool { self.pressed[Key::A] }
    pub fn b(&self) -> bool { self.pressed[Key::B] }
    pub fn l(&self) -> bool { self.pressed[Key::L] }
    pub fn r(&self) -> bool { self.pressed[Key::R] }
    pub fn start(&self) -> bool { self.pressed[Key::Start] }
    pub fn select(&self) -> bool { self.pressed[Key::Select] }
    pub fn new_up(&mut self) -> bool { let ret = self.new[Key::Up]; self.new[Key::Up] = false; ret }
    pub fn new_down(&mut self) -> bool { let ret = self.new[Key::Down]; self.new[Key::Down] = false; ret }
    pub fn new_left(&mut self) -> bool { let ret = self.new[Key::Left]; self.new[Key::Left] = false; ret }
    pub fn new_right(&mut self) -> bool { let ret = self.new[Key::Right]; self.new[Key::Right] = false; ret }
    pub fn new_a(&mut self) -> bool { let ret = self.new[Key::A]; self.new[Key::A] = false; ret }
    pub fn new_b(&mut self) -> bool { let ret = self.new[Key::B]; self.new[Key::B] = false; ret }
    pub fn new_l(&mut self) -> bool { let ret = self.new[Key::L]; self.new[Key::L] = false; ret }
    pub fn new_r(&mut self) -> bool { let ret = self.new[Key::R]; self.new[Key::R] = false; ret }
    pub fn new_start(&mut self) -> bool { let ret = self.new[Key::Start]; self.new[Key::Start] = false; ret }
    pub fn new_select(&mut self) -> bool { let ret = self.new[Key::Select]; self.new[Key::Select] = false; ret }
}

pub struct PointLight {
    pub pos: glam::Vec3,
    pub color: glam::Vec3,
    pub attenuation: glam::Vec2,
}

pub struct State {
    pub acc: f64,
    pub last: f64,
    pub tick: u64,

    pub rebinding: Option<Key>,
    pub keybindings: BiHashMap<winit::keyboard::KeyCode, Key>,
    pub keys: Keys,

    pub screen: framebuffer::Framebuffer,
    pub render_framebuffer: framebuffer::Framebuffer,
    pub shader_upscale: shader::Shader,
    pub audio: Option<audio::Assets>,

    pub projection: glam::Mat4,
    pub camera: (glam::Vec3, glam::Vec3, glam::Vec3),
    pub lighting: (glam::Vec3, glam::Vec3, glam::Vec3),
    pub point_lights: Vec<PointLight>,

    pub waker_ctx: std::task::Context<'static>,
    pub http_client: reqwest::Client,
    pub request: Option<std::pin::Pin<Box<dyn std::future::Future<Output = reqwest::Response>>>>,

    pub log: Vec<(u64, String)>,
}

pub fn now(ctx: &context::Context) -> f64 {
    ctx.performance.now() / 1000.0
}

pub fn default_keybindings() -> BiHashMap<winit::keyboard::KeyCode, Key> {
    BiHashMap::from_iter(vec![
        (winit::keyboard::KeyCode::KeyW, Key::Up),
        (winit::keyboard::KeyCode::KeyS, Key::Down),
        (winit::keyboard::KeyCode::KeyA, Key::Left),
        (winit::keyboard::KeyCode::KeyD, Key::Right),
        (winit::keyboard::KeyCode::Digit1, Key::A),
        (winit::keyboard::KeyCode::Digit2, Key::B),
        (winit::keyboard::KeyCode::KeyQ, Key::L),
        (winit::keyboard::KeyCode::KeyE, Key::R),
        (winit::keyboard::KeyCode::Tab, Key::Start),
        (winit::keyboard::KeyCode::Space, Key::Select),
    ])
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

        let waker = std::sync::Arc::new(WinitWaker::new());
        let cwaker = Box::leak(Box::new(waker.into()));
        let waker_ctx = std::task::Context::from_waker(cwaker);

        Self {
            acc: 0.0,
            last: now(ctx),
            // we initialize the tick to 1000, which allows us to use "0" as the default time for
            // various animation starts on entities without having them all play at game start
            tick: 1000,

            rebinding: None,
            keybindings: default_keybindings(),
            keys: Keys::new(),

            screen,
            render_framebuffer,
            shader_upscale,
            audio: None,

            projection: glam::Mat4::perspective_lh(
                std::f32::consts::PI / 4.0,
                context::RENDER_WIDTH / context::RENDER_HEIGHT,
                // 0.1,
                0.5,
                50.0,
            ),
            camera: (glam::Vec3::new(0.0, 0.0, 0.0), glam::Vec3::new(0.0, 0.0, 1.0), glam::Vec3::new(0.0, 1.0, 0.0)),
            lighting: (
                glam::Vec3::new(1.0, 1.0, 1.0),
                glam::Vec3::new(1.0, 1.0, 1.0),
                glam::Vec3::new(1.0, -1.0, 1.0),
            ),
            point_lights: Vec::new(),

            waker_ctx,
            http_client: reqwest::Client::new(),
            request: None,

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
        ambient: &glam::Vec3,
        color: &glam::Vec3,
        dir: &glam::Vec3,
    ) {
        self.lighting = (ambient.clone(), color.clone(), dir.clone());
    }

    pub fn add_point_light(
        &mut self,
        _ctx: &context::Context,
        pos: &glam::Vec3,
        color: &glam::Vec3,
        attenuation: &glam::Vec2,
    ) {
        self.point_lights.push(
            PointLight {
                pos: pos.clone(),
                color: color.clone(),
                attenuation: attenuation.clone(),
            },
        );
    }

    pub fn clear_point_lights(&mut self, _ctx: &context::Context) {
        self.point_lights.clear();
    }

    pub fn view(&self) -> glam::Mat4 {
        glam::Mat4::look_to_lh(
            self.camera.0,
            self.camera.1,
            self.camera.2,
        )
    }

    pub fn bind_3d_helper(&mut self, ctx: &context::Context, shader: &shader::Shader, plc: usize) {
        shader.bind(ctx);
        shader.set_mat4(ctx, "projection", &self.projection);
        shader.set_mat4(ctx, "view", &self.view());
        shader.set_vec3(
            ctx, "light_ambient_color",
            &self.lighting.0,
        );
        shader.set_vec3(
            ctx, "light_dir_color",
            &self.lighting.1,
        );
        shader.set_vec3(
            ctx, "light_dir",
            &self.lighting.2.normalize(),
        );
        shader.set_i32(
            ctx, &format!("light_count"),
            plc as _,
        );
    }

    pub fn bind_3d_no_point_lights(&mut self, ctx: &context::Context, shader: &shader::Shader) {
        self.bind_3d_helper(ctx, shader, 0);
    }

    pub fn bind_3d(&mut self, ctx: &context::Context, shader: &shader::Shader) {
        let plc = self.point_lights.len().min(5);
        self.bind_3d_helper(ctx, shader, plc);
        if plc > 0 {
            let lpos: Vec<_> = self.point_lights.iter().take(plc).map(|l| l.pos).collect();
            shader.set_vec3_array(
                ctx, &format!("light_pos[0]"),
                &lpos,
            );
            let lcolor: Vec<_> = self.point_lights.iter().take(plc).map(|l| l.color).collect();
            shader.set_vec3_array(
                ctx, &format!("light_color[0]"),
                &lcolor,
            );
            let lattenuation: Vec<_> = self.point_lights.iter().take(plc).map(|l| l.attenuation).collect();
            shader.set_vec2_array(
                ctx, &format!("light_attenuation[0]"),
                &lattenuation,
            );
        }
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

    pub fn mouse_moved<G>(
        &mut self,
        ctx: &context::Context,
        x: f32, y: f32,
        game: &mut G
    ) where G: Game
    {
        let rx = ((x - self.screen.offsets.x) * context::RENDER_WIDTH / self.screen.dims.x) as i32;
        let ry = ((y - self.screen.offsets.y) * context::RENDER_HEIGHT / self.screen.dims.y) as i32;
        if !(rx < 0 || rx >= context::RENDER_WIDTH as i32 || ry < 0 || ry >= context::RENDER_HEIGHT as i32) {
            game.mouse_move(ctx, self, rx, ry);
        }
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
            game.finish_title(self);
        }
        game.mouse_press(ctx, self);
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
        if key == winit::keyboard::KeyCode::F12 {
            self.keybindings = default_keybindings();
            self.rebinding = None;
            self.write_log("Reset keybindings!");
        } else if let Some(k) = self.rebinding {
            self.keybindings.insert(key, k);
            self.rebinding = None;
        } else if let Some(k) = self.keybindings.get_by_left(&key) {
            self.keys.pressed[*k] = true;
            self.keys.new[*k] = true;
        }
    }

    pub fn key_released(
        &mut self,
        _ctx: &context::Context,
        key: winit::keyboard::KeyCode,
    ) {
        if let Some(k) = self.keybindings.get_by_left(&key) {
            self.keys.pressed[*k] = false;
        }
    }

    /// Return the first keybinding for the given virtual key
    pub fn keybinding_for(&self, k: &Key) -> Option<String> {
        if let Some(kc) = self.keybindings.get_by_right(k) {
            Some(format!("{:?}", kc))
        } else {
            None
        }
    }

    pub fn rebind_key(&mut self, k: &Key) {
        self.rebinding = Some(*k);
    }

    pub fn request<F>(&mut self, ctx: &context::Context, f: F)
    where F: Fn(&reqwest::Client) -> reqwest::RequestBuilder
    {
        let builder = f(&self.http_client);
        let fut = async {
            builder.send().await.expect("failed to send HTTP request")
        };
        self.request = Some(Box::pin(fut));
    }

    pub fn requesting(&self) -> bool { self.request.is_some() }

    pub fn request_returned<G>(&mut self, ctx: &context::Context, game: &mut G, res: reqwest::Response)
    where G: Game
    {
        game.request_return(ctx, self, res);
        self.request = None;
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
