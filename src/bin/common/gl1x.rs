// wrap some GL 1.x stuff that this demo needs, but gl-rs doesn't wrap
use std;
use std::mem;
use std::ffi::CString;
use osvr_sys;

pub const PROJECTION: u32 = 0x1701;
pub const MODELVIEW: u32 = 0x1700;
pub const POLYGON: u32 = 0x0009;

pub fn init() {
    unsafe {
        glBegin = Some(std::mem::transmute(osvr_sys::SDL_GL_GetProcAddress(CString::new("glBegin").unwrap().as_ptr())));
        glEnd = Some(std::mem::transmute(osvr_sys::SDL_GL_GetProcAddress(CString::new("glEnd").unwrap().as_ptr())));
        glPushMatrix = Some(std::mem::transmute(osvr_sys::SDL_GL_GetProcAddress(CString::new("glPushMatrix").unwrap().as_ptr())));
        glPopMatrix = Some(std::mem::transmute(osvr_sys::SDL_GL_GetProcAddress(CString::new("glPopMatrix").unwrap().as_ptr())));
        glLoadIdentity = Some(std::mem::transmute(osvr_sys::SDL_GL_GetProcAddress(CString::new("glLoadIdentity").unwrap().as_ptr())));
        glMultMatrixd = Some(std::mem::transmute(osvr_sys::SDL_GL_GetProcAddress(CString::new("glMultMatrixd").unwrap().as_ptr())));
        glMatrixMode = Some(std::mem::transmute(osvr_sys::SDL_GL_GetProcAddress(CString::new("glMatrixMode").unwrap().as_ptr())));
        glVertex3f = Some(std::mem::transmute(osvr_sys::SDL_GL_GetProcAddress(CString::new("glVertex3f").unwrap().as_ptr())));
        glNormal3f = Some(std::mem::transmute(osvr_sys::SDL_GL_GetProcAddress(CString::new("glNormal3f").unwrap().as_ptr())));
        glColor3fv = Some(std::mem::transmute(osvr_sys::SDL_GL_GetProcAddress(CString::new("glColor3fv").unwrap().as_ptr())));
        glScaled = Some(std::mem::transmute(osvr_sys::SDL_GL_GetProcAddress(CString::new("glScaled").unwrap().as_ptr())));
    }
}

pub fn LoadIdentity() { unsafe { (glLoadIdentity.unwrap())() } }
pub fn Begin(mode: u32) { unsafe { (glBegin.unwrap())(mode) } }
pub fn End() { unsafe { (glEnd.unwrap())() } }
pub fn PushMatrix() { unsafe { (glPushMatrix.unwrap())() } }
pub fn PopMatrix() { unsafe { (glPopMatrix.unwrap())() } }
pub fn MultMatrixd(matrix: &[f64]) {
    assert!(matrix.len() == 16);
    unsafe { (glMultMatrixd.unwrap())(&matrix[0] as *const f64) }
}
pub fn MatrixMode(mode: u32) { unsafe { (glMatrixMode.unwrap())(mode) } }
pub fn Vertex3f(x: f32, y: f32, z: f32) { unsafe { (glVertex3f.unwrap())(x, y, z) } }
pub fn Normal3f(x: f32, y: f32, z: f32) { unsafe { (glNormal3f.unwrap())(x, y, z) } }
pub fn Color3fv(color: &[f32]) {
    assert!(color.len() == 3);
    unsafe { (glColor3fv.unwrap())(&color[0] as *const f32) }
}
pub fn Scaled(x: f64, y: f64, z: f64) { unsafe { (glScaled.unwrap())(x, y, z) } }

static mut glBegin: Option<extern "C" fn(mode: u32)> = None;
static mut glEnd: Option<extern "C" fn()> = None;
static mut glPushMatrix: Option<extern "C" fn()> = None;
static mut glPopMatrix: Option<extern "C" fn()> = None;
static mut glLoadIdentity: Option<extern "C" fn()> = None;
static mut glMultMatrixd: Option<extern "C" fn(matrix: *const f64)> = None;
static mut glMatrixMode: Option<extern "C" fn(mode: u32)> = None;
static mut glVertex3f: Option<extern "C" fn(x: f32, y: f32, z: f32)> = None;
static mut glNormal3f: Option<extern "C" fn(x: f32, y: f32, z: f32)> = None;
static mut glColor3fv: Option<extern "C" fn(color: *const f32)> = None;
static mut glScaled: Option<extern "C" fn(x: f64, y: f64, z: f64)> = None;
