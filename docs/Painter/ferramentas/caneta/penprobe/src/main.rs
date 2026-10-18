use std::io::Write;
use std::num::NonZeroU32;
use std::rc::Rc;
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::window::{Window, WindowId};

struct App {
    w: Option<Rc<Window>>,
    s: Option<softbuffer::Surface<Rc<Window>, Rc<Window>>>,
    t0: std::time::Instant,
    moves: u32,
}

impl ApplicationHandler for App {
    fn resumed(&mut self, el: &ActiveEventLoop) {
        if self.w.is_none() {
            let w = Rc::new(
                el.create_window(Window::default_attributes().with_title("TESTE DA CANETA"))
                    .unwrap(),
            );
            let c = softbuffer::Context::new(w.clone()).unwrap();
            self.s = Some(softbuffer::Surface::new(&c, w.clone()).unwrap());
            println!("READY");
            self.w = Some(w);
        }
    }
    fn window_event(&mut self, el: &ActiveEventLoop, _id: WindowId, ev: WindowEvent) {
        let ms = self.t0.elapsed().as_millis();
        match &ev {
            WindowEvent::CursorMoved { .. } => {
                self.moves += 1;
                if self.moves % 50 == 1 {
                    println!("{ms} MOVE (n={})", self.moves);
                }
            }
            WindowEvent::MouseInput { .. }
            | WindowEvent::Touch(_)
            | WindowEvent::CursorEntered { .. }
            | WindowEvent::CursorLeft { .. }
            | WindowEvent::MouseWheel { .. } => println!("{ms} {ev:?}"),
            WindowEvent::CloseRequested => el.exit(),
            WindowEvent::RedrawRequested => {
                let w = self.w.as_ref().unwrap();
                let sz = w.inner_size();
                if let (Some(wd), Some(h)) = (NonZeroU32::new(sz.width), NonZeroU32::new(sz.height)) {
                    let s = self.s.as_mut().unwrap();
                    s.resize(wd, h).unwrap();
                    let mut b = s.buffer_mut().unwrap();
                    b.fill(0x00_30_50_80);
                    b.present().unwrap();
                }
            }
            _ => {}
        }
        std::io::stdout().flush().ok();
    }
}

fn main() {
    let el = EventLoop::new().unwrap();
    el.run_app(&mut App { w: None, s: None, t0: std::time::Instant::now(), moves: 0 })
        .unwrap();
}
