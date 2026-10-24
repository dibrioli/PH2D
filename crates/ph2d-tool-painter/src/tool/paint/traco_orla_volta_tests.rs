//! ⭐ **O traço que VOLTA por cima de si** (bug #58, 3.ª volta — smoke do dono depois de `3574ae0122`): o gate (f)
//! da lei da orla, partido de [`super::traco_orla_lei_tests`] pelo tecto de LOC; os ajudantes do oráculo vêm de lá.

use super::traco_orla_bordas_tests::{H, W, glaze_round, padrao};
use super::traco_orla_lei_tests::{Monta, Traco, caminho, cp, e_medido, pinta};
use crate::tool::PainterTool;
use ph2d_editor_core::tool::{CanvasPaintTool, PointerPhase};

/// Os pincéis do dono para o traço que VOLTA: o Glaze Round de fábrica e o Digital por omissão (Accumulate
/// ligado e desligado, Strength 1 e 0,6, Flow 1 e 0,3), cada um com Wet 0,8 e com Burnt 0,8.
fn pinceis_da_volta() -> Vec<(String, PainterTool)> {
    let mut out = Vec::new();
    for (orla, liga) in [
        (
            "Wet 0,8",
            PainterTool::set_traco_molhada as fn(&mut PainterTool, f32),
        ),
        ("Burnt 0,8", PainterTool::set_traco_queimada),
    ] {
        out.push((
            format!("Glaze Round · {orla}"),
            glaze_round(|t| liga(t, 0.8)),
        ));
        for acum in [false, true] {
            for forca in [1.0, 0.6] {
                for flow in [1.0, 0.3] {
                    let t = padrao(|t| {
                        t.paint.brush.accumulate = acum;
                        t.set_brush_strength(forca);
                        t.set_brush_flow(flow);
                        liga(t, 0.8);
                    });
                    out.push((
                        format!("Digital · Acc {acum} · S {forca} · F {flow} · {orla}"),
                        t,
                    ));
                }
            }
        }
    }
    out
}

/// A posição (px, sub-píxel) em que `e` desce de `½` na coluna `x`, de cima para dentro (`None` sem orla à vista).
fn borda_na_coluna(t: &PainterTool, tr: &Traco, x: usize, y0: usize, y1: usize) -> Option<f32> {
    let mut antes: Option<(usize, f32)> = None;
    for y in y0..y1 {
        let Some(e) = e_medido(t, tr, y * W + x) else {
            continue;
        };
        if let Some((ya, ea)) = antes
            && ea >= 0.5
            && e < 0.5
        {
            return Some(ya as f32 + (ea - 0.5) / (ea - e) * (y - ya) as f32);
        }
        antes = Some((y, e));
    }
    None
}

/// O retrato do traço ABERTO agora (sem levantar).
fn retrato(t: &PainterTool) -> Traco {
    let s = t
        .paint
        .plano_do_traco
        .as_ref()
        .expect("o traço pinta no plano");
    let dec = |c: &[u8]| [0, 1, 2, 3].map(|k| f32::from(c[k]) / 255.0);
    let plano: Vec<[f32; 4]> = s.plano.chunks(4).map(dec).collect();
    let tecto = t.paint.brush.traco.tecto;
    Traco {
        tela: t.canvas_rgba.to_vec(),
        base: s.base.chunks(4).map(dec).collect(),
        v: plano
            .iter()
            .map(|p| if tecto > 0.0 { p[3].min(tecto) } else { p[3] })
            .collect(),
        plano,
        tau: s.fronteira(),
    }
}

