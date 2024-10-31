use std::sync::Once;
use web_sys::{js_sys, WebGl2RenderingContext};

pub struct ShaderProgram {
    pub program: web_sys::WebGlProgram,
    pub vertex_buffer: web_sys::WebGlBuffer,
}

static INIT: Once = Once::new();
static mut SHADER_PROGRAM: Option<ShaderProgram> = None;

impl ShaderProgram {
    pub fn instance(gl: &WebGl2RenderingContext) -> &'static ShaderProgram {
        unsafe {
            INIT.call_once(|| {
                SHADER_PROGRAM = Some(ShaderProgram::new(gl).expect("Failed to create shader program"));
            });
            SHADER_PROGRAM.as_ref().unwrap()
        }
    }
    pub fn new(gl: &WebGl2RenderingContext) -> Result<Self, String> {
        // 顶点着色器
        const VERTEX_SHADER: &str = r#"#version 300 es
            in vec2 aPosition;
            in vec2 aTexCoord;
            
            uniform mat3 uTransform;
            out vec2 vTexCoord;
            
            void main() {
                vec3 position = uTransform * vec3(aPosition, 1.0);
                gl_Position = vec4(position.xy, 0.0, 1.0);
                vTexCoord = aTexCoord;
            }
        "#;

        // 片段着色器
        const FRAGMENT_SHADER: &str = r#"#version 300 es
            precision mediump float;
            
            uniform sampler2D uTexture;
            uniform float uOpacity;
            
            in vec2 vTexCoord;
            out vec4 fragColor;
            
            void main() {
                vec4 texColor = texture(uTexture, vTexCoord);
                fragColor = vec4(texColor.rgb, texColor.a * uOpacity);
            }
        "#;

        // 编译着色器程序
        let vert_shader = compile_shader(
            gl,
            WebGl2RenderingContext::VERTEX_SHADER,
            VERTEX_SHADER,
        )?;
        let frag_shader = compile_shader(
            gl,
            WebGl2RenderingContext::FRAGMENT_SHADER,
            FRAGMENT_SHADER,
        )?;

        // 创建程序
        let program = gl.create_program().ok_or("Unable to create shader program")?;
        gl.attach_shader(&program, &vert_shader);
        gl.attach_shader(&program, &frag_shader);
        gl.link_program(&program);

        // 检查链接状态
        if !gl.get_program_parameter(&program, WebGl2RenderingContext::LINK_STATUS)
            .as_bool()
            .unwrap_or(false)
        {
            return Err(gl
                .get_program_info_log(&program)
                .unwrap_or_else(|| "Unknown error creating program".into()));
        }

        // 创建顶点缓冲区
        let vertex_buffer = gl.create_buffer().ok_or("Failed to create buffer")?;
        gl.bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, Some(&vertex_buffer));

        // 矩形的顶点数据（位置和纹理坐标）
        let vertices: [f32; 24] = [
            // 位置(x,y)    // 纹理坐标(u,v)
            0.0, 0.0,       0.0, 0.0,  // 左上
            1.0, 0.0,       1.0, 0.0,  // 右上
            0.0, 1.0,       0.0, 1.0,  // 左下
            0.0, 1.0,       0.0, 1.0,  // 左下
            1.0, 0.0,       1.0, 0.0,  // 右上
            1.0, 1.0,       1.0, 1.0,  // 右下
        ];

        // 将顶点数据传输到GPU
        unsafe {
            let vert_array = js_sys::Float32Array::view(&vertices);
            gl.buffer_data_with_array_buffer_view(
                WebGl2RenderingContext::ARRAY_BUFFER,
                &vert_array,
                WebGl2RenderingContext::STATIC_DRAW,
            );
        }

        Ok(ShaderProgram {
            program,
            vertex_buffer,
        })
    }

    pub fn bind(&self, gl: &WebGl2RenderingContext) {
        gl.use_program(Some(&self.program));
        gl.bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, Some(&self.vertex_buffer));

        // 设置顶点属性
        let pos_attr = gl.get_attrib_location(&self.program, "aPosition") as u32;
        let tex_attr = gl.get_attrib_location(&self.program, "aTexCoord") as u32;

        // 位置属性
        gl.vertex_attrib_pointer_with_i32(
            pos_attr,
            2,
            WebGl2RenderingContext::FLOAT,
            false,
            4 * 4,  // stride: 4 个 float (2 position + 2 texcoord) * 4 bytes
            0,      // offset for position
        );
        gl.enable_vertex_attrib_array(pos_attr);

        // 纹理坐标属性
        gl.vertex_attrib_pointer_with_i32(
            tex_attr,
            2,
            WebGl2RenderingContext::FLOAT,
            false,
            4 * 4,  // stride: 4 个 float * 4 bytes
            2 * 4,  // offset: 2 个 float * 4 bytes
        );
        gl.enable_vertex_attrib_array(tex_attr);
    }

    pub fn set_transform(&self, gl: &WebGl2RenderingContext, transform: &glam::DMat3) {
        let transform_location = gl.get_uniform_location(&self.program, "uTransform");
        let transform_array: [f32; 9] = transform.to_cols_array().iter().map(|&x| x as f32).collect::<Vec<f32>>().try_into().unwrap();
        gl.uniform_matrix3fv_with_f32_array(
            transform_location.as_ref(),
            false,
            &transform_array,
        );
    }

    pub fn set_opacity(&self, gl: &WebGl2RenderingContext, opacity: f64) {
        let opacity_location = gl.get_uniform_location(&self.program, "uOpacity");
        gl.uniform1f(opacity_location.as_ref(), opacity as f32);
    }
}

fn compile_shader(
    gl: &WebGl2RenderingContext,
    shader_type: u32,
    source: &str,
) -> Result<web_sys::WebGlShader, String> {
    let shader = gl
        .create_shader(shader_type)
        .ok_or_else(|| String::from("Unable to create shader object"))?;
    gl.shader_source(&shader, source);
    gl.compile_shader(&shader);

    if gl
        .get_shader_parameter(&shader, WebGl2RenderingContext::COMPILE_STATUS)
        .as_bool()
        .unwrap_or(false)
    {
        Ok(shader)
    } else {
        Err(gl
            .get_shader_info_log(&shader)
            .unwrap_or_else(|| "Unknown error creating shader".into()))
    }
}