//! B29.1 (missão 4K, decisão 3): a precisão da tinta com Flow baixo — o tecto do traço (`m_buf`) e a tela são u8,
//! e um passo `< ½` LSB arredonda para zero: o texel ESTANCA longe do alvo `255·Π(1 − flow)`. A sonda imprime o
//! carimbo real; o modelo por texel (o MESMO depósito de `bands/deposito.rs`, preso ao carimbo ao bit) prova as
//! três variantes: `u8 + u8` (hoje) · `u16 + u8` (só o tecto) · `u16 + recomposição` (a cura).

use super::*;
use crate::blend::BrushBlend;
use crate::falloff::Falloff;

/// `n` dabs no mesmo centro, preto sobre branco, Strength 1, Accumulate off: (tela no centro, alvo).
fn centro(flow: f32, n: u32) -> (u8, f32) {
    let (w, h) = (16u32, 16u32);
    let spec = BrushSpec {
        radius_px: 5.0,
        color: [0.0, 0.0, 0.0],
        blend: BrushBlend::Mix,
        falloff: Falloff::Constant,
        hardness: 1.0,
        strength: 1.0,
        flow,
        ..Default::default()
    };
    let mut dst = vec![255u8; (w * h * 4) as usize];
    let mut mask = vec![0u8; (w * h) as usize];
    for _ in 0..n {
        let _ = stamp_dab_textured_masked(
            &mut dst,
            w,
            h,
            [8.0, 8.0],
            &spec,
            spec.strength,
            false,
            None,
            None,
            None,
            Some(&mut mask),
            None,
            [1.0, 0.0],
            None,
        );
    }
    let alvo = 255.0 * (1.0 - flow).powi(n as i32);
    (dst[(8 * w as usize + 8) * 4], alvo)
}

#[test]
#[ignore = "sonda B29.1: imprime onde a tinta estanca com Flow baixo"]
fn sonda_b29_1_a_tinta_estanca_com_flow_baixo() {
    for (flow, ns) in [(0.02_f32, [50u32, 150, 400]), (0.002, [500, 2000, 4000])] {
        for n in ns {
            let (tela, alvo) = centro(flow, n);
            println!(
                "[b29.1] flow {flow:<5} {n:>5} dabs: tela {tela:>3} · alvo {alvo:>6.1} · erro {:>6.1} níveis",
                f32::from(tela) - alvo
            );
        }
    }
}

/// A cura proposta, a ligar em `bands/deposito.rs` (B29.1, parcial): o passo do tecto arredondado a 1/65535 — em `u8`
/// um passo `< ½` LSB some.
fn tecto_u16(m: u16, add: f32) -> u16 {
    ((f32::from(m) / 65535.0 + add).clamp(0.0, 1.0) * 65535.0 + 0.5) as u16
}

/// …e a tela recomposta do pré-traço (Mix, cor constante, sem pigmento): `pré·(1 − m) + cor·m`, UM arredondamento
/// por texel em vez de um por dab. Só o tecto `u16` não cura: a tela `u8` estanca no mesmo nível (`prev·a < ½`).
fn recompoe(pre: u8, cor: f32, m: u16) -> u8 {
    let m = f32::from(m) / 65535.0;
    crate::dab::encode(f32::from(pre) / 255.0 * (1.0 - m) + cor * m)
}

/// O depósito de UM texel sob `n` dabs (w = g = tecto = 1, Mix, preto sobre branco): `largo` = tecto `u16`,
/// `recompor` = a tela recomposta do pré-traço ([`recompoe`]) em vez da source-over por dab.
fn modelo(flow: f32, n: u32, largo: bool, recompor: bool) -> u8 {
    use crate::stroke_cover::{Area, cover_add};
    let (pre, cor) = (255u8, 0.0f32);
    let (mut m8, mut m16, mut tela) = (0u8, 0u16, pre);
    for _ in 0..n {
        let m = if largo {
            f32::from(m16) / 65535.0
        } else {
            f32::from(m8) / 255.0
        };
        let Some(add) = cover_add(m, 1.0, 1.0, 1.0, flow, Area::Nao) else {
            continue;
        };
        if largo {
            m16 = tecto_u16(m16, add);
        } else {
            m8 = ((m + add) * 255.0 + 0.5) as u8;
        }
        tela = if recompor {
            recompoe(pre, cor, m16)
        } else {
            let a = add / (1.0 - m).max(1e-4);
            let prev = f32::from(tela) / 255.0;
            crate::dab::encode(prev + (cor - prev) * a)
        };
    }
    tela
}

