//! 2D and 3D camera.

use crate::{
    get_context,
    texture::RenderTarget,
    window::{screen_height, screen_width},
};
use glam::{vec2, vec3, Mat4, Vec2, Vec3};

#[derive(Debug, Clone, Copy)]
pub enum Projection {
    Perspective,
    Orthographics,
}

#[derive(Debug, Clone)]
pub enum Camera {
    Camera2D {
        /// Rotation in degrees
        rotation: f32,
        /// Scaling, should be (1.0, 1.0) by default
        zoom: Vec2,
        /// Rotation and zoom origin
        target: Vec2,
        /// Displacement from target
        offset: Vec2,
    },
    Camera3D {
        /// Camera position
        position: Vec3,
        /// Camera target it looks-at
        target: Vec3,
        /// Camera up vector (rotation over its axis)
        up: Vec3,
        /// Camera field-of-view aperture in Y (degrees)
        /// in perspective, used as near plane width in orthographic
        fovy: f32,
        /// Camera projection type, perspective or orthographics
        projection: Projection,
    },
}

#[derive(Clone, Debug)]
pub struct RenderState {
    pub depth_enabled: bool,
    pub render_target: Option<RenderTarget>,

    pub aspect: Option<f32>,

    ///
    pub camera: Camera,
    /// Rectangle on the screen where this camera's output is drawn
    /// Numbers are pixels in window-spae, x, y, width, height
    pub viewport: Option<(i32, i32, i32, i32)>,
}

impl Default for RenderState {
    fn default() -> Self {
        RenderState {
            depth_enabled: false,
            render_target: None,
            aspect: None,

            camera: Camera::Camera2D {
                target: vec2(0., 0.),
                zoom: vec2(1., 1.),
                offset: vec2(0., 0.),
                rotation: 0.,
            },
            viewport: None,
        }
    }
}

impl RenderState {
    const Z_NEAR: f32 = 1.1;
    const Z_FAR: f32 = 100.0;

    pub fn matrix(&self) -> Mat4 {
        match self.camera {
            Camera::Camera2D {
                target,
                rotation,
                zoom,
                offset,
            } => {
                // gleaned from https://github.com/raysan5/raylib/blob/master/src/core.c#L1528

                // The camera in world-space is set by
                //   1. Move it to target
                //   2. Rotate by -rotation and scale by (1/zoom)
                //      When setting higher scale, it's more intuitive for the world to become bigger (= camera become smaller),
                //      not for the camera getting bigger, hence the invert. Same deal with rotation.
                //   3. Move it by (-offset);
                //      Offset defines target transform relative to screen, but since we're effectively "moving" screen (camera)
                //      we need to do it into opposite direction (inverse transform)

                // Having camera transform in world-space, inverse of it gives the modelview transform.
                // Since (A*B*C)' = C'*B'*A', the modelview is
                //   1. Move to offset
                //   2. Rotate and Scale
                //   3. Move by -target
                let mat_origin = Mat4::from_translation(vec3(-target.x, -target.y, 0.0));
                let mat_rotation =
                    Mat4::from_axis_angle(vec3(0.0, 0.0, 1.0), rotation.to_radians());
                let mat_scale = Mat4::from_scale(vec3(zoom.x, zoom.y, 1.0));
                let mat_translation = Mat4::from_translation(vec3(offset.x, offset.y, 0.0));

                mat_translation * ((mat_scale * mat_rotation) * mat_origin)
            }
            Camera::Camera3D {
                fovy,
                position,
                target,
                up,
                projection,
            } => {
                let aspect = self.aspect.unwrap_or(screen_width() / screen_height());
                match projection {
                    Projection::Perspective => {
                        Mat4::perspective_rh_gl(fovy, aspect, Self::Z_NEAR, Self::Z_FAR)
                            * Mat4::look_at_rh(position, target, up)
                    }
                    Projection::Orthographics => {
                        let top = fovy / 2.0;
                        let right = top * aspect;

                        Mat4::orthographic_rh_gl(
                            -right,
                            right,
                            -top,
                            top,
                            Self::Z_NEAR,
                            Self::Z_FAR,
                        ) * Mat4::look_at_rh(position, target, up)
                    }
                }
            }
        }
    }
}

// /// Set active 2D or 3D camera
// pub fn set_camera(camera: &Camera) {
//     let context = get_context();

//     // flush previous camera draw calls
//     context.perform_render_passes();

//     context
//         .gl
//         .render_pass(camera.render_target.map(|rt| rt.render_pass));
//     context.gl.viewport(camera.viewport);
//     context.gl.depth_test(camera.depth_enabled);
//     context.camera_matrix = Some(camera.matrix());
// }

// /// Reset default 2D camera mode
// pub fn set_default_camera() {
//     let context = get_context();

//     // flush previous camera draw calls
//     context.perform_render_passes();

//     context.gl.render_pass(None);
//     context.gl.depth_test(false);
//     context.camera_matrix = None;
// }
