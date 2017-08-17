extern crate osvr_sys;
extern crate gl;
extern crate sdl2;
use std::ffi::CString;
use std::ptr;
use std::mem;
use std::vec::Vec;

mod gl1x {
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
}

extern "C" fn myButtonCallback(userdata: *mut ::std::os::raw::c_void, timestamp: *const osvr_sys::OSVR_TimeValue, report: *const osvr_sys::OSVR_ButtonReport)
{
    unsafe {
        println!("Button state: {}", (*report).state);
        // quit when buttons are pressed
        let quit = userdata as *mut bool;
        *quit = (*report).state != 0;
    }
}

fn main() {
    let mut quit = false;

    unsafe {
        let title = CString::new("Rust OSVR example").unwrap();
        let opengl_str = CString::new("OpenGL").unwrap();
        let context = osvr_sys::osvrClientInit(title.as_ptr(), 0);

        let mut leftButton1: osvr_sys::OSVR_ClientInterface = mem::zeroed();
        osvr_sys::osvrClientGetInterface(context, CString::new("/controller/left/1").unwrap().as_ptr(), &mut leftButton1);
        osvr_sys::osvrRegisterButtonCallback(leftButton1, Some(myButtonCallback), std::mem::transmute(&mut quit));

        let mut rightButton1: osvr_sys::OSVR_ClientInterface = mem::zeroed();
        osvr_sys::osvrClientGetInterface(context, CString::new("/controller/right/1").unwrap().as_ptr(), &mut rightButton1);
        osvr_sys::osvrRegisterButtonCallback(rightButton1, Some(myButtonCallback), std::mem::transmute(&mut quit));

        let library: osvr_sys::OSVR_GraphicsLibraryOpenGL = mem::zeroed();
        let mut render: osvr_sys::OSVR_RenderManager = mem::zeroed();
        let mut renderOGL: osvr_sys::OSVR_RenderManagerOpenGL = mem::zeroed();
        

        if 0 != osvr_sys::osvrCreateRenderManagerOpenGL(context, opengl_str.as_ptr(), library, &mut render, &mut renderOGL) {
            eprintln!("Could not create render manager");
            return;
        }
        // why don't we reach here!

        let mut openResults: osvr_sys::OSVR_OpenResultsOpenGL = std::mem::zeroed();

        if 0 != osvr_sys::osvrRenderManagerOpenDisplayOpenGL(renderOGL, &mut openResults) {
            eprintln!("Could not open display");
            return;
        }

        // we have a GL context, so we can load GL function pointers
        gl::load_with(|s| {
            let name = CString::new(s).unwrap();
            osvr_sys::SDL_GL_GetProcAddress(name.as_ptr())
        });
        gl1x::init();

        setup_rendering();
        
        osvr_sys::osvrClientUpdate(context);

        let mut renderParams: osvr_sys::OSVR_RenderParams = std::mem::zeroed();
        osvr_sys::osvrRenderManagerGetDefaultRenderParams(&mut renderParams);

        let mut renderInfoCollection: osvr_sys::OSVR_RenderInfoCollection = std::mem::zeroed();
        if 0 != osvr_sys::osvrRenderManagerGetRenderInfoCollection(render, renderParams, &mut renderInfoCollection) {
            eprintln!("Could not get render info");
            return;
        }

        let mut numRenderInfo: osvr_sys::OSVR_RenderInfoCount = 0 as usize;
        osvr_sys::osvrRenderManagerGetNumRenderInfoInCollection(renderInfoCollection, &mut numRenderInfo);

        let mut colorBuffers: Vec<osvr_sys::OSVR_RenderBufferOpenGL> = Vec::new();
        let mut depthBuffers: Vec<osvr_sys::GLuint> = Vec::new();

        let mut frameBuffer: osvr_sys::GLuint = 0; //< Groups a color buffer and a depth buffer
        gl::GenFramebuffers(1, &mut frameBuffer);
        gl::BindFramebuffer(gl::FRAMEBUFFER, frameBuffer);

        let mut registerBufferState: osvr_sys::OSVR_RenderManagerRegisterBufferState = std::mem::uninitialized();
        if 0 != osvr_sys::osvrRenderManagerStartRegisterRenderBuffers(&mut registerBufferState) {
            eprintln!("Could not start registering render buffers");
            return;
        }

        for i in 0..numRenderInfo {
            let mut renderInfo: osvr_sys::OSVR_RenderInfoOpenGL = std::mem::zeroed();

            if 0 != osvr_sys::osvrRenderManagerGetRenderInfoFromCollectionOpenGL(renderInfoCollection, i, &mut renderInfo) {
                eprintln!("Could not get render info {}", i);
                osvr_sys::osvrDestroyRenderManager(render);
                return;
            }

            let width: i32 = renderInfo.viewport.width as i32;
            let height: i32 = renderInfo.viewport.height as i32;

            let mut colorBufferName: osvr_sys::GLuint = 0;
            if 0 != osvr_sys::osvrRenderManagerCreateColorBufferOpenGL(width, height, gl::RGBA, &mut colorBufferName) {
                eprintln!("Could not create color buffer.");
                osvr_sys::osvrDestroyRenderManager(render);
                return;
            }

            let mut rb: osvr_sys::OSVR_RenderBufferOpenGL = std::mem::zeroed();
            rb.colorBufferName = colorBufferName;
            
            colorBuffers.push(rb);

            let mut depthrenderbuffer: osvr_sys::GLuint = 0;
            if 0 != osvr_sys::osvrRenderManagerCreateDepthBufferOpenGL(width, height, &mut depthrenderbuffer) {
                eprintln!("Could not create depth buffer.");
                osvr_sys::osvrDestroyRenderManager(render);
                return;
            }
            rb.depthStencilBufferName = depthrenderbuffer;
            depthBuffers.push(depthrenderbuffer);

            if 0 != osvr_sys::osvrRenderManagerRegisterRenderBufferOpenGL(registerBufferState, rb) {
                eprintln!("Could not register render buffer {}.", i);
                osvr_sys::osvrDestroyRenderManager(render);
                return;
            }
        }

        if 0 != osvr_sys::osvrRenderManagerFinishRegisterRenderBuffers(render, registerBufferState, 0) {
            eprintln!("Could not start finish registering render buffers.");
            quit = true;
        }

        while !quit {
            osvr_sys::osvrClientUpdate(context);

            let mut renderInfoCollection: osvr_sys::OSVR_RenderInfoCollection = std::mem::zeroed();
            if 0 != osvr_sys::osvrRenderManagerGetRenderInfoCollection(render, renderParams, &mut renderInfoCollection) {
                eprintln!("Could not get render info in the main loop.");
                osvr_sys::osvrDestroyRenderManager(render);
                return;
            }

            let mut numRenderInfo: osvr_sys::OSVR_RenderInfoCount = 0 as usize;
            osvr_sys::osvrRenderManagerGetNumRenderInfoInCollection(renderInfoCollection, &mut numRenderInfo);

            for i in 0..numRenderInfo {
                let mut renderInfo: osvr_sys::OSVR_RenderInfoOpenGL = std::mem::zeroed();
                osvr_sys::osvrRenderManagerGetRenderInfoFromCollectionOpenGL(renderInfoCollection, i, &mut renderInfo);

                // then draw your GL scene for this eye here!
                renderView(&renderInfo, frameBuffer, colorBuffers[i].colorBufferName, depthBuffers[i]);
            }

            let mut presentState: osvr_sys::OSVR_RenderManagerPresentState = std::mem::zeroed();
            if 0 != osvr_sys::osvrRenderManagerStartPresentRenderBuffers(&mut presentState) {
                eprintln!("Could not start presenting render buffers.");
                osvr_sys::osvrDestroyRenderManager(render);
                return;
            }

            let fullView = osvr_sys::OSVR_ViewportDescription { left: 0.0, lower: 0.0, width: 1.0, height: 1.0};
            for i in 0..numRenderInfo {
                let mut renderInfo: osvr_sys::OSVR_RenderInfoOpenGL = std::mem::zeroed();
                osvr_sys::osvrRenderManagerGetRenderInfoFromCollectionOpenGL(renderInfoCollection, i, &mut renderInfo);

                if 0 != osvr_sys::osvrRenderManagerPresentRenderBufferOpenGL(presentState, colorBuffers[i], renderInfo, fullView) {
                    eprintln!("Could not present render buffer {}.", i);
                    osvr_sys::osvrDestroyRenderManager(render);
                    return;
                }
            }

            osvr_sys::osvrRenderManagerReleaseRenderInfoCollection(renderInfoCollection);

            if 0 != osvr_sys::osvrRenderManagerFinishPresentRenderBuffers(render, presentState, renderParams, 0) {
                eprintln!("Could not finish presenting render buffers.");
                quit = true;
            }
        }

        // Clean up after ourselves.
        gl::DeleteFramebuffers(1, &frameBuffer);
        for i in 0..colorBuffers.len() {
            gl::DeleteTextures(1, &colorBuffers[i].colorBufferName);
            gl::DeleteRenderbuffers(1, &depthBuffers[i]);
        }

        osvr_sys::osvrClientFreeInterface(context, leftButton1);
        osvr_sys::osvrClientFreeInterface(context, rightButton1);

        osvr_sys::osvrDestroyRenderManager(render);
        osvr_sys::osvrClientShutdown(context);
    }
}

