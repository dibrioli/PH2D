//! **O PERCURSO NO DEVICE É O PERCURSO DA CPU** — a paridade do port de compute
//! ([doc 12](../../../docs/Flip/12_novo_motor_pesquisa.md) §14).
//!
//! ⚠️ **"Bit-a-bit" NÃO é a política deste projeto** — o compositor do Painter já declara que
//! runtime não é bit-idêntico entre backends (FMA), e o template do repo é *literais exatos por
//! gate CPU-only + épsilon documentado por gate `#[ignore]` contra o kernel canônico*. Aqui o
//! kernel canônico é o [`ph2d_flip_render::walk_pixel`], e o épsilon está MEDIDO abaixo.
//!
//! ⚠️ **A saída é UMA textura, e é a que o PRODUTO usa** (`rgba16float`, o `hdr` premult do
//! `FlipCompose`) — uma 2ª saída em `f32` só para o gate mediria um caminho enquanto o outro shipa.
//! O preço é honesto e está na barra abaixo: a precisão do readback deixou de ser a do kernel e
//! passou a ser a do FORMATO (meia precisão).
//!
//! ```text
//! cargo test -p ph2d-flip-render --release --test walk_gpu_parity -- --ignored --nocapture
//! ```

use ph2d_core::Vec2;
use ph2d_flip::{FlipDrawing, FlipStroke, Point, Rgba};
use ph2d_flip_render::{
    CameraRaw, DEFAULT_TILE, ScreenSpace, WalkPass, bin_segments, pack_drawing, walk_pixel,
};

fn device() -> Option<(wgpu::Device, wgpu::Queue)> {
    let instance = wgpu::Instance::default();
    let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::HighPerformance,
        compatible_surface: None,
        force_fallback_adapter: false,
    }))
    .ok()?;
    let (device, queue) = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
        label: Some("ph2d-flip walk parity"),
        required_features: wgpu::Features::empty(),
        required_limits: wgpu::Limits::default(),
        experimental_features: wgpu::ExperimentalFeatures::default(),
        memory_hints: wgpu::MemoryHints::Performance,
        trace: wgpu::Trace::Off,
    }))
    .expect("request_device");
    Some((device, queue))
}

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

fn stroke(pts: &[(f32, f32)], width: f32, hardness: f32, col: [f32; 3], op: f32) -> FlipStroke {
    let mut s = FlipStroke::new();
    for &(x, y) in pts {
        s.push_point(Point {
            pos: Vec2::new(x, y),
            width,
            opacity: op,
            color: Rgba::new(col[0], col[1], col[2], 1.0),
        });
    }
    s.hardness = hardness;
    s
}

