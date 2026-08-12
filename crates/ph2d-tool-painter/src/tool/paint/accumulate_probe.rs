//! **Sonda de medição do ACCUMULATE** — o estudo comparativo com o Blender (Enio, 2026-08-12:
//! *"Apenas Accumulate deve ser estudo e comparado com o blender"*).
//!
//! Não é gate: são sondas `#[ignore]` que MEDEM a lei que shipa, para o estudo
//! ([`docs/Painter/35_accumulate_vs_blender.md`]) citar números do PRODUTO em vez de raciocínio.
//!
//! ⚠️ **A fixture tem de conter o fenômeno.** O `white_canvas` dos gates usa disco DURO
//! (`hardness 1`, `Falloff::Constant`) — nele o perfil do dab vale 1 ou 0, então a lei do acúmulo
//! fica invisível no miolo e o OMBRO, que é onde ela vive, nem existe. Esta sonda usa o pincel
//! MACIO (`Falloff::Smooth`, `hardness 0`), que é o do produto.
//!
//! Rodar: `cargo test -p ph2d-tool-painter --release accumulate_probe -- --ignored --nocapture`

use crate::tool::PainterTool;
use ph2d_editor_core::ids as core_ids;
use ph2d_editor_core::tool::{
    CanvasPaintTool, CanvasPointer, PanelEvent, PointerPhase, RasterEditTool, Tool,
};
use ph2d_painter_brush::Falloff;

const SIZE: u32 = 64;
const Y: f32 = 32.0;
const X0: f32 = 20.0;
const X1: f32 = 44.0;
/// Onde se mede: o MEIO do segmento, que todo dab do caminho atravessa.
const PROBE_X: u32 = 32;

pub(in crate::tool::paint) fn cp(pos: [f32; 2], phase: PointerPhase) -> CanvasPointer {
    CanvasPointer {
        pos,
        pressure: 1.0,
        tilt: [0.0, 0.0],
        phase,
    }
}

/// Tela BRANCA opaca + pincel PRETO macio (o do produto). `strength` é o único knob que o chamador
/// escolhe além dos que a sonda varre.
pub(in crate::tool::paint) fn soft_tool(strength: f32, accumulate: bool) -> PainterTool {
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; (SIZE * SIZE * 4) as usize], SIZE, SIZE);
    t.paint.brush.radius_px = 8.0;
    t.paint.brush.hardness = 0.0;
    t.paint.brush.falloff = Falloff::Smooth;
    t.paint.brush.color = [0.0, 0.0, 0.0];
    let seed = t.paint.brush;
    for slot in &mut t.paint.brush_by_mode {
        *slot = seed;
    }
    t.set_brush_strength(strength);
    if accumulate {
        t.handle_panel_event(PanelEvent::Click(core_ids::PAINTER_BRUSH_ACCUMULATE));
    }
    t
}

/// Opacidade da tinta no ponto de sonda: preto sobre branco ⇒ `alpha = (255 − r) / 255`.
pub(in crate::tool::paint) fn alpha(t: &PainterTool) -> f32 {
    let i = ((Y as u32 * SIZE) + PROBE_X) as usize * 4;
    (255.0 - f32::from(t.canvas_rgba[i])) / 255.0
}

/// `n` passadas de ida-e-volta **DENTRO de uma pincelada** (um Down, um Up).
pub(in crate::tool::paint) fn one_stroke(t: &mut PainterTool, n: usize) {
    t.on_canvas_pointer(cp([X0, Y], PointerPhase::Down));
    for _ in 0..n {
        t.on_canvas_pointer(cp([X1, Y], PointerPhase::Move));
        t.on_canvas_pointer(cp([X0, Y], PointerPhase::Move));
    }
    t.on_canvas_pointer(cp([X0, Y], PointerPhase::Up));
}

/// `n` pincelada**S** separadas (um Down/Up por passada) sobre o MESMO caminho.
fn separate_strokes(t: &mut PainterTool, n: usize) {
    for _ in 0..n {
        t.on_canvas_pointer(cp([X0, Y], PointerPhase::Down));
        t.on_canvas_pointer(cp([X1, Y], PointerPhase::Move));
        t.on_canvas_pointer(cp([X0, Y], PointerPhase::Move));
        t.on_canvas_pointer(cp([X0, Y], PointerPhase::Up));
    }
}

/// **Sonda 1 — a lei DENTRO de uma pincelada.** É o gesto que o Enio reportou: esfregar vai-e-volta
/// sem soltar. A pergunta é se a opacidade sobe, e para onde ela converge.
#[test]
#[ignore = "sonda de medicao (estudo do Accumulate); roda com --ignored --nocapture"]
fn measure_accumulate_within_one_stroke() {
    println!("\n=== DENTRO de UMA pincelada (ida-e-volta sem soltar) ===");
    println!("alpha no centro; teto teorico OFF = strength, ON = 1.0\n");
    for &s in &[0.3f32, 0.5, 1.0] {
        for &acc in &[false, true] {
            let mut row = format!(
                "strength {s:.1}  accumulate {:<3}  ",
                if acc { "ON" } else { "off" }
            );
            for &n in &[1usize, 2, 5, 15] {
                let mut t = soft_tool(s, acc);
                one_stroke(&mut t, n);
                row.push_str(&format!("n={n:<3}{:.4}  ", alpha(&t)));
            }
            println!("{row}");
        }
    }
}

