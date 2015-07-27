extern crate gl;
extern crate glfw;

use gl::types::*;
use glfw::{Context, OpenGlProfileHint, WindowHint, WindowMode};

const VERTEX_SHADER_SOURCE: &'static str = "
    #version 150 core

    in vec2 position;

    void main() {
        gl_Position = vec4(position, 0.0, 1.0);
    }
";

const FRAGMENT_SHADER_SOURCE: &'static str = "
    #version 150 core

    out vec4 out_color;

    void main() {
        out_color = vec4(1.0, 1.0, 1.0, 1.0);
    }
";

unsafe fn compile_shader(shader_type: GLenum, source: &str) -> Result<GLuint, String> {
    let shader = gl::CreateShader(shader_type);
    let source_ptr = source.as_bytes().as_ptr() as *const GLchar;
    let source_len = source.len() as GLint;
    gl::ShaderSource(shader, 1, &source_ptr, &source_len);
    gl::CompileShader(shader);

    let mut status = gl::FALSE as GLint;
    gl::GetShaderiv(shader, gl::COMPILE_STATUS, &mut status);

    if status == gl::TRUE as GLint {
        Ok(shader)
    } else {
        let mut log_len = 0;
        gl::GetShaderiv(shader, gl::INFO_LOG_LENGTH, &mut log_len);
        if log_len == 0 { return Err(String::new()) }

        let mut buf = Vec::with_capacity(log_len as usize);
        buf.set_len(log_len as usize - 1); // Subtract 1 to ignore the trailing null.
        gl::GetShaderInfoLog(shader, log_len, std::ptr::null_mut(),
        buf.as_mut_ptr() as *mut GLchar);

        Err(String::from_utf8_lossy(&buf).into_owned())
    }
}

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

    let vertex_shader;
    let fragment_shader;
    let shader_program;
    let mut vao;
    let mut vbo;

    unsafe {
        // Create a vertex array object.
        vao = 0;
        gl::GenVertexArrays(1, &mut vao);
        gl::BindVertexArray(vao);

        // Create a vertex buffer object and copy the vertex data to it.
        vbo = 0;
        gl::GenBuffers(1, &mut vbo);

        static VERTICES: [GLfloat; 6] = [
             0.0,  0.5,
             0.5, -0.5,
            -0.5, -0.5,
        ];

        gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
        gl::BufferData(gl::ARRAY_BUFFER,
                       (VERTICES.len() * std::mem::size_of::<GLfloat>()) as GLsizeiptr,
                       VERTICES.as_ptr() as *const GLvoid,
                       gl::STATIC_DRAW);

        // Compile the vertex and fragment shaders.
        vertex_shader = compile_shader(gl::VERTEX_SHADER, VERTEX_SHADER_SOURCE).unwrap();
        fragment_shader = compile_shader(gl::FRAGMENT_SHADER, FRAGMENT_SHADER_SOURCE).unwrap();

        // Link the vertex and fragment shaders into a shader program.
        shader_program = gl::CreateProgram();
        gl::AttachShader(shader_program, vertex_shader);
        gl::AttachShader(shader_program, fragment_shader);
        gl::BindFragDataLocation(shader_program, 0, b"out_color\0".as_ptr() as *const GLchar);
        gl::LinkProgram(shader_program);
        gl::UseProgram(shader_program);

        let position_attrib = gl::GetAttribLocation(shader_program,
                                                    b"position\0".as_ptr() as *const GLchar);
        gl::VertexAttribPointer(position_attrib as GLuint, 2, gl::FLOAT, gl::FALSE, 0,
                                std::ptr::null());
        gl::EnableVertexAttribArray(position_attrib as GLuint);
    }

    while !window.should_close() {
        glfw.poll_events();
        for (_, event) in glfw::flush_messages(&events) {
            handle_window_event(&mut window, event);
        }

        unsafe {
            // Clear the screen to black.
            gl::ClearColor(0.0, 0.0, 0.0, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);

            // Draw a triangle from the 3 vertices.
            gl::DrawArrays(gl::TRIANGLES, 0, 3);
        }

        window.swap_buffers();
    }

    unsafe {
        gl::DeleteProgram(shader_program);
        gl::DeleteShader(fragment_shader);
        gl::DeleteShader(vertex_shader);
        gl::DeleteBuffers(1, &vbo);
        gl::DeleteVertexArrays(1, &vao);
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
