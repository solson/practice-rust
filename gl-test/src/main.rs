extern crate gl;
extern crate glfw;

use gl::types::*;
use glfw::{Context, OpenGlProfileHint, WindowHint, WindowMode};

fn main() {
    let mut glfw = glfw::init(glfw::FAIL_ON_ERRORS).unwrap();

    glfw.window_hint(WindowHint::ContextVersion(3, 2));
    glfw.window_hint(WindowHint::OpenGlProfile(OpenGlProfileHint::Core));
    glfw.window_hint(WindowHint::OpenGlForwardCompat(true));
    glfw.window_hint(WindowHint::Resizable(false));

    let (mut window, events) = glfw.create_window(800, 600, "OpenGL", WindowMode::Windowed)
        .expect("Failed to create GLFW window.");

    // Listen for keyboard events on this window.
    window.set_key_polling(true);

    // Make this window's OpenGL context the current context. This must be done before calling
    // `gl::load_with`.
    window.make_current();

    // Load OpenGL function pointers.
    gl::load_with(|symbol| window.get_proc_address(symbol));

    static VERTICES: [GLfloat; 6] = [
         0.0,  0.5,
         0.5, -0.5,
        -0.5, -0.5,
    ];

    let mut vbo = 0;

    unsafe {
        gl::GenBuffers(1, &mut vbo);
        gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
        gl::BufferData(gl::ARRAY_BUFFER,
                       (VERTICES.len() * std::mem::size_of::<GLfloat>()) as GLsizeiptr,
                       &VERTICES as *const _ as *const GLvoid,
                       gl::STATIC_DRAW);
    }

    while !window.should_close() {
        glfw.poll_events();

        for (_, event) in glfw::flush_messages(&events) {
            handle_window_event(&mut window, event);
        }

        window.swap_buffers();
    }
}

fn handle_window_event(window: &mut glfw::Window, event: glfw::WindowEvent) {
    use glfw::{Action, Key, WindowEvent};

    match event {
        WindowEvent::Key(Key::Escape, _, Action::Press, _) => {
            window.set_should_close(true);
        },
        _ => {},
    }
}
