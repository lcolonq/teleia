// from <EGL/egl.h>
pub const EGL_NONE: EGLenum = 0x3038;
pub const EGL_WIDTH: EGLenum = 0x3057;
pub const EGL_HEIGHT: EGLenum = 0x3056;
pub const EGL_GL_TEXTURE_2D: EGLenum = 0x30B1;

// from <EGL/eglext.h>
pub const EGL_LINUX_DMA_BUF_EXT: EGLenum = 0x3270;
pub const EGL_LINUX_DRM_FOURCC_EXT: EGLenum = 0x3271;
pub const EGL_DMA_BUF_PLANE0_FD_EXT: EGLenum = 0x3272;
pub const EGL_DMA_BUF_PLANE0_OFFSET_EXT: EGLenum = 0x3273;
pub const EGL_DMA_BUF_PLANE0_PITCH_EXT: EGLenum = 0x3274;

#[repr(C)]
pub struct EGLDisplayTarget(());
pub type EGLDisplay = *mut EGLDisplayTarget;

#[repr(C)]
pub struct EGLContextTarget(());
pub type EGLContext = *mut EGLContextTarget;

#[repr(C)]
pub struct EGLClientBufferTarget(());
pub type EGLClientBuffer = *mut EGLClientBufferTarget;

#[repr(C)]
pub struct EGLImageTarget(());
pub type EGLImage = *mut EGLImageTarget;

#[repr(C)]
pub struct MustCastToFunctionTarget(());
pub type MustCastToFunction = *mut MustCastToFunctionTarget;

pub type EGLenum = core::ffi::c_uint;
pub type EGLBoolean = core::ffi::c_uint;

pub type EGLint = core::ffi::c_int;

pub type EGLAttrib = libc::intptr_t;

pub type PFNEGLEXPORTDMABUFIMAGEQUERYMESAPROC = fn(EGLDisplay, EGLImage, *mut core::ffi::c_int, *mut core::ffi::c_int, u64) -> EGLBoolean;
pub type PFNEGLEXPORTDMABUFIMAGEMESAPROC = fn(EGLDisplay, EGLImage, *mut core::ffi::c_int, *mut EGLint, *mut EGLint) -> EGLBoolean;

#[link(name = "EGL")]
unsafe extern "C" {
    pub fn eglCreateImage(d: EGLDisplay, c: EGLContext, t: EGLenum, buf: EGLClientBuffer, attribs: *const EGLAttrib) -> EGLImage;
    pub fn eglGetError() -> EGLint;
    pub fn eglGetProcAddress(nm: *const core::ffi::c_char) -> MustCastToFunction;
}

#[link(name = "GL")]
unsafe extern "C" {
    pub fn glEGLImageTargetTexture2DOES(t: u32, img: EGLImage);
}
