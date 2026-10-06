//! A GEOMETRIA do quadro da mancha do Solid na aguada (BUGS #36, item 2 de 2026-10-05): o que
//! muda a cada evento e quanto o composite caminha com UMA caixa contra faixas.

use super::measure_shape_system::{cp, tool};
use crate::tool::PainterTool;
use crate::tool::paint::Region;
use crate::tool::paint::media::PaintMedia;
use ph2d_editor_core::tool::{CanvasPaintTool, PointerPhase};

/// O laço de 1000 px da régua (`diag_a_mancha_de_1000px_na_aguada`) em `lado²`.
fn laco(lado: u32) -> impl Fn(usize) -> [f32; 2] {
    #[allow(clippy::cast_precision_loss)]
    let c = (lado / 2) as f32;
    move |i: usize| {
        #[allow(clippy::cast_precision_loss)]
        let a = i as f32 / 120.0 * std::f32::consts::TAU;
        [c + 500.0 * a.cos(), c + 500.0 * a.sin()]
    }
}

/// A janela de LEITURA de uma saída `(x0, y0, x1, y1)` (fim exclusivo) com `pad`, recortada à tela.
fn leitura(r: (usize, usize, usize, usize), pad: usize, lado: usize) -> usize {
    let (x0, y0) = (r.0.saturating_sub(2 * pad), r.1.saturating_sub(2 * pad));
    let (x1, y1) = ((r.2 + 2 * pad).min(lado), (r.3 + 2 * pad).min(lado));
    (x1 - x0) * (y1 - y0)
}

/// SONDA — por quadro, os texels de cobertura que mudaram, e a área de leitura do composite com
/// uma caixa contra faixas de `H` linhas (fundidas quando a fusão não caminha mais que as duas mais
/// o custo fixo `o`); e o custo do composite em função da janela (`a + b·texels`).
/// `cargo test -p ph2d-tool-painter --profile smoke --lib diag_a_geometria_do_quadro_da_mancha -- --ignored --nocapture`
#[test]
#[ignore = "diagnóstico"]
fn diag_a_geometria_do_quadro_da_mancha() {
    let lado = 2048usize;
    let pt = laco(lado as u32);
    let mut t: PainterTool = tool(lado as u32, PaintMedia::Watercolor, 20.0);
    t.set_brush_color_srgb8([30, 60, 220]);
    t.paint.brush.style_solid = true;
    t.on_canvas_pointer(cp(pt(0), PointerPhase::Down));
    let pad = t.alcance_da_janela().pad;
    let hs = [8usize, 16, 32, 64, 128];
    let custos_fixos = [0usize, 20_000, 60_000];
    let mut caixa = 0usize;
    let mut faixas = vec![vec![(0usize, 0usize); custos_fixos.len()]; hs.len()];
    let mut mudaram = 0usize;
    for i in 1..=120 {
        let antes = t.paint.stroke_coverage.clone();
        t.on_canvas_pointer(cp(pt(i), PointerPhase::Move));
        t.paint_tick(1.0 / 60.0);
        // As linhas que mudaram: (y, x_min, x_max).
        let mut linhas: Vec<(usize, usize, usize)> = Vec::new();
        for y in 0..lado {
            let (a, b) = (
                &antes[y * lado..(y + 1) * lado],
                &t.paint.stroke_coverage[y * lado..(y + 1) * lado],
            );
            if a == b {
                continue;
            }
            let x0 = a.iter().zip(b).position(|(p, q)| p != q).expect("difere");
            let x1 = lado
                - 1
                - a.iter()
                    .rev()
                    .zip(b.iter().rev())
                    .position(|(p, q)| p != q)
                    .expect("difere");
            mudaram += a.iter().zip(b).filter(|(p, q)| p != q).count();
            linhas.push((y, x0, x1));
        }
        if linhas.is_empty() {
            continue;
        }
        let bb = linhas.iter().fold((lado, lado, 0, 0), |r, &(y, a, b)| {
            (r.0.min(a), r.1.min(y), r.2.max(b + 1), r.3.max(y + 1))
        });
        caixa += leitura(bb, pad, lado);
        for (hi, &h) in hs.iter().enumerate() {
            // As faixas de `h` linhas, cada uma com a caixa das linhas dela.
            let mut bandas: Vec<(usize, usize, usize, usize)> = Vec::new();
            for &(y, a, b) in &linhas {
                match bandas.last_mut() {
                    Some(r) if y / h == (r.1) / h => {
                        *r = (r.0.min(a), r.1, r.2.max(b + 1), y + 1);
                    }
                    _ => bandas.push((a, y, b + 1, y + 1)),
                }
            }
            for (oi, &o) in custos_fixos.iter().enumerate() {
                // Funde a vizinha quando caminhar a fusão custa menos que as duas mais `o`.
                let mut fundidas: Vec<(usize, usize, usize, usize)> = Vec::new();
                for &r in &bandas {
                    if let Some(u) = fundidas.last_mut() {
                        let m = (u.0.min(r.0), u.1.min(r.1), u.2.max(r.2), u.3.max(r.3));
                        if leitura(m, pad, lado)
                            <= leitura(*u, pad, lado) + leitura(r, pad, lado) + o
                        {
                            *u = m;
                            continue;
                        }
                    }
                    fundidas.push(r);
                }
                let area: usize = fundidas.iter().map(|&r| leitura(r, pad, lado)).sum();
                faixas[hi][oi].0 += area;
                faixas[hi][oi].1 += fundidas.len();
            }
        }
    }
    eprintln!(
        "pad {pad} · texels de cobertura que mudaram {:.0}/quadro",
        mudaram as f64 / 120.0
    );
    eprintln!(
        "uma caixa: {:.0} px de leitura/quadro",
        caixa as f64 / 120.0
    );
    for (hi, h) in hs.iter().enumerate() {
        for (oi, o) in custos_fixos.iter().enumerate() {
            eprintln!(
                "faixas de {h:>3} linhas, custo fixo {o:>6} px: {:>8.0} px de leitura/quadro em {:.1} janelas",
                faixas[hi][oi].0 as f64 / 120.0,
                faixas[hi][oi].1 as f64 / 120.0
            );
        }
    }
    // O custo do composite em função da janela: o mínimo de 15 corridas por tamanho.
    let c = pt(60);
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let (cx, cy) = (c[0] as u32, c[1] as u32);
    for l in [1u32, 64, 128, 256, 512] {
        let mut melhor = f64::MAX;
        for _ in 0..15 {
            t.paint.wet_frame_dirty = Some(Region {
                x: cx - l / 2,
                y: cy - l / 2,
                w: l,
                h: l,
            });
            let px0 = t.wash.window_px;
            let q = std::time::Instant::now();
            t.apply_watercolor(false);
            melhor = melhor.min(q.elapsed().as_secs_f64() * 1e3);
            assert!(t.wash.window_px > px0, "o composite correu");
        }
        let px = leitura((0, 0, l as usize, l as usize), pad, usize::MAX);
        eprintln!("composite de uma saída {l}²: {melhor:.3} ms ({px} px de leitura)");
    }
}
