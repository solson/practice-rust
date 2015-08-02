extern crate gl;
extern crate glfw;
extern crate imagefmt;
extern crate time;

mod math;

use gl::types::*;
use glfw::{Context, OpenGlProfileHint, WindowHint, WindowMode};
use std::mem;
use std::ptr;

macro_rules! gl_str {
    ($string_literal:expr) => (
        concat!($string_literal, '\0').as_bytes().as_ptr() as *const GLchar
    )
}

const VERTEX_SHADER_SOURCE: &'static str = "
    #version 150

    in vec2 position;
    in vec3 color;
    in vec2 texcoord;

    out vec3 Color;
    out vec2 Texcoord;

    uniform mat4 model;
    uniform mat4 view;
    uniform mat4 proj;

    void main() {
        Color = color;
        Texcoord = texcoord;
        gl_Position = proj * view * model * vec4(position, 0.0, 1.0);
    }
";

const FRAGMENT_SHADER_SOURCE: &'static str = "
    #version 150

    in vec3 Color;
    in vec2 Texcoord;

    out vec4 out_color;

    uniform sampler2D tex_kitten;
    uniform sampler2D tex_puppy;
    uniform float time;

    void main() {
        float mix_factor = (sin(time * 3.0) + 1.0) / 2.0;
        vec4 col_kitten = texture(tex_kitten, Texcoord);
        vec4 col_puppy = texture(tex_puppy, Texcoord);
        vec4 mixed_texture = mix(col_kitten, col_puppy, mix_factor);
        out_color = mix(vec4(Color, 1.0), mixed_texture, 0.25);
    }
";

#[derive(Copy, Clone, Debug, PartialEq)]
#[repr(C, packed)]
struct Vertex {
    // Position.
    x: GLfloat, y: GLfloat,

    // Color.
    r: GLfloat, g: GLfloat, b: GLfloat,

    // Texture.
    s: GLfloat, t: GLfloat,
}

