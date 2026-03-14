use crate::{renderer, wad};
use std::sync::Arc;
use winit::{
    application::ApplicationHandler, dpi::PhysicalSize, event::WindowEvent,
    event_loop::ActiveEventLoop, window::Window,
};

pub struct State {
    render_state: renderer::RenderState,
    wad: wad::Wad,
}

impl State {
    pub async fn new(window: Arc<Window>, wad_name: &String) -> Self {
        let wad = wad::Wad::load(&wad_name);

        Self {
            render_state: renderer::RenderState::new(window, wad.get_palette()).await,
            wad,
        }
    }

    pub fn render(&mut self) {
        self.render_state.render();
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.render_state.resize(width, height);
    }
}

pub struct App {
    wad_name: String,
    state: Option<State>,
}

impl App {
    pub fn new(wad_name: &str) -> Self {
        Self {
            wad_name: wad_name.to_string(),
            state: None,
        }
    }
}

impl ApplicationHandler<State> for App {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        let window_attributes = Window::default_attributes();

        let window = Arc::new(
            event_loop
                .create_window(window_attributes)
                .expect("Failed to create window"),
        );

        let _ = window.request_inner_size(PhysicalSize::new(960, 600));

        self.state = Some(pollster::block_on(State::new(window, &self.wad_name)));
    }

    #[allow(unused_mut)]
    fn user_event(&mut self, _event_loop: &ActiveEventLoop, mut event: State) {
        self.state = Some(event);
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        let state = match &mut self.state {
            Some(state) => state,
            None => return,
        };

        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(size) => {
                state.resize(size.width, size.height);
            },
            WindowEvent::RedrawRequested => {
                state.render();
            },
            _ => {},
        }
    }
}
