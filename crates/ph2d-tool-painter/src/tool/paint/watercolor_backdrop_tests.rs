//! O despejo da água ([`PainterTool::grow_wet_soak`]) em LINHAS PARALELAS dá o byte do laço em
//! série de antes (ADR-0173).

use super::*;
use std::sync::Arc;

/// O corpo de ANTES da passagem a linhas paralelas, copiado À LETRA (só `self` → `t`). É o oráculo
/// do gate: a reescrita muda a ARRUMAÇÃO (fatias de linha em vez do laço duplo), e isso prova-se
/// contra o código que ela substituiu, não contra um valor escrito à mão.
fn grow_de_antes(t: &mut PainterTool, dt_s: f32) -> Option<Region> {
    let wet = t.paint.brush.wet_rewet;
    let (center, radius) = t.paint.wet_soak_pos?;
    if wet <= 0.0 || dt_s <= 0.0 || radius <= 0.0 {
        return None;
    }
    let (fw, fh) = t.source_size;
    let (fw, fh) = (fw as usize, fh as usize);
    if t.paint.wet_soak.len() != fw * fh {
        t.paint.wet_soak = vec![0u8; fw * fh];
    }
    let add = (SOAK_RATE_PER_S * dt_s).clamp(1.0, 255.0) as u16;
    let (cx, cy) = (center[0], center[1]);
    let ci =
        (cy.clamp(0.0, (fh - 1) as f32) as usize) * fw + (cx.clamp(0.0, (fw - 1) as f32) as usize);
    let center_soak = f32::from(t.paint.wet_soak[ci]) / 255.0;
    let r = radius * (1.0 + center_soak);
    let inv_r = 1.0 / r;
    let x0 = (cx - r).floor().max(0.0) as usize;
    let y0 = (cy - r).floor().max(0.0) as usize;
    let x1 = ((cx + r).ceil() as i64).clamp(0, fw as i64) as usize;
    let y1 = ((cy + r).ceil() as i64).clamp(0, fh as i64) as usize;
    if x0 >= x1 || y0 >= y1 {
        return None;
    }
    let (sel, prot, alock) = t.wet_splat_gates();
    let gated = sel.is_some() || prot.is_some() || alock.is_some();
    let soak = &mut t.paint.wet_soak;
    let mut grew = false;
    for y in y0..y1 {
        let dy = (y as f32 + 0.5) - cy;
        let base = y * fw;
        for x in x0..x1 {
            let dx = (x as f32 + 0.5) - cx;
            let dn = (dx * dx + dy * dy).sqrt() * inv_r;
            if dn >= 1.0 {
                continue;
            }
            let idx = base + x;
            let keep = if gated {
                super::super::watercolor_accum::splat_keep(
                    sel.as_deref().map(Vec::as_slice),
                    prot.as_deref().map(Vec::as_slice),
                    alock.as_deref().map(Vec::as_slice),
                    idx,
                )
            } else {
                1.0
            };
            if keep <= 0.0 {
                continue;
            }
            let w = (1.0 - dn).min(0.6) / 0.6;
            let cur = soak[idx];
            let next = (u16::from(cur) + (f32::from(add) * w * keep) as u16).min(255) as u8;
            if next != cur {
                soak[idx] = next;
                grew = true;
            }
        }
    }
    if grew {
        t.paint.wet_soak_active = true;
    }
    grew.then(|| Region {
        x: x0 as u32,
        y: y0 as u32,
        w: (x1 - x0) as u32,
        h: (y1 - y0) as u32,
    })
}

const N: u32 = 256;

/// Um disco CORTADO pela borda da tela, sobre uma água já despejada que chega a saturar, com e sem
/// uma selecção em degradê (as portas: `keep` fracionário e `keep = 0`).
fn ferramenta(com_seleccao: bool) -> PainterTool {
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; (N * N * 4) as usize], N, N);
    t.set_paint_media(PaintMedia::Watercolor);
    t.set_brush_wet_rewet(0.5);
    t.paint.wet_soak = (0..N * N)
        .map(|i| {
            let (x, y) = (i % N, i / N);
            // Metade de baixo a caminho da saturação: o `min(255)` é exercido.
            if y > 150 {
                240 + (x % 16) as u8
            } else {
                ((x * 7 + y * 13) % 200) as u8
            }
        })
        .collect();
    t.paint.wet_soak_pos = Some(([230.5, 170.25], 41.0));
    if com_seleccao {
        t.paint.selection_active = true;
        t.paint.selection_mask = Arc::new((0..N * N).map(|i| ((i % N) * 255 / N) as u8).collect());
    }
    t
}

#[test]
fn o_despejo_da_agua_em_paralelo_da_o_byte_da_serie() {
    for com_seleccao in [false, true] {
        for dt in [1.0 / 60.0, 0.05, 3.0] {
            let (mut novo, mut antes) = (ferramenta(com_seleccao), ferramenta(com_seleccao));
            assert_eq!(
                novo.wet_splat_gates().0.is_some(),
                com_seleccao,
                "a porta armou?"
            );
            let antes_da_agua = novo.paint.wet_soak.clone();
            let rn = novo.grow_wet_soak(dt);
            let ra = grow_de_antes(&mut antes, dt);
            assert_eq!(
                rn.map(|r| (r.x, r.y, r.w, r.h)),
                ra.map(|r| (r.x, r.y, r.w, r.h)),
                "região (seleção {com_seleccao}, dt {dt})"
            );
            assert_eq!(novo.paint.wet_soak_active, antes.paint.wet_soak_active);
            assert!(
                novo.paint.wet_soak == antes.paint.wet_soak,
                "o plano da água diverge (seleção {com_seleccao}, dt {dt})"
            );
            // CONTROLO: a fixtura mexe mesmo na água (senão a igualdade é de duas cópias intactas).
            let mudou = novo
                .paint
                .wet_soak
                .iter()
                .zip(&antes_da_agua)
                .filter(|(a, b)| a != b)
                .count();
            assert!(mudou > 500, "o despejo tem de mudar a água: mudou {mudou}");
        }
    }
}
