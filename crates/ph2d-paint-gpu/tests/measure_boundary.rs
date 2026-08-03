//! **A FRONTEIRA — a Fase 1 do ADR-0146 aplicada a esta wave.**
//!
//! O compute é o lado fácil: 17,3 M visitas de texel é trabalho de microssegundos num device. O que
//! decide se a wave existe é o TRANSPORTE — subir a região e lê-la de volta —, e é por isso que ele
//! se mede ANTES de qualquer fiação.
//!
//! ⚠️ **A comparação é contra o número do PRODUTO, não contra um banco de teste**: o log de smoke
//! mediu o carimbo em **18,30 ms por entrega** (doc 33 §1), pela porta do artista, na máquina dele.
//! Um `18,30` de sonda própria seria outra fixture e a §5.40 já pagou por essa confusão.
//!
//! ⚠️ **E o número que sai daqui é PESSIMISTA sob carga**, o que é a direção segura: o `run()` gasta
//! CPU em criar buffers e copiar para staging, então uma máquina disputada o infla. Se ele ganhar
//! com a máquina cheia, ganha.

use ph2d_paint_gpu::{GpuDab, Region, StampPass};

/// A escala do ARTISTA, derivada do log de smoke: 177 dabs por lote, pegada de 312 px (raio ~155),
/// 17,3 M visitas — o que dá uma bbox de figura de ~1,7 M px depois da redundância de ~10×.
const DABS: usize = 177;
const RADIUS: f32 = 155.0;
const RW: u32 = 1440;
const RH: u32 = 1216;
const LUT_N: usize = 4096;
/// O que o produto MEDIU no carimbo da CPU, por entrega. A régua desta sonda.
const CPU_MS: f64 = 18.30;

fn dabs() -> Vec<GpuDab> {
    (0..DABS)
        .map(|i| {
            #[allow(clippy::cast_precision_loss)]
            let t = (i as f32) / (DABS as f32) * std::f32::consts::TAU;
            GpuDab {
                center: [
                    f32::from(u16::try_from(RW / 2).unwrap_or(700)) + t.cos() * 500.0,
                    f32::from(u16::try_from(RH / 2).unwrap_or(600)) + t.sin() * 430.0,
                ],
                radius: RADIUS,
                coverage: 0.7,
                color: [0.2, 0.4, 0.8],
                _pad0: 0.0,
                m0: [1.0, 0.0],
                m1: [0.0, 1.0],
                _pad1: [0.0; 4],
            }
        })
        .collect()
}

#[test]
#[ignore = "mede a FRONTEIRA: precisa de adapter; rode com `-- --ignored --nocapture`"]
fn the_boundary_is_what_decides_this_wave() {
    let Some(gpu) = ph2d_gpu::GpuContext::new(ph2d_gpu::GpuContext::default_instance(), None).ok()
    else {
        eprintln!("sem adapter: skip");
        return;
    };
    let pass = StampPass::new(&gpu);
    let n = (RW as usize) * (RH as usize);
    let base = vec![128u8; n * 4];
    let lut: Vec<f32> = (0..LUT_N)
        .map(|i| {
            #[allow(clippy::cast_precision_loss)]
            let t = i as f32 / (LUT_N - 1) as f32;
            (1.0 - t * t).max(0.0)
        })
        .collect();
    let list = dabs();

    #[allow(clippy::cast_precision_loss)]
    let mpx = n as f64 / 1.0e6;
    eprintln!(
        "[fronteira] regiao {RW}x{RH} = {mpx:.2} M px ({:.1} MB por sentido) | {DABS} dabs r={RADIUS}",
        mpx * 4.0
    );

    // A 1a corrida paga alocação e caminho frio — descartada, como toda sonda desta linha.
    let _ = pass.run(&base, region(), &lut, &list, false);
    let mut samples = Vec::new();
    for _ in 0..7 {
        let t0 = std::time::Instant::now();
        let out = pass.run(&base, region(), &lut, &list, false);
        samples.push(t0.elapsed().as_secs_f64() * 1000.0);
        assert_eq!(out.len(), base.len());
        // O CONTROLE: se o passe não pintou nada, o relógio mede um no-op e o veredito é vazio.
        assert!(
            out.iter().any(|&b| b != 128),
            "o passe não pintou — a fixture não contém o fenômeno"
        );
    }
    samples.sort_by(f64::total_cmp);
    let med = samples[samples.len() / 2];
    eprintln!(
        "[fronteira] upload + compute + readback: {med:.2} ms (min {:.2}, max {:.2})",
        samples[0],
        samples[samples.len() - 1]
    );
    eprintln!(
        "[fronteira] o carimbo da CPU no PRODUTO: {CPU_MS:.2} ms  =>  {:.2}x",
        CPU_MS / med
    );
    eprintln!(
        "[fronteira] veredito: {}",
        if med * 2.0 < CPU_MS {
            "a v1 sem mudanca de posse JA' GANHA -- S3 (fiacao) e a proxima fatia"
        } else {
            "o transporte domina -- S4 (residencia no device) e' obrigatoria, com a porta bring_home"
        }
    );
}

fn region() -> Region {
    Region {
        x: 0,
        y: 0,
        w: RW,
        h: RH,
    }
}
