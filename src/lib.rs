extern crate osvr_sys;
extern crate gl;
use std::ffi::CString;
use std::vec::Vec;


#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
    }
}

pub struct TimeValue(osvr_sys::OSVR_TimeValue);
impl TimeValue {
    pub fn seconds(&self) -> i64 {
        let &TimeValue(t) = self;
        t.seconds
    }
    pub fn microseconds(&self) -> i32 {
        let &TimeValue(t) = self;
        t.microseconds
    }
}

pub struct ButtonReport(osvr_sys::OSVR_ButtonReport);
impl ButtonReport {
    pub fn sensor(&self) -> i32 {
        let &ButtonReport(b) = self;
        b.sensor
    }
    pub fn state(&self) -> u8 {
        let &ButtonReport(b) = self;
        b.state
    }
}

pub struct Context {
    ctxt: osvr_sys::OSVR_ClientContext
}

impl Context {
    pub fn new(app_identifier: &str) -> Context {
        Context { ctxt: unsafe { osvr_sys::osvrClientInit(CString::new(app_identifier).unwrap().as_ptr(), 0) } }
    }

    pub fn update(&self) {
        unsafe { osvr_sys::osvrClientUpdate(self.ctxt) };
    }
}

impl Drop for Context {
    fn drop(&mut self) {
        unsafe { osvr_sys::osvrClientShutdown(self.ctxt) };
    }
}

pub struct RenderManager {
    render: osvr_sys::OSVR_RenderManager,
    render_gl: osvr_sys::OSVR_RenderManagerOpenGL,
    library: osvr_sys::OSVR_GraphicsLibraryOpenGL,
    render_params: osvr_sys::OSVR_RenderParams,
    frame_buffer: osvr_sys::GLuint,
    color_buffers: Vec<osvr_sys::OSVR_RenderBufferOpenGL>,
    depth_buffers: Vec<osvr_sys::GLuint>
}

impl RenderManager {
    pub fn new(context: &Context) -> Option<RenderManager> {
        unsafe {
            let library: osvr_sys::OSVR_GraphicsLibraryOpenGL = std::mem::zeroed();
            let mut render: osvr_sys::OSVR_RenderManager = std::mem::zeroed();
            let mut render_gl: osvr_sys::OSVR_RenderManagerOpenGL = std::mem::zeroed();
            
            if 0 != osvr_sys::osvrCreateRenderManagerOpenGL(context.ctxt, CString::new("OpenGL").unwrap().as_ptr(), library, &mut render, &mut render_gl) {
                eprintln!("Could not create RenderManager");
                None
            } else {
                let mut openResults: osvr_sys::OSVR_OpenResultsOpenGL = std::mem::zeroed();
                if 0 != osvr_sys::osvrRenderManagerOpenDisplayOpenGL(render_gl, &mut openResults) {
                    eprintln!("Could not open GL display");
                    None
                } else {
                    gl::load_with(|s| {
                        osvr_sys::SDL_GL_GetProcAddress(CString::new(s).unwrap().as_ptr())
                    });

                    Some(RenderManager {
                        library: library,
                        render: render,
                        render_gl: render_gl,
                        render_params: std::mem::zeroed(),
                        frame_buffer: 0,
                        color_buffers: Vec::new(),
                        depth_buffers: Vec::new()
                    })
                }
            }
        }
    }

