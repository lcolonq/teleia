use super::egl::*;

#[link(name = "glfw3")]
unsafe extern "C" {
    pub fn glfwGetEGLDisplay() -> EGLDisplay;
    pub fn glfwGetEGLContext(w: *mut glfw::ffi::GLFWwindow) -> EGLContext; 
}
