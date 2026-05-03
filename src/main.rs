use winit::error::EventLoopError;
use winit::event_loop::EventLoop;
use crate::app::App;

mod app;
pub mod camera;
mod render;
mod simulation;

fn main() -> Result<(),EventLoopError> {
    let event_loop = EventLoop::new()?;
    let mut app = App::new();
    event_loop.run_app(&mut app)
}