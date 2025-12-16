use std::collections::HashMap;

#[cfg(target_arch = "wasm32")]
use std::sync::{Arc, Mutex};

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
pub struct AudioPlayingHandle {
    node: web_sys::AudioBufferSourceNode,
    gain: web_sys::GainNode,
}

#[cfg(target_arch = "wasm32")]
impl AudioPlayingHandle {
    pub fn stop(&self, _ctx: &Context) {
        self.node.stop().expect("failed to stop audio");
    }
    pub fn fade_out(&self, ctx: &Context, time: f32) {
        let t = ctx.audio.current_time() + time as f64;
        self.gain.gain().set_target_at_time(0.0, ctx.audio.current_time(), time as f64)
            .expect("failed to fade out audio");
        // self.node.stop_with_when(t).expect("failed to stop audio while fading out");
    }
}

#[cfg(target_arch = "wasm32")]
pub struct Audio {
    pub buffer: Arc<Mutex<Option<web_sys::AudioBuffer>>>,
}

#[cfg(target_arch = "wasm32")]
impl Audio {
    pub fn new(ctx: &Context, bytes: &[u8]) -> Self {
        let sbuffer = Arc::new(Mutex::new(None));
        let sclone = sbuffer.clone();
        let ret = Audio {
            buffer: sclone,
        };
        let jsp = ctx.audio.decode_audio_data(&js_sys::Uint8Array::from(bytes).buffer()).expect("failed to decode audio");
        let promise = wasm_bindgen_futures::JsFuture::from(jsp);
        wasm_bindgen_futures::spawn_local(async move {
            if let Some(data) = promise.await.ok() {
                *sbuffer.lock().unwrap() = Some(web_sys::AudioBuffer::from(data));
            }
            ()
        });
        ret
    }

    pub fn from_samples(ctx: &Context, sample_rate: f32, samples: &[f32]) -> Self {
        let buf = ctx.audio.create_buffer(1, samples.len() as u32, sample_rate)
            .expect("failed to create audio buffer");
        buf.copy_to_channel(samples, 0).expect("failed to populate audio samples");
        Audio {
            buffer: Arc::new(Mutex::new(Some(buf)))
        }
    }

    pub fn play(&self, ctx: &Context, looping: Option<(Option<f64>, Option<f64>)>) -> Option<AudioPlayingHandle> {
        let source = ctx.audio.create_buffer_source().ok()?;
        if let Some(ab) = &*self.buffer.lock().unwrap() {
            source.set_buffer(Some(&ab));
        } else { return None };
        if let Some((ms, me)) = looping {
            source.set_loop(true);
            if let Some(s) = ms { source.set_loop_start(s) }
            if let Some(e) = me { source.set_loop_end(e) }
        }
        let gain = ctx.audio.create_gain().ok()?;
        gain.gain().set_value_at_time(0.5, ctx.audio.current_time()).ok()?;
        gain.connect_with_audio_node(&ctx.audio.destination()).ok()?;
        source.connect_with_audio_node(&gain).ok()?;
        source.start().ok()?;
        Some(AudioPlayingHandle { node: source, gain })
    }
}

#[cfg(target_arch = "wasm32")]
pub struct Assets {
    pub ctx: Context,
    pub audio: HashMap<String, Audio>,
    pub music_node: Option<AudioPlayingHandle>,
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
            ms.node.buffer().is_some()
        } else { false }
    }

    pub fn play_music(&mut self, name: &str, start: Option<f64>, end: Option<f64>) {
        if let Some(s) = &self.music_node {
            let _ = s.stop(&self.ctx);
        }
        if let Some(a) = self.audio.get(name) {
            self.music_node = a.play(&self.ctx, Some((start, end)));
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub struct Context {
    manager: kira::manager::AudioManager,
}

#[cfg(not(target_arch = "wasm32"))]
impl Context {
    pub fn new() -> Self {
        Self {
            manager: kira::manager::AudioManager::new(kira::manager::AudioManagerSettings::default())
                .expect("failed to create audio manager"),
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub struct Audio {
    data: kira::sound::static_sound::StaticSoundData,
}

#[cfg(not(target_arch = "wasm32"))]
impl Audio {
    pub fn new(_ctx: &Context, bytes: &'static [u8]) -> Self {
        Self {
            data: kira::sound::static_sound::StaticSoundData::from_cursor(std::io::Cursor::new(bytes))
                .expect("failed to decode audio"),
        }
    }

    pub fn play(
        &self,
        ctx: &mut Context,
        looping: Option<(Option<f64>, Option<f64>)>
    ) -> Result<kira::sound::static_sound::StaticSoundHandle, String>
    {
        let sd = if let Some((ss, se)) = looping {
            let start = if let Some(s) = ss { s } else { 0.0 };
            if let Some(e) = se {
                self.data.loop_region(start..e)
            } else {
                self.data.loop_region(start..)
            }
        } else {
            self.data.clone()
        };
        match ctx.manager.play(sd) {
            Ok(h) => Ok(h),
            Err(e) => Err(e.to_string()),
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub struct Assets {
    pub ctx: Context,
    pub audio: HashMap<String, Audio>,
    pub music_handle: Option<kira::sound::static_sound::StaticSoundHandle>, 
}

#[cfg(not(target_arch = "wasm32"))]
impl Assets {
    pub fn new<F>(f : F) -> Self where F: Fn(&Context) -> HashMap<String, Audio> {
        let ctx = Context::new();

        let audio = f(&ctx);

        Self {
            ctx,
            audio,
            music_handle: None,
        }
    }

    pub fn play_sfx(&mut self, name: &str) {
        if let Some(a) = self.audio.get(name) {
            if let Err(e) = a.play(&mut self.ctx, None) {
                log::warn!("failed to play sound {}: {}", name, e);
            }
        }
    }

    pub fn is_music_playing(&self) -> bool {
        if let Some(mh) = &self.music_handle {
            mh.state() == kira::sound::PlaybackState::Playing
        } else { false }
    }

    pub fn play_music(&mut self, name: &str, start: Option<f64>, end: Option<f64>) {
        if let Some(s) = &mut self.music_handle {
            let _ = s.stop(kira::tween::Tween::default());
        }
        if let Some(a) = self.audio.get(name) {
            match a.play(&mut self.ctx, Some((start, end))) {
                Ok(h) => {
                    self.music_handle = Some(h);
                },
                Err(e) => {
                    log::warn!("failed to play music {}: {}", name, e);
                }
            }
        }
    }
}

pub trait AudioPlayback {
    fn play_sfx(&mut self, name: &str);
    fn play_music(&mut self, name: &str, start: Option<f64>, end: Option<f64>);
}
impl AudioPlayback for Option<Assets> {
    fn play_sfx(&mut self, name: &str) {
        if let Some(a) = self { a.play_sfx(name); }
    }
    fn play_music(&mut self, name: &str, start: Option<f64>, end: Option<f64>) {
        if let Some(a) = self { a.play_music(name, start, end); }
    }
}
