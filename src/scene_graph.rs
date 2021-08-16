use crate::{
    camera::RenderState,
    color::Color,
    file::{load_file, FileError},
    get_context,
    material::Material,
    math::{vec2, vec3, Mat4, Rect},
    window::miniquad::*,
};

pub struct Model {
    bindings: Bindings,
}

pub async fn load_model(path: &str) -> Result<Model, FileError> {
    let bytes = load_file(path).await?;

    let (gltf, buffers, images) = gltf::import_slice(&bytes).unwrap();
    assert!(gltf.meshes().len() == 1);

    let mesh = gltf.meshes().next().unwrap();

    assert!(mesh.primitives().len() == 1);

    let primitive = mesh.primitives().next().unwrap();

    let reader = primitive.reader(|buffer| Some(&buffers[buffer.index()]));

    let indices: Vec<u16> = reader
        .read_indices()
        .unwrap()
        .into_u32()
        .map(|ix| ix as u16)
        .collect::<Vec<_>>();
    let vertices: Vec<[f32; 3]> = reader.read_positions().unwrap().collect::<Vec<_>>();
    let uvs: Vec<[f32; 2]> = reader
        .read_tex_coords(0)
        .unwrap()
        .into_f32()
        .collect::<Vec<_>>();

    let normals: Vec<[f32; 3]> = reader.read_normals().unwrap().collect::<Vec<_>>();

    //println!("{:#?}", vertices);

    let ctx = &mut get_context().quad_context;
    let vertex_buffer = Buffer::immutable(ctx, BufferType::VertexBuffer, &vertices);
    let normals_buffer = Buffer::immutable(ctx, BufferType::VertexBuffer, &normals);
    let uvs_buffer = Buffer::immutable(ctx, BufferType::VertexBuffer, &uvs);
    let index_buffer = Buffer::immutable(ctx, BufferType::IndexBuffer, &indices);
    let bindings = Bindings {
        vertex_buffers: vec![vertex_buffer, uvs_buffer, normals_buffer],
        index_buffer,
        images: vec![Texture::empty(), Texture::empty()],
    };

    Ok(Model { bindings })
}

pub fn square() -> Model {
    let ctx = &mut get_context().quad_context;

    let width = 1.0;
    let height = 1.0;
    let length = 1.0;
    let indices = [0u16, 1, 2, 0, 2, 3];

    let vertices = [
        vec3(-width / 2., height / 2., -length / 2.),
        vec3(-width / 2., height / 2., length / 2.),
        vec3(width / 2., height / 2., length / 2.),
        vec3(width / 2., height / 2., -length / 2.),
    ];
    let uvs = [vec2(0., 1.), vec2(0., 0.), vec2(1., 0.), vec2(1., 1.)];
    let normals = [
        vec3(0., 1., 0.),
        vec3(0., 1., 0.),
        vec3(0., 1., 0.),
        vec3(0., 1., 0.),
    ];

    let vertex_buffer = Buffer::immutable(ctx, BufferType::VertexBuffer, &vertices);
    let normals_buffer = Buffer::immutable(ctx, BufferType::VertexBuffer, &normals);
    let uvs_buffer = Buffer::immutable(ctx, BufferType::VertexBuffer, &uvs);
    let index_buffer = Buffer::immutable(ctx, BufferType::IndexBuffer, &indices);
    let bindings = Bindings {
        vertex_buffers: vec![vertex_buffer, uvs_buffer, normals_buffer],
        index_buffer,
        images: vec![Texture::empty(), Texture::empty()],
    };

    Model { bindings }
}

use crate::quad_gl::QuadGl;

pub struct SpriteLayer<'a> {
    gl: QuadGl,
    render_state: &'a RenderState,
}

impl<'a> SpriteLayer<'a> {
    pub fn new(mut gl: QuadGl, render_state: &'a RenderState) -> SpriteLayer<'a> {
        SpriteLayer { gl, render_state }
    }

    pub fn gl(&mut self) -> &mut QuadGl {
        &mut self.gl
    }
}

pub struct SceneGraph {
    models: Vec<(Model, Mat4)>,
    layers_cache: Vec<QuadGl>,
    default_material: Material,
}

impl SceneGraph {
    pub fn new(ctx: &mut miniquad::Context) -> SceneGraph {
        let shader = Shader::new(ctx, shader::VERTEX, shader::FRAGMENT, shader::meta())
            .unwrap_or_else(|e| panic!("Failed to load shader: {}", e));

        let default_material = Material::new2(
            ctx,
            shader,
            PipelineParams {
                depth_test: Comparison::LessOrEqual,
                depth_write: true,
                ..Default::default()
            },
            vec![],
            vec![],
        )
        .unwrap();

        SceneGraph {
            models: vec![],
            layers_cache: vec![QuadGl::new(ctx), QuadGl::new(ctx), QuadGl::new(ctx)],
            default_material,
        }
    }

    pub fn add_model(&mut self, model: Model) -> usize {
        self.models.push((model, Mat4::IDENTITY));
        self.models.len() - 1
    }

    pub fn sprite_layer<'a>(&mut self, render_state: &'a RenderState) -> SpriteLayer<'a> {
        let mut gl = self.layers_cache.pop().unwrap();
        let render_pass = render_state.render_target.map(|rt| rt.render_pass);
        gl.render_pass(render_pass);