fn ConvertProjectionMatrix(matrix: osvr_sys::OSVR_ProjectionMatrix) -> osvr_sys::OSVR_ProjectionMatrix 
{
    let mut ret: osvr_sys::OSVR_ProjectionMatrix = unsafe { std::mem::zeroed() };
    ret.bottom = matrix.bottom;
    ret.top = matrix.top;
    ret.left = matrix.left;
    ret.right = matrix.right;
    ret.nearClip = matrix.nearClip;
    ret.farClip = matrix.farClip;
    ret
}

fn setup_rendering()
{
    unsafe { gl::Enable(gl::DEPTH_TEST) };
}

fn renderView(renderInfo: &osvr_sys::OSVR_RenderInfoOpenGL, frameBuffer: osvr_sys::GLuint, colorBuffer: osvr_sys::GLuint, depthBuffer: osvr_sys::GLuint)
{
    unsafe {
        gl::BindFramebuffer(gl::FRAMEBUFFER, frameBuffer);

        // Set color and depth buffers for the frame buffer
        gl::FramebufferTexture2D(gl::FRAMEBUFFER, gl::COLOR_ATTACHMENT0, gl::TEXTURE_2D, colorBuffer, 0);
        gl::FramebufferRenderbuffer(gl::FRAMEBUFFER, gl::DEPTH_ATTACHMENT, gl::RENDERBUFFER, depthBuffer);

        // Set the list of draw buffers.
        let drawBuffer: osvr_sys::GLenum = gl::COLOR_ATTACHMENT0;
        gl::DrawBuffers(1, &drawBuffer); // "1" is the size of DrawBuffers

        // Always check that our framebuffer is ok
        if gl::CheckFramebufferStatus(gl::FRAMEBUFFER) != gl::FRAMEBUFFER_COMPLETE {
            eprintln!("RenderView: Incomplete Framebuffer");
            return;
        }

        // Set the viewport to cover our entire render texture.
        gl::Viewport(0, 0, renderInfo.viewport.width as i32, renderInfo.viewport.height as i32);

        // Set the OpenGL projection matrix
        let mut projection: [f64; 16] = std::mem::uninitialized();
        let mut temp: osvr_sys::OSVR_ProjectionMatrix;
        temp.bottom = renderInfo.projection.bottom;
        osvr_sys::OSVR_Projection_to_OpenGL(&mut projection[0] as *mut f64, ConvertProjectionMatrix(renderInfo.projection));

        gl1x::MatrixMode(gl1x::PROJECTION);
        gl1x::LoadIdentity();
        gl1x::MultMatrixd(&projection);

        /// Put the transform into the OpenGL ModelView matrix
        let mut modelView: [f64; 16] = std::mem::uninitialized();
        osvr_sys::OSVR_PoseState_to_OpenGL(&mut modelView[0] as *mut f64, renderInfo.pose);
        gl1x::MatrixMode(gl1x::MODELVIEW);
        gl1x::LoadIdentity();
        gl1x::MultMatrixd(&modelView);

        // Clear the screen to black and clear depth
        gl::ClearColor(0.0, 0.0, 0.0, 1.0);
        gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);

        // =================================================================
        // This is where we draw our world and hands and any other objects.
        // We're in World Space.  To find out about where to render objects
        // in OSVR spaces (like left/right hand space) we need to query the
        // interface and handle the coordinate tranforms ourselves.

        // Draw a cube with a 5-meter radius as the room we are floating in.
        draw_cube(5.0);
    }
}

