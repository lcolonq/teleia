pub mod utils;
pub mod ui;
pub mod context;
pub mod state;
pub mod framebuffer;
pub mod shader;
pub mod mesh;
pub mod texture;
pub mod font;
pub mod shadow;
pub mod audio;

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

pub fn event_loop_body<G>(event: winit::event::Event<()>, elwt: &winit::event_loop::EventLoopWindowTarget<()>)
    where G: state::Game + 'static,
{
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
}

pub async fn run<'a, F, G, Fut>(gnew: F)
where
    Fut: std::future::Future<Output = G>,
    G: state::Game + 'static,
    F: (Fn(&'a context::Context) -> Fut),
{
    #[cfg(target_arch = "wasm32")]
    {
        console_log::init_with_level(log::Level::Debug).unwrap();
        console_error_panic_hook::set_once();
        tracing_wasm::set_as_global_default();
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        env_logger::Builder::new()
            .filter(None, log::LevelFilter::Info)
            .init();
    }
    log::info!("hello computer, starting up...");

    let event_loop = winit::event_loop::EventLoop::new()
        .expect("failed to initialize event loop");

    #[cfg(target_arch = "wasm32")]
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

    #[cfg(not(target_arch = "wasm32"))]
    let (window, gl) = {
        use glutin::config::GlConfig;
        use glutin::context::NotCurrentGlContext;
        use glutin::display::{GlDisplay, GetGlDisplay};
        use glutin::surface::GlSurface;
        use raw_window_handle::HasRawWindowHandle;
        use glutin_winit::GlWindow;
        let window_builder = winit::window::WindowBuilder::new()
            .with_title("teleia")
            .with_maximized(true)
            .with_decorations(false);
        let template = glutin::config::ConfigTemplateBuilder::new();
        let display_builder = glutin_winit::DisplayBuilder::new().with_window_builder(Some(window_builder));
        let (window, gl_config) = display_builder
            .build(&event_loop, template, |configs| {
                configs.reduce(|a, c| {
                    if c.num_samples() > a.num_samples() { c } else { a }
                }).expect("failed to obtain select configuration")
            }).expect("failed to obtain opengl display");
        let window = window.expect("failed to create window");
        let raw_window_handle = window.raw_window_handle();
        let gl_display = gl_config.display();
        let context_attributes = glutin::context::ContextAttributesBuilder::new()
            // .with_context_api(glutin::context::ContextApi::OpenGl(Some(glutin::context::Version {
            //     major: 3,
            //     minor: 3,
            // })))
            .build(Some(raw_window_handle));
        unsafe {
            let not_current_gl_context = gl_display.create_context(&gl_config, &context_attributes)
                .expect("failed to obtain opengl context");
            let attrs = window.build_surface_attributes(Default::default());
            let gl_surface = gl_display.create_window_surface(&gl_config, &attrs)
                .expect("failed to create opengl surface");
            let gl_context = not_current_gl_context.make_current(&gl_surface)
                .expect("failed to set openglt context");
            let gl = glow::Context::from_loader_function_cstr(|s| gl_display.get_proc_address(s));
            gl_surface
                .set_swap_interval(&gl_context, glutin::surface::SwapInterval::Wait(std::num::NonZeroU32::new(1).unwrap()))
                .expect("failed to set swap interval");
            (window, gl)
        }
    };

    let ctx = Box::leak(Box::new(context::Context::new(window, gl)));

    #[cfg(target_arch = "wasm32")]
    {
        ctx.maximize_canvas();
    }

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


    #[cfg(target_arch = "wasm32")]
    {
        event_loop.set_control_flow(winit::event_loop::ControlFlow::Wait);
        event_loop.spawn(|event, elwt| {
            event_loop_body::<G>(event, elwt);
        });
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        event_loop.set_control_flow(winit::event_loop::ControlFlow::Poll);
        event_loop.run(|event, elwt| {
            event_loop_body::<G>(event, elwt);
        }).expect("window closed");
    }
}
