use amethyst_core::Axis2;
use amethyst_core::cgmath::Ortho;
use amethyst_core::specs::{Component, DenseVecStorage, Join, System, ReadExpect, ReadStorage, WriteStorage};
use amethyst_renderer::{Camera, ScreenDimensions};

/// `Component` attached to the camera's entity that allows automatically adjusting the camera's matrix according to preferences in the "mode" field.
#[derive(Default)]
pub struct NormalOrthoCamera {
    /// How the camera's matrix is changed when the window's aspect ratio changes. See `CameraNormalizeMode` for more info.
    pub mode: CameraNormalizeMode,
}

impl NormalOrthoCamera {
    pub fn camera_offsets(&self, ratio: f32) -> (f32,f32,f32,f32) {
        self.mode.camera_offsets(ratio)
    }
}

impl Component for NormalOrthoCamera {
    type Storage = DenseVecStorage<Self>;
}

/// Settings that decide how to scale the camera's matrix when the aspect ratio changes.
pub enum CameraNormalizeMode {
    /// Using an aspect ratio of 1:1, tries to ajust the matrix values of the camera so
    /// that the direction opposite to the stretch_direction is always [0,1].
    /// Scene space can be lost on the specified stretch_direction.
    Lossy {stretch_direction: Axis2},
    
    /// Scales the render dynamically to ensure no space is lost in the [0,1] range on any axis.
    Shrink,
}

impl CameraNormalizeMode {
    /// Get the camera matrix offsets according to the specified options.
    pub fn camera_offsets(&self, aspect_ratio: f32) -> (f32,f32,f32,f32) {
        match self {
            &CameraNormalizeMode::Lossy {ref stretch_direction} => {
                match stretch_direction {
                    Axis2::X => {
                        CameraNormalizeMode::lossy_x(aspect_ratio)
                    },
                    Axis2::Y => {
                        CameraNormalizeMode::lossy_y(aspect_ratio)
                    },
                }
            },
            &CameraNormalizeMode::Shrink => {
                if aspect_ratio > 1.0 {
                    CameraNormalizeMode::lossy_x(aspect_ratio)
                } else if aspect_ratio < 1.0 {
                    CameraNormalizeMode::lossy_y(aspect_ratio)
                } else {
                    (0.0,1.0,0.0,1.0)
                }
            },
        }
    }
    
    fn lossy_x(aspect_ratio: f32) -> (f32,f32,f32,f32) {
        let offset = (aspect_ratio - 1.0) / 2.0;
        (-offset, 1.0 + offset, 0.0, 1.0)
    }

    fn lossy_y(aspect_ratio: f32) -> (f32,f32,f32,f32) {
        let offset = (1.0 / aspect_ratio - 1.0) / 2.0;
        (0.0, 1.0, -offset, 1.0 + offset)
    }
}

impl Default for CameraNormalizeMode {
    fn default() -> Self {
        CameraNormalizeMode::Shrink
    }
}

/// System that automatically changes the camera matrix according to the settings in the `NormalOrthoCamera` attached to the camera entity.
#[derive(Default)]
pub struct NormalOrthoCameraSystem {
    aspect_ratio_cache: f32,
}

impl<'a> System<'a> for NormalOrthoCameraSystem {
    type SystemData = (ReadExpect<'a, ScreenDimensions>, WriteStorage<'a, Camera>, ReadStorage<'a, NormalOrthoCamera>);
    fn run(&mut self, (dimensions, mut cameras, ortho_cameras): Self::SystemData) {
        let aspect = dimensions.aspect_ratio();
        if aspect != self.aspect_ratio_cache {
            self.aspect_ratio_cache = aspect;

            for (mut camera, ortho_camera) in (&mut cameras, &ortho_cameras).join() {
                let offsets = ortho_camera.camera_offsets(aspect);
                camera.proj = Ortho {
                    left: offsets.0,
                    right: offsets.1,
                    bottom: offsets.2,
                    top: offsets.3,
                    near: 0.1,
                    far: 1000.0,
                }.into();
            }
        }
    }
}
