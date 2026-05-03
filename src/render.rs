use std::sync::Arc;
use winit::event::WindowEvent;
use winit::window::Window;
use easy_gpu::assets::*;
use easy_gpu::assets_manager::*;
use easy_gpu::wgpu::{TextureFormat, VertexFormat};
use easy_gpu::Renderer;
use crate::camera::{Camera, CameraController, CameraRaw};
use crate::simulation::{Particle, Simulation};

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex{
    position: [f32;2],
}
impl Vertex {
    pub fn new(position: [f32;2]) -> Self {
        Vertex{position}
    }
}
impl GpuVertex for Vertex {
    fn buffer_layout() -> BufferLayout {
        BufferLayout::new()
            .stride(size_of::<Self>() as u64)
            .attribute(0,0,VertexFormat::Float32x2)
    }
}

pub struct Render{
    pub core: Renderer,
    camera_buffer: Handle<Buffer>,
    camera_controller: CameraController,
    camera: Camera,
    camera_raw: CameraRaw,
    pub(crate) material: Handle<Material>,
    pub(crate) mesh: Handle<Mesh>,
}

impl Render{
    pub fn new(window: Arc<Window>)->(Self,Simulation){
        let mut renderer = pollster::block_on(Renderer::new(window.clone()));

        let scale = 4.0;
        let vertices = [
            Vertex::new([-scale, -scale]),
            Vertex::new([scale, -scale]),
            Vertex::new([scale, scale]),
            Vertex::new([-scale, scale])
        ];
        let indices = [0, 1, 2, 0, 2, 3];
        let mesh = renderer.create_mesh(&vertices, &indices);

        let camera = Camera::new();
        let camera_controller = CameraController::new(4.0,0.15);
        let mut camera_raw = CameraRaw::new();
        camera_raw.update_view_proj(&camera);
        let camera_buffer = renderer.create_buffer_with_contents(
            BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            bytemuck::cast_slice(&[camera_raw])
        );

        let shader = renderer.load_shader(include_str!("particles.wgsl"));

        let pipeline_builder = RenderPipelineBuilder::new(shader.clone())
            .vertex_layout(Vertex::buffer_layout())
            .vertex_layout(Particle::buffer_layout())
            .material_layout(&[uniform(0)])
            .depth_format(TextureFormat::Depth24Plus)
            .depth_writes_enabled(false)
            .additive_alpha_blending();
        let pipeline = renderer.create_render_pipeline(pipeline_builder);

        let material_builder = MaterialBuilder::new(pipeline)
            .uniform(0,camera_buffer);
        let material = renderer.create_material(material_builder);

        let simulation = Simulation::new(&mut renderer,shader);

        (Self{
            core: renderer,
            camera_buffer,
            camera_controller,
            camera,
            camera_raw,
            material,
            mesh,
        },simulation)
    }

    pub fn update_camera(&mut self, event: &WindowEvent){
        self.camera_controller.process_events(event);
        self.camera_controller.update_camera(&mut self.camera, self.core.window_aspect());
        self.camera_raw.update_view_proj(&self.camera);
        self.core.write_buffer(self.camera_buffer,self.camera_raw);
    }
}