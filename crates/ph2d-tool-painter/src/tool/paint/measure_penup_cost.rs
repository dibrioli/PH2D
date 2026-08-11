//! **O que o PEN-UP custa** — a irmã do [`super::measure_pendown_cost`] no outro extremo do gesto.
//!
//! Aquele arquivo responde *"quanto custa COMEÇAR"*; este responde *"quanto custa FECHAR"*, e a
//! diferença não é cosmética: o pen-up é o único evento que **comita** — o relevo entra na camada, o
//! histórico grava o passo, e os dois trabalham em janelas que ninguém mediu separadamente.
//!
//! ⚠️ **A ablação é por ENTRADA, nunca por instrumentação:** `paint.stroke_undo = None` faz o
//! `close_stroke` pular o commit estrutural (a mesma porta que a §5.14 do doc 28 usou para atribuir
//! 91% do pen-up do impasto ao histórico), e o `set_shape_relief(0)` desliga o depósito de relevo pela
//! porta do artista. Os dois cruzados dão as quatro células da tabela.
//!
//! ⚠️ **E o traço medido é o SÉTIMO**, pela razão que o `measure_relief_systems` documenta: o primeiro
//! traço de um documento paga o *first-touch* dos planos canvas-shaped, e medi-lo como se fosse o preço
//! de todo traço é o erro que o doc 28 §5.13 nomeia.

use super::measure_impasto_cost::{cp, ms};
use crate::tool::PainterTool;
use ph2d_editor_core::tool::{CanvasPaintTool, PointerPhase, RasterEditTool};
use ph2d_painter_brush::TextureKind;

fn tool(side: u32, film: bool) -> PainterTool {
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; (side * side * 4) as usize], side, side);
    t.set_brush_shape_kind(TextureKind::Stripes as u8);
    t.set_brush_size_px(40.0);
    if film {
        t.set_shape_relief(1.0);
    }
    t
}

/// **DE QUE O PEN-UP É FEITO** — as quatro células que separam o commit de undo do commit do relevo.
///
/// Rodar: `cargo test -p ph2d-tool-painter --release what_a_pen_up_is_made_of -- --ignored --nocapture --test-threads=1`
#[test]
#[ignore = "sonda de estudo; roda sob demanda"]
fn what_a_pen_up_is_made_of() {
    println!(
        "\n=== O PEN-UP, PECA A PECA (600 px fixos, raio 20; mediana de 6 apos descartar o 1o) ===\n"
    );
    println!(
        "{:<10} {:>6} {:>10} {:>12} {:>12}",
        "config", "tela", "pen-up", "sem o undo", "delta"
    );
    for side in [2048u32, 4096] {
        for film in [false, true] {
            let mut cell = [0.0f64; 2];
            for (col, skip_undo) in [(0usize, false), (1, true)] {
                let mut t = tool(side, film);
                let cy = f32::from(u16::try_from(side / 2).unwrap_or(512));
                let mut ups = Vec::new();
                for k in 0..7u8 {
                    let y = cy + f32::from(k) * 6.0;
                    let x0 = cy - 300.0;
                    t.on_canvas_pointer(cp([x0, y], PointerPhase::Down));
                    for i in 1..=15u8 {
                        t.on_canvas_pointer(cp([x0 + 40.0 * f32::from(i), y], PointerPhase::Move));
                    }
                    if skip_undo {
                        // A ablação por ENTRADA: sem o `before`, o `close_stroke` não grava passo.
                        t.paint.stroke_undo = None;
                    }
                    ups.push(ms(&mut || {
                        t.on_canvas_pointer(cp([x0 + 600.0, y], PointerPhase::Up));
                    }));
                }
                ups.remove(0);
                ups.sort_by(f64::total_cmp);
                cell[col] = ups[ups.len() / 2];
            }
            println!(
                "{:<10} {side:>6} {:>10.2} {:>12.2} {:>12.2}",
                if film { "filme" } else { "digital" },
                cell[0],
                cell[1],
                cell[0] - cell[1]
            );
        }
    }
    println!();
}

