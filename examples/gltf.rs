use macroquad::prelude::*;

#[macroquad::main("3D")]
async fn main() {
    let model = load_model("examples/ship.gltf").await.unwrap();

    loop {
        let render_state = RenderState {
            camera: Camera::Camera2D {
                rotation: 0.,
                zoom: vec2(1. / 400., -1. / 400.),
                target: vec2(200., 200.),
                offset: vec2(0., 0.),
            },
            ..Default::default()
        };

        scene_graph().clear2(&render_state, Color::new(0.2, 0.2, 0.5, 1.0));

        let mut canvas = scene_graph().sprite_layer(&render_state);
        draw_text(&mut canvas, "WELCOME TO 3D WORLD", 10.0, 20.0, 30.0, BLACK);
        draw_text(&mut canvas, "TEXT BELOW!!!", 400.0, 400.0, 30.0, BLUE);
        draw_rectangle(&mut canvas, 300., 200., 100., 100., RED);
        scene_graph().draw_canvas(canvas);

        scene_graph().draw_model(
            &mut RenderState {
                camera: Camera::Camera3D {
                    position: vec3(-10., 7.5, 0.),
                    up: vec3(0., 1., 0.),
                    target: vec3(0., 0., 0.),
                    fovy: 45.,
                    projection: macroquad::camera::Projection::Perspective,
                },
                ..Default::default()
            },
            &model,
            Mat4::IDENTITY,
        );

        let mut canvas = scene_graph().sprite_layer(&render_state);

        draw_rectangle(&mut canvas, 100., 350., 100., 100., GREEN);
        draw_text(&mut canvas, "TEXT ABOVE!!!", 400.0, 300.0, 30.0, YELLOW);
        scene_graph().draw_canvas(canvas);

        next_frame().await
    }
}
