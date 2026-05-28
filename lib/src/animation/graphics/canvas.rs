use glow::HasContext;
use crate::common::data::imgcut::SpriteSheet;
use super::transform::WorldTransform;
use std::sync::Arc;

/// Represents an error encountered during the initialization or execution of the OpenGL rendering pipeline.
#[derive(Debug)]
pub enum CanvasError {
    /// Failed to create the main shader program object.
    ProgramCreation,
    /// Failed to allocate or initialize the vertex shader.
    VertexShaderCreation,
    /// Failed to allocate or initialize the fragment shader.
    FragmentShaderCreation,
    /// Failed to compile the shader source code. Contains the OpenGL compilation log.
    ShaderCompile(String),
    /// Failed to link the compiled shaders into the final program. Contains the OpenGL linker log.
    ProgramLink(String),
    /// Failed to generate a Vertex Array Object (VAO).
    VaoCreation,
    /// Failed to generate the Vertex Buffer Object (VBO) for positional data.
    VboCreation,
    /// Failed to generate the Texture Buffer Object (TBO) for UV mapping data.
    TboCreation,
    /// Failed to generate a texture object.
    TextureAllocation,
}

const VERTEX_SHADER_SOURCE: &str = r#"
#ifdef GL_ES
precision lowp float;
#endif
in vec2 a_position;
in vec2 a_texcoord;
uniform mat3 u_transform;
out vec2 v_texcoord;

void main() {
vec3 pos = u_transform * vec3(a_position, 1.0);
gl_Position = vec4(pos.xy, 0.0, 1.0);
v_texcoord = a_texcoord;
}
"#;

const FRAGMENT_SHADER_SOURCE: &str = r#"
#ifdef GL_ES
precision lowp float;
#endif
uniform sampler2D u_texture;
uniform float u_opacity;
uniform int u_is_glow;
in vec2 v_texcoord;
out vec4 f_color;

void main() {
vec4 tex_color = texture(u_texture, v_texcoord);

if (u_is_glow == 1) {
float brightness = max(tex_color.r, max(tex_color.g, tex_color.b));
f_color = vec4(tex_color.rgb, brightness) * u_opacity;
} else {
f_color = tex_color * u_opacity;
}
}
"#;

/// An OpenGL-based hardware-accelerated rendering pipeline.
///
/// This struct manages the active OpenGL state, including buffers, shader programs,
/// and texture handles required to draw the calculated geometry.
pub struct GlowRenderer {
    program: glow::Program,
    vertex_array: glow::VertexArray,
    vbo: glow::Buffer,
    tbo: glow::Buffer,
    texture: Option<glow::Texture>,
    last_image_id: usize,
}

impl GlowRenderer {
    /// Initializes a new rendering pipeline and allocates required OpenGL buffers and shader programs.
    ///
    /// # Arguments
    /// * `gl_context` - A reference to the active OpenGL context.
    ///
    /// # Returns
    /// A `Result` containing the initialized `GlowRenderer`, or a `CanvasError` if OpenGL buffer or shader allocation fails.
    pub fn new(gl_context: &glow::Context) -> Result<Self, CanvasError> {
        unsafe {
            let program = compile_program(gl_context, VERTEX_SHADER_SOURCE, FRAGMENT_SHADER_SOURCE)?;
            let vertex_array = gl_context.create_vertex_array().map_err(|_| CanvasError::VaoCreation)?;
            let vbo = gl_context.create_buffer().map_err(|_| CanvasError::VboCreation)?;
            let tbo = gl_context.create_buffer().map_err(|_| CanvasError::TboCreation)?;

            gl_context.bind_vertex_array(Some(vertex_array));

            gl_context.bind_buffer(glow::ARRAY_BUFFER, Some(vbo));
            let position_location = gl_context.get_attrib_location(program, "a_position").unwrap_or(0);
            gl_context.enable_vertex_attrib_array(position_location);
            gl_context.vertex_attrib_pointer_f32(position_location, 2, glow::FLOAT, false, 0, 0);

            gl_context.bind_buffer(glow::ARRAY_BUFFER, Some(tbo));
            let texture_location = gl_context.get_attrib_location(program, "a_texcoord").unwrap_or(1);
            gl_context.enable_vertex_attrib_array(texture_location);
            gl_context.vertex_attrib_pointer_f32(texture_location, 2, glow::FLOAT, false, 0, 0);

            gl_context.bind_vertex_array(None);

            Ok(Self {
                program,
                vertex_array,
                vbo,
                tbo,
                texture: None,
                last_image_id: 0,
            })
        }
    }