/// **Sonda 2 — pinceladas SEPARADAS.** O controle: entre traços o build-up existe nos dois modos
/// (cada traço recomeça a própria cobertura), então esta tabela isola o que o flag de fato governa.
#[test]
#[ignore = "sonda de medicao (estudo do Accumulate); roda com --ignored --nocapture"]
fn measure_accumulate_across_separate_strokes() {
    println!("\n=== pinceladas SEPARADAS (um Down/Up por passada) ===\n");
    for &s in &[0.3f32, 0.5, 1.0] {
        for &acc in &[false, true] {
            let mut row = format!(
                "strength {s:.1}  accumulate {:<3}  ",
                if acc { "ON" } else { "off" }
            );
            for &n in &[1usize, 2, 5, 15] {
                let mut t = soft_tool(s, acc);
                separate_strokes(&mut t, n);
                row.push_str(&format!("n={n:<3}{:.4}  ", alpha(&t)));
            }
            println!("{row}");
        }
    }
}

/// **Sonda 3 — a dependência de ESPAÇAMENTO**, que é a diferença estrutural entre as duas leis.
///
/// O OFF é um teto por texel (`m += w·(cap − m)`), então mais dabs só aproximam o MESMO teto: o
/// espaçamento quase não entra. O ON é source-over por dab, então o número de dabs multiplica —
/// e é exatamente aí que o Blender tem o mesmo comportamento e um knob para o compensar
/// (*Adjust Strength for Spacing*, o nosso `space_attenuation`).
#[test]
#[ignore = "sonda de medicao (estudo do Accumulate); roda com --ignored --nocapture"]
fn measure_accumulate_spacing_dependence() {
    println!("\n=== dependencia de ESPACAMENTO (uma passada, strength 0.5) ===");
    println!("mesmo CAMINHO; so muda quantos dabs o motor emite nele\n");
    for &atten in &[false, true] {
        for &acc in &[false, true] {
            let mut row = format!(
                "space_atten {:<3}  accumulate {:<3}  ",
                if atten { "ON" } else { "off" },
                if acc { "ON" } else { "off" }
            );
            let mut vals = Vec::new();
            for &sp in &[0.05f32, 0.10, 0.20, 0.40] {
                let mut t = soft_tool(0.5, acc);
                if atten {
                    t.handle_panel_event(PanelEvent::Click(core_ids::PAINTER_BRUSH_SPACE_ATTEN));
                }
                t.handle_panel_event(PanelEvent::SetValue(
                    core_ids::PAINTER_BRUSH_SPACING,
                    f64::from(sp),
                ));
                one_stroke(&mut t, 1);
                let a = alpha(&t);
                vals.push(a);
                row.push_str(&format!("sp={sp:.2} {a:.4}  "));
            }
            let (lo, hi) = vals
                .iter()
                .fold((f32::MAX, 0.0f32), |(l, h), &v| (l.min(v), h.max(v)));
            row.push_str(&format!(
                "| razao {:.2}x",
                if lo > 0.0 { hi / lo } else { 0.0 }
            ));
            println!("{row}");
        }
    }
}

/// **Sonda 5 — o OMBRO, que é onde a lei vive quando a Strength é 1.0.**
///
/// ⚠️ Esta sonda existe por causa de uma lição paga (doc 25 §13.10): *"eu medi a modulação no EIXO
/// do traço e escrevi «invisível» — o eixo satura em QUALQUER lei; as contas vivem no OMBRO"*. Com
/// `strength = 1.0` o centro de um pincel macio satura numa passada nos DOIS modos, então uma
/// tabela medida só no centro diz **1.0000 contra 1.0000** e conclui, errado, que o flag é inerte.
/// O perfil perpendicular é o que separa as duas leis.
#[test]
#[ignore = "sonda de medicao (estudo do Accumulate); roda com --ignored --nocapture"]
fn measure_accumulate_on_the_shoulder() {
    println!("\n=== o PERFIL PERPENDICULAR do traco (o ombro) ===");
    println!("alpha por distancia ao eixo; raio 8, falloff Smooth\n");
    for &s in &[1.0f32, 0.5] {
        for &n in &[1usize, 15] {
            for &acc in &[false, true] {
                let mut t = soft_tool(s, acc);
                one_stroke(&mut t, n);
                let mut row = format!(
                    "strength {s:.1}  n={n:<3} accumulate {:<3} ",
                    if acc { "ON" } else { "off" }
                );
                for dy in [0u32, 2, 4, 6, 8, 10] {
                    let i = (((Y as u32 + dy) * SIZE) + PROBE_X) as usize * 4;
                    let a = (255.0 - f32::from(t.canvas_rgba[i])) / 255.0;
                    row.push_str(&format!("d{dy:<2}={a:.3} "));
                }
                println!("{row}");
            }
        }
        println!();
    }
}

