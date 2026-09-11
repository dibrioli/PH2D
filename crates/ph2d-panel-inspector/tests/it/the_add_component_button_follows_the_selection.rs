//! ⭐ **O `+` do cabeçalho aparece se, e só se, houver um objeto sob o Inspector — e é ELE que
//! ganha o clique ali** (ADR-0166 / F3).
//!
//! # ⚠️ Este ficheiro já esteve verde sobre um botão MORTO
//!
//! A 1.ª versão perguntava *"o id foi registado no índice?"*, e a resposta era **sim** — o `+` era
//! registado no `paint_head`, e depois a **alça de arrasto** do painel, registada no fim do quadro,
//! cobria a banda do título e **ganhava-lhe** no passeio back-to-front do `HitIndex`. O botão
//! pintava-se, acendia sob o rato, e clicar nele não fazia nada. O Enio apanhou-o no 1.º smoke.
//!
//! ⇒ **A pergunta certa não é «foi registado», é «QUEM ganha o clique naquele ponto»** — que é a
//! mesma lição que a costura da booleana do vetor pagou duas vezes ([27 §8] do Vector Module):
//! *um `Click` sintético passa com o chip morto; só o gesto real mede a segunda costura*.
//!
//! O X (`INSP_CLOSE`) sobrevive à alça porque é **re-registado depois dela** — e a nota que o diz
//! está no `paint.rs` desde 2026-05-24. O `+` nasceu sem esse re-registo.

use ph2d_editor_core::ids;
use ph2d_editor_core::interaction::HitIndex;
use ph2d_editor_core::screens::hero::InspectorTransformInfo;
use ph2d_editor_core::zones::Rect;
use ph2d_panel_inspector::{InspectorPanel, InspectorState, set_current_inspector_transform};
use ph2d_ui_testkit::MockPanelHost;

const VIEWPORT: Rect = Rect {
    x: 0.0,
    y: 0.0,
    w: 1600.0,
    h: 900.0,
};

fn transform() -> InspectorTransformInfo {
    InspectorTransformInfo {
        entity_bits: 0xABCD_1234,
        translation: [0.0, 0.0],
        rotation_rad: 0.0,
        scale: [1.0, 1.0],
        skew_rad: [0.0, 0.0],
    }
}

/// Pinta o painel e devolve **quem ganha o clique** no centro do `+`, mais a lista de rects.
///
/// ⚠️ O vencedor sai de um `HitIndex` reconstruído **na ordem em que o painel regista**, e
/// perguntado pelo `hit(x, y)` — o mesmo que o despacho real usa. Contar registos não responde.
fn winner_at_the_plus(sel: Option<InspectorTransformInfo>) -> (Option<ph2d_a11y::NodeId>, bool) {
    let mut host = MockPanelHost::with_panel::<InspectorPanel>();
    let mut state = InspectorState::default();
    set_current_inspector_transform(sel);
    let rects = host.paint::<InspectorPanel>(&mut state, VIEWPORT);
    set_current_inspector_transform(None);

    let registered = rects.iter().any(|(n, _)| *n == ids::INSP_ADD_COMPONENT);
    let Some((_, r)) = rects
        .iter()
        .find(|(n, _)| *n == ids::INSP_ADD_COMPONENT)
        .copied()
    else {
        return (None, registered);
    };
    let mut hits = HitIndex::new();
    for (id, rect) in &rects {
        hits.register(*id, *rect);
    }
    (hits.hit(r.x + r.w * 0.5, r.y + r.h * 0.5), registered)
}

/// ⭐ **Com objeto: o `+` está lá E ganha o clique.**
///
/// (Mutação: tirar o re-registo do fim do `paint.rs` ⇒ o vencedor passa a ser a alça de arrasto,
/// e a mensagem NOMEIA quem roubou o clique — que é o que faltava na 1.ª versão deste gate.)
#[test]
fn the_plus_wins_the_click_over_the_drag_handle() {
    let (winner, registered) = winner_at_the_plus(Some(transform()));
    assert!(
        registered,
        "com um objeto selecionado o + tem de estar la' — senao nao ha' porta para anexar"
    );
    assert_eq!(
        winner,
        Some(ids::INSP_ADD_COMPONENT),
        "o + esta' pintado e MORTO sob o dedo: quem ganha o clique no centro dele e' {winner:?} \
         (a alca de arrasto do painel regista-se no fim do quadro e cobre a banda do titulo — o X \
         sobrevive porque e' RE-REGISTADO depois dela, e o + tem de o ser tambem)"
    );
}

/// ⚠️ **E a metade de AUSÊNCIA:** sem seleção não há a quem anexar, o handler recusa o clique, e um
/// botão que se pinta e não faz nada é o defeito que esta fase inteira existe para apagar. A
/// resposta certa é ele **não estar lá** — nem pintado, nem no índice.
#[test]
fn without_a_selection_the_plus_does_not_exist_at_all() {
    let (winner, registered) = winner_at_the_plus(None);
    assert!(
        !registered,
        "sem selecao o + foi registado no indice: um alvo clicavel sobre um botao que ninguem pintou"
    );
    assert_ne!(winner, Some(ids::INSP_ADD_COMPONENT));
}
