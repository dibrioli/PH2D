//! **As fileiras de refluxo não são oferecidas a um texto que cavalga um caminho.**
//!
//! ## O defeito, medido em 2026-08-30
//!
//! O motor descarta o `wrap_width` sempre que o texto não está pousado num ponto — a guarda está
//! escrita em `shells/desktop/src/vec_glyph.rs::wrapped_lines`:
//!
//! ```text
//! Some(w) if w > 0.0 && matches!(placement, TextPlacement::At(_)) => w,
//! _ => { out.extend(text.split('\n')); return out; }
//! ```
//!
//! Num `TextPlacement::OnPath` o texto segue o arco e o número **não é lido por ninguém**. O
//! painel, porém, pintava `Width: Auto | Fixed` e o slider da largura à mesma: o artista escolhia
//! *Fixed*, arrastava, e nada se movia.
//!
//! ## Por que era invisível a todos os gates que já existiam
//!
//! O slider é declarado, pintado, registado, ligado ao chip e **encaminhado** — o clique chega à
//! ferramenta e o valor chega ao documento. O que não acontece é o passo seguinte: o consumidor
//! recebe-o e **descarta-o**. É a segunda espécie de knob morto que a caça de 2026-08-30 nomeou
//! (*o consumidor projecta o valor fora*), e nenhuma sonda de «quem lê este campo?» a vê — ele
//! **é** lido.
//!
//! ⚠️ A lei da cura já estava escrita neste módulo, para o BOTÃO de prender:
//! *«um botão que não faz nada é pior que um botão que falta, e este é o único jeito de o painel
//! saber a diferença»* (`state_textpath.rs`). Aqui ela vale para um slider.

use ph2d_editor_core::zones::Rect;
use ph2d_panel_vector::state::VectorPanelState;
use ph2d_panel_vector::{VectorPanel, ids};
use ph2d_ui_testkit::MockPanelHost;

const VIEWPORT: Rect = Rect {
    x: 0.0,
    y: 0.0,
    w: 1600.0,
    h: 4000.0,
};

/// Um texto seleccionado, com refluxo FIXO armado — o estado em que as fileiras existem.
fn publish(on_path: bool) {
    ph2d_panel_vector::set_current_text_visible(true);
    ph2d_panel_vector::set_current_text(Some("uma frase que refluiria".into()));
    ph2d_panel_vector::set_current_text_wrap(Some(200.0));
    ph2d_panel_vector::set_current_textpath(on_path, 0.25, false);
}

/// Os três ids que a secção de refluxo pinta.
const WRAP_ROWS: [(&str, ph2d_a11y::NodeId); 3] = [
    ("Width:Auto", ids::VECTOR_TEXT_WRAP_AUTO),
    ("Width:Fixed", ids::VECTOR_TEXT_WRAP_FIXED),
    ("Wrap width", ids::VECTOR_TEXT_WRAP_W),
];

/// **Controle POSITIVO: fora de um caminho, as fileiras existem.**
///
/// Sem ele, um painel que nunca pintasse o refluxo passaria no teste de baixo — e a secção
/// estaria morta em vez de correctamente escondida.
#[test]
fn a_free_standing_text_still_gets_its_wrap_rows() {
    publish(false);
    let mut host = MockPanelHost::with_panel::<VectorPanel>();
    let mut st = VectorPanelState;
    for (name, id) in WRAP_ROWS {
        assert!(
            host.painted_rect::<VectorPanel>(&mut st, VIEWPORT, id)
                .is_some(),
            "a fileira {name} sumiu para um texto que NÃO está num caminho — o refluxo funciona \
             aí, e esconder a fileira seria tirar uma feature que existe"
        );
    }
}

/// **Sobre um caminho, nenhuma delas é oferecida.**
///
/// Mutação que tem de sangrar: tirar o `if state::linked() { return y; }` de
/// `paint_text_sections::wrap_rows`.
#[test]
fn a_text_riding_a_path_is_not_offered_a_wrap_width() {
    publish(true);
    let mut host = MockPanelHost::with_panel::<VectorPanel>();
    let mut st = VectorPanelState;
    for (name, id) in WRAP_ROWS {
        assert!(
            host.painted_rect::<VectorPanel>(&mut st, VIEWPORT, id)
                .is_none(),
            "a fileira {name} foi pintada para um texto que cavalga um caminho. O motor descarta \
             o `wrap_width` fora de `TextPlacement::At` (`vec_glyph::wrapped_lines`), logo o \
             artista escolhe *Fixed*, arrasta a largura, e NADA se move."
        );
    }
}
