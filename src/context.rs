use glow::HasContext;

#[cfg(target_arch = "wasm32")]
use winit::platform::web::WindowExtWebSys;

#[cfg(target_arch = "wasm32")]
#[link(wasm_import_module = "./helpers.js")]
extern {
    fn js_track_resized_setup();
    fn js_poll_resized() -> bool;
}

#[cfg(not(target_arch = "wasm32"))]
pub struct Context {
    pub render_width: f32,
    pub render_height: f32,
    pub glfw: std::cell::RefCell<glfw::Glfw>,
    pub window: std::cell::RefCell<glfw::PWindow>,
    pub gl: glow::Context,
    pub emptyvao: glow::VertexArray,
    pub start_instant: std::time::Instant,
}


#[cfg(target_arch = "wasm32")]
pub struct Context {
    pub render_width: f32,
    pub render_height: f32,
    pub window: winit::window::Window,
    pub gl: glow::Context,
    pub emptyvao: glow::VertexArray,
    pub performance: web_sys::Performance,
}

impl Context {
    pub fn compute_upscale(&self, windoww: u32, windowh: u32) -> u32 {
        let mut ratio = 1;
        loop {
            if (self.render_width as u32) * ratio > windoww
                || (self.render_height as u32) * ratio > windowh
            {
                break;
            }
            ratio += 1;
        }
        (ratio - 1).max(1)
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn new(glfw: std::cell::RefCell<glfw::Glfw>, window: std::cell::RefCell<glfw::PWindow>, gl: glow::Context, render_width: f32, render_height: f32) -> Self {
        let emptyvao = unsafe {
            gl.create_vertex_array().expect("failed to initialize vao")
        };
        let ret = Self {
            render_width, render_height,
            glfw, window,
            gl,
            emptyvao,
            start_instant: std::time::Instant::now(),
        };
        ret.init();
        ret
    }

    #[cfg(target_arch = "wasm32")]
    pub fn new(window: winit::window::Window, gl: glow::Context, render_width: f32, render_height: f32) -> Self {
        let emptyvao = unsafe {
            gl.create_vertex_array().expect("failed to initialize vao")
        };

        #[cfg(target_arch = "wasm32")]
        unsafe { js_track_resized_setup(); }

        let ret = Self {
            render_width, render_height,
            window,
            gl,
            emptyvao,

            #[cfg(target_arch = "wasm32")]
            performance: web_sys::window().expect("failed to find window")
                .performance().expect("failed to get performance"),
        };
        ret.init();
        ret
    }

    pub fn init(&self) {
        unsafe {
            self.gl.clear_color(0.1, 0.1, 0.1, 1.0);
            self.gl.clear_depth_f32(1.0);

            self.gl.enable(glow::DEPTH_TEST);
            self.gl.depth_func(glow::LEQUAL);

            self.gl.enable(glow::BLEND);
            self.gl.blend_func(glow::SRC_ALPHA, glow::ONE_MINUS_SRC_ALPHA);

            self.gl.enable(glow::STENCIL_TEST);

            self.gl.cull_face(glow::FRONT);
        }
    }

    #[cfg(target_arch = "wasm32")]
    pub fn maximize_canvas(&self) {
        web_sys::window()
            .and_then(|win| win.document())
            .and_then(|doc| {
                let inner_size = {
                    let browser_window = doc.default_view()
                        .or_else(web_sys::window)
                        .unwrap();
                    winit::dpi::PhysicalSize::new(
                        browser_window.inner_width().unwrap().as_f64().unwrap(),
                        browser_window.inner_height().unwrap().as_f64().unwrap(),
                    )
                };
                self.window.canvas().unwrap().set_width(inner_size.width as _);
                self.window.canvas().unwrap().set_height(inner_size.height as _);
                let _ = self.window.request_inner_size(inner_size);
                Some(())
            })
            .expect("failed to resize canvas");
    }

    #[cfg(target_arch = "wasm32")]
    pub fn resize_necessary(&self) -> bool {
        unsafe {
            js_poll_resized()
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn resize_necessary(&self) -> bool {
        false
    }

    pub fn clear_color(&self, color: glam::Vec4) {
        unsafe {
            self.gl.clear_color(color.x, color.y, color.z, color.w);
        }
    }

    pub fn clear_depth(&self) {
        unsafe {
            self.gl.clear(glow::DEPTH_BUFFER_BIT);
        }
    }

    pub fn clear(&self) {
        unsafe {
            self.gl.stencil_mask(0xff);
            self.gl.clear(glow::COLOR_BUFFER_BIT | glow::DEPTH_BUFFER_BIT | glow::STENCIL_BUFFER_BIT);
        }
    }

    pub fn begin_stencil(&self) {
        unsafe {
            self.gl.stencil_func(glow::ALWAYS, 1, 0xff);
            self.gl.stencil_op(glow::KEEP, glow::KEEP, glow::REPLACE);
        }
    }

    pub fn use_stencil(&self) {
        unsafe {
            self.gl.stencil_func(glow::EQUAL, 1, 0xff);
            self.gl.stencil_op(glow::KEEP, glow::KEEP, glow::KEEP);
        }
    }

    pub fn end_stencil(&self) {
        unsafe {
            self.gl.stencil_func(glow::ALWAYS, 1, 0xff);
            self.gl.stencil_op(glow::KEEP, glow::KEEP, glow::KEEP);
        }
    }

    pub fn render_no_geometry(&self) {
        unsafe {
            self.gl.bind_vertex_array(Some(self.emptyvao));
            self.gl.draw_arrays(glow::TRIANGLE_STRIP, 0, 4);
        }
    }

    pub fn reset_blend(&self) {
        unsafe {
            self.gl.blend_func(glow::SRC_ALPHA, glow::ONE_MINUS_SRC_ALPHA);
        }
    }

    pub fn inverse_blend(&self) {
        unsafe {
            self.gl.blend_func(
                glow::ONE_MINUS_DST_COLOR,
                glow::ZERO,
            );
        }
    }

    pub fn enable_culling(&self) {
        unsafe {
            self.gl.enable(glow::CULL_FACE);
        }
    }

    pub fn disable_culling(&self) {
        unsafe {
            self.gl.disable(glow::CULL_FACE);
        }
    }
}
