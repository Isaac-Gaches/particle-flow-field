use std::sync::Arc;
use winit::event::WindowEvent;
use winit::window::Window;
use easy_gpu::assets::*;
use easy_gpu::assets_manager::*;
use easy_gpu::wgpu::{TextureFormat, VertexFormat, VertexStepMode};
use easy_gpu::Renderer;
use crate::camera::{Camera, CameraController, CameraRaw};

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
    pub material: Handle<Material>,
    pub mesh: Handle<Mesh>,

    pub compute_pipeline: Handle<ComputePipeline>,
    pub bind_group: Handle<ComputeBindGroup>,
    pub positions: Handle<Buffer>,
    pub colours: Handle<Buffer>,
    pub num_particles: u32,
}

impl Render{
    pub fn new(window: Arc<Window>)->Self{
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
        let camera_controller = CameraController::new(2.0,0.05);
        let mut camera_raw = CameraRaw::new();
        camera_raw.update_view_proj(&camera);

        let camera_buffer = renderer.create_buffer_with_contents(
            BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            bytemuck::cast_slice(&[camera_raw])
        );

        let shader = renderer.load_shader(include_str!("particles.wgsl"));

        let pipeline = RenderPipelineBuilder::new(shader.clone())
            .vertex_layout(Vertex::buffer_layout())
            .vertex_layout(
                BufferLayout::new()
                    .step_mode(VertexStepMode::Instance)
                    .stride(size_of::<[f32;4]>() as u64)
                    .attribute(1,0,VertexFormat::Float32x3)
            )
            .vertex_layout(
                BufferLayout::new()
                    .step_mode(VertexStepMode::Instance)
                    .stride(size_of::<[f32;4]>() as u64)
                    .attribute(2,0,VertexFormat::Float32x3)
            )
            .material_layout(&[uniform(0)])
            .depth_format(TextureFormat::Depth24Plus)
            .depth_writes_enabled(false)
            .additive_alpha_blending()
            .build(&mut renderer);

        let material = MaterialBuilder::new(pipeline)
            .uniform(0,camera_buffer)
            .build(&mut renderer);

        let w = 203.;
        let s = 3.0;
        let o = -(w*s)/2.;

        let size = w*w*w;

        let mut positions:Vec<[f32;4]> = Vec::with_capacity(size as usize);
        let mut velocities:Vec<[f32;4]> = Vec::with_capacity(size as usize);
        let mut colours:Vec<[f32;4]> = Vec::with_capacity(size as usize);

        for i in 0..size as u32{
            let x = i as f32 %w;
            let y = (i as f32/w).floor() %w;
            let z = (i as f32/(w*w)).floor();

            positions.push([o + x*s,o +y*s,o +z*s,0.]);
            velocities.push([0.,0.,0.,0.]);
            colours.push([x/w,y/w/2.,1.2-z/w,1.0]);
        };

        let positions = renderer.create_buffer_with_contents(
            BufferUsages::VERTEX | BufferUsages::STORAGE,
            bytemuck::cast_slice(positions.as_slice())
        );
        let velocities = renderer.create_buffer_with_contents(
            BufferUsages::STORAGE,
            bytemuck::cast_slice(velocities.as_slice())
        );
        let colours = renderer.create_buffer_with_contents(
            BufferUsages::VERTEX | BufferUsages::STORAGE,
            bytemuck::cast_slice(colours.as_slice())
        );

        let compute_pipeline = ComputePipelineBuilder::new(shader)
            .bind_group_layout(&[
                storage(0,false),
                storage(1,false),
                storage(2,false),
            ])
            .build(&mut renderer);

        let bind_group = ComputeBindGroupBuilder::new(compute_pipeline.clone())
            .storage(0,positions)
            .storage(1,velocities)
            .storage(2,colours)
            .build(&mut renderer);

        Self{
            core: renderer,
            camera_buffer,
            camera_controller,
            camera,
            camera_raw,
            material,
            mesh,
            compute_pipeline,
            bind_group,
            positions,
            colours,
            num_particles: size as u32,
        }
    }

    pub fn update(&mut self){
        let frame = self.core.begin_frame();

         frame.compute(
             self.bind_group,
             self.compute_pipeline,
             (self.num_particles/256 + 1,1,1),
         );

        frame.draw_manual_batch(
            vec![self.positions,self.colours],
            self.material,
            self.mesh,
            0..self.num_particles
        );

        self.core.render();
    }

    pub fn update_camera(&mut self, event: &WindowEvent){
        self.camera_controller.process_events(event);
        self.camera_controller.update_camera(&mut self.camera, self.core.window_aspect());
        self.camera_raw.update_view_proj(&self.camera);
        self.core.write_buffer(self.camera_buffer,self.camera_raw);
    }
}