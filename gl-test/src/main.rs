extern crate gl;
extern crate glfw;
extern crate imagefmt;
extern crate time;

use gl::types::*;
use glfw::{Context, OpenGlProfileHint, WindowHint, WindowMode};
use std::mem;
use std::ptr;

#[derive(Copy, Clone, Debug, PartialEq)]
struct Vec4([GLfloat; 4]);

impl Vec4 {
    fn zero() -> Self {
        Vec4([0.0, 0.0, 0.0, 0.0])
    }

    fn x(&self) -> GLfloat { self[0] }
    fn y(&self) -> GLfloat { self[1] }
    fn z(&self) -> GLfloat { self[2] }
    fn w(&self) -> GLfloat { self[3] }
}

impl std::ops::Index<usize> for Vec4 {
    type Output = GLfloat;

    fn index(&self, i: usize) -> &GLfloat {
        &self.0[i]
    }
}

impl std::ops::IndexMut<usize> for Vec4 {
    fn index_mut(&mut self, i: usize) -> &mut GLfloat {
        &mut self.0[i]
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
struct Mat4([[GLfloat; 4]; 4]);

impl Mat4 {
    fn zero() -> Self {
        Mat4([
            [0.0, 0.0, 0.0, 0.0],
            [0.0, 0.0, 0.0, 0.0],
            [0.0, 0.0, 0.0, 0.0],
            [0.0, 0.0, 0.0, 0.0],
        ])
    }

    fn identity() -> Self {
        Mat4([
            [1.0, 0.0, 0.0, 0.0],
            [0.0, 1.0, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ])
    }

    fn scale(x: GLfloat, y: GLfloat, z: GLfloat) -> Self {
        Mat4([
            [x,   0.0, 0.0, 0.0],
            [0.0, y,   0.0, 0.0],
            [0.0, 0.0, z,   0.0],
            [0.0, 0.0, 0.0, 1.0],
        ])
    }

    fn translate(x: GLfloat, y: GLfloat, z: GLfloat) -> Self {
        Mat4([
            [1.0, 0.0, 0.0, x  ],
            [0.0, 1.0, 0.0, y  ],
            [0.0, 0.0, 1.0, z  ],
            [0.0, 0.0, 0.0, 1.0],
        ])
    }

    /// A matrix representing a rotation around the X-axis by the given angle (in radians).
    fn rotate_x(angle: GLfloat) -> Self {
        let cos = angle.cos();
        let sin = angle.sin();

        Mat4([
            [1.0, 0.0,  0.0, 0.0],
            [0.0, cos, -sin, 0.0],
            [0.0, sin,  cos, 0.0],
            [0.0, 0.0,  0.0, 1.0],
        ])
    }

    /// A matrix representing a rotation around the Y-axis by the given angle (in radians).
    fn rotate_y(angle: GLfloat) -> Self {
        let cos = angle.cos();
        let sin = angle.sin();

        Mat4([
            [ cos, 0.0, sin, 0.0],
            [ 0.0, 1.0, 0.0, 0.0],
            [-sin, 0.0, cos, 0.0],
            [ 0.0, 0.0, 0.0, 1.0],
        ])
    }

    /// A matrix representing a rotation around the Z-axis by the given angle (in radians).
    fn rotate_z(angle: GLfloat) -> Self {
        let cos = angle.cos();
        let sin = angle.sin();

        Mat4([
            [cos, -sin, 0.0, 0.0],
            [sin,  cos, 0.0, 0.0],
            [0.0,  0.0, 1.0, 0.0],
            [0.0,  0.0, 0.0, 1.0],
        ])
    }
}

impl std::ops::Index<usize> for Mat4 {
    type Output = [GLfloat; 4];

    fn index(&self, i: usize) -> &[GLfloat; 4] {
        &self.0[i]
    }
}

impl std::ops::IndexMut<usize> for Mat4 {
    fn index_mut(&mut self, i: usize) -> &mut [GLfloat; 4] {
        &mut self.0[i]
    }
}

impl std::ops::Mul<Mat4> for Mat4 {
    type Output = Mat4;

    fn mul(self, other: Mat4) -> Mat4 {
        let mut result = Mat4::zero();

        for i in 0..4 {
            for j in 0..4 {
                for k in 0..4 {
                    result[i][j] += self[i][k] * other[k][j];
                }
            }
        }

        result
    }
}

impl std::ops::Mul<Vec4> for Mat4 {
    type Output = Vec4;

    fn mul(self, vec: Vec4) -> Vec4 {
        let mut result = Vec4::zero();

        for i in 0..4 {
            for j in 0..4 {
                result[i] += self[i][j] * vec[j];
            }
        }

        result
    }
}

#[test]
fn test_math() {
    let scale = Mat4::scale(2.0, 2.0, 2.0);
    let trans = Mat4::translate(1.0, 2.0, 3.0);
    let combined = trans * scale;

    let original = Vec4([3.0, 3.0, 3.0, 1.0]);
    let expected = Vec4([7.0, 8.0, 9.0, 1.0]);

    assert_eq!(expected, combined * original);
}

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

    void main() {
        Color = color;
        Texcoord = texcoord;
        gl_Position = vec4(position, 0.0, 1.0);
    }
";

const FRAGMENT_SHADER_SOURCE: &'static str = "
    #version 150

    in vec3 Color;
    in vec2 Texcoord;

    out vec4 out_color;

    uniform sampler2D texKitten;
    uniform sampler2D texPuppy;
    uniform float time;

    void main() {
        float mix_factor = (sin(time * 3.0) + 1.0) / 2.0;
        vec4 colKitten = texture(texKitten, Texcoord);
        vec4 colPuppy = texture(texPuppy, Texcoord);
        vec4 mixedTexture = mix(colKitten, colPuppy, mix_factor);
        out_color = mix(vec4(Color, 1.0), mixedTexture, 0.25);
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
        gl::Uniform1i(gl::GetUniformLocation(shader_program, gl_str!("texKitten")), 0);

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
        gl::Uniform1i(gl::GetUniformLocation(shader_program, gl_str!("texPuppy")), 1);

        gl::GenerateMipmap(gl::TEXTURE_2D);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::REPEAT as GLint);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::REPEAT as GLint);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER,
                          gl::LINEAR_MIPMAP_LINEAR as GLint);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER,
                          gl::LINEAR_MIPMAP_LINEAR as GLint);
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
