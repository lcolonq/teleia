use std::{cell::RefCell, collections::HashMap};

#[cfg(target_arch = "wasm32")]
pub struct Context {
    pub audio: web_sys::AudioContext,
}

#[cfg(target_arch = "wasm32")]
impl Context {
    pub fn new() -> Self {
        let audio = web_sys::AudioContext::new()
            .expect("failed to create audio context");
        Self {
            audio,
        }
    }
}

#[cfg(target_arch = "wasm32")]
pub struct Audio {
    pub buffer: &'static RefCell<Option<web_sys::AudioBuffer>>,
    //pub source: &'static web_sys::AudioBufferSourceNode,
}

#[cfg(target_arch = "wasm32")]
impl Audio {
    pub fn new(ctx: &Context, bytes: &[u8]) -> Self {
        let sbuffer: &_ = Box::leak(Box::new(RefCell::new(None)));
        let sclone: &'static RefCell<Option<web_sys::AudioBuffer>> =
            <&_>::clone(&sbuffer);
        let ret = Audio {
            buffer: sclone,
        };
        let jsp = ctx.audio.decode_audio_data(&js_sys::Uint8Array::from(bytes).buffer()).expect("failed to decode audio");
        let promise = wasm_bindgen_futures::JsFuture::from(jsp);
        wasm_bindgen_futures::spawn_local(async {
            if let Some(data) = promise.await.ok() {
                *sbuffer.borrow_mut() = Some(web_sys::AudioBuffer::from(data));
            }
            ()
        });
        ret
    }

    pub fn play(&self, ctx: &Context, looping: Option<(Option<f64>, Option<f64>)>) -> Option<web_sys::AudioBufferSourceNode> {
        let source = ctx.audio.create_buffer_source().ok()?;
        source.set_buffer((&*self.buffer.borrow()).as_ref());
        if let Some((ms, me)) = looping {
            source.set_loop(true);
            if let Some(s) = ms { source.set_loop_start(s) }
            if let Some(e) = me { source.set_loop_end(e) }
        }
        source.connect_with_audio_node(&ctx.audio.destination()).ok()?;
        source.start().ok()?;
        Some(source)
    }
}

#[cfg(target_arch = "wasm32")]
pub struct Assets {
    pub ctx: Context,

    pub audio: HashMap<String, Audio>,

    pub music_node: Option<web_sys::AudioBufferSourceNode>, 
}

#[cfg(target_arch = "wasm32")]
impl Assets {
    pub fn new<F>(f : F) -> Self where F: Fn(&Context) -> HashMap<String, Audio> {
        let ctx = Context::new();

        let audio = f(&ctx);

        Self {
            ctx,
            audio,
            music_node: None,
        }
    }

    pub fn play_sfx(&mut self, name: &str) {
        if let Some(a) = self.audio.get(name) {
            a.play(&self.ctx, None);
        }
    }

    pub fn is_music_playing(&self) -> bool {
        if let Some(ms) = &self.music_node {
            ms.buffer().is_some()
        } else { false }
    }

    pub fn play_music(&mut self, name: &str, start: Option<f64>, end: Option<f64>) {
        if let Some(s) = &self.music_node {
            let _ = s.stop();
        }
        if let Some(a) = self.audio.get(name) {
            self.music_node = a.play(&self.ctx, Some((start, end)));
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub struct Context {
}

#[cfg(not(target_arch = "wasm32"))]
impl Context {
    pub fn new() -> Self {
        Self {
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub struct Audio {
}

#[cfg(not(target_arch = "wasm32"))]
impl Audio {
    pub fn new(ctx: &Context, _bytes: &[u8]) -> Self {
        Self {
        }
    }

    pub fn play(&self, ctx: &Context, looping: Option<(Option<f64>, Option<f64>)>) {
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub struct Assets {
    pub ctx: Context,
    pub audio: HashMap<String, Audio>,
}

#[cfg(not(target_arch = "wasm32"))]
impl Assets {
    pub fn new<F>(f : F) -> Self where F: Fn(&Context) -> HashMap<String, Audio> {
        let ctx = Context::new();

        let audio = f(&ctx);

        Self {
            ctx,
            audio,
        }
    }

    pub fn play_sfx(&mut self, name: &str) {
        if let Some(a) = self.audio.get(name) {
            a.play(&self.ctx, None);
        }
    }

    pub fn is_music_playing(&self) -> bool {
        false
    }

    pub fn play_music(&mut self, name: &str, start: Option<f64>, end: Option<f64>) {
    }
}