/// **Sonda 4 — o RELEVO (impasto) sob o mesmo gesto.** O flag governa a COR e nunca alcança o
/// corpo da tinta: esta sonda mede as duas metades no MESMO traço, que é o que torna a assimetria
/// um número em vez de uma frase.
#[test]
#[ignore = "sonda de medicao (estudo do Accumulate); roda com --ignored --nocapture"]
fn measure_accumulate_reaches_colour_but_not_relief() {
    println!("\n=== a MESMA tinta, as duas metades (strength 1.0) ===\n");
    for &acc in &[false, true] {
        for &n in &[1usize, 5] {
            let mut t = soft_tool(1.0, acc);
            t.paint.brush.impasto = true;
            let seed = t.paint.brush;
            for slot in &mut t.paint.brush_by_mode {
                *slot = seed;
            }
            one_stroke(&mut t, n);
            let i = (Y as usize * SIZE as usize) + PROBE_X as usize;
            let h = t
                .layers
                .active()
                .and_then(|a| t.heights.get(&a))
                .and_then(|hs| hs.get(i).copied())
                .unwrap_or(0.0);
            println!(
                "accumulate {:<3}  n={n:<3}  COR alpha={:.4}   RELEVO h={h:.4}",
                if acc { "ON" } else { "off" },
                alpha(&t)
            );
        }
    }
}

/// O relevo no ponto de sonda (0 se a camada não tem plano de altura).
pub(in crate::tool::paint) fn relief(t: &PainterTool) -> f32 {
    let i = (Y as usize * SIZE as usize) + PROBE_X as usize;
    t.layers
        .active()
        .and_then(|a| t.heights.get(&a))
        .and_then(|hs| hs.get(i).copied())
        .unwrap_or(0.0)
}

/// Um tool de impasto: o pincel macio + o corpo ligado nos três slots de relevo.
pub(in crate::tool::paint) fn impasto_tool(accumulate: bool) -> PainterTool {
    let mut t = soft_tool(1.0, accumulate);
    t.paint.brush.impasto = true;
    let seed = t.paint.brush;
    for slot in &mut t.paint.brush_by_mode {
        *slot = seed;
    }
    t
}

/// **Sonda 6 — o ACCUMULATE do RELEVO (a D3).** É o gesto que o Enio reportou, medido no corpo da
/// tinta em vez de na cor.
#[test]
#[ignore = "sonda de medicao (estudo do Accumulate); roda com --ignored --nocapture"]
fn measure_relief_accumulates_along_the_arc() {
    println!("\n=== o RELEVO sob ida-e-volta na MESMA pincelada ===\n");
    for &acc in &[false, true] {
        let mut row = format!("accumulate {:<3}  ", if acc { "ON" } else { "off" });
        for &n in &[1usize, 2, 5, 15] {
            let mut t = impasto_tool(acc);
            one_stroke(&mut t, n);
            row.push_str(&format!("n={n:<3}{:.4}  ", relief(&t)));
        }
        println!("{row}");
    }

    println!("\n=== e a INDEPENDENCIA DE ESPACAMENTO (uma passada) ===");
    println!("mesmo CAMINHO; so muda quantos dabs o motor emite\n");
    for &acc in &[false, true] {
        let mut row = format!("accumulate {:<3}  ", if acc { "ON" } else { "off" });
        let mut vals = Vec::new();
        for &sp in &[0.05f32, 0.10, 0.20] {
            let mut t = impasto_tool(acc);
            t.handle_panel_event(PanelEvent::SetValue(
                core_ids::PAINTER_BRUSH_SPACING,
                f64::from(sp),
            ));
            one_stroke(&mut t, 1);
            let h = relief(&t);
            vals.push(h);
            row.push_str(&format!("sp={sp:.2} {h:.4}  "));
        }
        let (lo, hi) = vals
            .iter()
            .fold((f32::MAX, 0.0f32), |(l, h), &v| (l.min(v), h.max(v)));
        row.push_str(&format!(
            "| razao {:.2}x",
            if lo > 0.0 { hi / lo } else { 0.0 }
        ));
        println!("{row}");
    }

    println!("\n=== UMA passada RETA (o gate que torna o toggle honesto) ===\n");
    for &acc in &[false, true] {
        let mut t = impasto_tool(acc);
        t.on_canvas_pointer(cp([X0, Y], PointerPhase::Down));
        t.on_canvas_pointer(cp([X1, Y], PointerPhase::Move));
        t.on_canvas_pointer(cp([X1, Y], PointerPhase::Up));
        println!(
            "accumulate {:<3}  h={:.4}",
            if acc { "ON" } else { "off" },
            relief(&t)
        );
    }

    println!("\n=== o TAP (decisao (i): 1o dab recebe o espacamento nominal) ===\n");
    for &acc in &[false, true] {
        let mut t = impasto_tool(acc);
        t.on_canvas_pointer(cp([32.0, 32.0], PointerPhase::Down));
        t.on_canvas_pointer(cp([32.0, 32.0], PointerPhase::Up));
        println!(
            "accumulate {:<3}  tap h={:.4}",
            if acc { "ON" } else { "off" },
            relief(&t)
        );
    }
}