/// **O COMMIT DO RELEVO É PROPORCIONAL À JANELA?** — a pergunta que decide se encolhê-la compra algo.
///
/// A tabela acima diz que o commit do filme é **plano na tela** (1,79 contra 1,95 ms), logo é trabalho
/// de janela e não de plano. Falta saber se ele *cresce com a janela*: se crescer, a margem constante
/// de 28 px que o `grow_region` acrescenta ao bbox é uma fração do trabalho, e ela é maior quanto menor
/// for o pincel.
#[test]
#[ignore = "sonda de estudo; roda sob demanda"]
fn is_the_relief_commit_proportional_to_its_window() {
    const SIDE: u32 = 2048;
    println!("\n=== O COMMIT DO RELEVO CONTRA A JANELA (2048, sem o commit de undo; ms) ===\n");
    println!(
        "{:<8} {:>10} {:>14} {:>12}",
        "raio", "pen-up", "bbox+28 (kpx)", "ns/texel"
    );
    for r in [20.0f32, 40.0, 80.0] {
        let mut t = tool(SIDE, true);
        t.set_brush_size_px(r * 2.0);
        let cy = f32::from(u16::try_from(SIDE / 2).unwrap_or(512));
        let mut ups = Vec::new();
        for k in 0..7u8 {
            let y = cy + f32::from(k) * (r * 2.0 + 4.0);
            let x0 = cy - 300.0;
            t.on_canvas_pointer(cp([x0, y], PointerPhase::Down));
            for i in 1..=15u8 {
                t.on_canvas_pointer(cp([x0 + 40.0 * f32::from(i), y], PointerPhase::Move));
            }
            t.paint.stroke_undo = None;
            ups.push(ms(&mut || {
                t.on_canvas_pointer(cp([x0 + 600.0, y], PointerPhase::Up));
            }));
        }
        ups.remove(0);
        ups.sort_by(f64::total_cmp);
        let up = ups[ups.len() / 2];
        // A janela que o commit percorre: o bbox do traço crescido pelas duas constantes.
        let win = f64::from((600.0 + 2.0 * r + 56.0) * (2.0 * r + 56.0));
        println!(
            "{r:<8.0} {up:>10.2} {:>14.0} {:>12.1}",
            win / 1000.0,
            up * 1e6 / win
        );
    }
    println!();
}

/// **DE QUE O COMMIT DO RELEVO É FEITO** — os quatro blocos, medidos no código que shipa.
///
/// A tabela irmã excluiu a tela (plano) e a janela (plano). Sem knob que separe as peças, a atribuição
/// vem do split `cfg(test)` que o [`super::impasto_live::spans`] instala nas fronteiras que os blocos
/// já tinham.
#[test]
#[ignore = "sonda de estudo; roda sob demanda"]
fn what_the_relief_commit_is_made_of() {
    println!("\n=== O COMMIT DO RELEVO, BLOCO A BLOCO (600 px, raio 20; ms por traco) ===\n");
    println!(
        "{:<8} {:>10} {:>10} {:>10} {:>12} {:>10}",
        "tela", "cobertura", "material", "patches", "re-derive", "soma"
    );
    for side in [2048u32, 4096] {
        let mut t = tool(side, true);
        let cy = f32::from(u16::try_from(side / 2).unwrap_or(512));
        let mut per: Vec<Vec<f64>> = vec![Vec::new(); 4];
        let mut first = [0.0f64; 4];
        for k in 0..7u8 {
            let y = cy + f32::from(k) * 6.0;
            let x0 = cy - 300.0;
            t.on_canvas_pointer(cp([x0, y], PointerPhase::Down));
            for i in 1..=15u8 {
                t.on_canvas_pointer(cp([x0 + 40.0 * f32::from(i), y], PointerPhase::Move));
            }
            t.paint.stroke_undo = None;
            let _ = super::impasto_live::spans::take(); // zera: o que se mede é ESTE traço
            t.on_canvas_pointer(cp([x0 + 600.0, y], PointerPhase::Up));
            // ⚠️ **MEDIANA por bloco, não média.** O `mats` é alocado UMA vez por documento (28 MB a
            // 2048², 117 a 4096²) e o traço que o estreia paga o *first-touch* inteiro; numa média de
            // seis ele sozinho dizia 3,70 ms onde a mediana diz o preço de um commit. É a lição do
            // §5.13 outra vez, agora por bloco.
            let sp = super::impasto_live::spans::take();
            if k == 0 {
                first = sp;
            } else {
                for (i, v) in sp.iter().enumerate() {
                    per[i].push(*v);
                }
            }
        }
        let m = |v: &mut Vec<f64>| {
            v.sort_by(f64::total_cmp);
            v[v.len() / 2]
        };
        let s: Vec<f64> = (0..4).map(|i| m(&mut per[i])).collect();
        println!(
            "{side:<8} {:>10.2} {:>10.2} {:>10.2} {:>12.2} {:>10.2}",
            s[0],
            s[1],
            s[2],
            s[3],
            s.iter().sum::<f64>()
        );
        // ⚠️ **E o PRIMEIRO traço do documento, que a mediana existe para descartar** — é ele que
        // estreia os planos da camada (o `mats` mede 28 MB a 2048² e 117 a 4096²) e paga o
        // *first-touch* inteiro. Descartá-lo é certo para saber o que um traço custa; ESCONDÊ-LO seria
        // perder um hitch de verdade, que acontece uma vez por documento.
        println!(
            "{:<8} {:>10.2} {:>10.2} {:>10.2} {:>12.2} {:>10.2}   <- o 1o traco",
            "",
            first[0],
            first[1],
            first[2],
            first[3],
            first.iter().sum::<f64>()
        );
    }
    println!();
}
