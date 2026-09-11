//! **O device pinta o que a CPU pinta.** É este gate que torna a wave segura.
//!
//! ⚠️ **O oráculo é a função REAL do produto** (`stamp_dab_textured_masked`), não uma reescrita —
//! por isso o `ph2d-painter-brush` está em `[dev-dependencies]` e o `src/` não o toca (o padrão que
//! a `ph2d-flip-render` já usa pelo mesmo motivo: *machete-safe*, e a crate segue sem conseguir ter
//! opinião sobre a lei do falloff).
//!
//! ⚠️ **"Bit-a-bit" NÃO é a política deste projeto** — o compositor declara que runtime não é
//! bit-idêntico entre backends (contração FMA). O template é o do `ImpastoLightPass`: pior delta
//! **E** quantos bytes diferem, porque *quão longe* e *quantos* são perguntas diferentes — tirar o
//! `+0.5` do `quantise` moveu 2375 bytes por UM nível e passava sob um limite de magnitude.

use ph2d_paint_gpu::{GpuDab, Region, StampPass};
use ph2d_painter_brush::{BrushSpec, stamp_dab_textured_masked};

const W: u32 = 192;
const H: u32 = 160;
/// Nós do perfil. Quanto mais fino, menor o degrau que a amostragem *nearest* do device introduz
/// contra a avaliação EXATA que a CPU faz — e o gate MEDE o que sobra em vez de o supor.
const LUT_N: usize = 65_536;

fn spec() -> BrushSpec {
    BrushSpec {
        radius_px: 26.0,
        color: [0.15, 0.55, 0.9],
        ..BrushSpec::default()
    }
}

/// Um arco com sobreposição de verdade — é a sobreposição que expõe a ida-e-volta por `u8` entre
/// dabs, que É a lei (o device a reproduz de propósito; guardar `f32` no registrador seria *mais
/// preciso* e por isso divergente).
fn dabs(n: usize) -> Vec<([f32; 2], f32)> {
    (0..n)
        .map(|i| {
            #[allow(clippy::cast_precision_loss)]
            let t = (i as f32) / (n as f32) * std::f32::consts::TAU;
            (
                [96.0 + t.cos() * 44.0, 80.0 + t.sin() * 44.0],
                0.55_f32 + 0.3 * t.sin(),
            )
        })
        .collect()
}

fn cpu(spec: &BrushSpec, list: &[([f32; 2], f32)], alpha_lock: bool) -> Vec<u8> {
    let mut buf = base();
    for (c, cov) in list {
        let _ = stamp_dab_textured_masked(
            &mut buf,
            W,
            H,
            *c,
            spec,
            *cov,
            alpha_lock,
            None,
            None,
            None,
            None,
            [1.0, 0.0],
        );
    }
    buf
}

fn base() -> Vec<u8> {
    // Fundo VARIADO: um fundo chato faria qualquer blend concordar, e o gate seria verde por vácuo.
    (0..(W as usize) * (H as usize))
        .flat_map(|i| {
            let x = u8::try_from(i % 251).unwrap_or(0);
            let y = u8::try_from((i / 97) % 199).unwrap_or(0);
            [x, y, 255 - x, 200_u8.saturating_add(y % 55)]
        })
        .collect()
}

/// A TABELA que sobe ao device — cheia pela função que o produto usa. O device nunca a re-deriva.
fn lut(spec: &BrushSpec) -> Vec<f32> {
    #[allow(clippy::cast_precision_loss)]
    (0..LUT_N)
        .map(|i| spec.falloff_weight(i as f32 / (LUT_N - 1) as f32))
        .collect()
}

fn gpu_dabs(spec: &BrushSpec, list: &[([f32; 2], f32)]) -> Vec<GpuDab> {
    // O footprint como mapa linear, avaliado nos vetores da BASE — a mesma porta que o `stamp_band`
    // usa, aqui só amostrada duas vezes. `the_footprint_is_a_linear_map` prova a premissa.
    let fp = spec.dab_footprint([1.0, 0.0]);
    let e0 = fp.apply([1.0, 0.0]);
    let e1 = fp.apply([0.0, 1.0]);
    list.iter()
        .map(|(c, cov)| GpuDab {
            center: *c,
            radius: spec.radius_px,
            coverage: *cov,
            color: spec.color,
            _pad0: 0.0,
            m0: [e0[0], e1[0]],
            m1: [e0[1], e1[1]],
            _pad1: [0.0; 4],
        })
        .collect()
}

