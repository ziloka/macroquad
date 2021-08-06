use crate::{
    camera::RenderState,
    color::Color,
    file::{load_file, FileError},
    get_context,
    math::{vec3, Mat4, Rect},
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
    let normals: Vec<[f32; 3]> = reader.read_normals().unwrap().collect::<Vec<_>>();

    //println!("{:#?}", vertices);

    let ctx = &mut get_context().quad_context;
    let vertex_buffer = Buffer::immutable(ctx, BufferType::VertexBuffer, &vertices);
    let normals_buffer = Buffer::immutable(ctx, BufferType::VertexBuffer, &normals);
    let index_buffer = Buffer::immutable(ctx, BufferType::IndexBuffer, &indices);
    let bindings = Bindings {
        vertex_buffers: vec![vertex_buffer, normals_buffer],
        index_buffer,
        images: vec![],
    };

    Ok(Model { bindings })
}

use crate::quad_gl::QuadGl;

pub struct SpriteLayer {
    gl: QuadGl,
    render_state: RenderState,
}

impl SpriteLayer {
    pub fn new(mut gl: QuadGl, render_state: RenderState) -> SpriteLayer {
        SpriteLayer { gl, render_state }
    }

    pub fn gl(&mut self) -> &mut QuadGl {
        &mut self.gl
    }
}

pub struct SceneGraph {
    models: Vec<(Model, Mat4)>,
    layers_cache: Vec<QuadGl>,
    pipeline: miniquad::Pipeline,
}

impl SceneGraph {
    pub fn new(ctx: &mut miniquad::Context) -> SceneGraph {
        let shader = Shader::new(ctx, shader::VERTEX, shader::FRAGMENT, shader::meta())
            .unwrap_or_else(|e| panic!("Failed to load shader: {}", e));

        let pipeline = Pipeline::with_params(
            ctx,
            &[BufferLayout::default(), BufferLayout::default()],
            &[
                VertexAttribute::with_buffer("position", VertexFormat::Float3, 0),
                VertexAttribute::with_buffer("normal", VertexFormat::Float3, 1),
            ],
            shader,
            PipelineParams {
                depth_test: Comparison::LessOrEqual,
                depth_write: true,
                ..Default::default()
            },
        );

        SceneGraph {
            models: vec![],
            layers_cache: vec![QuadGl::new(ctx), QuadGl::new(ctx), QuadGl::new(ctx)],
            pipeline,
        }
    }

    pub fn add_model(&mut self, model: Model) -> usize {
        self.models.push((model, Mat4::IDENTITY));
        self.models.len() - 1
    }

    pub fn sprite_layer(&mut self, render_state: RenderState) -> SpriteLayer {
        SpriteLayer::new(self.layers_cache.pop().unwrap(), render_state)
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

        let screen_mat = glam::Mat4::orthographic_rh_gl(0., width, height, 0., -1., 1.);
        canvas.gl().draw(context, screen_mat);

        self.layers_cache.push(canvas.gl);
    }

    pub fn draw_model(&mut self, render_state: &RenderState, model: &Model, transform: Mat4) {
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

        ctx.apply_pipeline(&self.pipeline);

        ctx.apply_bindings(&model.bindings);
        ctx.apply_uniforms(&shader::Uniforms {
            projection: render_state.matrix(),
            model: transform,
        });
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
    attribute vec3 position;
    attribute vec3 normal;

    varying lowp vec4 color;

    uniform mat4 Model;
    uniform mat4 Projection;

    void main() {
        color = vec4(dot(normal, vec3(0.0, 1.0, 0.0)), dot(normal, vec3(0.0, -1.0, 0.0)), dot(normal, vec3(-0.2, -0.8, -0.3)), 1);
        gl_Position = Projection * Model * vec4(position, 1);
    }"#;

    pub const FRAGMENT: &str = r#"#version 100
    varying lowp vec4 color;

    void main() {
        gl_FragColor = vec4(1.0, 0.0, 0.0, 1) * (max(color.x, 0.0) + max(color.y, 0.0));
    }"#;

    pub fn meta() -> ShaderMeta {
        ShaderMeta {
            images: vec![],
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
