pub mod utils;
pub mod ui;
pub mod context;
pub mod state;
pub mod framebuffer;
pub mod shader;
pub mod mesh;
pub mod texture;
pub mod scene;
pub mod font;
pub mod shadow;
pub mod audio;
pub mod net;
use std::ops::Rem;

#[cfg(target_arch = "wasm32")]
use winit::platform::web::EventLoopExtWebSys;

#[cfg(target_arch = "wasm32")]
use winit::platform::web::WindowExtWebSys;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::JsCast;

static mut CTX: Option<*const context::Context> = None;
static mut ST: Option<*mut state::State> = None;
static mut G: Option<*mut std::ffi::c_void> = None;

pub fn contextualize<F, G>(mut f: F)
where
    G: state::Game + 'static,
    F: FnMut(&context::Context, &mut state::State, &mut G) {
    unsafe {
        match (CTX, ST, G) {
            (Some(c), Some(s), Some(g)) => f(&*c, &mut*s, &mut*(g as *mut G)),
            _ => log::info!("context not set"),
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn run<'a, F, G, Fut>(title: &str, gnew: F)
where
    Fut: std::future::Future<Output = G>,
    G: state::Game + 'static,
    F: (Fn(&'a context::Context) -> Fut),
{
    env_logger::Builder::new()
        .filter(None, log::LevelFilter::Info)
        .init();

    log::info!("hello computer, starting up...");

    let (sdl, window, gl, mut event_loop, _gl_context) = {
        let sdl = sdl2::init().expect("failed to initialize SDL2");
        let video = sdl.video().expect("failed to initialize SDL2 video");
        // let gl_attr = video.gl_attr();
        // gl_attr.set_context_profile(sdl2::video::GLProfile::Core);
        // gl_attr.set_context_version(3, 0);
        let window = video
            .window(title, 640, 360)
            .opengl()
            // .fullscreen_desktop()
            .resizable()
            .build()
            .unwrap();
        let gl_context = window.gl_create_context().unwrap();
        let gl = unsafe {
            glow::Context::from_loader_function(|s| video.gl_get_proc_address(s) as *const _)
        };
        let event_loop = sdl.event_pump().unwrap();
        (sdl, window, gl, event_loop, gl_context)
    };

    let ctx = Box::leak(Box::new(context::Context::new(sdl, window, gl)));
    let game = Box::leak(Box::new(gnew(ctx).await));
    let st = Box::leak(Box::new(state::State::new(&ctx)));

    unsafe {
        CTX = Some(ctx as _);
        ST = Some(st as _);
        G = Some(game as *mut G as *mut std::ffi::c_void);
    }

    'running: loop {
        for event in event_loop.poll_iter() {
            match event {
                sdl2::event::Event::Quit {..} => {
                    log::info!("bye!");
                    break 'running;
                },
                sdl2::event::Event::Window { win_event: sdl2::event::WindowEvent::Resized(_, _), .. } => {
                    st.handle_resize(&ctx);
                },
                sdl2::event::Event::Window { win_event: sdl2::event::WindowEvent::FocusLost, .. } => {
                    st.keys = state::Keys::new();
                },
                sdl2::event::Event::MouseMotion { x, y, .. } => {
                    st.mouse_moved(&ctx, x as f32, y as f32, game);
                },
                sdl2::event::Event::MouseButtonDown {..} => {
                    st.mouse_pressed(&ctx, game)
                },
                sdl2::event::Event::MouseButtonUp {..} => {
                    st.mouse_released(&ctx)
                },
                sdl2::event::Event::KeyDown { keycode: Some(key), repeat: false, .. } => {
                    st.key_pressed(&ctx, key)
                },
                sdl2::event::Event::KeyUp { keycode: Some(key), repeat: false, .. } => {
                    st.key_released(&ctx, key)
                },
                _ => {},
            }
        }
        if ctx.resize_necessary() {
            st.handle_resize(&ctx);
        }
        if let Some(f) = &mut st.request {
            match std::future::Future::poll(f.as_mut(), &mut st.waker_ctx) {
                std::task::Poll::Pending => {},
                std::task::Poll::Ready(res) => {
                    st.request = None;
                    match res {
                        Ok(r) => st.request_returned(&ctx, game, r),
                        Err(e) => log::warn!("error during HTTP request: {}", e),
                    }
                },
            }
        }
        st.run_update(&ctx, game);
        st.run_render(&ctx, game);
        ctx.window.gl_swap_window();
    }

    // event_loop.set_control_flow(winit::event_loop::ControlFlow::Poll);
    // event_loop.run(|event, elwt| {
    //     event_loop_body::<G>(event, elwt);
    // }).expect("window closed");
}

#[cfg(target_arch = "wasm32")]
pub async fn run<'a, F, G, Fut>(gnew: F)
where
    Fut: std::future::Future<Output = G>,
    G: state::Game + 'static,
    F: (Fn(&'a context::Context) -> Fut),
{
    console_log::init_with_level(log::Level::Debug).unwrap();
    console_error_panic_hook::set_once();
    tracing_wasm::set_as_global_default();

    log::info!("hello computer, starting up...");

    let event_loop = winit::event_loop::EventLoop::new()
        .expect("failed to initialize event loop");

    let (window, gl) = {
        let window = winit::window::WindowBuilder::new()
            .with_maximized(true)
            .with_decorations(false)
            .build(&event_loop)
            .expect("failed to initialize window");
        let gl = web_sys::window()
            .and_then(|win| win.document())
            .and_then(|doc| {
                let dst = doc.get_element_by_id("teleia-parent")?;
                let canvas = web_sys::Element::from(window.canvas().expect("failed to find canvas"));
                dst.append_child(&canvas).ok()?;
                let c = canvas.dyn_into::<web_sys::HtmlCanvasElement>().ok()?;
                let webgl2_context = c.get_context("webgl2").ok()??
                    .dyn_into::<web_sys::WebGl2RenderingContext>().ok()?;
                Some(glow::Context::from_webgl2_context(webgl2_context))
            })
            .expect("couldn't add canvas to document");
        (window, gl)
    };

    let ctx = Box::leak(Box::new(context::Context::new(window, gl)));
    ctx.maximize_canvas();
    let game = Box::leak(Box::new(gnew(ctx).await));
    let st = Box::leak(Box::new(state::State::new(&ctx)));

    unsafe {
        CTX = Some(ctx as _);
        ST = Some(st as _);
        G = Some(game as *mut G as *mut std::ffi::c_void);
    }

    event_loop.set_control_flow(winit::event_loop::ControlFlow::Wait);
    event_loop.spawn(|event, elwt| {
        contextualize(|ctx, st, game: &mut G| {
            match &event {
                winit::event::Event::WindowEvent {
                    event: wev,
                    window_id,
                    ..
                } => match wev {
                    winit::event::WindowEvent::CloseRequested
                        if *window_id == ctx.window.id() => elwt.exit(),
                    winit::event::WindowEvent::Resized{..} => {
                        #[cfg(target_arch = "wasm32")]
                        ctx.maximize_canvas();
                        st.handle_resize(&ctx);
                    },
                    winit::event::WindowEvent::Focused(false) => {
                        st.keys = state::Keys::new();
                    },
                    winit::event::WindowEvent::CursorMoved { position, ..} => {
                        st.mouse_moved(&ctx, position.x as f32, position.y as f32, game);
                    },
                    winit::event::WindowEvent::MouseInput {
                        state,
                        ..
                    } => match state {
                        winit::event::ElementState::Pressed => {
                            st.mouse_pressed(&ctx, game)
                        },
                        winit::event::ElementState::Released => {
                            st.mouse_released(&ctx)
                        },
                    }
                    winit::event::WindowEvent::KeyboardInput {
                        event: winit::event::KeyEvent {
                            physical_key: winit::keyboard::PhysicalKey::Code(key),
                            state,
                            repeat: false,
                            ..
                        },
                        ..
                    } => match state {
                        winit::event::ElementState::Pressed => {
                            st.key_pressed(&ctx, *key)
                        },
                        winit::event::ElementState::Released => {
                            st.key_released(&ctx, *key)
                        },
                    }
                    _ => {},
                },
                
                winit::event::Event::AboutToWait => {
                    if ctx.resize_necessary() {
                        #[cfg(target_arch = "wasm32")]
                        ctx.maximize_canvas();
                        st.handle_resize(&ctx);
                    }
                    if let Some(f) = &mut st.request {
                        match std::future::Future::poll(f.as_mut(), &mut st.waker_ctx) {
                            std::task::Poll::Pending => {},
                            std::task::Poll::Ready(res) => {
                                st.request = None;
                                match res {
                                    Ok(r) => st.request_returned(&ctx, game, r),
                                    Err(e) => log::warn!("error during HTTP request: {}", e),
                                }
                            },
                        }
                        // f.poll();
                    }
                    st.run_update(&ctx, game);
                    st.run_render(&ctx, game);
                    ctx.window.request_redraw();
                },

                _ => {},
            }
        });
    });
}

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg(target_arch = "wasm32")]
struct TestGame {
    font: font::Bitmap,
    tt: font::TrueType,
    cube: mesh::Mesh,
    fox: scene::Scene,
    tex: texture::Texture,
    shader: shader::Shader,
}
impl TestGame {
    pub async fn new(ctx: &context::Context) -> Self {
        Self {
            font: font::Bitmap::new(ctx),
            tt: font::TrueType::new(ctx),
            cube: mesh::Mesh::from_obj(ctx, include_bytes!("assets/meshes/cube.obj")),
            fox: scene::Scene::from_gltf(ctx, include_bytes!("assets/scenes/fox.glb")),
            tex: texture::Texture::new(ctx, include_bytes!("assets/textures/test.png")),
            shader: shader::Shader::new(ctx, include_str!("assets/shaders/scene/vert.glsl"), include_str!("assets/shaders/scene/frag.glsl")),
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
        self.fox.reflect_animation("Run", (st.tick as f32 / 60.0).rem(3.0));
        st.bind_3d(ctx, &self.shader);
        self.shader.set_position_3d(
            ctx,
            &glam::Mat4::from_scale_rotation_translation(
                glam::Vec3::new(0.005, 0.005, 0.005),
                glam::Quat::from_rotation_y(st.tick as f32 / 60.0),
                glam::Vec3::new(0.0, -0.2, 0.0),
            ),
        );
        self.tex.bind(ctx);
        self.fox.render(ctx, &self.shader);
        self.font.render_text(ctx, &glam::Vec2::new(0.0, 0.0), "he's all FIXED up");
        self.tt.render_text(ctx, &glam::Vec2::new(10.0, 10.0), "tESTge");
        Some(())
    }
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub async fn main_js_test() {
    run(TestGame::new).await;
}