/// ⭐ **(f) O contorno da primeira perna não se mexe quando o traço volta** (smoke do dono depois de `3574ae0122`:
/// *«no segundo movimento (quando o traço muda de direção) faz os edges mudarem de forma estranha, se
/// encolhendo»*): um traço para a direita que volta para a esquerda por cima de si; na faixa da 1.ª perna onde a
/// 2.ª não deposita nada, a tela antes e depois da volta é a MESMA (`|Δ| ≤ 1`) e a borda (`e = ½`) não anda mais
/// de 0,5 px.
#[test]
fn o_contorno_da_primeira_perna_nao_se_mexe_quando_o_traco_volta() {
    let ida = caminho(&[[30.0, 100.0], [290.0, 100.0]]);
    let volta = caminho(&[[290.0, 100.0], [150.0, 110.0]]);
    // A faixa só da 1.ª perna: x < 150 − raio (20) − W e a célula (≥ 10).
    let (x0, x1) = (40usize, 110usize);
    let mut falhas = Vec::new();
    for (nome, mut t) in pinceis_da_volta() {
        pinta(&mut t, &ida, |_| 4);
        let antes = retrato(&t);
        for p in &volta[1..] {
            t.on_canvas_pointer(cp(*p, PointerPhase::Move));
            t.paint_tick(1.0 / 60.0);
        }
        let depois = retrato(&t);
        t.on_canvas_pointer(cp(*volta.last().unwrap(), PointerPhase::Up));
        let maxv = |tr: &Traco| tr.v.iter().copied().fold(0.0, f32::max);
        // A fronteira é o nível de UMA passagem: ½ do máximo da 1.ª perna, e a volta não o sobe.
        let uma = 0.5 * maxv(&antes);
        if (antes.tau - uma).abs() > 0.5 / 255.0 || depois.tau.to_bits() != antes.tau.to_bits() {
            falhas.push(format!(
                "{nome}: τ {:.4} → {:.4}, ½ da 1.ª perna {uma:.4}",
                antes.tau, depois.tau
            ));
            continue;
        }
        let (mut pior, mut mudados) = (0u8, 0usize);
        for y in 0..H {
            for x in x0..x1 {
                let i = (y * W + x) * 4;
                let d = (0..4)
                    .map(|k| antes.tela[i + k].abs_diff(depois.tela[i + k]))
                    .max()
                    .unwrap();
                pior = pior.max(d);
                mudados += usize::from(d > 1);
            }
        }
        let xc = (x0 + x1) / 2;
        let (ba, bd) = (
            borda_na_coluna(&t, &antes, xc, 60, 100),
            borda_na_coluna(&t, &depois, xc, 60, 100),
        );
        let anda = match (ba, bd) {
            (Some(a), Some(b)) => (b - a).abs(),
            (None, None) => 0.0,
            _ => f32::INFINITY,
        };
        eprintln!(
            "{nome}: max v {:.3} → {:.3} · borda {ba:?} → {bd:?} (anda {anda:.2} px) · tela máx |Δ| {pior}, {mudados} texels",
            maxv(&antes),
            maxv(&depois)
        );
        if pior > 1 || anda > 0.5 {
            falhas.push(nome.clone());
        }
    }
    assert!(falhas.is_empty(), "a 1.ª perna mexe: {falhas:#?}");
}

/// ⛔ Sonda + fotos (`PH2D_FOTO_DIR`, `PH2D_FOTO_SUFIXO`): o traço que VOLTA por cima de si, o quadro na volta
/// (`1_na_volta`) e o quadro depois de voltar (`2_voltou`), ×2.
#[test]
#[ignore = "sonda: escreve PNG em PH2D_FOTO_DIR"]
fn foto_do_traco_que_volta() {
    let Ok(dir) = std::env::var("PH2D_FOTO_DIR") else {
        return;
    };
    let sufixo = std::env::var("PH2D_FOTO_SUFIXO").unwrap_or_else(|_| "antes".into());
    let ida = caminho(&[[30.0, 100.0], [290.0, 100.0]]);
    let volta = caminho(&[[290.0, 100.0], [150.0, 110.0]]);
    let cenas: [(&str, Monta); 3] = [
        ("glaze_wet08", || glaze_round(|t| t.set_traco_molhada(0.8))),
        ("glaze_burnt08", || {
            glaze_round(|t| t.set_traco_queimada(0.8))
        }),
        ("digital_acc_s06_f03_wet08", || {
            padrao(|t| {
                t.paint.brush.accumulate = true;
                t.set_brush_flow(0.3);
                t.set_traco_molhada(0.8);
            })
        }),
    ];
    for (nome, mk) in cenas {
        let mut t = mk();
        let grava = |t: &PainterTool, sub: &str| {
            let px = &t.canvas_rgba;
            let (y0, lh, zoom) = (50, 100, 2);
            let mut c = super::tests::look_watercolor_arc::Canvas::new(W * zoom, lh * zoom);
            for y in 0..lh * zoom {
                for x in 0..W * zoom {
                    let i = ((y0 + y / zoom) * W + x / zoom) * 4;
                    c.set(x, y, [px[i], px[i + 1], px[i + 2]]);
                }
            }
            let p = std::path::Path::new(&dir).join(format!("{nome}_{sub}_{sufixo}.png"));
            super::tests::look_watercolor_arc::write_png(&p, &c).expect("PNG");
        };
        pinta(&mut t, &ida, |_| 4);
        grava(&t, "1_na_volta");
        for p in &volta[1..] {
            t.on_canvas_pointer(cp(*p, PointerPhase::Move));
            t.paint_tick(1.0 / 60.0);
        }
        grava(&t, "2_voltou");
        t.on_canvas_pointer(cp(*volta.last().unwrap(), PointerPhase::Up));
    }
}
