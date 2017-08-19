extern crate gl;
extern crate osvr;
pub mod common;
use common::gl1x;
use common::cube;

fn main() {
    let mut quit = false;

    let context = osvr::Context::new("Rust OSVR example");

    let left_button_1 = osvr::Interface::new(&context, "/controller/left/1");
    left_button_1.register_button_callback(my_button_callback, &mut quit);
    let right_button_1 = osvr::Interface::new(&context, "/controller/right/1");
    right_button_1.register_button_callback(my_button_callback, &mut quit);
    
    let mut render = osvr::RenderManager::new(&context).unwrap();

    gl1x::init();
    setup_rendering();

    context.update();
    render.register_buffers();

    while !quit {
        context.update();

        render.render_eyes(|render_info, frame_buffer, color_buffer, depth_buffer| {
            osvr::glutil::bind_buffers(frame_buffer, color_buffer, depth_buffer);
            osvr::glutil::set_viewport(render_info);

            let projection = osvr::glutil::get_projection(render_info);
            let modelview = osvr::glutil::get_modelview(render_info);

            unsafe {
                gl1x::MatrixMode(gl1x::PROJECTION);
                gl1x::LoadIdentity();
                gl1x::MultMatrixd(&projection);

                gl1x::MatrixMode(gl1x::MODELVIEW);
                gl1x::LoadIdentity();
                gl1x::MultMatrixd(&modelview);

                // Clear the screen to black and clear depth
                gl::ClearColor(0.0, 0.0, 0.0, 1.0);
                gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
            }

            // =================================================================
            // This is where we draw our world and hands and any other objects.
            // We're in World Space.  To find out about where to render objects
            // in OSVR spaces (like left/right hand space) we need to query the
            // interface and handle the coordinate tranforms ourselves.

            // Draw a cube with a 5-meter radius as the room we are floating in.
            cube::draw_cube(5.0);
        });
    }
}

extern "C" fn my_button_callback(userdata: &mut bool, timestamp: &osvr::TimeValue, report: &osvr::ButtonReport)
{
    println!("Button state: {}", report.state());
    println!("Time {} s, {} us", timestamp.seconds(), timestamp.microseconds());
    // quit when buttons are pressed
    *userdata = report.state() != 0;
}

fn setup_rendering()
{
    unsafe {
        gl::Enable(gl::DEPTH_TEST)
    };
}