/// A cena de paridade: **dez perguntas diferentes num desenho só** — a estrela que cruza a si
/// mesma (a topologia da foto do Enio), um traço duro, um macio, um de opacidade < 1 (a regra do
/// GP), um afilado (largura variando ⇒ o raio interpolado da quadratura) e **dois pontilhados**
/// (Dots e Squares), que exercitam a OUTRA lista de dabs: o `arc_len`, o dono meio-aberto de cada
/// conta e a conta da PONTA — mais um **sub-pixel** que afina atravessando 1 px, o unico regime
/// onde o fade nao e' a identidade, e um de **tampa CHATA** que se enrola de volta sobre o proprio
/// comeco (o unico jeito de provar que a truncagem e' por-SEGMENTO e nao um semi-plano global), e um
/// **X de UM traço** com `self_overlap` + opacidade 0,5 — sem a opacidade a flag e' invisivel (tinta
/// opaca ja satura), e sem o cruzamento a particao de passagens nao roda.
fn scene() -> FlipDrawing {
    let mut d = FlipDrawing::new();
    let (cx, cy, outer) = (60.0_f32, 60.0, 44.0);
    let mut corners: Vec<(f32, f32)> = (0..5)
        .map(|k| {
            let a = -std::f32::consts::FRAC_PI_2 + (k as f32) * 4.0 * std::f32::consts::PI / 5.0;
            (cx + outer * a.cos(), cy + outer * a.sin())
        })
        .collect();
    corners.push(corners[0]);
    let mut estrela = Vec::new();
    for w in corners.windows(2) {
        let (a, b) = (w[0], w[1]);
        let n = 12;
        for k in 0..=n {
            let t = k as f32 / n as f32;
            estrela.push((a.0 + (b.0 - a.0) * t, a.1 + (b.1 - a.1) * t));
        }
    }
    d.strokes
        .push(stroke(&estrela, 14.0, 0.45, [0.1, 0.1, 0.1], 1.0));
    let reta: Vec<(f32, f32)> = (0..=20).map(|k| (10.0 + k as f32 * 5.0, 120.0)).collect();
    d.strokes
        .push(stroke(&reta, 16.0, 1.0, [0.9, 0.2, 0.1], 1.0));
    let macio: Vec<(f32, f32)> = (0..=20).map(|k| (10.0 + k as f32 * 5.0, 145.0)).collect();
    d.strokes
        .push(stroke(&macio, 16.0, 0.1, [0.1, 0.5, 0.9], 1.0));
    let meio: Vec<(f32, f32)> = (0..=20).map(|k| (10.0 + k as f32 * 5.0, 170.0)).collect();
    d.strokes
        .push(stroke(&meio, 16.0, 0.6, [0.2, 0.8, 0.3], 0.5));
    // Afilado: a largura varia ponto a ponto.
    let mut cone = FlipStroke::new();
    for k in 0..=20 {
        let t = k as f32 / 20.0;
        cone.push_point(Point {
            pos: Vec2::new(10.0 + k as f32 * 5.0, 195.0),
            width: 3.0 + 18.0 * t,
            opacity: 1.0,
            color: Rgba::new(0.6, 0.2, 0.8, 1.0),
        });
    }
    cone.hardness = 0.5;
    d.strokes.push(cone);
    // **O TIP PONTILHADO**, os dois sabores. `dot_spacing` default (2,0 = vão de um diâmetro) na
    // fileira de discos e apertado na de quadrados, para as duas caírem em regimes diferentes de
    // `bead_range`. Dureza < 1 de propósito: é o `f_bead_of` que a borda macia exercita.
    let contas: Vec<(f32, f32)> = (0..=20).map(|k| (10.0 + k as f32 * 5.0, 220.0)).collect();
    let mut dots = stroke(&contas, 12.0, 0.5, [0.9, 0.7, 0.1], 1.0);
    dots.tip = ph2d_flip::StrokeTip::Dots;
    dots.dot_spacing = 2.0;
    d.strokes.push(dots);
    let quad: Vec<(f32, f32)> = (0..=20).map(|k| (10.0 + k as f32 * 5.0, 245.0)).collect();
    let mut squares = stroke(&quad, 12.0, 0.8, [0.1, 0.8, 0.8], 1.0);
    squares.tip = ph2d_flip::StrokeTip::Squares;
    squares.dot_spacing = 1.25;
    d.strokes.push(squares);
    // **O FADE SUB-PIXEL** — e ele exige uma largura ABAIXO de 1 px, senão o atalho do caso comum
    // dispara em todo segmento e a cena inteira fica CEGA à mudança (a fixture tem de conter o
    // fenômeno). O traço afina de 1,6 px a 0,1: ele atravessa a fronteira do atalho no meio, então
    // os DOIS ramos correm no mesmo traço.
    let mut fino = FlipStroke::new();
    for k in 0..=20 {
        let t = k as f32 / 20.0;
        fino.push_point(Point {
            pos: Vec2::new(10.0 + k as f32 * 5.0, 270.0),
            width: 1.6 - 1.5 * t,
            opacity: 1.0,
            color: Rgba::new(0.3, 0.3, 0.9, 1.0),
        });
    }
    fino.hardness = 1.0;
    d.strokes.push(fino);
    // **A TAMPA CHATA**, e a forma importa: um "J" que volta e passa POR CIMA do próprio começo
    // cortado. Um semi-plano global apagaria a tinta da volta; a truncagem por-segmento a deixa.
    let mut chato = stroke(
        &[
            (30.0, 282.0),
            (30.0, 300.0),
            (55.0, 300.0),
            (55.0, 278.0),
            (18.0, 278.0),
        ],
        13.0,
        0.7,
        [0.9, 0.4, 0.6],
        1.0,
    );
    chato.cap = (ph2d_flip::Cap::Flat, ph2d_flip::Cap::Flat);
    d.strokes.push(chato);
    // **O SELF OVERLAP** — um X de um traço só, DENSO (o regime real, pós-`resample_smooth`) e a
    // opacidade em 0,5: é o único regime onde a flag muda um pixel.
    let mut xx = Vec::new();
    for w in [
        (20.0_f32, 320.0),
        (60.0, 360.0),
        (60.0, 320.0),
        (20.0, 360.0),
    ]
    .windows(2)
    {
        for k in 0..20 {
            let t = k as f32 / 20.0;
            xx.push((
                w[0].0 + (w[1].0 - w[0].0) * t,
                w[0].1 + (w[1].1 - w[0].1) * t,
            ));
        }
    }
    xx.push((20.0, 360.0));
    let mut cruzado = stroke(&xx, 9.0, 1.0, [0.2, 0.6, 0.3], 0.5);
    cruzado.self_overlap = true;
    d.strokes.push(cruzado);
    d
}