static MATSPEC: [f32; 4] = [ 0.5, 0.5, 0.5, 0.0 ];
static RED_COL: [f32; 3] = [ 1.0, 0.0, 0.0 ];
static GRN_COL: [f32; 3] = [ 0.0, 1.0, 0.0 ];
static BLU_COL: [f32; 3] = [ 0.0, 0.0, 1.0 ];
static YEL_COL: [f32; 3] = [ 1.0, 1.0, 0.0 ];
static LIGHTBLU_COL: [f32; 3] = [ 0.0, 1.0, 1.0 ];
static PUR_COL: [f32; 3] = [ 1.0, 0.0, 1.0 ];

fn draw_cube(radius: f64)
{
    gl1x::PushMatrix();
    gl1x::Scaled(radius, radius, radius);
    //gl::Materialfv(GL_FRONT, GL_SPECULAR, MATSPEC);
    //gl::Materialf(GL_FRONT, GL_SHININESS, 64.0);
    gl1x::Begin(gl1x::POLYGON);
    gl1x::Color3fv(&LIGHTBLU_COL);
    //gl::Materialfv(GL_FRONT_AND_BACK, GL_AMBIENT, LIGHTBLU_COL);
    //gl::Materialfv(GL_FRONT_AND_BACK, GL_DIFFUSE, LIGHTBLU_COL);
    gl1x::Normal3f(0.0, 0.0, -1.0);
    gl1x::Vertex3f(1.0, 1.0, -1.0);
    gl1x::Vertex3f(1.0, -1.0, -1.0);
    gl1x::Vertex3f(-1.0, -1.0, -1.0);
    gl1x::Vertex3f(-1.0, 1.0, -1.0);
    gl1x::End();
    gl1x::Begin(gl1x::POLYGON);
    gl1x::Color3fv(&BLU_COL);
    //gl::Materialfv(GL_FRONT_AND_BACK, GL_AMBIENT, BLU_COL);
    //gl::Materialfv(GL_FRONT_AND_BACK, GL_DIFFUSE, BLU_COL);
    gl1x::Normal3f(0.0, 0.0, 1.0);
    gl1x::Vertex3f(-1.0, 1.0, 1.0);
    gl1x::Vertex3f(-1.0, -1.0, 1.0);
    gl1x::Vertex3f(1.0, -1.0, 1.0);
    gl1x::Vertex3f(1.0, 1.0, 1.0);
    gl1x::End();
    gl1x::Begin(gl1x::POLYGON);
    gl1x::Color3fv(&YEL_COL);
    //gl::Materialfv(GL_FRONT_AND_BACK, GL_AMBIENT, YEL_COL);
    //gl::Materialfv(GL_FRONT_AND_BACK, GL_DIFFUSE, YEL_COL);
    gl1x::Normal3f(0.0, -1.0, 0.0);
    gl1x::Vertex3f(1.0, -1.0, 1.0);
    gl1x::Vertex3f(-1.0, -1.0, 1.0);
    gl1x::Vertex3f(-1.0, -1.0, -1.0);
    gl1x::Vertex3f(1.0, -1.0, -1.0);
    gl1x::End();
    gl1x::Begin(gl1x::POLYGON);
    gl1x::Color3fv(&GRN_COL);
    //gl::Materialfv(GL_FRONT_AND_BACK, GL_AMBIENT, GRN_COL);
    //gl::Materialfv(GL_FRONT_AND_BACK, GL_DIFFUSE, GRN_COL);
    gl1x::Normal3f(0.0, 1.0, 0.0);
    gl1x::Vertex3f(1.0, 1.0, 1.0);
    gl1x::Vertex3f(1.0, 1.0, -1.0);
    gl1x::Vertex3f(-1.0, 1.0, -1.0);
    gl1x::Vertex3f(-1.0, 1.0, 1.0);
    gl1x::End();
    gl1x::Begin(gl1x::POLYGON);
    gl1x::Color3fv(&PUR_COL);
    //gl::Materialfv(GL_FRONT_AND_BACK, GL_AMBIENT, PUR_COL);
    //gl::Materialfv(GL_FRONT_AND_BACK, GL_DIFFUSE, PUR_COL);
    gl1x::Normal3f(-1.0, 0.0, 0.0);
    gl1x::Vertex3f(-1.0, 1.0, 1.0);
    gl1x::Vertex3f(-1.0, 1.0, -1.0);
    gl1x::Vertex3f(-1.0, -1.0, -1.0);
    gl1x::Vertex3f(-1.0, -1.0, 1.0);
    gl1x::End();
    gl1x::Begin(gl1x::POLYGON);
    gl1x::Color3fv(&RED_COL);
    //gl::Materialfv(GL_FRONT_AND_BACK, GL_AMBIENT, RED_COL);
    //gl::Materialfv(GL_FRONT_AND_BACK, GL_DIFFUSE, RED_COL);
    gl1x::Normal3f(1.0, 0.0, 0.0);
    gl1x::Vertex3f(1.0, -1.0, 1.0);
    gl1x::Vertex3f(1.0, -1.0, -1.0);
    gl1x::Vertex3f(1.0, 1.0, -1.0);
    gl1x::Vertex3f(1.0, 1.0, 1.0);
    gl1x::End();
    gl1x::PopMatrix();
}
