mod ffi;

use byteorder::{LE, ReadBytesExt, WriteBytesExt};
use simple_eyre::eyre::Context;

use std::io::{IoSlice, IoSliceMut};
// use std::io::Write;
// use std::os::fd::FromRawFd;
use std::os::unix::net::{AncillaryData, SocketAncillary};
use std::sync::mpsc::{Receiver, channel};
use std::{os::unix::net::UnixDatagram, thread::{spawn, JoinHandle}};

// use glow::HasContext;
use ffi::egl;
use ffi::glfw;

use crate::{context, texture, utils};

#[derive(Debug)]
struct ReceivedTexture {
    fd: i32,
    width: i32, height: i32,
    fourcc: u32, offset: i32, stride: i32,
}

#[allow(dead_code)]
pub struct Server {
    pub tex: texture::Texture,
    thread: JoinHandle<()>,
    receiver: Receiver<ReceivedTexture>,
}
impl Server {
    pub fn new(ctx: &context::Context, path: &str) -> Self {
        let _ = std::fs::remove_file(path);
        let sock = UnixDatagram::bind(path).expect("failed to bind texture server socket");

        let (sender, receiver) = channel::<ReceivedTexture>();
        let thread = spawn(move || {
            let mut buf = [0; 1024];
            let mut ancillary_buf = [0; 128];
            let mut ancillary = SocketAncillary::new(&mut ancillary_buf);
            loop {
                match sock.recv_vectored_with_ancillary(&mut [IoSliceMut::new(&mut buf)], &mut ancillary) {
                    Ok(_) => {
                        let res: utils::Erm<()> = try {
                            let mut cur = std::io::Cursor::new(&buf);
                            let width = cur.read_i32::<LE>().wrap_err("failed to read width")?;
                            let height = cur.read_i32::<LE>().wrap_err("failed to read height")?;
                            let fourcc = cur.read_u32::<LE>().wrap_err("failed to read fourcc")?;
                            let offset = cur.read_i32::<LE>().wrap_err("failed to read offset")?;
                            let stride = cur.read_i32::<LE>().wrap_err("failed to read stride")?;
                            for ar in ancillary.messages() {
                                match ar {
                                    Ok(AncillaryData::ScmRights(mut sr)) => {
                                        if let Some(fd) = sr.next() {
                                            let rt = ReceivedTexture {
                                                fd, width, height,
                                                fourcc, offset, stride
                                            };
                                            if let Err(e) = sender.send(rt) {
                                                log::warn!("error on channel: {:?}", e);
                                                break;
                                            }
                                        }
                                    },
                                    Ok(_) => log::info!("message is not SCM_RIGHTS"),
                                    Err(e) => log::warn!("failed to read ancillary message: {:?}", e),
                                }
                            };
                        };
                        if let Err(e) = res {
                            log::warn!("error when receiving texture: {}", e);
                        }
                    },
                    Err(e) => log::warn!("texture server failed to receive: {}", e),
                }
            }
        });
        Self {
            tex: texture::Texture::new_empty(ctx),
            thread,
            receiver,
        }
    }
    pub fn update(&self, ctx: &context::Context) {
        if let Ok(rt) = self.receiver.try_recv() {
            log::info!("received file descriptor: {:?}", rt);
            unsafe {
                let window = ctx.window.borrow_mut().0.ptr;
                let d = glfw::glfwGetEGLDisplay();
                let c = glfw::glfwGetEGLContext(window);
                log::info!("d: {:?}, c: {:?}", d, c);
                let img = egl::eglCreateImage(d, std::ptr::null_mut(), egl::EGL_LINUX_DMA_BUF_EXT, std::ptr::null_mut(), [
                    egl::EGL_WIDTH as egl::EGLAttrib, rt.width as _,
                    egl::EGL_HEIGHT as _, rt.height as _,
                    egl::EGL_LINUX_DRM_FOURCC_EXT as _, rt.fourcc as _,
                    egl::EGL_DMA_BUF_PLANE0_FD_EXT as _, rt.fd as _,
                    egl::EGL_DMA_BUF_PLANE0_OFFSET_EXT as _, rt.offset as _,
                    egl::EGL_DMA_BUF_PLANE0_PITCH_EXT as _, rt.stride as _,
                    egl::EGL_NONE as _,
                ].as_ptr());
                let _ = libc::close(rt.fd);
                log::info!("error here: {}", egl::eglGetError());
                self.tex.bind(ctx);
                egl::glEGLImageTargetTexture2DOES(glow::TEXTURE_2D, img);
                log::info!("set up texture: {}", egl::eglGetError());
            }
        }
    }
}

pub fn share_texture(ctx: &context::Context, path: &str, tex: &texture::Texture) -> utils::Erm<()> {
    unsafe {
        #[allow(non_snake_case)]
        let eglExportDMABUFImageQueryMESA =
            std::mem::transmute::<egl::MustCastToFunction, egl::PFNEGLEXPORTDMABUFIMAGEQUERYMESAPROC>(
                egl::eglGetProcAddress(c"eglExportDMABUFImageQueryMESA".as_ptr())
            );
        #[allow(non_snake_case)]
        let eglExportDMABUFImageMESA =
            std::mem::transmute::<egl::MustCastToFunction, egl::PFNEGLEXPORTDMABUFIMAGEMESAPROC>(
                egl::eglGetProcAddress(c"eglExportDMABUFImageMESA".as_ptr())
            );
        let window = ctx.window.borrow_mut().0.ptr;
        let d = glfw::glfwGetEGLDisplay();
        let c = glfw::glfwGetEGLContext(window);
        let img = egl::eglCreateImage(d, c, egl::EGL_GL_TEXTURE_2D,
            std::ptr::with_exposed_provenance_mut(tex.tex.0.get() as usize),
            std::ptr::null_mut()
        );
        let mut imgfd: std::ffi::c_int = 0;
        let mut fourcc: std::ffi::c_int = 0;
        let mut offset: egl::EGLint = 0;
        let mut stride: egl::EGLint = 0;
        eglExportDMABUFImageQueryMESA(d, img, &mut fourcc as *mut _, std::ptr::null_mut(), 0);
        eglExportDMABUFImageMESA(d, img,
            &mut imgfd as *mut _, &mut stride as *mut _, &mut offset as *mut _
        );
        log::info!("fd: {imgfd}, fourcc: {fourcc}, offset: {offset}, stride: {stride}");
        let mut buf: Vec<u8> = Vec::new();
        buf.write_i32::<LE>(tex.width)?;
        buf.write_i32::<LE>(tex.height)?;
        buf.write_u32::<LE>(fourcc as u32)?;
        buf.write_i32::<LE>(offset)?;
        buf.write_i32::<LE>(stride)?;
        let mut ancillary_buf = [0; 128];
        let mut ancillary = SocketAncillary::new(&mut ancillary_buf[..]);
        ancillary.add_fds(&[imgfd]);
        let sock = UnixDatagram::unbound().expect("failed to create unix socket");
        sock.send_vectored_with_ancillary_to(&[IoSlice::new(&buf)], &mut ancillary, path)?;
        let _ = libc::close(imgfd);
        Ok(())
    }
}