/// 🔴 **O GATE.** O mesmo desenho, os dois percursos.
#[test]
#[ignore = "requires a GPU adapter; run with --ignored"]
fn the_device_walk_is_the_cpu_walk() {
    let Some((device, queue)) = device() else {
        println!("sem adapter -- skip");
        return;
    };
    let (w, h) = (128_u32, 376);
    let sc = ScreenSpace::from_camera(&camera(w, h));
    let data = pack_drawing(&scene());
    let bins = bin_segments(&data, &sc, DEFAULT_TILE);
    let gpu = WalkPass::new(&device).run(&device, &queue, &data, &sc, &bins);

    let (mut pior, mut onde, mut n_alto, mut som, mut n_tinta) = (0.0_f32, (0, 0), 0u32, 0.0, 0u32);
    for y in 0..h {
        for x in 0..w {
            let i = (y * w + x) as usize;
            let cpu = walk_pixel(&bins, &data, &sc, [x as f32 + 0.5, y as f32 + 0.5]);
            for c in 0..4 {
                let d = (gpu[i][c] - cpu[c]).abs();
                if d > pior {
                    pior = d;
                    onde = (x, y);
                }
                if d > 1.0 / 255.0 {
                    n_alto += 1;
                }
            }
            if cpu[3] > 0.0 {
                som += (gpu[i][3] - cpu[3]).abs();
                n_tinta += 1;
            }
        }
    }
    println!(
        "\n  pior |Δ| {:.3e} em {onde:?}   |   {n_alto} canais acima de 1/255   |   \
         erro medio no alfa com tinta {:.3e} ({n_tinta} px)",
        pior,
        som / n_tinta.max(1) as f32
    );
    // **MEDIDO na RTX: pior |Δ| 4,883e-4**, ZERO canais acima de 1/255, erro médio no alfa 1,25e-4.
    //
    // ⚠️ **Esse número é o FORMATO, não o kernel, e a aritmética o nomeia:** `4,883e-4 = 2⁻¹¹` é
    // exatamente o arredondamento de meia precisão em magnitude 1 (`rgba16float` tem 10 bits de
    // mantissa ⇒ ulp `2⁻¹⁰`, erro de arredondamento metade disso). Enquanto a saída era um buffer
    // `f32` o mesmo desenho media **4,05e-6** — 120× ABAIXO do quantum do alvo do produto, ou seja
    // o kernel nunca foi o limite. Ganhamos medir o que shipa e perdemos resolução de medição; o
    // trade é deliberado (uma 2ª saída em `f32` seria um caminho medido e outro shipado).
    //
    // A barra é **DERIVADA**: `1e-3` ≈ 2× o quantum do formato (folga para outra implementação de
    // transcendental) e ainda **3,9× mais apertada** que meio nível de byte, que é a resolução em
    // que qualquer pessoa pode ver a diferença.
    assert!(
        pior <= 1e-3,
        "o percurso do device divergiu do da CPU: pior |Δ| {pior:.3e} em {onde:?} \
         ({n_alto} canais acima de 1/255)"
    );
}

