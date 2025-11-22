#![allow(dead_code, unused_variables)]
use std::{collections::HashMap, fmt::Display};
use bimap::BiHashMap;
use enum_map::{enum_map, Enum, EnumMap};
use serde::{Serialize, Deserialize};
use strum::EnumIter;

use crate::{audio, context, framebuffer, mesh, shader, utils};

const DELTA_TIME: f64 = 0.016; // todo

// pub struct WinitWaker {}
// impl WinitWaker {
//     fn new() -> Self { Self {} }
// }
// impl std::task::Wake for WinitWaker {
//     fn wake(self: std::sync::Arc<Self>) {}
// }

// pub struct Response {
//     pub url: String,
//     pub status: reqwest::StatusCode,
//     pub body: bytes::Bytes,
// }

pub trait Game {
    fn initialize(&self, ctx: &context::Context, st: &State) -> utils::Erm<()> { Ok(()) }
    fn finalize(&self, ctx: &context::Context, st: &State) -> utils::Erm<()> { Ok(()) }
    fn initialize_audio(
        &self, ctx: &context::Context, st: &State,
        actx: &audio::Context
    ) -> HashMap<String, audio::Audio> {
        HashMap::new()
    }
    fn finish_title(&mut self, st: &mut State) {}
    fn mouse_move(&mut self, ctx: &context::Context, st: &mut State, x: i32, y: i32) {}
    fn mouse_press(&mut self, ctx: &context::Context, st: &mut State) {}
    // fn request_return(&mut self, ctx: &context::Context, st: &mut State, res: Response) {}
    fn update(&mut self, ctx: &context::Context, st: &mut State) -> utils::Erm<()> { Ok(()) }
    fn render(&mut self, ctx: &context::Context, st: &mut State) -> utils::Erm<()> { Ok(()) }
}

#[derive(Debug, Enum, EnumIter, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Key {
    Up, Down, Left, Right,
    A, B, X, Y, L, R,
    Start, Select,
}
pub struct Keys {
    pub pressed: EnumMap<Key, bool>,
    pub new: EnumMap<Key, bool>,
}
impl Keys {
    pub fn new() -> Self {
        Self {
            pressed: enum_map! {
                Key::Up => false, Key::Down => false, Key::Left => false, Key::Right => false,
                Key::A => false, Key::B => false, Key::X => false, Key::Y => false,
                Key::L => false, Key::R => false,
                Key::Start => false, Key::Select => false,
            },
            new: enum_map! {
                Key::Up => false, Key::Down => false, Key::Left => false, Key::Right => false,
                Key::A => false, Key::B => false, Key::X => false, Key::Y => false,
                Key::L => false, Key::R => false,
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
    pub fn x(&self) -> bool { self.pressed[Key::X] }
    pub fn y(&self) -> bool { self.pressed[Key::Y] }
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
    pub fn new_x(&mut self) -> bool { let ret = self.new[Key::X]; self.new[Key::X] = false; ret }
    pub fn new_y(&mut self) -> bool { let ret = self.new[Key::Y]; self.new[Key::Y] = false; ret }
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

type Timestamp = f64;

#[cfg(target_arch = "wasm32")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Keycode {
    pub kc: winit::keyboard::KeyCode,
}
#[cfg(target_arch = "wasm32")]
impl Keycode {
    pub fn new(kc: winit::keyboard::KeyCode) -> Self { Self { kc } }
}
#[cfg(target_arch = "wasm32")]
impl Display for Keycode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.kc)
    }
}
#[cfg(target_arch = "wasm32")]
impl Serialize for Keycode {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where S: serde::Serializer {
        self.kc.serialize(serializer)
    }
}
#[cfg(target_arch = "wasm32")]
impl<'de> Deserialize<'de> for Keycode {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where D: serde::Deserializer<'de> {
        let kc = winit::keyboard::KeyCode::deserialize(deserializer)?;
        Ok(Self { kc })
    }
}

#[cfg(not(target_arch = "wasm32"))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Keycode {
    pub kc: glfw::Key
}
#[cfg(not(target_arch = "wasm32"))]
impl Keycode {
    pub fn new(kc: glfw::Key) -> Self { Self { kc } }
}
#[cfg(not(target_arch = "wasm32"))]
impl Display for Keycode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.kc)
    }
}
#[cfg(not(target_arch = "wasm32"))]
impl Serialize for Keycode {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where S: serde::Serializer {
        (self.kc as i32).serialize(serializer)
    }
}
#[cfg(not(target_arch = "wasm32"))]
impl<'de> Deserialize<'de> for Keycode {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where D: serde::Deserializer<'de> {
        i32::deserialize(deserializer)
            .map(|x| unsafe {
                std::mem::transmute(x)
            })
    }
}

