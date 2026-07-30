//! **Toda row que a fórmula dobra tem um widget** (FASE C.1 do plano 12) — o U2 da
//! auditoria de 2026-07-29, que é o defeito por trás de *"não tem scroll nem barra de
//! scroll"* e do screenshot com `+1 more rows`.
//!
//! **O mecanismo medido:** `BODY_SLOTS = 12`, uma row custava `1 + knobs`, e quando
//! `used + need > BODY_SLOTS` o pintor imprimia `+N more rows` e fazia **`return`**. Com 4
//! rows de Turbulence as rows 0-1 eram pintadas e as **rows 2-3 ficavam com ZERO widgets**
//! — sem hit rect, sem store, sem clique — *enquanto a fórmula que o objeto roda continha
//! as quatro*. Uma `Fade by Distance` sozinha comia **9 dos 12 slots**.
//!
//! ⚠️ **Por que os gates de row eram todos verdes:** todos usam UMA ou DUAS rows. A fixture
//! não continha o fenômeno, e não havia como ela conter por acidente.
//!
//! **A cura não foi scroll**, e a medição é o argumento: toda row de knob tinha **128 px
//! mortos** (40% do sheet — `ctrl_w` computado como 168 e descartado no braço numérico).
//! Dois knobs numéricos por linha levam o pior caso do catálogo de **5 slots para 3** — 4
//! rows garantidas, 6 no caso típico — sem um 2º eixo de scroll dentro de um painel que já
//! rola.

use ph2d_editor_core::ids;
use ph2d_editor_core::interaction::{ContextMenuKind, ContextMenuRequest, WidgetEvent};
use ph2d_editor_core::panel::PanelHostInternal;
use ph2d_editor_core::zones::Rect;
use ph2d_panel_timeline::TimelinePanel;
use ph2d_panel_timeline::state::TimelinePanelState;
use ph2d_ui_testkit::MockPanelHost;

const VIEWPORT: Rect = Rect {
    x: 0.0,
    y: 0.0,
    w: 1600.0,
    h: 900.0,
};

fn publish_one_track(entity: u64, prop: ph2d_timeline::PropKind) -> u64 {
    use ph2d_timeline::{TimelineIntent, TimelineState, TimelineViewSnapshot, apply_intent};
    let mut st = TimelineState::new();
    let mut ph = ph2d_core::Playhead::new(1.0 / 60.0);
    apply_intent(&mut st, &mut ph, TimelineIntent::Bind { entity, prop });
    let target = st.doc.binding_for(entity, prop).unwrap().target.get();
    let mut snap = TimelineViewSnapshot::default();
    snap.rebuild(&mut st, &ph, false);
    ph2d_panel_timeline::set_current_timeline(Some(snap));
    target
}

/// Abre o card e empilha `ids`, pela porta que o artista usa (a galeria).
fn card_with(host: &mut MockPanelHost, state: &mut TimelinePanelState, target: u64, ids_: &[&str]) {
    host.store_mut().open_context_menu(ContextMenuRequest {
        x: 40.0,
        y: 50.0,
        kind: ContextMenuKind::TimelineTrack { target },
    });
    host.store_mut().close_context_menu();
    host.apply_panel_event::<TimelinePanel>(state, WidgetEvent::Click(ids::CTX_MENU_TL_EXPR));
    for id in ids_ {
        host.apply_panel_event::<TimelinePanel>(
            state,
            WidgetEvent::Click(ids::expr_gallery_id(id)),
        );
    }
}

/// **Toda row do stack tem widgets pintados — e a fórmula contém todas elas.**
///
/// A fixture usa **QUATRO** rows da receita mais cara que o catálogo tem, que é exactamente
/// a forma que os gates de uma-ou-duas-rows não podiam alcançar.
#[test]
fn every_row_the_formula_folds_has_a_widget() {
    let target = publish_one_track(31, ph2d_timeline::PropKind::TranslationX);
    let mut host = MockPanelHost::with_panel::<TimelinePanel>();
    let mut state = TimelinePanelState::default();
    // `shake` tem 4 knobs (absorveu o Detail/Roughness do turbulence) — a receita mais
    // cara do catálogo, e a que o report do `+1 more rows` usava.
    card_with(
        &mut host,
        &mut state,
        target,
        &["shake", "shake", "shake", "shake"],
    );
    let m = state.expr_modal.as_ref().expect("o card abriu");
    assert_eq!(m.stack.rows.len(), 4, "PREMISSA: quatro rows no stack");
    let formula = m.stack.to_formula();
    assert_eq!(
        formula.matches("wiggle").count(),
        4,
        "PREMISSA: a fórmula que o objeto roda contém as QUATRO rows: {formula}"
    );

    let regs = host.paint::<TimelinePanel>(&mut state, VIEWPORT);
    for ri in 0..4 {
        // O X de remover é o widget mínimo: sem ele a row é inalcançável, e a única saída
        // do artista é apagar uma row ACIMA dela.
        let rm = ids::expr_remove_id(ri);
        assert!(
            regs.iter().any(|(id, _)| *id == rm),
            "a row {ri} dirige o objeto e não tem um pixel de UI — o `+N more rows` \
             deixava-a assim, e a fórmula continha-a"
        );
        // ...e os knobs dela também.
        for ki in 0..4 {
            let k = ids::expr_knob_id(ri, ki);
            assert!(
                regs.iter().any(|(id, _)| *id == k),
                "o knob {ki} da row {ri} não foi pintado"
            );
        }
    }
}