/// **SONDA** — o número que decide: quanto custa um frame de 1080p NO DEVICE.
///
/// ⚠️ **Mede o que o PRODUTO faz:** `prepare` uma vez + `record` N vezes num submit só, **sem
/// readback nenhum**. O readback de 33 MB que o `run` faz é do harness — incluí-lo mediria o
/// PCIe, não o percurso.
#[test]
#[ignore = "sonda de medicao; roda com --ignored"]
fn measure_what_a_frame_costs_on_the_device() {
    let Some((device, queue)) = device() else {
        println!("sem adapter -- skip");
        return;
    };
    let (w, h) = (1920_u32, 1080);
    let sc = ScreenSpace::from_camera(&camera(w, h));
    println!("\n=== O CUSTO DE UM FRAME NO DEVICE (1920x1080) ===");
    println!("  tracos   segs   bin(CPU) ms [min-max]   walk(GPU) ms   ns/px   vs CPU serial");
    for tile in [8_u32, 16, 32, 64] {
        println!("  --- ladrilho {tile} px ---");
        for n in [1_usize, 10, 50, 200] {
            let mut d = FlipDrawing::new();
            let mut seed = 0x85EB_CA6B_u32;
            let mut rnd = || {
                seed ^= seed << 13;
                seed ^= seed >> 17;
                seed ^= seed << 5;
                (seed >> 8) as f32 / (1 << 24) as f32
            };
            for _ in 0..n {
                let (x0, y0) = (rnd() * w as f32 * 0.2, rnd() * h as f32);
                let (x1, y1) = (w as f32 * 0.8 + rnd() * w as f32 * 0.2, rnd() * h as f32);
                let bow = (rnd() - 0.5) * h as f32 * 0.6;
                let pts: Vec<(f32, f32)> = (0..40)
                    .map(|k| {
                        let t = k as f32 / 39.0;
                        (
                            x0 + (x1 - x0) * t,
                            y0 + (y1 - y0) * t + bow * (std::f32::consts::PI * t).sin(),
                        )
                    })
                    .collect();
                d.strokes
                    .push(stroke(&pts, 12.0, 0.5, [0.1, 0.1, 0.1], 1.0));
            }
            let data = pack_drawing(&d);
            // ⚠️ **O lado da CPU é MEDIANA de 9, o 1º descartado** — e não é higiene: com UMA amostra
            // não-aquecida ele media 1,33 / 2,30 / 4,00 ms em três corridas seguidas do mesmo binário
            // (o `bin_segments` aloca, e o alocador tem memória entre chamadas). *Um número que não
            // reproduz não é achado, é ruído com casas decimais* — e foi sobre uma dessas amostras que
            // o §14 concluiu "o binner é 45% do frame".
            let mut amostras = Vec::new();
            let mut bins = bin_segments(&data, &sc, tile);
            for _ in 0..9 {
                let t0 = std::time::Instant::now();
                bins = bin_segments(&data, &sc, tile);
                amostras.push(t0.elapsed().as_secs_f64() * 1e3);
            }
            amostras.sort_by(|a, b| a.partial_cmp(b).unwrap());
            // ⚠️ **MÍNIMO, não mediana** — e a escolha do redutor é parte da fixture: aqui toda amostra
            // faz o trabalho IDÊNTICO (mesmos dados, mesma chamada), então o mínimo é o que a máquina de
            // fato consegue e o resto é carga alheia. (Onde uma amostra é estruturalmente diferente — o
            // 1º move de um traço, que não compõe — o mínimo é o redutor ERRADO; ver doc 28 §5.12.)
            let bin_ms = amostras[0];
            let (bin_lo, bin_hi) = (amostras[0], amostras[amostras.len() - 1]);
            let pass = WalkPass::new(&device);
            let tex = device.create_texture(&wgpu::TextureDescriptor {
                label: Some("walk perf target"),
                size: wgpu::Extent3d {
                    width: w,
                    height: h,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: ph2d_flip_render::WALK_TARGET_FORMAT,
                usage: wgpu::TextureUsages::STORAGE_BINDING,
                view_formats: &[],
            });
            let view = tex.create_view(&wgpu::TextureViewDescriptor::default());
            // Piso de fills VAZIO — a sonda mede o percurso, não a composição do fill.
            let floor = device.create_texture(&wgpu::TextureDescriptor {
                label: Some("walk perf empty fills"),
                size: wgpu::Extent3d {
                    width: w,
                    height: h,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: ph2d_flip_render::WALK_TARGET_FORMAT,
                usage: wgpu::TextureUsages::TEXTURE_BINDING,
                view_formats: &[],
            });
            let floor_view = floor.create_view(&wgpu::TextureViewDescriptor::default());
            let job = pass
                .prepare(&device, &data, &sc, &bins, &view, &floor_view)
                .expect("job");
            // Aquece (compilação de pipeline, primeira submissão) e depois mede REPS dispatches.
            {
                let mut enc =
                    device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
                pass.record(&mut enc, &job);
                queue.submit(Some(enc.finish()));
                let _ = device.poll(wgpu::PollType::wait_indefinitely());
            }
            const REPS: u32 = 16;
            let t1 = std::time::Instant::now();
            let mut enc =
                device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
            for _ in 0..REPS {
                pass.record(&mut enc, &job);
            }
            queue.submit(Some(enc.finish()));
            let _ = device.poll(wgpu::PollType::wait_indefinitely());
            let ms = t1.elapsed().as_secs_f64() * 1e3 / f64::from(REPS);
            let px = f64::from(w) * f64::from(h);
            // Os números seriais de CPU do doc 12 §10.2, para a razão ficar ao lado.
            let cpu = match n {
                1 => 18.4,
                10 => 93.4,
                50 => 412.8,
                _ => 1593.7,
            };
            println!(
                "  {n:6}   {:4}   {bin_ms:6.2} [{bin_lo:.2}-{bin_hi:.2}]   {ms:12.2}   {:5.1}   {:8.0}x",
                data.points.len() - data.strokes.len(),
                ms * 1e6 / px,
                cpu / ms
            );
        }
    }
}

/// **SONDA** — os dois percursos sobre uma fileira de CONTAS, lado a lado em ASCII. É ela que
/// nomeia *qual* conta discorda em vez de deixar o gate apontar um pixel solto.
#[test]
#[ignore = "sonda; roda com --ignored --nocapture"]
fn measure_the_bead_row_in_both_walks() {
    let Some((device, queue)) = device() else {
        println!("sem adapter -- skip");
        return;
    };
    for (nome, tip, spacing, hard) in [
        ("DOTS   ", ph2d_flip::StrokeTip::Dots, 2.0_f32, 0.5_f32),
        ("SQUARES", ph2d_flip::StrokeTip::Squares, 1.25, 0.8),
    ] {
        let (w, h) = (128_u32, 32_u32);
        let sc = ScreenSpace::from_camera(&camera(w, h));
        let pts: Vec<(f32, f32)> = (0..=20).map(|k| (10.0 + k as f32 * 5.0, 16.0)).collect();
        let mut st = stroke(&pts, 12.0, hard, [0.1, 0.8, 0.8], 1.0);
        st.tip = tip;
        st.dot_spacing = spacing;
        let mut d = FlipDrawing::new();
        d.strokes.push(st);
        let data = pack_drawing(&d);
        let bins = bin_segments(&data, &sc, DEFAULT_TILE);
        let gpu = WalkPass::new(&device).run(&device, &queue, &data, &sc, &bins);
        let glyph = |a: f32| match (a * 9.0).round() as i32 {
            0 => '.',
            n if n >= 9 => '#',
            n => char::from_digit(n as u32, 10).unwrap(),
        };
        println!("\n  {nome}  spacing {spacing}");
        // Os RUNS acesos por linha: `start..=end` de cada bloco com alfa > 0,5. Um mapa em ASCII
        // engana o olho na contagem de colunas; um par de números não.
        let runs = |row: &[f32]| {
            let mut v: Vec<(u32, u32)> = Vec::new();
            let mut open: Option<u32> = None;
            for (x, a) in row.iter().enumerate() {
                match (*a > 0.5, open) {
                    (true, None) => open = Some(x as u32),
                    (false, Some(s)) => {
                        v.push((s, x as u32 - 1));
                        open = None;
                    }
                    _ => {}
                }
            }
            if let Some(s) = open {
                v.push((s, row.len() as u32 - 1));
            }
            v
        };
        for y in [11_u32, 13, 16, 19, 21] {
            let c: Vec<f32> = (0..w)
                .map(|x| walk_pixel(&bins, &data, &sc, [x as f32 + 0.5, y as f32 + 0.5])[3])
                .collect();
            let g: Vec<f32> = (0..w).map(|x| gpu[(y * w + x) as usize][3]).collect();
            let (rc, rg) = (runs(&c), runs(&g));
            println!("    y={y:2}  cpu {rc:?}");
            if rc != rg {
                println!("          gpu {rg:?}   <-- DIVERGE");
            }
        }
        let _ = glyph;
    }
}