    pub fn register_buffers(&mut self) {
        unsafe {
            osvr_sys::osvrRenderManagerGetDefaultRenderParams(&mut self.render_params);

            let mut renderInfoCollection: osvr_sys::OSVR_RenderInfoCollection = std::mem::zeroed();
            if 0 != osvr_sys::osvrRenderManagerGetRenderInfoCollection(self.render, self.render_params, &mut renderInfoCollection) {
                panic!("Could not get render info");
            }

            let mut numRenderInfo: osvr_sys::OSVR_RenderInfoCount = 0 as usize;
            osvr_sys::osvrRenderManagerGetNumRenderInfoInCollection(renderInfoCollection, &mut numRenderInfo);

            gl::GenFramebuffers(1, &mut self.frame_buffer);
            gl::BindFramebuffer(gl::FRAMEBUFFER, self.frame_buffer);

            let mut registerBufferState: osvr_sys::OSVR_RenderManagerRegisterBufferState = std::mem::uninitialized();
            if 0 != osvr_sys::osvrRenderManagerStartRegisterRenderBuffers(&mut registerBufferState) {
                panic!("Could not start registering render buffers");
            }

            for i in 0..numRenderInfo {
                let mut renderInfo: osvr_sys::OSVR_RenderInfoOpenGL = std::mem::zeroed();

                if 0 != osvr_sys::osvrRenderManagerGetRenderInfoFromCollectionOpenGL(renderInfoCollection, i, &mut renderInfo) {
                    panic!("Could not get render info {}", i);
                }

                let width: i32 = renderInfo.viewport.width as i32;
                let height: i32 = renderInfo.viewport.height as i32;

                let mut colorBufferName: osvr_sys::GLuint = 0;
                if 0 != osvr_sys::osvrRenderManagerCreateColorBufferOpenGL(width, height, gl::RGBA, &mut colorBufferName) {
                    panic!("Could not create color buffer.");
                }

                let mut rb: osvr_sys::OSVR_RenderBufferOpenGL = std::mem::zeroed();
                rb.colorBufferName = colorBufferName;
                
                self.color_buffers.push(rb);

                let mut depthrenderbuffer: osvr_sys::GLuint = 0;
                if 0 != osvr_sys::osvrRenderManagerCreateDepthBufferOpenGL(width, height, &mut depthrenderbuffer) {
                    panic!("Could not create depth buffer.");
                }
                rb.depthStencilBufferName = depthrenderbuffer;
                self.depth_buffers.push(depthrenderbuffer);

                if 0 != osvr_sys::osvrRenderManagerRegisterRenderBufferOpenGL(registerBufferState, rb) {
                    panic!("Could not register render buffer {}.", i);
                }
            }

            if 0 != osvr_sys::osvrRenderManagerFinishRegisterRenderBuffers(self.render, registerBufferState, 0) {
                panic!("Could not start finish registering render buffers.");
            }
        }
    }

    pub fn render_eyes<F>(&mut self, render_eye: F) 
    where F: Fn(&osvr_sys::OSVR_RenderInfoOpenGL, osvr_sys::GLuint, osvr_sys::GLuint, osvr_sys::GLuint)
    {
        if self.color_buffers.len() == 0 {
            panic!("Color buffers not registered when render_eyes() called. Did you forget to call register_buffers()?");
        }
        unsafe {
            let mut renderInfoCollection: osvr_sys::OSVR_RenderInfoCollection = std::mem::zeroed();
            if 0 != osvr_sys::osvrRenderManagerGetRenderInfoCollection(self.render, self.render_params, &mut renderInfoCollection) {
                panic!("Could not get render info in the main loop.");
            }

            let mut numRenderInfo: osvr_sys::OSVR_RenderInfoCount = 0 as usize;
            osvr_sys::osvrRenderManagerGetNumRenderInfoInCollection(renderInfoCollection, &mut numRenderInfo);

            for i in 0..numRenderInfo {
                let mut renderInfo: osvr_sys::OSVR_RenderInfoOpenGL = std::mem::zeroed();
                osvr_sys::osvrRenderManagerGetRenderInfoFromCollectionOpenGL(renderInfoCollection, i, &mut renderInfo);

                // then draw your GL scene for this eye here!
                (render_eye)(&renderInfo, self.frame_buffer, self.color_buffers[i].colorBufferName, self.depth_buffers[i]);
            }

            let mut presentState: osvr_sys::OSVR_RenderManagerPresentState = std::mem::zeroed();
            if 0 != osvr_sys::osvrRenderManagerStartPresentRenderBuffers(&mut presentState) {
                panic!("Could not start presenting render buffers.");
                return;
            }

            let fullView = osvr_sys::OSVR_ViewportDescription { left: 0.0, lower: 0.0, width: 1.0, height: 1.0};
            for i in 0..numRenderInfo {
                let mut renderInfo: osvr_sys::OSVR_RenderInfoOpenGL = std::mem::zeroed();
                osvr_sys::osvrRenderManagerGetRenderInfoFromCollectionOpenGL(renderInfoCollection, i, &mut renderInfo);

                if 0 != osvr_sys::osvrRenderManagerPresentRenderBufferOpenGL(presentState, self.color_buffers[i], renderInfo, fullView) {
                    panic!("Could not present render buffer {}.", i);
                    return;
                }
            }

            osvr_sys::osvrRenderManagerReleaseRenderInfoCollection(renderInfoCollection);

            if 0 != osvr_sys::osvrRenderManagerFinishPresentRenderBuffers(self.render, presentState, self.render_params, 0) {
                panic!("Could not finish presenting render buffers.");
            }
        }
    }
}

