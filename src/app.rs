use std::sync::Arc;
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::ActiveEventLoop;
use winit::window::{Window, WindowId};
use crate::render::Render;
use crate::simulation::Simulation;

pub struct App{
    window: Option<Arc<Window>>,
    renderer: Option<Render>,
    simulation: Option<Simulation>,
}

impl App{
    pub fn new()->Self{
        Self{
            window: None,
            renderer: None,
            simulation: None,
        }
    }
}

impl ApplicationHandler for App{
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window = Arc::new(
            event_loop
                .create_window(Window::default_attributes())
                .unwrap(),
        );

        let (renderer,simulation) = Render::new(window.clone());

        self.window = Some(window);
        self.renderer = Some(renderer);
        self.simulation = Some(simulation);
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _window_id: WindowId, event: WindowEvent)  {
        let renderer = self.renderer.as_mut().unwrap();
        let simulation = self.simulation.as_mut().unwrap();
        let _window = self.window.as_mut().unwrap();

        renderer.update_camera(&event);

        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }

            WindowEvent::Resized(size) => {
                renderer.core.resize_surface(size);
            }

            WindowEvent::RedrawRequested => {
                let frame = renderer.core.begin_frame();

                simulation.compute(frame);
                
                frame.draw(
                    simulation.particles,
                    renderer.material,
                    renderer.mesh,
                    0..simulation.num_particles
                );

                renderer.core.render();
            }

            _ => {}
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        if let Some(window) = &self.window {
            window.request_redraw();
        }
    }
}

#[repr(C)]
#[derive(Copy,Clone,bytemuck::Pod, bytemuck::Zeroable)]
struct Camera{
    num:f32,
}