fn context() -> Option<ph2d_gpu::GpuContext> {
    ph2d_gpu::GpuContext::new(ph2d_gpu::GpuContext::default_instance(), None).ok()
}

/// ⚠️ **A premissa que o `GpuDab` faz sobre o footprint vira TESTE aqui.** O device recebe duas
/// linhas de matriz porque um deform de dab é LINEAR; se não fosse, o carimbo sairia deformado de
/// um jeito que só uma screenshot mostraria. Este gate não precisa de GPU.
#[test]
fn the_footprint_is_a_linear_map() {
    for angle in [0.0_f32, 0.7, 2.9] {
        let mut s = spec();
        s.dab_angle_deg = 37;
        s.dab_flatten = 0.6;
        let fp = s.dab_footprint([angle.cos(), angle.sin()]);
        let (e0, e1) = (fp.apply([1.0, 0.0]), fp.apply([0.0, 1.0]));
        for v in [[0.3_f32, -0.8], [-1.0, 0.25], [0.0, 0.0], [0.61, 0.61]] {
            let direct = fp.apply(v);
            let via = [e0[0] * v[0] + e1[0] * v[1], e0[1] * v[0] + e1[1] * v[1]];
            let err = (direct[0] - via[0]).abs().max((direct[1] - via[1]).abs());
            assert!(
                err < 1e-6,
                "o footprint NÃO é linear em {v:?}: {direct:?} vs {via:?} (erro {err:e})"
            );
        }
    }
}

fn worst_and_count(a: &[u8], b: &[u8]) -> (u8, usize) {
    let mut worst = 0u8;
    let mut n = 0usize;
    for (x, y) in a.iter().zip(b) {
        let d = x.abs_diff(*y);
        if d > 0 {
            n += 1;
            worst = worst.max(d);
        }
    }
    (worst, n)
}

#[test]
#[ignore = "precisa de adapter — rode com `-- --ignored` na máquina com GPU"]
fn the_device_paints_what_the_cpu_paints() {
    let Some(gpu) = context() else {
        eprintln!("sem adapter: skip");
        return;
    };
    let pass = StampPass::new(&gpu);
    let s = spec();
    let region = Region {
        x: 0,
        y: 0,
        w: W,
        h: H,
    };
    for n in [1usize, 12, 90] {
        for alpha_lock in [false, true] {
            let list = dabs(n);
            let want = cpu(&s, &list, alpha_lock);
            let got = pass
                .run(&base(), region, &lut(&s), &gpu_dabs(&s, &list), alpha_lock)
                .expect("o passe recusou");
            let painted = want.iter().zip(&base()).filter(|(a, b)| a != b).count();
            assert!(
                painted > 0,
                "a fixture não pintou nada (n={n}) — ela não contém o fenômeno"
            );
            let (worst, diff) = worst_and_count(&want, &got);
            assert!(
                worst <= 1,
                "n={n} alpha_lock={alpha_lock}: pior delta {worst} (>1 nível)"
            );
            // ⚠️ **A barra fica ENTRE dois números medidos, não num palpite.** Correto na RTX com os
            // 65 536 nós que shipam: **1 (n=1), 9-12 (n=12), 15-18 (n=90) de 122.880 = 0,015%**. Com
            // a lei do `u8` entre dabs removida — a mutação que eu quase shipei — sobe a **7744 =
            // 6,3%**. 0,25% deixa ~17× de folga sobre o correto (para outro adaptador contrair um
            // FMA diferente) e ainda pega a mutação por 25×.
            //
            // ⚠️ **O que sobra é a ESCADA da tabela, e isso está medido, não suposto:** varrendo os
            // nós com todo o resto igual, 1 024 REPROVA (2 níveis) · 16 384 → 71 · 65 536 → 18 ·
            // 262 144 → 8. A hipótese de que a divergência vinha do blend foi refutada — trocar
            // `stamp_rgba` pelo `blend_over` do produto deixou os seis números **idênticos**.
            //
            // ⚠️ E é por isso que `n=1` NÃO basta como fixture: sem sobreposição a ida-e-volta por
            // `u8` não tem o que fazer, e a mesma mutação mede 14 bytes — dentro de qualquer barra.
            // A fixture tem de conter o fenômeno, e o fenômeno é a sobreposição.
            assert!(
                diff * 400 <= want.len(),
                "n={n} alpha_lock={alpha_lock}: {diff} de {} bytes diferem (>0,25%)",
                want.len()
            );
            eprintln!(
                "n={n:>3} alpha_lock={alpha_lock:<5} pior delta {worst}, {diff} bytes diferem"
            );
        }
    }
}
