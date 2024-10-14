use winit::platform::web::EventLoopExtWebSys;

pub mod utils;
pub mod ui;
pub mod request;
pub mod context;
pub mod state;
pub mod framebuffer;
pub mod shader;
pub mod mesh;
pub mod texture;
pub mod font;
pub mod audio;
pub mod shadow;
pub mod module;

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

    let window = winit::window::WindowBuilder::new()
        .with_maximized(true)
        .with_decorations(false)
        .build(&event_loop)
        .expect("failed to initialize window");

    let ctx = Box::leak(Box::new(context::Context::new(window)));
    ctx.maximize_canvas();
    let game = Box::leak(Box::new(gnew(ctx).await));
    let st = Box::leak(Box::new(state::State::new(&ctx)));
    // request = Some(Box::new(async {
    //     "foo".to_owned()
    // }));

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
                        button,
                        state,
                        ..
                    } => match state {
                        winit::event::ElementState::Pressed => {
                            st.mouse_pressed(&ctx, *button, game)
                        },
                        winit::event::ElementState::Released => {
                            st.mouse_released(&ctx, *button)
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
                        ctx.maximize_canvas();
                        st.handle_resize(&ctx);
                    }
                    if let Some(f) = &mut st.request {
                        match std::future::Future::poll(f.as_mut(), &mut st.waker_ctx) {
                            std::task::Poll::Pending => {},
                            std::task::Poll::Ready(res) => {
                                st.request_returned(&ctx, game, res);
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
