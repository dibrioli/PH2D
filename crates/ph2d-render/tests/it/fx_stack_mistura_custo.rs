//! **O PREÇO das leis de mistura e do degradê assado** (Motion, ZERAR família B — o dono: *«sim se
//! a performance for boa, pois aqui podemos ter milhares de objetos»*). Sonda manual:
//!
//! `PH2D_GPU=1 bash scripts/ph2d-run.sh cargo test -p ph2d-render --test it --release
//! sonda_preco_da_mistura -- --ignored --nocapture`
//!
//! Um degrau pontual custa por PIXEL, não por forma: a camada inteira (`Whole Layer`) paga a área
//! da cena, e o `Each Shape` paga a área do ÁTLAS — `n` células de [`CELULA`] px (uma forma de 16 px
//! e a margem). Por isso as colunas são ÁREAS: a camada a 1600×972 e a 4K, e o átlas de 4 096 e de
//! 32 761 formas. Variantes no MESMO processo, blocos intercalados e rodados, o MÍNIMO
//! (`docs/DevOps/MEDIR_VELOCIDADE.md`).

use std::sync::Arc;
use std::time::Instant;

use ph2d_ecs::FxOp;
use ph2d_render::{FxOpGpu, FxStackPass, make_output_texture};

use crate::fx_stack_common::{make_src, try_headless_gpu};

/// O lado de uma célula do átlas — uma forma de 16 px com 4 px de margem de cada lado.
const CELULA: u32 = 24;

/// Uma textura OPACA e colorida (o ramo `src.a > 0` corre em todo pixel; as leis HSL veem cor).
#[expect(clippy::cast_possible_truncation, reason = "bytes de teste")]
fn fonte(gpu: &ph2d_gpu::GpuContext, w: u32, h: u32) -> wgpu::Texture {
    let mut b = vec![255u8; (w * h * 4) as usize];
    for y in 0..h {
        for x in 0..w {
            let o = ((y * w + x) * 4) as usize;
            b[o] = (x * 7) as u8;
            b[o + 1] = (y * 5) as u8;
            b[o + 2] = ((x + y) * 3) as u8;
        }
    }
    make_src(gpu, w, h, &b)
}

fn degrau(kind: u8, blend: u8) -> FxOpGpu {
    let mut op = FxOp::new(kind);
    op.blend = blend;
    FxOpGpu::resolve(&op, [1.0, 0.0, 0.0, 1.0])
}

#[expect(clippy::cast_precision_loss, reason = "4096 amostras")]
fn variantes() -> Vec<(String, Vec<FxOpGpu>)> {
    let mut v: Vec<(String, Vec<FxOpGpu>)> = vec![("pilha vazia".into(), vec![])];
    for b in 0..FxOp::BLEND_KINDS {
        v.push((
            format!("Color Overlay · lei {b:>2}"),
            vec![degrau(FxOp::COLOR_OVERLAY, b)],
        ));
    }
    let mut oito = FxOp::new(FxOp::GRADIENT_MAP);
    oito.stop_count = 8;
    for i in 0..8 {
        oito.stops[i] = [i as f32 / 7.0, 0.5, 1.0 - i as f32 / 7.0, 1.0];
        oito.stop_pos[i] = i as f32 / 7.0;
    }
    let n = FxOp::RAMP_LUT_TEXELS;
    let lut: Arc<[[f32; 4]]> = (0..n)
        .map(|i| [i as f32 / n as f32, 0.3, 0.6, 1.0])
        .collect();
    let com_lut = |blend: u8| FxOpGpu {
        ramp_lut: Some(Arc::clone(&lut)),
        ..degrau(FxOp::GRADIENT_MAP, blend)
    };
    v.push((
        "Gradient Map · 2 stops".into(),
        vec![degrau(FxOp::GRADIENT_MAP, 0)],
    ));
    v.push((
        "Gradient Map · 8 stops".into(),
        vec![FxOpGpu::resolve(&oito, [1.0, 0.0, 0.0, 1.0])],
    ));
    v.push((format!("Gradient Map · tabela {n}"), vec![com_lut(0)]));
    v.push((
        format!("Gradient Map · tabela {n} · Hue"),
        vec![com_lut(16)],
    ));
    v
}

#[test]
#[ignore = "sonda: manual — --release --nocapture, com placa"]
fn sonda_preco_da_mistura_e_do_degrade() {
    let Some(gpu) = try_headless_gpu() else {
        return;
    };
    let lado = |n: u32| n.isqrt().max(1) * CELULA;
    let areas = [
        ("1600x972".to_string(), 1600u32, 972u32),
        ("4K".into(), 3840, 2160),
        ("atlas 4096".into(), lado(4096), lado(4096)),
        ("atlas 32761".into(), lado(32761), lado(32761)),
    ];
    let vs = variantes();
    let carga = std::fs::read_to_string("/proc/loadavg").unwrap_or_default();
    println!("\ncarga: {}  · células de {CELULA} px", carga.trim());
    let mut tabela = vec![vec![f64::MAX; areas.len()]; vs.len()];
    for (a, (_, w, h)) in areas.iter().enumerate() {
        let mut pass = FxStackPass::new(&gpu);
        let (src, dst) = (fonte(&gpu, *w, *h), make_output_texture(&gpu, *w, *h));
        let corre = |pass: &mut FxStackPass, ops: &[FxOpGpu]| {
            let t = Instant::now();
            pass.run(&gpu, &src, &dst, *w, *h, ops, &[]);
            let _ = gpu.device.poll(wgpu::PollType::wait_indefinitely());
            t.elapsed().as_secs_f64() * 1000.0
        };
        for (_, ops) in &vs {
            let _ = corre(&mut pass, ops); // aquece pipelines e texturas
        }
        for bloco in 0..9 {
            for k in 0..vs.len() {
                let i = (k + bloco) % vs.len();
                let ms = corre(&mut pass, &vs[i].1);
                tabela[i][a] = tabela[i][a].min(ms);
            }
        }
    }
    print!("{:<34}", "variante (ms, mínimo de 9)");
    for (nome, w, h) in &areas {
        print!(" {:>18}", format!("{nome} ({w}x{h})"));
    }
    println!();
    for (i, (nome, _)) in vs.iter().enumerate() {
        print!("{nome:<34}");
        for v in &tabela[i] {
            print!(" {v:>18.3}");
        }
        println!();
    }
}