    fn upload_texture(&mut self, gl_context: &glow::Context, sheet: &SpriteSheet) -> Result<(), CanvasError> {
        unsafe {
            let Some(image) = &sheet.image_data else {
                return Ok(());
            };

            let current_image_id = Arc::as_ptr(image) as usize;

            if self.last_image_id == current_image_id && self.texture.is_some() {
                return Ok(());
            }

            let texture_id = if let Some(existing_texture) = self.texture {
                gl_context.bind_texture(glow::TEXTURE_2D, Some(existing_texture));
                existing_texture
            } else {
                let new_texture = gl_context.create_texture().map_err(|_| CanvasError::TextureAllocation)?;
                gl_context.bind_texture(glow::TEXTURE_2D, Some(new_texture));

                gl_context.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_WRAP_S, glow::CLAMP_TO_EDGE as i32);
                gl_context.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_WRAP_T, glow::CLAMP_TO_EDGE as i32);
                gl_context.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_MIN_FILTER, glow::LINEAR as i32);
                gl_context.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_MAG_FILTER, glow::LINEAR as i32);

                new_texture
            };

            let pixels = image.pixels();
            let mut data: Vec<u8> = Vec::with_capacity(pixels.len() * 4);

            let gamma_value: f32 = 1.9;
            let inverse_gamma = 1.0 / gamma_value;
            let to_linear = |byte_value: u8| -> f32 { (byte_value as f32 / 255.0).powf(gamma_value) };
            let to_monitor = |value: f32| -> u8 { (value.powf(inverse_gamma) * 255.0 + 0.5).clamp(0.0, 255.0) as u8 };

            for pixel in pixels {
                let alpha_byte = pixel[3];

                if alpha_byte == 0 {
                    data.extend_from_slice(&[0, 0, 0, 0]);
                    continue;
                }

                let red_linear = to_linear(pixel[0]);
                let green_linear = to_linear(pixel[1]);
                let blue_linear = to_linear(pixel[2]);
                let alpha_linear = alpha_byte as f32 / 255.0;

                let red_monitor = to_monitor(red_linear) as f32;
                let green_monitor = to_monitor(green_linear) as f32;
                let blue_monitor = to_monitor(blue_linear) as f32;

                data.push((red_monitor * alpha_linear) as u8);
                data.push((green_monitor * alpha_linear) as u8);
                data.push((blue_monitor * alpha_linear) as u8);
                data.push(alpha_byte);
            }

            gl_context.tex_image_2d(
                glow::TEXTURE_2D, 0, glow::RGBA8 as i32,
                image.width() as i32, image.height() as i32, 0,
                glow::RGBA, glow::UNSIGNED_BYTE, Some(&data),
            );

            self.texture = Some(texture_id);
            self.last_image_id = current_image_id;

            Ok(())
        }
    }

    pub fn paint(
        &mut self,
        gl_context: &glow::Context,
        viewport_width: f32,
        viewport_height: f32,
        parts: &[WorldTransform],
        sheet: &SpriteSheet,
        pan_x: f32,
        pan_y: f32,
        zoom: f32,
    ) -> Result<(), CanvasError> {
        unsafe {
            self.upload_texture(gl_context, sheet)?;

            if self.texture.is_none() { return Ok(()); }

            gl_context.disable(glow::DEPTH_TEST);
            gl_context.depth_mask(false);
            gl_context.disable(glow::CULL_FACE);

            gl_context.use_program(Some(self.program));
            gl_context.bind_vertex_array(Some(self.vertex_array));
            gl_context.active_texture(glow::TEXTURE0);
            gl_context.bind_texture(glow::TEXTURE_2D, self.texture);

            let projection = [
                2.0 / viewport_width, 0.0, 0.0,
                0.0, -2.0 / viewport_height, 0.0,
                -1.0, 1.0, 1.0,
            ];

            let center_x = viewport_width / 2.0;
            let center_y = viewport_height / 2.0;

            let camera = [
                zoom, 0.0, 0.0,
                0.0, zoom, 0.0,
                center_x + pan_x * zoom, center_y + pan_y * zoom, 1.0
            ];

            let view_matrix = multiply_mat3(&projection, &camera);

            let u_transform = gl_context.get_uniform_location(self.program, "u_transform");
            let u_opacity = gl_context.get_uniform_location(self.program, "u_opacity");
            let u_texture = gl_context.get_uniform_location(self.program, "u_texture");
            let u_is_glow = gl_context.get_uniform_location(self.program, "u_is_glow");

            gl_context.uniform_1_i32(u_texture.as_ref(), 0);
            gl_context.enable(glow::BLEND);

            for part in parts {
                if part.hidden || part.opacity < 0.005 { continue; }

                gl_context.uniform_1_i32(u_is_glow.as_ref(), if part.glow > 0 { 1 } else { 0 });

                if part.glow > 0 {
                    gl_context.blend_func_separate(glow::ONE, glow::ONE, glow::ONE, glow::ONE);
                } else {
                    gl_context.blend_func(glow::ONE, glow::ONE_MINUS_SRC_ALPHA);
                }

                let Some(cut) = sheet.cuts_map.get(&part.sprite_index) else { continue; };

                let sprite_width = cut.original_size.x;
                let sprite_height = cut.original_size.y;
                let pivot_x = part.pivot.x;
                let pivot_y = part.pivot.y;

                let final_matrix = multiply_mat3(&view_matrix, &part.matrix);

                gl_context.uniform_matrix_3_f32_slice(u_transform.as_ref(), false, &final_matrix);
                gl_context.uniform_1_f32(u_opacity.as_ref(), part.opacity);

                let vertices: [f32; 12] = [
                    -pivot_x,               -pivot_y,
                    sprite_width - pivot_x, -pivot_y,
                    -pivot_x,               sprite_height - pivot_y,

                    -pivot_x,               sprite_height - pivot_y,
                    sprite_width - pivot_x, -pivot_y,
                    sprite_width - pivot_x, sprite_height - pivot_y,
                ];

                gl_context.bind_buffer(glow::ARRAY_BUFFER, Some(self.vbo));
                gl_context.buffer_data_u8_slice(glow::ARRAY_BUFFER, bytemuck::cast_slice(&vertices), glow::DYNAMIC_DRAW);

                let uv_coordinates = cut.uv_coordinates;
                let texture_coordinates: [f32; 12] = [
                    uv_coordinates.min.x, uv_coordinates.min.y,
                    uv_coordinates.max.x, uv_coordinates.min.y,
                    uv_coordinates.min.x, uv_coordinates.max.y,

                    uv_coordinates.min.x, uv_coordinates.max.y,
                    uv_coordinates.max.x, uv_coordinates.min.y,
                    uv_coordinates.max.x, uv_coordinates.max.y,
                ];

                gl_context.bind_buffer(glow::ARRAY_BUFFER, Some(self.tbo));
                gl_context.buffer_data_u8_slice(glow::ARRAY_BUFFER, bytemuck::cast_slice(&texture_coordinates), glow::DYNAMIC_DRAW);

                gl_context.draw_arrays(glow::TRIANGLES, 0, 6);
            }

            gl_context.blend_func(glow::ONE, glow::ONE_MINUS_SRC_ALPHA);

            Ok(())
        }
    }
}

