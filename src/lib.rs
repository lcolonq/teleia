use winit::platform::web::EventLoopExtWebSys;

pub mod utils;
pub mod request;
pub mod context;
pub mod state;
pub mod framebuffer;
pub mod shader;
pub mod mesh;
pub mod texture;
pub mod font;
pub mod audio;

pub fn run<F, G>(gnew: F) where G: state::Game + 'static, F: (Fn(&context::Context) -> G) {
    console_log::init_with_level(log::Level::Debug).unwrap();
    console_error_panic_hook::set_once();
    tracing_wasm::set_as_global_default();
    log::info!("HELLO COMPUTER HELLO CLONKHEAD :)");

    let event_loop = winit::event_loop::EventLoop::new()
        .expect("failed to initialize event loop");

    let window = winit::window::WindowBuilder::new()
        .with_maximized(true)
        .with_decorations(false)
        .build(&event_loop)
        .expect("failed to initialize window");

    let ctx = context::Context::new(window);
    ctx.maximize_canvas();
    let mut game = gnew(&ctx);
    let mut st = state::State::new(&ctx);
    st.write_log("test");
    st.write_log("foo");
    st.write_log("bar");
    st.write_log("baz");

    event_loop.set_control_flow(winit::event_loop::ControlFlow::Wait);
    event_loop.spawn(move |event, elwt| {
        match event {
            winit::event::Event::WindowEvent {
                event: wev,
                window_id,
                ..
            } => match wev {
                winit::event::WindowEvent::CloseRequested
                    if window_id == ctx.window.id() => elwt.exit(),
                winit::event::WindowEvent::Resized{..} => {
                    ctx.maximize_canvas();
                    st.handle_resize(&ctx);
                },
                winit::event::WindowEvent::MouseInput {
                    button,
                    state,
                    ..
                } => match state {
                    winit::event::ElementState::Pressed => {
                        st.mouse_pressed(&ctx, button, &mut game)
                    },
                    winit::event::ElementState::Released => {
                        st.mouse_released(&ctx, button)
                    },
                }
                winit::event::WindowEvent::KeyboardInput {
                    event: winit::event::KeyEvent {
                        physical_key: winit::keyboard::PhysicalKey::Code(key),
                        state,
                        ..
                    },
                    ..
                } => match state {
                    winit::event::ElementState::Pressed => {
                        st.key_pressed(&ctx, key)
                    },
                    winit::event::ElementState::Released => {
                        st.key_released(&ctx, key)
                    },
                }
                _ => {},
            },
                
            winit::event::Event::AboutToWait => {
                if ctx.resize_necessary() {
                    ctx.maximize_canvas();
                    st.handle_resize(&ctx);
                }
                st.run_update(&ctx, &mut game);
                st.run_render(&ctx, &mut game);
                ctx.window.request_redraw();
            },

            _ => {},
        }
    });
}
