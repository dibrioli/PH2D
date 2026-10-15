use winit::application::ApplicationHandler;
use winit::event::{WindowEvent, ElementState};
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::window::{Window, WindowId};
struct App { w: Option<Window>, t0: std::time::Instant }
impl ApplicationHandler for App {
    fn resumed(&mut self, el: &ActiveEventLoop) {
        if self.w.is_none() {
            let w = el.create_window(Window::default_attributes().with_title("imeprobe")).unwrap();
            w.set_ime_allowed(std::env::var("NOIME").is_err());
            w.set_ime_cursor_area(winit::dpi::PhysicalPosition::new(10, 10), winit::dpi::PhysicalSize::new(2, 16));
            println!("READY"); 
            self.w = Some(w);
        }
    }
    fn window_event(&mut self, el: &ActiveEventLoop, _id: WindowId, ev: WindowEvent) {
        let ms = self.t0.elapsed().as_millis();
        match ev {
            WindowEvent::KeyboardInput { event, .. } => {
                if event.state == ElementState::Pressed {
                    println!("{ms} KEY logical={:?} text={:?} phys={:?}", event.logical_key, event.text, event.physical_key);
                }
            }
            WindowEvent::Ime(i) => println!("{ms} IME {:?}", i),
            WindowEvent::Focused(f) => println!("{ms} FOCUSED {f}"),
            WindowEvent::CloseRequested => el.exit(),
            WindowEvent::RedrawRequested => {}
            _ => {}
        }
        use std::io::Write; std::io::stdout().flush().ok();
    }
}
fn main() {
    let el = EventLoop::new().unwrap();
    let mut app = App { w: None, t0: std::time::Instant::now() };
    el.run_app(&mut app).unwrap();
}