/// **Os knobs numéricos vêm em DUAS colunas, e a linha não tem mais 128 px mortos.**
///
/// O oráculo é a GEOMETRIA que o paint registrou: dois knobs na mesma faixa de `y` e em `x`
/// diferentes é o que "duas colunas" significa, e nenhuma outra afirmação sobre o layout
/// pode ser feita sem olhar os retângulos.
///
/// **Mutação que deve sangrar:** `knob_slot` devolvendo sempre `col = 0`.
#[test]
fn two_numeric_knobs_share_a_line() {
    let target = publish_one_track(32, ph2d_timeline::PropKind::TranslationY);
    let mut host = MockPanelHost::with_panel::<TimelinePanel>();
    let mut state = TimelinePanelState::default();
    card_with(&mut host, &mut state, target, &["shake"]);
    let regs = host.paint::<TimelinePanel>(&mut state, VIEWPORT);

    let rect_of = |ki: usize| {
        regs.iter()
            .find(|(id, _)| *id == ids::expr_knob_id(0, ki))
            .map(|(_, r)| *r)
            .unwrap_or_else(|| panic!("o knob {ki} não foi pintado"))
    };
    let (a, b) = (rect_of(0), rect_of(1));
    assert_eq!(
        a.y, b.y,
        "os dois primeiros knobs numéricos do Shake têm de dividir a LINHA"
    );
    assert!(
        b.x > a.x + a.w,
        "e sentar em colunas separadas, sem sobrepor: {a:?} vs {b:?}"
    );
    // A segunda coluna tem de usar de fato a metade direita: os 128 px mortos eram lá.
    let sheet_right = a.x + (b.x - a.x) * 2.0;
    assert!(
        b.x + b.w > sheet_right - 100.0,
        "a coluna da direita tem de alcançar a borda do sheet — o defeito era 128 px \
         mortos exactamente aí ({:?})",
        b
    );
    // ...e a 2ª LINHA de knobs desce (o Shake tem 4 knobs = 2 linhas).
    let c = rect_of(2);
    assert!(
        c.y > a.y,
        "o terceiro knob abre uma linha nova: {a:?} vs {c:?}"
    );
    assert_eq!(c.x, a.x, "e volta para a coluna da esquerda");
}

/// **Um knob de TEXTO fica sozinho na linha.**
///
/// ⚠️ A metade que impede a cura de virar o defeito: parear dois campos de texto de 72 px
/// seria trocar um aperto por outro, e um `Link` carrega um NOME (`Ball.x`) enquanto um
/// `Text` carrega uma fórmula.
///
/// **Mutação que deve sangrar:** tratar `Link`/`Text` como numérico em `knob_slot`.
#[test]
fn a_text_knob_gets_the_whole_line() {
    let target = publish_one_track(33, ph2d_timeline::PropKind::TranslationX);
    let mut host = MockPanelHost::with_panel::<TimelinePanel>();
    let mut state = TimelinePanelState::default();
    // `follow` = 1 Link + 2 numéricos.
    card_with(&mut host, &mut state, target, &["follow"]);
    let regs = host.paint::<TimelinePanel>(&mut state, VIEWPORT);
    let rect_of = |ki: usize| {
        regs.iter()
            .find(|(id, _)| *id == ids::expr_knob_id(0, ki))
            .map(|(_, r)| *r)
            .unwrap_or_else(|| panic!("o knob {ki} não foi pintado"))
    };
    let (link, mult, off) = (rect_of(0), rect_of(1), rect_of(2));
    assert!(
        mult.y > link.y,
        "o Target é um link: ninguém divide a linha com ele ({link:?} vs {mult:?})"
    );
    assert_eq!(
        mult.y, off.y,
        "e os dois numéricos que sobram dividem a próxima"
    );
    assert!(
        link.w > mult.w,
        "o campo de link é mais largo que uma caixa de coluna: {} vs {}",
        link.w,
        mult.w
    );
}
