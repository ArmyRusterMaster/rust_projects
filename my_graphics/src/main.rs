use softbuffer::{Context, Surface};
use std::num::NonZeroU32;
use std::sync::Arc;
use winit::event::{Event, WindowEvent};
use winit::event_loop::EventLoop;
use winit::window::WindowBuilder;

fn main() {
    let event_loop = EventLoop::new().unwrap();
    // Обертываем окно в Arc, чтобы удовлетворить требования трейтов
    let window = Arc::new(WindowBuilder::new().build(&event_loop).unwrap());
    
    // Передаем Arc<Window> в контекст и поверхность
    let context = Context::new(window.clone()).unwrap();
    let mut surface = Surface::new(&context, window.clone()).unwrap();

    event_loop.run(move |event, elwt| {
        match event {
            Event::WindowEvent { event: WindowEvent::RedrawRequested, .. } => {
                let size = window.inner_size();
                let width = NonZeroU32::new(size.width).unwrap();
                let height = NonZeroU32::new(size.height).unwrap();

                surface.resize(width, height).unwrap();

                let mut buffer = surface.buffer_mut().unwrap();
                for index in 0..(size.width * size.height) as usize {
                    buffer[index] = 0xFF0000; // Красный
                }
                buffer.present().unwrap();
            }
            Event::WindowEvent { event: WindowEvent::CloseRequested, .. } => elwt.exit(),
            _ => window.request_redraw(),
        }
    }).unwrap();
}

