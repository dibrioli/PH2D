//! Medida (ignorada): o custo da composição na CPU — `cargo test --profile smoke … mede_compose -- --ignored --nocapture`.

use ph2d_pixel_core::blend::BlendMode;
use ph2d_pixel_core::layers;
use ph2d_pixel_core::{NewSprite, Op, Patch, Purpose, Rect, Sprite, apply, compose_window};

#[test]
#[ignore = "medida, não lei"]
fn mede_compose() {
    for (lado, n) in [(64u32, 5usize), (256, 8), (512, 20), (1024, 20), (2048, 8)] {
        let mut s = Sprite::new(NewSprite { width: lado, height: lado, ..NewSprite::default() }).expect("s");
        let mut top = s.layers()[0].id;
        for i in 0..n {
            let (op, id) = layers::new_layer(&mut s, top).expect("l");
            apply(&mut s, op).expect("l");
            let r = Rect::new(0, 0, lado, lado);
            let pixels = (0..r.area()).flat_map(|k| [(k * 7) as u8, (k * 13) as u8, i as u8 * 9, 200]).collect();
            let frame = s.frames()[0].id;
            apply(&mut s, Op::Patch(Patch { layer: id, frame, bbox: Some(r), rect: r, pixels })).expect("p");
            let m = BlendMode::ALL[i % 19];
            if let Some(op) = layers::edit(&s, id, |l| l.blend = m) { apply(&mut s, op).expect("m"); }
            top = id;
        }
        let f = s.frames()[0].id;
        let mut best = f64::MAX;
        for _ in 0..5 {
            let t = std::time::Instant::now();
            let v = compose_window(&s, f, None, s.bounds(), Purpose::View);
            std::hint::black_box(v);
            best = best.min(t.elapsed().as_secs_f64() * 1e3);
        }
        let mut pincel = f64::MAX;
        for _ in 0..20 {
            let t = std::time::Instant::now();
            std::hint::black_box(compose_window(&s, f, None, Rect::new(10, 10, 16, 16), Purpose::View));
            pincel = pincel.min(t.elapsed().as_secs_f64() * 1e3);
        }
        let fatias = std::thread::available_parallelism().map_or(1, |n| n.get() / 2).max(1) as u32;
        let mut faixas = f64::MAX;
        for _ in 0..5 {
            let t = std::time::Instant::now();
            std::thread::scope(|sc| {
                let alto = lado.div_ceil(fatias);
                for k in 0..fatias {
                    let y = k * alto;
                    if y >= lado { break; }
                    let r = Rect::new(0, y as i32, lado, alto.min(lado - y));
                    let s = &s;
                    sc.spawn(move || std::hint::black_box(compose_window(s, f, None, r, Purpose::View)));
                }
            });
            faixas = faixas.min(t.elapsed().as_secs_f64() * 1e3);
        }
        println!("MEDE {lado}x{lado} {n} camadas: cheio {best:.2} ms · {fatias} faixas {faixas:.2} ms · janela 16x16 {pincel:.4} ms");
    }
}
