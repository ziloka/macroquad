use macroquad::prelude::*;

#[macroquad::main("3D")]
async fn main() {
    let model = load_model("../lowres/assets/ship.gltf").await.unwrap();

    loop {
        clear_background(LIGHTGRAY);

        let mut canvas = scene_graph().sprite_layer(&Default::default());
        draw_text(&mut canvas, "WELCOME TO 3D WORLD", 10.0, 20.0, 30.0, BLACK);
        draw_text(&mut canvas, "TEXT BELOW!!!", 400.0, 600.0, 30.0, BLUE);
        draw_rectangle(&mut canvas, 300., 300., 100., 100., RED);
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

        let mut canvas = scene_graph().sprite_layer(&Default::default());
        draw_rectangle(&mut canvas, 500., 600., 100., 100., GREEN);
        draw_text(&mut canvas, "TEXT ABOVE!!!", 400.0, 300.0, 30.0, YELLOW);
        scene_graph().draw_canvas(canvas);

        next_frame().await
    }
}