static VERTICES: [Vertex; 4] = [
    Vertex { x: -0.5, y:  0.5, r: 1.0, g: 0.0, b: 0.0, s: 0.0, t: 0.0 }, // Top-left
    Vertex { x:  0.5, y:  0.5, r: 0.0, g: 1.0, b: 0.0, s: 1.0, t: 0.0 }, // Top-right
    Vertex { x:  0.5, y: -0.5, r: 0.0, g: 0.0, b: 1.0, s: 1.0, t: 1.0 }, // Bottom-right
    Vertex { x: -0.5, y: -0.5, r: 1.0, g: 1.0, b: 1.0, s: 0.0, t: 1.0 }, // Bottom-left
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
        gl::GetShaderInfoLog(shader, log_len, ptr::null_mut(), buf.as_mut_ptr() as *mut GLchar);

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
    let mut textures = [0; 2];

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
                                mem::size_of::<Vertex>() as GLint, ptr::null());

        let position_attrib = gl::GetAttribLocation(shader_program, gl_str!("color"));
        gl::EnableVertexAttribArray(position_attrib as GLuint);
        gl::VertexAttribPointer(position_attrib as GLuint, 3, gl::FLOAT, gl::FALSE,
                                mem::size_of::<Vertex>() as GLint,
                                (2 * mem::size_of::<GLfloat>()) as *const GLvoid);

        let position_attrib = gl::GetAttribLocation(shader_program, gl_str!("texcoord"));
        gl::EnableVertexAttribArray(position_attrib as GLuint);
        gl::VertexAttribPointer(position_attrib as GLuint, 2, gl::FLOAT, gl::FALSE,
                                mem::size_of::<Vertex>() as GLint,
                                (5 * mem::size_of::<GLfloat>()) as *const GLvoid);

        // Create and load textures.
        gl::GenTextures(2, textures.as_mut_ptr());

        gl::ActiveTexture(gl::TEXTURE0);
        gl::BindTexture(gl::TEXTURE_2D, textures[0]);
        let image = imagefmt::read("sample.png", imagefmt::ColFmt::RGB).unwrap();
        gl::TexImage2D(gl::TEXTURE_2D, 0, gl::RGB as GLint, image.w as GLint, image.h as GLint,
                       0, gl::RGB, gl::UNSIGNED_BYTE, image.buf.as_ptr() as *const GLvoid);
        gl::Uniform1i(gl::GetUniformLocation(shader_program, gl_str!("tex_kitten")), 0);

        gl::GenerateMipmap(gl::TEXTURE_2D);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::REPEAT as GLint);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::REPEAT as GLint);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER,
                          gl::LINEAR_MIPMAP_LINEAR as GLint);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER,
                          gl::LINEAR_MIPMAP_LINEAR as GLint);

        gl::ActiveTexture(gl::TEXTURE1);
        gl::BindTexture(gl::TEXTURE_2D, textures[1]);
        let image = imagefmt::read("sample2.png", imagefmt::ColFmt::RGB).unwrap();
        gl::TexImage2D(gl::TEXTURE_2D, 0, gl::RGB as GLint, image.w as GLint, image.h as GLint,
                       0, gl::RGB, gl::UNSIGNED_BYTE, image.buf.as_ptr() as *const GLvoid);
        gl::Uniform1i(gl::GetUniformLocation(shader_program, gl_str!("tex_puppy")), 1);

        gl::GenerateMipmap(gl::TEXTURE_2D);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::REPEAT as GLint);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::REPEAT as GLint);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER,
                          gl::LINEAR_MIPMAP_LINEAR as GLint);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER,
                          gl::LINEAR_MIPMAP_LINEAR as GLint);
    }

    let view = math::Mat4::look_at(
        math::Vec3([1.2, 1.2, 1.2]),
        math::Vec3([0.0, 0.0, 0.0]),
        math::Vec3([0.0, 0.0, 1.0]));
    let proj = math::Mat4::perspective(math::TAU / 8.0, 800.0 / 600.0, 1.0, 10.0);

    let model_uniform = unsafe { gl::GetUniformLocation(shader_program, gl_str!("model")) };
    let view_uniform = unsafe { gl::GetUniformLocation(shader_program, gl_str!("view")) };
    let proj_uniform = unsafe { gl::GetUniformLocation(shader_program, gl_str!("proj")) };

    unsafe {
        gl::UniformMatrix4fv(view_uniform, 1, gl::FALSE, &view[0][0]);
        gl::UniformMatrix4fv(proj_uniform, 1, gl::FALSE, &proj[0][0]);
    }

    let time_uniform = unsafe { gl::GetUniformLocation(shader_program, gl_str!("time")) };
    let time_start = time::precise_time_ns();

    while !window.should_close() {
        glfw.poll_events();
        for (_, event) in glfw::flush_messages(&events) {
            handle_window_event(&mut window, event);
        }

        unsafe {
            // Update the `time` uniform.
            let time_now = time::precise_time_ns();
            let elapsed_seconds = (time_now - time_start) as f32 / 1e9;
            gl::Uniform1f(time_uniform, elapsed_seconds);

            // Vary the model matrix over time.
            let scale = (elapsed_seconds * 5.0).sin() * 0.25 + 0.75;
            let model =
                math::Mat4::rotate_z(math::TAU / 2.0 * elapsed_seconds) *
                math::Mat4::scale(scale, scale, scale);
            gl::UniformMatrix4fv(model_uniform, 1, gl::FALSE, &model[0][0]);

            // Clear the screen to black.
            gl::ClearColor(0.0, 0.0, 0.0, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);

            // Draw the triangles described by the elements array.
            gl::DrawElements(gl::TRIANGLES, ELEMENTS.len() as GLint, gl::UNSIGNED_INT,
                             ptr::null());
        }

        window.swap_buffers();
    }

    unsafe {
        gl::DeleteTextures(2, textures.as_ptr());
        gl::DeleteProgram(shader_program);
        gl::DeleteShader(fragment_shader);
        gl::DeleteShader(vertex_shader);
        gl::DeleteBuffers(1, &ebo);
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
