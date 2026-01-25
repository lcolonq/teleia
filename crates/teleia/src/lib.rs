#![feature(try_blocks)]

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
pub mod physics;
pub mod save;
pub mod level2d;

pub use utils::{erm, install_error_handler, Erm};
pub use audio::AudioPlayback;
pub use simple_eyre::eyre::WrapErr;

#[cfg(target_arch = "wasm32")]
use winit::platform::web::EventLoopExtWebSys;

#[cfg(target_arch = "wasm32")]
use winit::platform::web::WindowExtWebSys;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::JsCast;

#[cfg(not(target_arch = "wasm32"))]
use glfw::Context;

use bitflags::bitflags;
bitflags! {
    pub struct Options: u32 {
        const OVERLAY  = 0b00000001;
        const HIDDEN   = 0b00000010;
        const NORESIZE = 0b00000100;
    }
}

static mut CTX: Option<*const context::Context> = None;
static mut ST: Option<*mut state::State> = None;
static mut G: Option<*mut std::ffi::c_void> = None;

pub fn contextualize<F, G, X>(mut f: F) -> X
where
    G: state::Game + 'static,
    F: FnMut(&context::Context, &mut state::State, &mut G) -> X {
    unsafe {
        match (CTX, ST, G) {
            (Some(c), Some(s), Some(g)) => f(&*c, &mut*s, &mut*(g as *mut G)),
            _ => panic!("context not set"),
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub fn run<'a, F, G>(title: &str, w: u32, h: u32, options: Options, gnew: F) -> Erm<()>
where
    G: state::Game + 'static,
    F: (Fn(&'a context::Context) -> G),
{
    env_logger::Builder::new()
        .filter(None, log::LevelFilter::Info)
        .init();
    install_error_handler();

    log::info!("hello computer, starting up...");

    let (rglfw, rwindow, gl, events) = {
        use glfw::fail_on_errors;
        let mut glfw = glfw::init(glfw::fail_on_errors!()).expect("failed to initialize GLFW");
        // let gl_attr = video.gl_attr();
        // gl_attr.set_context_profile(sdl2::video::GLProfile::Core);
        // gl_attr.set_context_version(3, 0);
        let (mut window, events) = glfw.with_primary_monitor(|glfw, primary| {
            if options.contains(Options::HIDDEN) {
                glfw.window_hint(glfw::WindowHint::Visible(false));
                glfw.create_window(w as _, h as _, title, glfw::WindowMode::Windowed)
                    .expect("failed to create window")
            } else if options.contains(Options::OVERLAY) {
                let mon = primary.expect("failed to get monitor");
                let mode = mon.get_video_mode().expect("failed to get video mode");
                glfw.window_hint(glfw::WindowHint::RedBits(Some(mode.red_bits)));
                glfw.window_hint(glfw::WindowHint::GreenBits(Some(mode.green_bits)));
                glfw.window_hint(glfw::WindowHint::BlueBits(Some(mode.blue_bits)));
                glfw.window_hint(glfw::WindowHint::RefreshRate(Some(mode.refresh_rate)));
                glfw.window_hint(glfw::WindowHint::Resizable(false));
                glfw.window_hint(glfw::WindowHint::Decorated(false));
                glfw.window_hint(glfw::WindowHint::Floating(true));
                glfw.window_hint(glfw::WindowHint::TransparentFramebuffer(true));
                unsafe {
                    // glfw.window_hint(glfw::WindowHint::MousePassthrough(true));
                    glfw::ffi::glfwWindowHint(0x0002000D, 1); // mouse passthrough
                }
                glfw.create_window(mode.width, mode.height, title, glfw::WindowMode::FullScreen(mon))
                    .expect("failed to create window")
            } else {
                glfw.create_window(w as _, h as _, title, glfw::WindowMode::Windowed)
                    .expect("failed to create window")
            }
        });
        window.make_current();
        window.set_key_polling(true);
        window.set_mouse_button_polling(true);
        window.set_size_polling(true);
        window.set_focus_polling(true);
        window.set_cursor_pos_polling(true);
        let gl = unsafe {
            glow::Context::from_loader_function(|s| window.get_proc_address(s) as *const _)
        };
        glfw.set_swap_interval(glfw::SwapInterval::Sync(1));
        (glfw, window, gl, events)
    };
    let glfw = std::cell::RefCell::new(rglfw);
    let window = std::cell::RefCell::new(rwindow);

    let ctx = Box::leak(Box::new(context::Context::new(
        glfw, window, gl,
        w as f32, h as f32, options,
    )));
    let game = Box::leak(Box::new(gnew(ctx)));
    let st = Box::leak(Box::new(state::State::new(&ctx)));

    unsafe {
        CTX = Some(ctx as _);
        ST = Some(st as _);
        G = Some(game as *mut G as *mut std::ffi::c_void);
    }

    game.initialize(ctx, st)?;
    'running: loop {
        if ctx.window.borrow().should_close() {
            game.finalize(ctx, st)?;
            log::info!("bye!");
            break 'running;
        }
        ctx.glfw.borrow_mut().poll_events();
        for (_, event) in glfw::flush_messages(&events) {
            match event {
                glfw::WindowEvent::Size(_, _) => st.handle_resize(&ctx),
                glfw::WindowEvent::Focus(false) => {
                    st.keys = state::Keys::new();
                },
                glfw::WindowEvent::CursorPos(x, y) => {
                    st.mouse_moved(&ctx, x as f32, y as f32, game);
                }
                glfw::WindowEvent::MouseButton(_, glfw::Action::Press, _) => {
                    st.mouse_pressed(&ctx, game)
                },
                glfw::WindowEvent::MouseButton(_, glfw::Action::Release, _) => {
                    st.mouse_released(&ctx)
                },
                glfw::WindowEvent::Key(key, _, glfw::Action::Press, _) => {
                    st.key_pressed(&ctx, state::Keycode::new(key))
                },
                glfw::WindowEvent::Key(key, _, glfw::Action::Release, _) => {
                    st.key_released(&ctx, state::Keycode::new(key))
                },
                _ => {},
            }
        }
        if ctx.resize_necessary() {
            st.handle_resize(&ctx);
        }
        // if let Some(f) = &mut st.request {
        //     match std::future::Future::poll(f.as_mut(), &mut st.waker_ctx) {
        //         std::task::Poll::Pending => {},
        //         std::task::Poll::Ready(res) => {
        //             st.request = None;
        //             match res {
        //                 Ok(r) => st.request_returned(&ctx, game, r),
        //                 Err(e) => log::warn!("error during HTTP request: {}", e),
        //             }
        //         },
        //     }
        // }
        st.run_update(&ctx, game)?;
        st.run_render(&ctx, game)?;
        ctx.window.borrow_mut().swap_buffers();
    }
    Ok(())
}

#[cfg(target_arch = "wasm32")]
pub fn run<'a, F, G>(w: u32, h: u32, options: Options, gnew: F)
where
    G: state::Game + 'static,
    F: (Fn(&'a context::Context) -> G),
{
    console_log::init_with_level(log::Level::Debug).unwrap();
    console_error_panic_hook::set_once();
    tracing_wasm::set_as_global_default();
    // install_error_handler();

    log::info!("hello computer, starting up...");

    let event_loop = winit::event_loop::EventLoop::new()
        .expect("failed to initialize event loop");

    let resize = !options.contains(Options::NORESIZE);
    let (window, gl) = {
        let window = winit::window::WindowBuilder::new()
            .with_maximized(resize)
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

    let ctx = Box::leak(Box::new(context::Context::new(window, gl, w as f32, h as f32, options)));
    ctx.maximize_canvas();
    let game = Box::leak(Box::new(gnew(ctx)));
    let st = Box::leak(Box::new(state::State::new(&ctx)));

    unsafe {
        CTX = Some(ctx as _);
        ST = Some(st as _);
        G = Some(game as *mut G as *mut std::ffi::c_void);
    }

    let _ = game.initialize(ctx, st);
    let res = std::rc::Rc::new(std::cell::RefCell::new(Ok(())));
    let result = res.clone();
    event_loop.set_control_flow(winit::event_loop::ControlFlow::Wait);
    event_loop.spawn(move |event, elwt| {
        let res: Erm<()> = contextualize(|ctx, st, game: &mut G| {
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
                            st.key_pressed(&ctx, state::Keycode { kc: *key })
                        },
                        winit::event::ElementState::Released => {
                            st.key_released(&ctx, state::Keycode { kc: *key })
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
                    // if let Some(f) = &mut st.request {
                    //     match std::future::Future::poll(f.as_mut(), &mut st.waker_ctx) {
                    //         std::task::Poll::Pending => {},
                    //         std::task::Poll::Ready(res) => {
                    //             st.request = None;
                    //             match res {
                    //                 Ok(r) => st.request_returned(&ctx, game, r),
                    //                 Err(e) => log::warn!("error during HTTP request: {}", e),
                    //             }
                    //         },
                    //     }
                    //     // f.poll();
                    // }
                    st.run_update(&ctx, game)?;
                    st.run_render(&ctx, game)?;
                    ctx.window.request_redraw();
                },

                _ => {},
            }
            Ok(())
        });
        if let Err(e) = res {
            *result.borrow_mut() = Err(e);
            elwt.exit();
        }
    });
    let _ = game.finalize(ctx, st);
    if let Err(e) = res.replace(Ok(())) {
        panic!("{}", e);
    }
}