pub struct State {
    pub tick: u64,
    pub nextframe: Timestamp,
    pub fps: u32,
    pub frames_this_second: u32,
    pub start_this_second: Timestamp,

    pub rebinding: Option<Key>,
    pub keybindings: BiHashMap<Keycode, Key>,
    pub keys: Keys,

    pub screen: framebuffer::Framebuffer,
    pub render_framebuffer: framebuffer::Framebuffer,
    pub render_dims: glam::Vec2,
    pub shader_upscale: shader::Shader,
    pub mesh_square: mesh::Mesh,
    pub audio: Option<audio::Assets>,

    pub projection: glam::Mat4,
    pub camera: (glam::Vec3, glam::Vec3, glam::Vec3),
    pub lighting: (glam::Vec3, glam::Vec3, glam::Vec3),
    pub point_lights: Vec<PointLight>,

    // pub waker_ctx: std::task::Context<'static>,
    // pub http_client: reqwest::Client,
    // pub request: Option<std::pin::Pin<Box<dyn std::future::Future<Output = reqwest::Result<Response>>>>>,

    pub log: Vec<(u64, String)>,
}

#[cfg(target_arch = "wasm32")]
pub fn now(ctx: &context::Context) -> Timestamp {
    ctx.performance.now() / 1000.0
}

#[cfg(not(target_arch = "wasm32"))]
pub fn now(ctx: &context::Context) -> Timestamp {
    let elapsed = ctx.start_instant.elapsed();
    let ms = elapsed.as_millis();
    (ms as f64) / 1000.0
}

#[cfg(target_arch = "wasm32")]
pub fn default_keybindings() -> BiHashMap<Keycode, Key> {
    BiHashMap::from_iter(vec![
        (Keycode::new(winit::keyboard::KeyCode::KeyW), Key::Up),
        (Keycode::new(winit::keyboard::KeyCode::KeyS), Key::Down),
        (Keycode::new(winit::keyboard::KeyCode::KeyA), Key::Left),
        (Keycode::new(winit::keyboard::KeyCode::KeyD), Key::Right),
        (Keycode::new(winit::keyboard::KeyCode::Digit1), Key::A),
        (Keycode::new(winit::keyboard::KeyCode::Digit2), Key::B),
        (Keycode::new(winit::keyboard::KeyCode::Digit3), Key::X),
        (Keycode::new(winit::keyboard::KeyCode::Digit4), Key::Y),
        (Keycode::new(winit::keyboard::KeyCode::KeyQ), Key::L),
        (Keycode::new(winit::keyboard::KeyCode::KeyE), Key::R),
        (Keycode::new(winit::keyboard::KeyCode::Tab), Key::Start),
        (Keycode::new(winit::keyboard::KeyCode::Space), Key::Select),
    ])
}

#[cfg(not(target_arch = "wasm32"))]
pub fn default_keybindings() -> BiHashMap<Keycode, Key> {
    BiHashMap::from_iter(vec![
        (Keycode::new(glfw::Key::W), Key::Up),
        (Keycode::new(glfw::Key::S), Key::Down),
        (Keycode::new(glfw::Key::A), Key::Left),
        (Keycode::new(glfw::Key::D), Key::Right),
        (Keycode::new(glfw::Key::Num1), Key::A),
        (Keycode::new(glfw::Key::Num2), Key::B),
        (Keycode::new(glfw::Key::Num3), Key::X),
        (Keycode::new(glfw::Key::Num4), Key::Y),
        (Keycode::new(glfw::Key::Q), Key::L),
        (Keycode::new(glfw::Key::E), Key::R),
        (Keycode::new(glfw::Key::Tab), Key::Start),
        (Keycode::new(glfw::Key::Space), Key::Select),
    ])
}

