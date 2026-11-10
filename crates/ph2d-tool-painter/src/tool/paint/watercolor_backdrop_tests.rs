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

/// **B16.9: o pen-down repõe o memo do substrato só na caixa que o composite preencheu** — e o que
/// fica é o estado da tela inteira reposta (tudo `NaN`). O sentinela plantado FORA da caixa mede o
/// alcance do reset: o de antes (a tela inteira) apagava-o.
#[test]
fn o_pen_down_so_repoe_o_substrato_que_o_traco_tocou() {
    use ph2d_painter_brush::{Falloff, TextureKind, TextureMapping};
    let cp = |pos: [f32; 2], phase| CanvasPointer {
        pos,
        pressure: 1.0,
        tilt: [0.0, 0.0],
        phase,
    };
    let size = 256usize;
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; size * size * 4], size as u32, size as u32);
    t.paint.brush = BrushSpec {
        radius_px: 8.0,
        hardness: 1.0,
        falloff: Falloff::Constant,
        color: [0.2, 0.4, 0.6],
        space_attenuation: false,
        watercolor: true,
        fill: 0.5,
        depth: 1.5,
        granulation: 0.9,
        ..Default::default()
    };
    t.paint.brush.paper.kind = TextureKind::Voronoi;
    t.paint.brush.paper.mapping = TextureMapping::Tiled;
    t.paint.brush.paper_depth = 1.0;
    t.paint.brush_by_mode.fill(t.paint.brush);
    t.on_canvas_pointer(cp([32.0, 32.0], PointerPhase::Down));
    t.on_canvas_pointer(cp([60.0, 40.0], PointerPhase::Move));
    t.on_canvas_pointer(cp([60.0, 40.0], PointerPhase::Up));
    let memo = t.paint.wet_substrate.iter().filter(|v| !v.is_nan()).count();
    assert!(memo > 0, "a fixtura tem de memorizar o papel");
    let (c, _) = t.paint.wet_substrate_caixa.expect("a caixa do memo");
    let longe = 5 * size + 250;
    assert!((c.x + c.w) as usize <= 250, "o sentinela tem de ficar fora da caixa");
    t.paint.wet_substrate[longe] = 0.5;
    t.on_canvas_pointer(cp([200.0, 200.0], PointerPhase::Down));
    let sub = &t.paint.wet_substrate;
    let agora = t.paint.wet_substrate_caixa.map(|(r, _)| r);
    let dentro = |i: usize| {
        let (x, y) = ((i % size) as u32, (i / size) as u32);
        agora.is_some_and(|r| x >= r.x && x < r.x + r.w && y >= r.y && y < r.y + r.h)
    };
    assert!(agora.is_none_or(|r| r.x >= c.x + c.w || r.y >= c.y + c.h));
    let velho = sub.iter().enumerate().filter(|&(i, v)| i != longe && !v.is_nan() && !dentro(i));
    assert_eq!(velho.count(), 0, "o pen-down deixou memo de outro traço");
    assert_eq!(sub[longe].to_bits(), 0.5f32.to_bits(), "o reset varreu fora da caixa");
}

/// Sonda B16.9 (`--ignored`): o pen-down da aguada a 3840×2160 com 1 e 3 camadas — o reset do memo
/// só na caixa (A) contra a caixa = a tela inteira (B, o custo de antes), no MESMO processo, ABBA,
/// o mínimo de 6 de cada.
#[test]
#[ignore]
fn mede_o_pen_down_da_aguada_4k() {
    let cp = |pos: [f32; 2], phase| CanvasPointer {
        pos,
        pressure: 1.0,
        tilt: [0.0, 0.0],
        phase,
    };
    let (fw, fh) = (3840u32, 2160u32);
    for camadas in [1usize, 3] {
        let mut t = PainterTool::default();
        t.set_source(vec![255u8; (fw * fh * 4) as usize], fw, fh);
        for k in 1..camadas {
            let _ = t.add_raster_layer(format!("L{k}"));
        }
        t.paint.brush = BrushSpec {
            radius_px: 30.0,
            watercolor: true,
            ..Default::default()
        };
        t.paint.brush_by_mode.fill(t.paint.brush);
        let mut best = [f64::MAX; 2];
        for i in 0..12u32 {
            let v = [0usize, 1, 1, 0][(i % 4) as usize];
            let (x, y) = (300.0 + 280.0 * (i % 12) as f32, 400.0 + 100.0 * (i % 3) as f32);
            if v == 1 {
                t.paint.wet_substrate_caixa = Some((Region { x: 0, y: 0, w: fw, h: fh }, fw));
            }
            let t0 = std::time::Instant::now();
            t.on_canvas_pointer(cp([x, y], PointerPhase::Down));
            let ms = t0.elapsed().as_secs_f64() * 1e3;
            t.on_canvas_pointer(cp([x + 30.0, y], PointerPhase::Move));
            t.on_canvas_pointer(cp([x + 30.0, y], PointerPhase::Up));
            best[v] = best[v].min(ms);
        }
        println!(
            "[B16.9] 3840x2160 camadas {camadas}: pen-down caixa {:.2} ms | tela inteira {:.2} ms",
            best[0], best[1]
        );
    }
}

/// **B16.A2: o chão uniforme da aguada (falta o `wet_backdrop`) nasce uma vez por (cor, tamanho)**,
/// com o byte do laço de antes (copiado à letra) — antes eram 33 MB novos a cada quadro a 4K.
#[test]
fn o_chao_uniforme_da_aguada_nasce_uma_vez() {
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; 64 * 32 * 4], 64, 32);
    let n = 64 * 32;
    let de_antes = |t: &PainterTool, n: usize| {
        let p = t.cor_do_chao();
        let mut g = vec![0u8; n * 4];
        for px in g.as_chunks_mut::<4>().0 {
            px.copy_from_slice(&[p[0], p[1], p[2], 255]);
        }
        g
    };
    let a = t.chao_uniforme(n);
    let b = t.chao_uniforme(n);
    assert!(Arc::ptr_eq(&a, &b), "o 2.º quadro alocou outro chão");
    assert_eq!(*a, de_antes(&t, n), "o chão guardado não é o byte do de antes");
    let c = t.chao_uniforme(n / 2);
    assert_eq!(c.len(), n / 2 * 4, "tamanho novo ⇒ chão novo");
    assert_eq!(*c, de_antes(&t, n / 2));
}
