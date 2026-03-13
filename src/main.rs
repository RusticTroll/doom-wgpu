use doom_wgpu::app::App;
use winit::event_loop::EventLoop;

fn main() {
    env_logger::init();

    let event_loop = EventLoop::with_user_event().build().unwrap();
    let mut app = App::new();
    event_loop.run_app(&mut app).unwrap();
}