impl State {
    pub fn new(ctx: &context::Context) -> Self {
        let screen = framebuffer::Framebuffer::screen(ctx);
        let render_framebuffer = framebuffer::Framebuffer::new(
            ctx,
            &glam::Vec2::new(ctx.render_width, ctx.render_height),
            &glam::Vec2::new(0.0, 0.0),
        );
        let shader_upscale = shader::Shader::new_nolib(
            ctx,
            include_str!("assets/shaders/scale/vert.glsl"),
            include_str!("assets/shaders/scale/frag.glsl"),
        );
        let mesh_square = mesh::Mesh::from_obj(ctx, include_bytes!("assets/meshes/square.obj"));

        // let waker = std::sync::Arc::new(WinitWaker::new());
        // let cwaker = Box::leak(Box::new(waker.into()));
        // let waker_ctx = std::task::Context::from_waker(cwaker);

        let nextframe = now(ctx);

        Self {
            // we initialize the tick to 1000, which allows us to use "0" as the default time for
            // various animation starts on entities without having them all play at game start
            tick: 1000,

            nextframe,
            fps: 0,
            frames_this_second: 0,
            start_this_second: nextframe,

            rebinding: None,
            keybindings: default_keybindings(),
            keys: Keys::new(),

            screen,
            render_framebuffer,
            render_dims: glam::Vec2::new(ctx.render_width, ctx.render_height),
            shader_upscale,
            mesh_square,
            audio: None,

            projection: glam::Mat4::perspective_lh(
                std::f32::consts::PI / 4.0,
                ctx.render_width / ctx.render_height,
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

            // waker_ctx,
            // http_client: reqwest::Client::new(),
            // request: None,

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

    pub fn bind_framebuffer(&mut self, ctx: &context::Context, fb: &framebuffer::Framebuffer) {
        fb.bind(ctx);
        self.render_dims = fb.dims;
    }

    pub fn bind_render_framebuffer(&mut self, ctx: &context::Context) {
        self.render_framebuffer.bind(&ctx); self.render_dims = self.render_framebuffer.dims;
    }

    pub fn bind_screen(&mut self, ctx: &context::Context) {
        self.screen.bind(&ctx); self.render_dims = self.screen.dims;
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
                    2.0 / self.render_dims.x,
                    2.0 / self.render_dims.y,
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
        let rx = ((x - self.screen.offsets.x) * ctx.render_width / self.screen.dims.x) as i32;
        let ry = ((y - self.screen.offsets.y) * ctx.render_height / self.screen.dims.y) as i32;
        if !(rx < 0 || rx >= ctx.render_width as i32 || ry < 0 || ry >= ctx.render_height as i32) {
            game.mouse_move(ctx, self, rx, ry);
        }
    }

    pub fn mouse_pressed<G>(
        &mut self,
        ctx: &context::Context,
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
    ) {
    }

    pub fn key_pressed(
        &mut self,
        _ctx: &context::Context,
        key: Keycode,
    ) {
        #[cfg(target_arch = "wasm32")]
        let rebind = key.kc == winit::keyboard::KeyCode::F12;
        #[cfg(not(target_arch = "wasm32"))]
        let rebind = key.kc == glfw::Key::F12;
        if rebind {
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
        key: Keycode,
    ) {
        if let Some(k) = self.keybindings.get_by_left(&key) {
            self.keys.pressed[*k] = false;
        }
    }

    /// Return the first keybinding for the given virtual key
    pub fn keybinding_for(&self, k: Key) -> Option<String> {
        if let Some(kc) = self.keybindings.get_by_right(&k) {
            Some(format!("{}", kc))
        } else {
            None
        }
    }

    pub fn rebind_key(&mut self, k: Key) {
        self.rebinding = Some(k);
    }

    // pub fn request<F>(&mut self, f: F)
    // where F: Fn(&reqwest::Client) -> reqwest::RequestBuilder
    // {
    //     let builder = f(&self.http_client);
    //     let fut = async {
    //         let resp = builder.send().await?;
    //         let url = resp.url().clone().to_string();
    //         let status = resp.status().clone();
    //         let body = resp.bytes().await?;
    //         reqwest::Result::Ok(Response {
    //             url,
    //             status,
    //             body,
    //         })
    //     };
    //     self.request = Some(Box::pin(fut));
    // }
    // pub fn requesting(&self) -> bool { self.request.is_some() }
    // pub fn request_returned<G>(&mut self, ctx: &context::Context, game: &mut G, res: Response)
    // where G: Game
    // {
    //     game.request_return(ctx, self, res);
    // }

    pub fn run_update<G>(&mut self, ctx: &context::Context, game: &mut G) -> utils::Erm<()> where G: Game {
        let now = now(ctx);
        if now > self.nextframe {
            while self.nextframe < now { // find the next target frame that isn't in the past
                self.nextframe = self.nextframe + DELTA_TIME;
            }
            self.tick += 1;
            self.frames_this_second += 1;
            game.update(ctx, self)?;
        }
        if now - self.start_this_second > 1.0 { // track FPS
            self.start_this_second = now;
            self.fps = self.frames_this_second;
            self.frames_this_second = 0;
        }
        Ok(())
    }

    pub fn run_render<G>(&mut self, ctx: &context::Context, game: &mut G) -> utils::Erm<()> where G: Game {
        self.bind_render_framebuffer(ctx);

        game.render(ctx, self)?;

        self.bind_screen(ctx);
        ctx.clear_color(
            if ctx.options.contains(crate::Options::OVERLAY) {
                glam::Vec4::new(0.0, 0.0, 0.0, 0.0)
            } else {
                glam::Vec4::new(0.0, 0.0, 0.0, 1.0)
            }
        );
        ctx.clear();
        self.shader_upscale.bind(&ctx);
        self.render_framebuffer.bind_texture(&ctx);
        ctx.render_no_geometry();
        Ok(())
    }
}
