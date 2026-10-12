//! W18 — o Servo 0.7 EMBUTIDO, fora do ecrã: carrega o embed do YouTube (num iframe servido de uma origem
//! local, como uma página real), e a cada quadro novo lê os píxeis (`read_to_image`) — o caminho que o quadro
//! do PH2D usaria para pôr o tocador dentro do canvas. Mede quadros/s lidos, quantos são DISTINTOS (o vídeo
//! anda?), o custo de ler um quadro, CPU e RSS. Uso: servo_spike <id> <segundos> <pasta_saida>
use std::cell::Cell;
use std::io::{Read, Write};
use std::rc::Rc;
use std::sync::mpsc::{Sender, channel};
use std::time::{Duration, Instant};

use euclid::Point2D;
use servo::{
    EventLoopWaker, RenderingContext, ServoBuilder, SoftwareRenderingContext, WebView, WebViewBuilder,
    WebViewDelegate,
};

#[derive(Clone)]
struct Waker(Sender<()>);
impl EventLoopWaker for Waker {
    fn clone_box(&self) -> Box<dyn EventLoopWaker> {
        Box::new(self.clone())
    }
    fn wake(&self) {
        let _ = self.0.send(());
    }
}

struct Delegate {
    frame_ready: Cell<bool>,
}
impl WebViewDelegate for Delegate {
    fn notify_new_frame_ready(&self, _: WebView) {
        self.frame_ready.set(true);
    }
}

fn rusage_s() -> f64 {
    let mut ru: libc_rusage = unsafe { std::mem::zeroed() };
    unsafe { getrusage(0, &mut ru) };
    ru.u.0 as f64 + ru.u.1 as f64 / 1e6 + ru.s.0 as f64 + ru.s.1 as f64 / 1e6
}
#[repr(C)]
struct libc_rusage {
    u: (i64, i64),
    s: (i64, i64),
    rest: [i64; 14],
}
unsafe extern "C" {
    fn getrusage(who: i32, usage: *mut libc_rusage) -> i32;
}
fn rss_mb() -> f64 {
    let s = std::fs::read_to_string("/proc/self/statm").unwrap_or_default();
    s.split_whitespace().nth(1).and_then(|v| v.parse::<f64>().ok()).unwrap_or(0.0) * 4096.0 / 1048576.0
}

fn main() {
    let a: Vec<String> = std::env::args().collect();
    let (vid, secs, out) = (a[1].clone(), a[2].parse::<f64>().unwrap(), a[3].clone());
    std::fs::create_dir_all(&out).unwrap();
    rustls::crypto::aws_lc_rs::default_provider().install_default().unwrap();

    // A página: um iframe com o embed oficial, servida de http://localhost:<p>/ (o YouTube pede um referrer).
    let html = format!(
        "<!doctype html><body style=\"margin:0;background:#000\"><iframe width=560 height=315 frameborder=0 \
         allow=\"autoplay; encrypted-media\" referrerpolicy=\"strict-origin-when-cross-origin\" \
         src=\"https://www.youtube-nocookie.com/embed/{vid}?autoplay=1&mute=1&playsinline=1\"></iframe></body>"
    );
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    std::thread::spawn(move || {
        for s in listener.incoming().flatten() {
            let mut s = s;
            let mut buf = [0u8; 4096];
            let _ = s.read(&mut buf);
            let _ = write!(s, "HTTP/1.1 200 OK\r\ncontent-type: text/html\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{}", html.len(), html);
        }
    });

    let (tx, rx) = channel();
    let t_build = Instant::now();
    let ctx = Rc::new(SoftwareRenderingContext::new(winit_size(560, 315)).expect("contexto de software"));
    let _ = ctx.make_current();
    let servo = ServoBuilder::default().event_loop_waker(Box::new(Waker(tx))).build();
    let delegate = Rc::new(Delegate { frame_ready: Cell::new(false) });
    let url = url::Url::parse(&format!("http://localhost:{port}/")).unwrap();
    let webview = WebViewBuilder::new(&servo, ctx.clone()).url(url).delegate(delegate.clone()).build();
    let arranque_ms = t_build.elapsed().as_secs_f64() * 1e3;

    let t0 = Instant::now();
    let (mut lidos, mut distintos, mut fotos) = (0u32, 0u32, 0u32);
    let mut ultimo: u64 = 0;
    let mut ler_ms_min = f64::MAX;
    let mut ler_ms_soma = 0.0;
    let (mut cpu5, mut t5) = (None, None);
    while t0.elapsed().as_secs_f64() < secs {
        let _ = rx.recv_timeout(Duration::from_millis(8));
        servo.spin_event_loop();
        if delegate.frame_ready.replace(false) {
            webview.paint();
            let tl = Instant::now();
            let img = ctx.read_to_image(euclid::Box2D::new(Point2D::new(0, 0), Point2D::new(560, 315)));
            let dt = tl.elapsed().as_secs_f64() * 1e3;
            if let Some(img) = img {
                lidos += 1;
                ler_ms_min = ler_ms_min.min(dt);
                ler_ms_soma += dt;
                let h = img.as_raw().iter().step_by(97).fold(0xcbf29ce484222325u64, |h, &b| (h ^ b as u64).wrapping_mul(0x100000001b3));
                let t = t0.elapsed().as_secs_f64();
                if h != ultimo && t > 5.0 {
                    distintos += 1;
                    if distintos % 60 == 1 && fotos < 4 {
                        img.save(format!("{out}/servo_embutido_{fotos}.png")).unwrap();
                        fotos += 1;
                    }
                }
                ultimo = h;
            }
        }
        if t5.is_none() && t0.elapsed().as_secs_f64() > 5.0 {
            t5 = Some(Instant::now());
            cpu5 = Some(rusage_s());
        }
    }
    let dur = t5.map(|t| t.elapsed().as_secs_f64()).unwrap_or(1.0);
    let cpu = rusage_s() - cpu5.unwrap_or(0.0);
    let json = format!(
        "{{\"o_que\":\"Servo 0.7 embutido, SoftwareRenderingContext + read_to_image por quadro\",\"video\":\"{vid}\",\"segundos\":{secs},\
         \"arranque_ms\":{arranque_ms:.0},\"quadros_lidos\":{lidos},\"distintos_depois_5s\":{distintos},\"fps_distintos\":{:.2},\
         \"ler_quadro_ms_min\":{ler_ms_min:.3},\"ler_quadro_ms_media\":{:.3},\"cpu_nucleos_depois_5s\":{:.2},\"rss_mb\":{:.0}}}",
        distintos as f64 / dur,
        ler_ms_soma / lidos.max(1) as f64,
        cpu / dur,
        rss_mb()
    );
    std::fs::write(format!("{out}/servo_embutido.json"), &json).unwrap();
    println!("{json}");
    std::process::exit(0);
}

fn winit_size(w: u32, h: u32) -> dpi::PhysicalSize<u32> {
    dpi::PhysicalSize::new(w, h)
}
