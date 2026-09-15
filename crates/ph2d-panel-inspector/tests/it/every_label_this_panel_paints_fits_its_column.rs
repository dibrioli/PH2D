//! ⭐⭐⭐ **UM RÓTULO QUE NÃO CABE NA COLUNA DELE É UM RÓTULO CORTADO.**
//!
//! ⛔⛔ **Report do dono, 2026-09-14:** *«Label acima do campo numérico! Muito ruim!»*, e a cura
//! (o rótulo à esquerda, numa coluna que é uma FRACÇÃO da linha) trouxe a pergunta seguinte: *cabe?*
//!
//! Medido nesse dia com o sistema de texto REAL, à largura de omissão do Inspector:
//!
//! | | rótulos cortados |
//! |---|---|
//! | com a unidade no rótulo (`"Float Height (m)"`) | **20 de 39** |
//! | com a unidade no CAMPO (`"Float Height"` + chip `m`) | **1** |
//!
//! O `"Float Height (m)"` mede `92,1 px` numa coluna de `91,2` — ele perdia o `(m)` **e** o `t` do
//! *Height*. ⇒ a unidade mudou-se para dentro do campo
//! ([`sections::rows::num_row_unit`](../../src/sections/rows.rs)), que é onde ela é lida: ao lado
//! do valor.
//!
//! # ⚠️ A coluna é DERIVADA, não escrita
//!
//! `inspector-w` − 2×`panel-head-pad` − 2×`Spacing::Sm` (o recuo do cartão) é a largura real de uma
//! linha, e a coluna sai da porta [`ph2d_editor_core::widget::property_label_col_w`]. ⛔ Um número
//! escrito aqui mediria uma coluna que o produto já não tem.
//!
//! # ⛔ Por que a barra é a coluna e não «a coluna mais folga»
//!
//! A elisão é **correcta** — a coluna docada é arrastável, e um rótulo tem sempre de poder cortar.
//! O que este gate afirma é outra coisa: *à largura de OMISSÃO, o artista não devia ver nenhum*.

use ph2d_text::TextSystem;
use ph2d_tokens::{Spacing, TypeToken};

/// ⭐⭐⭐ **VAZIO — e foi a decisão de APARÊNCIA do dono que o esvaziou.**
///
/// Esta lista nasceu com **12** entradas, cada uma um nome comprido que era elidido numa coluna de
/// `84,2 px`, e eu devolvi-lhe a escolha: encurtar os nomes, ou dar ao rótulo uma fatia maior da
/// linha. Ele escolheu a segunda por outra razão — *«as caixas numéricas são muito grandes. Maiores
/// que as labels»* — e a coluna passou a `120 px`, onde o mais comprido do app
/// (*«Swim Line (weights)»*, `113,9`) **cabe**.
///
/// ⚠️ ***Uma decisão de aparência do dono resolveu, de graça, o item que eu lhe tinha devolvido como
/// escolha.*** ⛔ E a lista fica aqui, vazia: qualquer rótulo novo que não caiba reprova o gate.
const AINDA_CORTAM: &[&str] = &[];

/// A largura real de uma linha de card do Inspector, à largura de omissão do painel.
fn largura_de_uma_linha() -> f32 {
    let painel = ph2d_tokens::INSPECTOR_W_PX - 2.0 * ph2d_tokens::PANEL_HEAD_PAD_PX;
    // O `card_frame` recua `Spacing::Sm` de cada lado antes de pintar as rows.
    painel - 2.0 * Spacing::Sm.px()
}

#[test]
fn every_label_this_panel_paints_fits_its_column() {
    let rotulos = ph2d_panel_inspector::player_row_labels();
    // ⚠️ Piso de população: uma lista vazia passa trivialmente.
    assert!(
        rotulos.len() >= 40,
        "a §14 tem {} rotulos — a tabela encolheu?",
        rotulos.len()
    );
    let coluna = ph2d_editor_core::widget::property_label_col_w(0.0, largura_de_uma_linha());
    let fonte = TypeToken::Sm.px();
    let mut ts = TextSystem::new();
    let mut cortados = Vec::new();
    for r in &rotulos {
        let largura = ts.prefix_width(r, fonte);
        if largura > coluna && !AINDA_CORTAM.contains(r) {
            cortados.push(format!(
                "{r:?} mede {largura:.1} px numa coluna de {coluna:.1}"
            ));
        }
    }
    assert!(
        cortados.is_empty(),
        "{} rotulo(s) da §14 nao cabem na coluna deles a' largura de omissao:\n  {}\n\n\
         A unidade fisica vive no CAMPO desde 2026-09-14 (`num_row_unit`) — um rotulo que a \
         carregue outra vez volta a ser cortado.",
        cortados.len(),
        cortados.join("\n  ")
    );
}

/// ⚠️ **A metade de OBSOLESCÊNCIA** (`CLAUDE.md` §5.0) — uma tolerância que já não descreve nada
/// sai da lista.
#[test]
fn the_elision_tolerance_still_describes_something() {
    let rotulos = ph2d_panel_inspector::player_row_labels();
    let coluna = ph2d_editor_core::widget::property_label_col_w(0.0, largura_de_uma_linha());
    let fonte = TypeToken::Sm.px();
    let mut ts = TextSystem::new();
    let mortas: Vec<&&str> = AINDA_CORTAM
        .iter()
        .filter(|d| {
            !rotulos
                .iter()
                .any(|r| r == *d && ts.prefix_width(r, fonte) > coluna)
        })
        .collect();
    assert!(
        mortas.is_empty(),
        "entrada(s) STALE na tolerancia — o rotulo sumiu ou ja' cabe:\n  {mortas:?}"
    );
}
