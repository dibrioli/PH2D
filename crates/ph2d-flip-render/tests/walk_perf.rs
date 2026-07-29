//! **O CUSTO DO PERCURSO** — o terço do §11.3 do handoff que faltava
//! ([doc 12](../../../docs/Flip/12_novo_motor_pesquisa.md) §10).
//!
//! O §6 do doc 12 mediu o pedágio de **ÁREA** (quantos ladrilhos um traço obriga a tocar). Isto
//! mede a outra metade: **o que custa um pixel**. A integral gasta ~40 amostras por segmento
//! dentro do disco, e ninguém tinha multiplicado isso por uma tela.
//!
//! ⚠️ **A pergunta que DECIDE não é "quantos ms" — é a FORMA.** Este percurso vai virar um
//! dispatch de compute por ladrilho, onde cada pixel é uma thread; o número serial de CPU não é o
//! número do produto. O que o número serial responde, e o produto herda, é:
//!
//! > o custo de um pixel é função da densidade LOCAL, ou da cena INTEIRA?
//!
//! Se dobrar o número de traços numa parte da tela dobra o custo do pixel do outro lado, o binning
//! não está fazendo o trabalho dele e o desenho não escala — e isso é verdade em qualquer
//! dispositivo. Se o custo por pixel fica plano, o percurso é local, que é o requisito.
//!
//! Roda com:
//! ```text
//! cargo test -p ph2d-flip-render --release --test walk_perf -- --ignored --nocapture
//! ```

use ph2d_core::Vec2;
use ph2d_flip::{FlipDrawing, FlipStroke, Point, Rgba};
use ph2d_flip_render::{
    CameraRaw, DEFAULT_TILE, ScreenSpace, TileBins, bin_segments, pack_drawing, walk_pixel,
};
use std::time::Instant;

