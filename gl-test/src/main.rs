extern crate gl;
extern crate glfw;
extern crate time;

use gl::types::*;
use glfw::{Context, OpenGlProfileHint, WindowHint, WindowMode};
use std::mem;

macro_rules! gl_str {
    ($string_literal:expr) => (concat!($string_literal, '\0').as_bytes().as_ptr() as *const GLchar)
}

const VERTEX_SHADER_SOURCE: &'static str = "
    #version 150

    in vec2 position;
    in vec3 color;

    out vec3 Color;

    void main() {
        Color = color;
        gl_Position = vec4(position, 0.0, 1.0);
    }
";

const FRAGMENT_SHADER_SOURCE: &'static str = "
    #version 150

    in vec3 Color;

    out vec4 out_color;

    void main() {
        out_color = vec4(Color, 1.0);
    }
";

#[derive(Copy, Clone, Debug, PartialEq)]
#[repr(C, packed)]
struct Vertex {
    // Position.
    x: GLfloat, y: GLfloat,

    // Color.
    r: GLfloat, g: GLfloat, b: GLfloat,
}

static VERTICES: [Vertex; 4] = [
    Vertex { x: -0.5, y:  0.5, r: 1.0, g: 0.0, b: 0.0 }, // Top-left
    Vertex { x:  0.5, y:  0.5, r: 0.0, g: 1.0, b: 0.0 }, // Top-right
    Vertex { x:  0.5, y: -0.5, r: 0.0, g: 0.0, b: 1.0 }, // Bottom-right
    Vertex { x: -0.5, y: -0.5, r: 1.0, g: 1.0, b: 1.0 }, // Bottom-left
];

static ELEMENTS: [GLuint; 6] = [
    0, 1, 2, // Top-right triangle
    2, 3, 0, // Bottom-left triangle
];

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
    let mut vao = 0;
    let mut vbo = 0;
    let mut ebo = 0;

    unsafe {
        // Create a vertex array object.
        gl::GenVertexArrays(1, &mut vao);
        gl::BindVertexArray(vao);

        // Create a vertex buffer object and copy the vertex data to it.
        gl::GenBuffers(1, &mut vbo);
        gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
        gl::BufferData(gl::ARRAY_BUFFER,
                       mem::size_of_val(&VERTICES) as GLsizeiptr,
                       VERTICES.as_ptr() as *const GLvoid,
                       gl::STATIC_DRAW);

        // Create an element buffer object and copy the element data to it.
        gl::GenBuffers(1, &mut ebo);
        gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, ebo);
        gl::BufferData(gl::ELEMENT_ARRAY_BUFFER,
                       mem::size_of_val(&ELEMENTS) as GLsizeiptr,
                       ELEMENTS.as_ptr() as *const GLvoid,
                       gl::STATIC_DRAW);

        // Compile the vertex and fragment shaders.
        vertex_shader = compile_shader(gl::VERTEX_SHADER, VERTEX_SHADER_SOURCE).unwrap();
        fragment_shader = compile_shader(gl::FRAGMENT_SHADER, FRAGMENT_SHADER_SOURCE).unwrap();

        // Link the vertex and fragment shaders into a shader program.
        shader_program = gl::CreateProgram();
        gl::AttachShader(shader_program, vertex_shader);
        gl::AttachShader(shader_program, fragment_shader);
        gl::BindFragDataLocation(shader_program, 0, gl_str!("out_color"));
        gl::LinkProgram(shader_program);
        gl::UseProgram(shader_program);

        // Specify the layout of the vertex data.
        let position_attrib = gl::GetAttribLocation(shader_program, gl_str!("position"));
        gl::EnableVertexAttribArray(position_attrib as GLuint);
        gl::VertexAttribPointer(position_attrib as GLuint, 2, gl::FLOAT, gl::FALSE,
                                mem::size_of::<Vertex>() as GLint, std::ptr::null());

        let position_attrib = gl::GetAttribLocation(shader_program, gl_str!("color"));
        gl::EnableVertexAttribArray(position_attrib as GLuint);
        gl::VertexAttribPointer(position_attrib as GLuint, 3, gl::FLOAT, gl::FALSE,
                                mem::size_of::<Vertex>() as GLint,
                                std::ptr::null().offset(2 * mem::size_of::<GLfloat>() as isize));
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

            // Draw the triangles described by the elements array.
            gl::DrawElements(gl::TRIANGLES, ELEMENTS.len() as GLint, gl::UNSIGNED_INT,
                             std::ptr::null());
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
