use crate::renderer;
use std::sync::Arc;
use winit::{
    application::ApplicationHandler, dpi::PhysicalSize, event::WindowEvent,
    event_loop::ActiveEventLoop, window::Window,
};

pub struct State {
    render_state: renderer::RenderState,
}

impl State {
    pub async fn new(window: Arc<Window>) -> Self {
        let palette_bytes = include_bytes!("doom.pal");
        let mut alpha_bytes = Vec::with_capacity((palette_bytes.len() / 3) * 4);
        for chunk in palette_bytes.chunks(3) {
            alpha_bytes.extend_from_slice(chunk);
            alpha_bytes.push(255);
        }

        Self {
            render_state: renderer::RenderState::new(
                window,
                vec![*bytemuck::from_bytes(&alpha_bytes[..])],
            )
            .await,
        }
    }

    pub fn render(&mut self) {
        self.render_state.render();
    }
}

pub struct App {
    state: Option<State>,
}

impl App {
    pub fn new() -> Self {
        Self { state: None }
    }
}

impl ApplicationHandler<State> for App {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        let mut window_attributes = Window::default_attributes();

        let window = Arc::new(
            event_loop
                .create_window(window_attributes)
                .expect("Failed to create window"),
        );

        let _ = window.request_inner_size(PhysicalSize::new(320, 200));

        self.state = Some(pollster::block_on(State::new(window)));
    }

    #[allow(unused_mut)]
    fn user_event(&mut self, _event_loop: &ActiveEventLoop, mut event: State) {
        self.state = Some(event);
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        window_id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        let state = match &mut self.state {
            Some(state_i) => state_i,
            None => return,
        };

        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::RedrawRequested => {
                state.render();
            },
            _ => {},
        }
    }
}