const CASOS: [(f32, [u32; 3]); 2] = [(0.02, [50, 150, 400]), (0.002, [500, 2000, 4000])];

/// ⭐ B29.1: o modelo `u8 + u8` É o carimbo que shipou (ao bit, nos seis casos) — e é por isso que as outras duas
/// variantes valem como medição da cura.
#[test]
fn o_modelo_do_texel_e_o_carimbo_ao_bit() {
    for (flow, ns) in CASOS {
        for n in ns {
            assert_eq!(
                modelo(flow, n, false, false),
                centro(flow, n).0,
                "flow {flow} · {n} dabs"
            );
        }
    }
}

/// ⭐ B29.1 (red → green): com o tecto `u16` e a tela recomposta do pré-traço o Flow baixo chega ao alvo a ≤ 1
/// nível; o `u8` de hoje estanca (25 contra 0,1 · 250 contra 4,7) e só o tecto `u16` estanca no MESMO nível.
#[test]
fn com_o_tecto_u16_e_a_recomposicao_a_tinta_chega_ao_alvo() {
    for (flow, ns) in CASOS {
        for n in ns {
            let alvo = 255.0 * (1.0 - flow).powi(n as i32);
            let cura = f32::from(modelo(flow, n, true, true));
            assert!(
                (cura - alvo).abs() <= 1.0,
                "flow {flow} · {n}: {cura} contra {alvo:.1}"
            );
        }
    }
    // O vermelho de hoje e a recusa do «só u16», medidos no fim de cada série.
    for (flow, alvo_max) in [(0.02_f32, 400u32), (0.002, 2000)] {
        let alvo = 255.0 * (1.0 - flow).powi(alvo_max as i32);
        let hoje = f32::from(modelo(flow, alvo_max, false, false));
        let so_u16 = f32::from(modelo(flow, alvo_max, true, false));
        assert!(
            hoje - alvo > 10.0 && so_u16 - alvo > 10.0,
            "flow {flow}: hoje {hoje} · só u16 {so_u16}"
        );
    }
}

/// Sonda de custo (ignored, `--profile smoke`): o passo por texel numa tela 4K, `u8 + source-over` × `u16 +
/// recomposição`, intercalado, o MÍNIMO de 9 blocos. RAM: tecto 8,3 → 16,6 MB e o pré-traço RGBA 33,2 MB a 4K.
#[test]
#[ignore = "sonda B29.1: custo por texel a 4K"]
fn sonda_b29_1_custo_4k() {
    use crate::stroke_cover::{Area, cover_add};
    use std::hint::black_box;
    let n = 3840usize * 2160;
    let mut tela = vec![200u8; n * 4];
    let pre = vec![255u8; n * 4];
    let mut m8 = vec![10u8; n];
    let mut m16 = vec![2570u16; n];
    let flow = black_box(0.02f32);
    let mut melhor = [f64::MAX; 2];
    for bloco in 0..9 {
        for v in [bloco % 2, 1 - bloco % 2] {
            let t0 = std::time::Instant::now();
            for i in 0..n {
                if v == 0 {
                    let m = f32::from(m8[i]) / 255.0;
                    let Some(add) = cover_add(m, 1.0, 1.0, 1.0, flow, Area::Nao) else {
                        continue;
                    };
                    m8[i] = ((m + add) * 255.0 + 0.5) as u8;
                    let a = add / (1.0 - m).max(1e-4);
                    for c in 0..4 {
                        let p = f32::from(tela[i * 4 + c]) / 255.0;
                        tela[i * 4 + c] = crate::dab::encode(p + (0.0 - p) * a);
                    }
                } else {
                    let m = f32::from(m16[i]) / 65535.0;
                    let Some(add) = cover_add(m, 1.0, 1.0, 1.0, flow, Area::Nao) else {
                        continue;
                    };
                    m16[i] = tecto_u16(m16[i], add);
                    for c in 0..4 {
                        tela[i * 4 + c] = recompoe(pre[i * 4 + c], 0.0, m16[i]);
                    }
                }
            }
            melhor[v] = melhor[v].min(t0.elapsed().as_secs_f64() * 1e3);
            m8.fill(10);
            m16.fill(2570);
        }
    }
    black_box(&tela);
    println!(
        "[b29.1] 4K, um passo por texel (mínimo de 9): u8+source-over {:.1} ms · u16+recomposição {:.1} ms",
        melhor[0], melhor[1]
    );
}