impl Drop for RenderManager {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteFramebuffers(1, &self.frame_buffer);
            for i in 0..self.color_buffers.len() {
                gl::DeleteTextures(1, &self.color_buffers[i].colorBufferName);
                gl::DeleteRenderbuffers(1, &self.depth_buffers[i]);
            }

            osvr_sys::osvrDestroyRenderManager(self.render);
        }
    }
}

pub struct Interface<'a> {
    iface: osvr_sys::OSVR_ClientInterface,
    context: &'a Context
}

impl<'a> Interface<'a> {
    pub fn new(context: &'a Context, path: &str) -> Interface<'a> {
        unsafe {
            let mut interface = Interface {
                iface: std::mem::zeroed(),
                context: context
            };
            osvr_sys::osvrClientGetInterface(context.ctxt, CString::new(path).unwrap().as_ptr(), &mut interface.iface);
            interface
        }
    }
    /* XXX untested, because I have no such peripherals :) */
    pub fn register_button_callback<T>(&self,
                                    callback: extern "C" fn(&mut T, &TimeValue, &ButtonReport),
                                    userdata: &mut T)
    {
        unsafe {
            osvr_sys::osvrRegisterButtonCallback(self.iface, Some(std::mem::transmute(callback as *const ())), std::mem::transmute(userdata));
        }
    }
}

impl<'a> Drop for Interface<'a> {
    fn drop(&mut self) {
        unsafe {
            osvr_sys::osvrClientFreeInterface(self.context.ctxt, self.iface);
        }
    }
}

pub mod glutil {
    extern crate gl;
    extern crate osvr_sys;

    pub fn bind_buffers(frame_buffer: osvr_sys::GLuint, color_buffer: osvr_sys::GLuint, depth_buffer: osvr_sys::GLuint)
    {
        unsafe {
            gl::BindFramebuffer(gl::FRAMEBUFFER, frame_buffer);

            // Set color and depth buffers for the frame buffer
            gl::FramebufferTexture2D(gl::FRAMEBUFFER, gl::COLOR_ATTACHMENT0, gl::TEXTURE_2D, color_buffer, 0);
            gl::FramebufferRenderbuffer(gl::FRAMEBUFFER, gl::DEPTH_ATTACHMENT, gl::RENDERBUFFER, depth_buffer);

            // Set the list of draw buffers.
            let drawBuffer: osvr_sys::GLenum = gl::COLOR_ATTACHMENT0;
            gl::DrawBuffers(1, &drawBuffer); // "1" is the size of DrawBuffers

            // Always check that our framebuffer is ok
            if gl::CheckFramebufferStatus(gl::FRAMEBUFFER) != gl::FRAMEBUFFER_COMPLETE {
                panic!("RenderView: Incomplete Framebuffer");
                return;
            }
        }
    }
    
    pub fn set_viewport(render_info: &osvr_sys::OSVR_RenderInfoOpenGL) {
        unsafe {
            gl::Viewport(0, 0, render_info.viewport.width as i32, render_info.viewport.height as i32);
        }
    }

    fn convert_projection_matrix(matrix: osvr_sys::OSVR_ProjectionMatrix) -> osvr_sys::OSVR_ProjectionMatrix 
    {
        let mut ret: osvr_sys::OSVR_ProjectionMatrix = unsafe { ::std::mem::zeroed() };
        ret.bottom = matrix.bottom;
        ret.top = matrix.top;
        ret.left = matrix.left;
        ret.right = matrix.right;
        ret.nearClip = matrix.nearClip;
        ret.farClip = matrix.farClip;
        ret
    }

    pub fn get_projection(render_info: &osvr_sys::OSVR_RenderInfoOpenGL) -> [f64; 16]
    {
        unsafe {
            // Set the OpenGL projection matrix
            let mut projection: [f64; 16] = ::std::mem::uninitialized();
            let mut temp: osvr_sys::OSVR_ProjectionMatrix;
            temp.bottom = render_info.projection.bottom;
            osvr_sys::OSVR_Projection_to_OpenGL(&mut projection[0] as *mut f64, convert_projection_matrix(render_info.projection));
            projection
        }
    }

    pub fn get_modelview(render_info: &osvr_sys::OSVR_RenderInfoOpenGL) -> [f64; 16]
    {
        unsafe {
            /// Put the transform into the OpenGL ModelView matrix
            let mut model_view: [f64; 16] = ::std::mem::uninitialized();
            osvr_sys::OSVR_PoseState_to_OpenGL(&mut model_view[0] as *mut f64, render_info.pose);
            model_view
        }
    }
}
