use cgmath::{InnerSpace, Vector3};
use winit::event::{ElementState, MouseButton, WindowEvent};
use winit::keyboard::{KeyCode, PhysicalKey};

pub struct Camera{
    eye: cgmath::Point3<f32>,
    target: cgmath::Point3<f32>,
    up: cgmath::Vector3<f32>,
    aspect: f32,
    fov_y: f32,
    z_near: f32,
    z_far: f32,
}
impl Camera {
    pub fn new()->Self{
        Self {
            eye: (0.0,0.0, 500.0).into(),
            target: (0.0, 0.0, 0.0).into(),
            up: cgmath::Vector3::unit_y(),
            aspect: 1.0,
            fov_y: 50.0,
            z_near: 0.1,
            z_far: 100.0,
        }
    }
    pub fn build_view_matrix(&self) -> cgmath::Matrix4<f32> {
        let view = cgmath::Matrix4::look_at_rh(self.eye, self.target, self.up);
        view
    }
    pub fn build_proj_matrix(&self) -> cgmath::Matrix4<f32> {
        let proj = cgmath::perspective(cgmath::Deg(self.fov_y), self.aspect, self.z_near, self.z_far);
        proj
    }
}
#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraRaw {
    view: [[f32; 4]; 4],
    proj: [[f32; 4]; 4],
    opengl_to_wgpu: [[f32; 4]; 4],
}
impl CameraRaw {
    pub(crate) fn new() -> Self {
        use cgmath::SquareMatrix;
        Self {
            view: cgmath::Matrix4::identity().into(),
            proj: cgmath::Matrix4::identity().into(),
            opengl_to_wgpu: OPENGL_TO_WGPU_MATRIX.into(),
        }
    }
    pub(crate) fn update_view_proj(&mut self, camera: &Camera) {
        self.view = camera.build_view_matrix().into();
        self.proj = camera.build_proj_matrix().into();
    }
}
#[rustfmt::skip]
pub const OPENGL_TO_WGPU_MATRIX: cgmath::Matrix4<f32> = cgmath::Matrix4::new(
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 0.5, 0.5,
    0.0, 0.0, 0.0, 1.0,
);
pub struct CameraController {
    pub speed: f32,
    pub sensitivity: f32,

    forward: bool,
    backward: bool,
    left: bool,
    right: bool,

    mouse_pressed: bool,
    last_mouse_pos: Option<(f64, f64)>,

    yaw: f32,
    pitch: f32,
}
impl CameraController {
    pub fn new(speed: f32, sensitivity: f32) -> Self {
        Self {
            speed,
            sensitivity,
            forward: false,
            backward: false,
            left: false,
            right: false,
            mouse_pressed: false,
            last_mouse_pos: None,
            yaw: -90.0,
            pitch: 0.0,
        }
    }
    pub fn process_events(&mut self, event: &WindowEvent) {
        match event {
            WindowEvent::KeyboardInput { event, .. } => {
                let pressed = event.state == ElementState::Pressed;

                match event.physical_key {
                    PhysicalKey::Code(KeyCode::KeyW) => self.forward = pressed,
                    PhysicalKey::Code(KeyCode::KeyS) => self.backward = pressed,
                    PhysicalKey::Code(KeyCode::KeyA) => self.left = pressed,
                    PhysicalKey::Code(KeyCode::KeyD) => self.right = pressed,
                    _ => {}
                }
            }

            WindowEvent::MouseInput { state, button, .. } => {
                if *button == MouseButton::Left {
                    self.mouse_pressed = *state == ElementState::Pressed;
                    if !self.mouse_pressed {
                        self.last_mouse_pos = None;
                    }
                }
            }

            WindowEvent::CursorMoved { position, .. } => {
                if self.mouse_pressed {
                    if let Some((last_x, last_y)) = self.last_mouse_pos {
                        let dx = position.x - last_x;
                        let dy = position.y - last_y;

                        self.yaw += dx as f32 * self.sensitivity;
                        self.pitch -= dy as f32 * self.sensitivity;

                        self.pitch = self.pitch.clamp(-89.0, 89.0);
                    }
                    self.last_mouse_pos = Some((position.x, position.y));
                }
            }

            _ => {}
        }
    }
    pub fn update_camera(&self, camera: &mut Camera, aspect: f32) {
        let yaw_rad = self.yaw.to_radians();
        let pitch_rad = self.pitch.to_radians();

        let direction = Vector3 {
            x: yaw_rad.cos() * pitch_rad.cos(),
            y: pitch_rad.sin(),
            z: yaw_rad.sin() * pitch_rad.cos(),
        }
            .normalize();

        let right = direction.cross(camera.up).normalize();

        let mut movement = Vector3::new(0.0, 0.0, 0.0);

        if self.forward {
            movement += direction;
        }
        if self.backward {
            movement -= direction;
        }
        if self.right {
            movement += right;
        }
        if self.left {
            movement -= right;
        }

        if movement.magnitude() > 0.0 {
            let movement = movement.normalize() * self.speed;
            camera.eye += movement;
        }
        
        camera.target = camera.eye + direction;

        camera.aspect = aspect;
    }
}