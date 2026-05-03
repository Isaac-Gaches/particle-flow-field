use rayon::iter::IntoParallelIterator;
use easy_gpu::assets::*;
use easy_gpu::wgpu::*;
use easy_gpu::Renderer;
use easy_gpu::assets_manager::*;
use rayon::iter::ParallelIterator;
use easy_gpu::frame::Frame;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Particle {
    position: [f32; 3],
    pad1: u32,
    colour: [f32; 3],
    pad2: u32,
    velocity: [f32; 3],
    pad3:u32,
}
impl Particle{
    fn new(position: [f32;3],colour: [f32;3]) -> Self {
        Self{
            position,
            pad1:0,
            colour,
            pad2: 0,
            velocity: [0.,0.,0.],
            pad3: 0,
        }
    }
}
impl GpuInstance for Particle {
    fn buffer_layout() -> BufferLayout {
        BufferLayout::new()
            .stride(size_of::<Self>() as u64)
            .step_mode(VertexStepMode::Instance)
            .attribute(1,0,VertexFormat::Float32x3)
            .attribute(2,16,VertexFormat::Float32x4)
    }
}

pub struct Simulation{
    compute: Handle<ComputePipeline>,
    bind_group: Handle<ComputeBindGroup>,
    pub(crate) particles: Handle<Buffer>,
    pub(crate) num_particles: u32,
}
impl Simulation {
    pub fn new(renderer: &mut Renderer,shader: Handle<ShaderModule>) -> Simulation {
        let compute_builder = ComputePipelineBuilder::new(shader)
            .bind_group_layout(&[storage(0,false)]);

        let compute = renderer.create_compute_pipeline(compute_builder);

        let w = 80.;
        let s = 3.0;
        let o = -(w*s)/2.;

        let particles = (0..(w*w*w) as u32).into_par_iter().map(|i|{
            let x = i as f32 %w;
            let y = (i as f32/w).floor() %w;
            let z = (i as f32/(w*w)).floor();
            Particle::new(
                [o + x*s,o +y*s,o +z*s],
                [x/w,y/w/2.,1.2-z/w]
            )
        }).collect::<Vec<_>>();

        let particle_buffer = renderer.create_buffer_with_contents(
            BufferUsages::VERTEX | BufferUsages::STORAGE,
            bytemuck::cast_slice(particles.as_slice())
        );

        let bind_group_builder = ComputeBindGroupBuilder::new(compute.clone())
            .storage(0,particle_buffer);
        let bind_group = renderer.create_compute_bind_group(bind_group_builder);

        Self{
            compute,
            bind_group,
            particles: particle_buffer,
            num_particles: particles.len() as u32,
        }
    }

    pub fn compute(&self, frame: &mut Frame){
        frame.compute(
            self.bind_group,
            self.compute,
            ((self.num_particles as f32/256.).ceil() as u32 ,1,1)
        );
    }
}