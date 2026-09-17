//! ⭐⭐⭐ **QUE NÚMERO CADA ROW DO CARTÃO CARREGA** — irmão de
//! [`super::card_params_tests`] por RESPONSABILIDADE (HR-18): lá pergunta-se *QUAIS* params o
//! cartão mostra, aqui *QUE NÚMERO* cada um deles traz — a faixa do arrasto, os limites
//! digitáveis e a face em que o artista o lê.
//!
//! ⚠️ São duas perguntas mesmo, e a segunda nasceu inteira em 2026-09-05: até o cartão poder
//! receber um número escrito, ninguém tinha razão para comparar a FAIXA dele com a do painel —
//! e ela divergia em **109 de 454** rows escalares do catálogo.

use super::card_params_tests::open_every_section;
use super::*;
use crate::motion_state::MotionState;

/// ⭐⭐⭐ **O CARTÃO ARRASTA NA MESMA FAIXA QUE O PAINEL** — sobre **todo tipo de nó do registry**.
///
/// ⚠️ **A faixa de uma MAGNITUDE não é a do hint.** Ela depende do canal que o nó conduz (o
/// `Amount` do `motion.drive` mede graus numa Rotation e unidades de mundo num X/Y), do fio que
/// sai dele (um `value.*` veste a roupa de quem alimenta) e do valor vivo (`contain`, para uma
/// faixa nunca mentir sobre um número que já está lá). O painel resolve as três desde
/// 2026-08-14 — e o report que o obrigou foi do Enio: *«Scale não aceita mais que 4 em sua caixa
/// de texto e 4 não é quase nada para rot»*.
///
/// ⇒ Um cartão que lesse `hint.min`/`hint.max` traria esse defeito de volta **no dia em que o
/// painel saísse**, e traria-o calado. FALSIFICADO por o `stamp_card_params` voltar a copiar o
/// hint: os nós que declaram `param_channel_range` acusam de imediato.
///
/// ⚠️ **A comparação é DIRECTA, e é isso que a torna forte:** desde 2026-09-05 o cartão veste a
/// mesma FACE do painel (`mostrado = guardado × escala`), então os dois números têm de ser o
/// MESMO — faixa, passo, limites digitáveis e o valor. Eram `109` de `454` rows escalares a
/// discordar (o `motion.move::dx` lia `0.94` no cartão e `94 px` no painel).
#[test]
fn the_card_shows_and_drags_the_same_numbers_the_panel_does() {
    use crate::ParamRow;
    let base = MotionState::new();
    let tipos: Vec<String> = base
        .registry
        .manifests()
        .map(|m| m.name.to_string())
        .collect();
    drop(base);
    let mut divergem: Vec<String> = Vec::new();
    let mut comparados = 0usize;
    for nome in &tipos {
        let mut m = MotionState::new();
        let id = m.doc.graph.add_node(nome.clone());
        open_every_section(&mut m, id);
        ph2d_panel_motion_graph::set_graph_selection(vec![id.0]);
        let Some(painel) = build_params_snapshot(&m, ph2d_editor_core::ProjectSettings::default())
        else {
            continue;
        };
        let mut snap = ph2d_panel_motion_graph::snapshot_from(&m.doc.graph, &m.registry);
        stamp_card_params(&m, ph2d_editor_core::ProjectSettings::default(), &mut snap);
        let Some(view) = snap.nodes.iter().find(|v| v.id == id.0) else {
            continue;
        };
        for row in &painel.rows {
            let ParamRow::Scalar(r) = row else { continue };
            let Some(c) = view.params.iter().find(|c| c.hint.param == r.name) else {
                continue; // a falta é o assunto do gate irmão
            };
            comparados += 1;
            let mesma = |painel: f64, cartao: f32| {
                (painel - f64::from(cartao)).abs() <= 1e-3 * painel.abs().max(1.0)
            };
            #[expect(
                clippy::cast_possible_truncation,
                reason = "a face e' um f64 de escala"
            )]
            let face_igual = (r.display.scale as f32 - c.face_scale).abs()
                <= 1e-4 * (r.display.scale as f32).abs().max(1.0)
                && r.display.suffix == c.face_suffix;
            if !mesma(r.min, c.min)
                || !mesma(r.max, c.max)
                || !mesma(r.hard_min, c.hard_min)
                || !mesma(r.hard_max, c.hard_max)
                || !mesma(r.value, c.value)
                || !face_igual
            {
                divergem.push(format!(
                    "{nome}::{} painel [{:.4}, {:.4}] (duro [{:.4}, {:.4}], valor {:.4}, face {:.4}{:?}) contra cartao [{:.4}, {:.4}] (duro [{:.4}, {:.4}], valor {:.4}, face {:.4}{:?})",
                    r.name,
                    r.min, r.max, r.hard_min, r.hard_max, r.value, r.display.scale, r.display.suffix,
                    c.min, c.max, c.hard_min, c.hard_max, c.value, c.face_scale, c.face_suffix,
                ));
            }
        }
    }
    ph2d_panel_motion_graph::set_graph_selection(Vec::new());
    assert!(
        comparados > 300,
        "o gate ficaria vacuo com {comparados} rows"
    );
    assert!(
        divergem.is_empty(),
        "{} de {comparados} rows arrastam numa faixa diferente no cartao:\n  {}",
        divergem.len(),
        divergem.join("\n  ")
    );
}

/// **O CENSO DA FACE — quantos params o painel mostra numa unidade que o cartão ainda não veste.**
///
/// O painel converte uma LENGTH de metros para px (`RowDisplay::scale`) e põe o sufixo; o cartão
/// desenha o número guardado. ⚠️ *Isto é uma divergência REAL de produto* — o mesmo param lê-se
/// `0.94` num sítio e `94 px` noutro — e este censo é o número dela, para a wave que a fecha não
/// começar por um palpite.
///
/// `cargo test -p ph2d-app-motion --lib --release -- --ignored --nocapture how_many_params_wear_a_face`
#[test]
#[ignore = "sonda de censo, nao um gate"]
fn how_many_params_wear_a_face() {
    use crate::ParamRow;
    let base = MotionState::new();
    let tipos: Vec<String> = base
        .registry
        .manifests()
        .map(|m| m.name.to_string())
        .collect();
    drop(base);
    let (mut total, mut com_escala, mut com_sufixo) = (0usize, 0usize, 0usize);
    let mut nomes: Vec<String> = Vec::new();
    for nome in &tipos {
        let mut m = MotionState::new();
        let id = m.doc.graph.add_node(nome.clone());
        open_every_section(&mut m, id);
        ph2d_panel_motion_graph::set_graph_selection(vec![id.0]);
        let Some(painel) = build_params_snapshot(&m, ph2d_editor_core::ProjectSettings::default())
        else {
            continue;
        };
        for row in &painel.rows {
            let ParamRow::Scalar(r) = row else { continue };
            total += 1;
            if (r.display.scale - 1.0).abs() > 1e-9 {
                com_escala += 1;
                nomes.push(format!("{nome}::{}", r.name));
            }
            if !r.display.suffix.is_empty() {
                com_sufixo += 1;
            }
        }
    }
    ph2d_panel_motion_graph::set_graph_selection(Vec::new());
    eprintln!(
        "\n  {total} rows escalares · {com_escala} com ESCALA de face · {com_sufixo} com SUFIXO"
    );
    for n in nomes.iter().take(20) {
        eprintln!("    {n}");
    }
}