fn multiply_mat3(matrix_a: &[f32; 9], matrix_b: &[f32; 9]) -> [f32; 9] {
    [
        matrix_a[0]*matrix_b[0] + matrix_a[3]*matrix_b[1] + matrix_a[6]*matrix_b[2],
        matrix_a[1]*matrix_b[0] + matrix_a[4]*matrix_b[1] + matrix_a[7]*matrix_b[2],
        matrix_a[2]*matrix_b[0] + matrix_a[5]*matrix_b[1] + matrix_a[8]*matrix_b[2],

        matrix_a[0]*matrix_b[3] + matrix_a[3]*matrix_b[4] + matrix_a[6]*matrix_b[5],
        matrix_a[1]*matrix_b[3] + matrix_a[4]*matrix_b[4] + matrix_a[7]*matrix_b[5],
        matrix_a[2]*matrix_b[3] + matrix_a[5]*matrix_b[4] + matrix_a[8]*matrix_b[5],

        matrix_a[0]*matrix_b[6] + matrix_a[3]*matrix_b[7] + matrix_a[6]*matrix_b[8],
        matrix_a[1]*matrix_b[6] + matrix_a[4]*matrix_b[7] + matrix_a[7]*matrix_b[8],
        matrix_a[2]*matrix_b[6] + matrix_a[5]*matrix_b[7] + matrix_a[8]*matrix_b[8],
    ]
}

unsafe fn compile_program(gl_context: &glow::Context, vertex_shader_source: &str, fragment_shader_source: &str) -> Result<glow::Program, CanvasError> {
    unsafe {
        let program = gl_context.create_program().map_err(|_| CanvasError::ProgramCreation)?;

        let shader_version = if cfg!(target_arch = "wasm32") {
            "#version 300 es\n"
        } else {
            "#version 330\n"
        };

        let compiled_vertex_source = format!("{}{}", shader_version, vertex_shader_source);
        let compiled_fragment_source = format!("{}{}", shader_version, fragment_shader_source);

        let vertex_shader = gl_context.create_shader(glow::VERTEX_SHADER).map_err(|_| CanvasError::VertexShaderCreation)?;
        gl_context.shader_source(vertex_shader, &compiled_vertex_source);
        gl_context.compile_shader(vertex_shader);
        if !gl_context.get_shader_compile_status(vertex_shader) {
            return Err(CanvasError::ShaderCompile(gl_context.get_shader_info_log(vertex_shader)));
        }
        gl_context.attach_shader(program, vertex_shader);

        let fragment_shader = gl_context.create_shader(glow::FRAGMENT_SHADER).map_err(|_| CanvasError::FragmentShaderCreation)?;
        gl_context.shader_source(fragment_shader, &compiled_fragment_source);
        gl_context.compile_shader(fragment_shader);
        if !gl_context.get_shader_compile_status(fragment_shader) {
            return Err(CanvasError::ShaderCompile(gl_context.get_shader_info_log(fragment_shader)));
        }
        gl_context.attach_shader(program, fragment_shader);

        gl_context.link_program(program);
        if !gl_context.get_program_link_status(program) {
            return Err(CanvasError::ProgramLink(gl_context.get_program_info_log(program)));
        }

        gl_context.delete_shader(vertex_shader);
        gl_context.delete_shader(fragment_shader);

        Ok(program)
    }
}