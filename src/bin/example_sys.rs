extern crate osvr_sys;
extern crate gl;
extern crate sdl2;
use std::ffi::CString;
use std::ptr;
use std::mem;
use std::vec::Vec;
pub mod common;
use common::gl1x;
use common::cube;

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
        cube::draw_cube(5.0);
    }
}