        SpriteLayer::new(gl, render_state)
    }

    pub fn clear(&mut self, color: Color) {
        let ctx = &mut get_context().quad_context;
        let clear = PassAction::clear_color(color.r, color.g, color.b, color.a);

        ctx.begin_default_pass(clear);
        ctx.end_render_pass();
    }

    pub fn clear2(&mut self, render_state: &RenderState, color: Color) {
        let ctx = &mut get_context().quad_context;
        let clear = PassAction::clear_color(color.r, color.g, color.b, color.a);

        if let Some(pass) = render_state.render_target.map(|rt| rt.render_pass) {
            ctx.begin_pass(pass, clear);
        } else {
            ctx.begin_default_pass(clear);
        }
        ctx.end_render_pass();
    }

    pub fn draw_canvas(&mut self, mut canvas: SpriteLayer) {
        let context = &mut get_context().quad_context;

        let (width, height) = context.screen_size();

        let screen_mat = //glam::Mat4::orthographic_rh_gl(0., width, height, 0., -1., 1.);
            canvas.render_state.matrix();
        canvas.gl().draw(context, screen_mat);

        self.layers_cache.push(canvas.gl);
    }

    pub fn draw_model(&mut self, render_state: &mut RenderState, model: &Model, transform: Mat4) {
        // unsafe {
        //     miniquad::gl::glPolygonMode(miniquad::gl::GL_FRONT_AND_BACK, miniquad::gl::GL_LINE);
        // }
        let ctx = &mut get_context().quad_context;
        //let projection = self.camera.matrix();

        // let pass = get_context().gl.get_active_render_pass();
        if let Some(pass) = render_state.render_target.map(|rt| rt.render_pass) {
            ctx.begin_pass(pass, PassAction::Nothing);
        } else {
            ctx.begin_default_pass(PassAction::Nothing);
        }

        if let Some(ref material) = render_state.material {
            ctx.apply_pipeline(&material.pipeline_3d);
        } else {
            ctx.apply_pipeline(&self.default_material.pipeline_3d);
        }

        let mut bindings = model.bindings.clone();
        if let Some(ref mut material) = render_state.material {
            bindings.images[0] = material
                .textures_data
                .get("Texture")
                .copied()
                .unwrap_or_else(|| Texture::empty())
        }
        ctx.apply_bindings(&bindings);

        let projection = render_state.matrix();
        let time = (crate::time::get_time()) as f32;
        let time = glam::vec4(time, time.sin(), time.cos(), 0.);

        if let Some(ref mut material) = render_state.material {
            material.set_uniform("Projection", projection);
            material.set_uniform("Model", transform);
            material.set_uniform("_Time", time);

            ctx.apply_uniforms_from_bytes(
                material.uniforms_data.as_ptr(),
                material.uniforms_data.len(),
            );
        } else {
            ctx.apply_uniforms(&shader::Uniforms {
                projection,
                model: transform,
            });
        }

        ctx.draw(0, model.bindings.index_buffer.size() as i32 / 2, 1);

        ctx.end_render_pass();

        // unsafe {
        //     miniquad::gl::glPolygonMode(miniquad::gl::GL_FRONT_AND_BACK, miniquad::gl::GL_FILL);
        // }
    }

    pub fn set_transform(&mut self, model: usize, transform: Mat4) {
        self.models[model].1 = transform;
    }
}

mod shader {
    use miniquad::{ShaderMeta, UniformBlockLayout, UniformDesc, UniformType};

    pub const VERTEX: &str = r#"#version 100
    attribute vec3 in_position;
    attribute vec2 in_uv;
    attribute vec3 in_normal;

    varying lowp vec4 out_color;
    varying lowp vec2 out_uv;

    uniform mat4 Model;
    uniform mat4 Projection;

    void main() {
        out_color = vec4(dot(in_normal, vec3(0.0, 1.0, 0.0)), dot(in_normal, vec3(0.0, -1.0, 0.0)), dot(in_normal, vec3(-0.2, -0.8, -0.3)), 1);
        gl_Position = Projection * Model * vec4(in_position, 1);
        out_uv = in_uv;
    }"#;

    pub const FRAGMENT: &str = r#"#version 100
    varying lowp vec4 out_color;
    varying lowp vec2 out_uv;

    lowp float chessboard(lowp vec2 uv)
    {
	uv = floor(uv * 2.0);
    
        return mod(uv.x + uv.y, 2.0);
    }

    void main() {
        gl_FragColor = vec4(1.0, 0.0, 0.0, 1) * (max(out_color.x, 0.0) + max(out_color.y, 0.0));
    }"#;

    pub fn meta() -> ShaderMeta {
        ShaderMeta {
            images: vec!["Texture".to_string()],
            uniforms: UniformBlockLayout {
                uniforms: vec![
                    UniformDesc::new("Projection", UniformType::Mat4),
                    UniformDesc::new("Model", UniformType::Mat4),
                ],
            },
        }
    }

    #[repr(C)]
    pub struct Uniforms {
        pub projection: crate::math::Mat4,
        pub model: crate::math::Mat4,
    }
}