fn camera(w: u32, h: u32) -> CameraRaw {
    let sx = 2.0 / w as f32;
    let sy = -2.0 / h as f32;
    CameraRaw::new(
        [
            [sx, 0.0, 0.0, 0.0],
            [0.0, sy, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [-1.0, 1.0, 0.0, 1.0],
        ],
        [w as f32, h as f32],
        1.0,
    )
}

/// A cena de GESTO: arcos amplos atravessando a tela — o traço de animação, e o caso em que a
/// bbox de um traço é a tela inteira (a armadilha que o §6 mediu).
fn gesture_scene(n: usize, w: u32, h: u32, width: f32) -> FlipDrawing {
    let mut seed = 0x85EB_CA6Bu32;
    let mut rnd = move || {
        seed ^= seed << 13;
        seed ^= seed >> 17;
        seed ^= seed << 5;
        (seed >> 8) as f32 / (1 << 24) as f32
    };
    let mut d = FlipDrawing::new();
    for _ in 0..n {
        let (x0, y0) = (rnd() * w as f32 * 0.2, rnd() * h as f32);
        let (x1, y1) = (w as f32 * 0.8 + rnd() * w as f32 * 0.2, rnd() * h as f32);
        let bow = (rnd() - 0.5) * h as f32 * 0.6;
        let mut s = FlipStroke::new();
        for k in 0..40 {
            let t = k as f32 / 39.0;
            s.push_point(Point {
                pos: Vec2::new(
                    x0 + (x1 - x0) * t,
                    y0 + (y1 - y0) * t + bow * (std::f32::consts::PI * t).sin(),
                ),
                width,
                opacity: 1.0,
                color: Rgba::new(0.1, 0.1, 0.1, 1.0),
            });
        }
        s.hardness = 0.5;
        d.strokes.push(s);
    }
    d
}

/// Percorre a tela inteira e devolve `(ms, pixels com tinta)`.
fn walk_frame(
    bins: &TileBins,
    data: &ph2d_flip_render::FlipGpuData,
    sc: &ScreenSpace,
) -> (f64, u64) {
    let (w, h) = (sc.viewport[0] as u32, sc.viewport[1] as u32);
    let t0 = Instant::now();
    let mut inked = 0u64;
    for y in 0..h {
        for x in 0..w {
            let p = [x as f32 + 0.5, y as f32 + 0.5];
            if walk_pixel(bins, data, sc, p)[3] > 0.0 {
                inked += 1;
            }
        }
    }
    (t0.elapsed().as_secs_f64() * 1e3, inked)
}

/// **A MEDIÇÃO QUE DECIDE.** Custo de um frame a 1080p conforme a cena cresce.
#[test]
#[ignore = "sonda de medicao; roda com --ignored"]
fn measure_what_a_frame_of_the_new_engine_costs() {
    let (w, h) = (1920u32, 1080);
    let sc = ScreenSpace::from_camera(&camera(w, h));
    let px = f64::from(w) * f64::from(h);
    println!("\n=== O CUSTO DE UM FRAME (percurso SERIAL de CPU, 1920x1080) ===");
    println!("    'ns/px' e' o numero que porta para o compute; 'bin' roda 1x por frame\n");
    println!(
        "  tracos   segs   segs/tile(med|max)   bin ms   walk ms   ns/px   ns/px COM TINTA   \
         tinta%"
    );
    for n in [1usize, 10, 50, 200] {
        let data = pack_drawing(&gesture_scene(n, w, h, 12.0));
        let t0 = Instant::now();
        let bins = bin_segments(&data, &sc, DEFAULT_TILE);
        let bin_ms = t0.elapsed().as_secs_f64() * 1e3;
        let n_tiles = (bins.cols * bins.rows) as usize;
        let ocup: Vec<u32> = (0..n_tiles).map(|i| bins.ranges[i][1]).collect();
        let med = f64::from(ocup.iter().sum::<u32>()) / n_tiles as f64;
        let max = ocup.iter().copied().max().unwrap_or(0);
        let (walk_ms, inked) = walk_frame(&bins, &data, &sc);
        println!(
            "  {n:6}   {:4}   {med:6.2} | {max:4}        {bin_ms:6.2}   {walk_ms:7.1}   \
             {:5.0}   {:14.0}   {:5.1}",
            data.points.len() - data.strokes.len(),
            walk_ms * 1e6 / px,
            walk_ms * 1e6 / (inked.max(1) as f64),
            inked as f64 * 100.0 / px
        );
    }
}

/// **A FORMA.** O custo de um pixel tem de ser função da densidade LOCAL, não do tamanho da cena.
///
/// A sonda acima mede a tela inteira, onde mais traços = mais pixels com tinta, então o total sobe
/// por um motivo legítimo. Esta isola a pergunta: **um pixel de uma REGIÃO VAZIA**, com a cena
/// crescendo do outro lado da tela. Se o binning é local, o número não anda.
#[test]
#[ignore = "sonda de medicao; roda com --ignored"]
fn measure_whether_a_far_pixel_pays_for_the_scene() {
    let (w, h) = (1920u32, 1080);
    let sc = ScreenSpace::from_camera(&camera(w, h));
    println!("\n=== UM PIXEL LONGE PAGA PELA CENA? (canto vazio, cena crescendo) ===\n");
    println!("  tracos   segs   ns/px no CANTO VAZIO      ns/px na FAIXA densa");
    for n in [1usize, 10, 50, 200] {
        // A cena vive na metade de baixo; o canto superior esquerdo fica vazio.
        let mut d = FlipDrawing::new();
        for k in 0..n {
            let mut s = FlipStroke::new();
            let y = h as f32 * 0.6 + (k % 40) as f32 * 4.0;
            for j in 0..40 {
                s.push_point(Point {
                    pos: Vec2::new(40.0 + j as f32 * 46.0, y),
                    width: 12.0,
                    opacity: 1.0,
                    color: Rgba::new(0.1, 0.1, 0.1, 1.0),
                });
            }
            s.hardness = 0.5;
            d.strokes.push(s);
        }
        let data = pack_drawing(&d);
        let bins = bin_segments(&data, &sc, DEFAULT_TILE);
        // ⚠️ O `black_box` vai na ENTRADA, dentro do laço. Com ele só no fim, `walk_pixel` é pura
        // sobre argumentos invariantes e o LLVM a computa UMA vez — a 1ª versão desta sonda mediu
        // **3,3 ns** no canto E na faixa densa, o mesmo número, que é o retrato de nada rodando.
        let bench = |p: [f32; 2]| -> f64 {
            const REP: u32 = 20_000;
            let t0 = Instant::now();
            let mut sink = 0.0f32;
            for _ in 0..REP {
                let q = std::hint::black_box(p);
                sink += std::hint::black_box(walk_pixel(&bins, &data, &sc, q)[3]);
            }
            std::hint::black_box(sink);
            t0.elapsed().as_secs_f64() * 1e9 / f64::from(REP)
        };
        // ⚠️ **Os dois pontos são PROJETADOS pela câmera**, nunca escritos à mão: a câmera é
        // Y-flipada, e a 1ª versão desta sonda cravou `y` de tela — a "faixa densa" caiu em
        // espaço VAZIO e as duas colunas mediram o mesmo nada. O alfa vai impresso ao lado
        // exatamente para uma sonda que não encosta na tinta se denunciar.
        let vazio = sc.point_px([30.5, 30.5]);
        let denso = sc.point_px([960.5, h as f32 * 0.6]);
        let a_vazio = walk_pixel(&bins, &data, &sc, vazio)[3];
        let a_denso = walk_pixel(&bins, &data, &sc, denso)[3];
        println!(
            "  {n:6}   {:4}   {:9.1} (a={a_vazio:.2})   {:11.1} (a={a_denso:.2})",
            data.points.len() - data.strokes.len(),
            bench(vazio),
            bench(denso)
        );
    }
}
