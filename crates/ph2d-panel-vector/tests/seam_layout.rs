//! Seam do **AUTO LAYOUT** (plano UI/UX W2, ADR-0153) — os chips estão vivos sob o MOUSE, os
//! campos existem, e o que **não se aplica não é pintado**.
//!
//! O gesto é REAL (Down+Up sobre o retângulo que o painel pintou), e não um `WidgetEvent::Click`
//! sintético: o sintético prova a allowlist do painel mas **pula a checagem de focabilidade no
//! store** — a lacuna que já deixou as 36 células da matriz de física e os dez chips de ferramenta
//! do Painter *pintados, hit-registrados e mortos sob o ponteiro*.

use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::interaction::WidgetEvent;
use ph2d_editor_core::tool::PanelEvent;
use ph2d_editor_core::zones::Rect;
use ph2d_host::{PointerButton, PointerEvent, PointerKind, PointerSource};
use ph2d_panel_vector::state::{LayoutFlow, LayoutItem, VectorPanelState};
use ph2d_panel_vector::{VectorPanel, ids, state};
use ph2d_ui_testkit::MockPanelHost;

const VIEWPORT: Rect = Rect {
    x: 0.0,
    y: 0.0,
    w: 1600.0,
    h: 900.0,
};
const SEC: u128 = 1_000_000_000;

fn pointer(kind: PointerKind, x: f32, y: f32, t: u128) -> PointerEvent {
    PointerEvent {
        kind,
        x,
        y,
        button: PointerButton::Primary,
        source: PointerSource::Mouse,
        pressure: 1.0,
        timestamp_ns: t,
    }
}

/// Um fluxo em LINHA, tudo no neutro — a publicação que faz a seção existir.
fn row_flow() -> LayoutFlow {
    LayoutFlow {
        dir: ids::VECTOR_LAYOUT_DIR_ROW,
        gap: [0.0, 0.0],
        pad: [0.0; 4],
        align: ids::VECTOR_LAYOUT_ALIGN_START,
        justify: ids::VECTOR_LAYOUT_JUSTIFY_START,
    }
}

fn clear() {
    state::set_frame_clip(None);
    state::set_layout_flow(None);
    state::set_layout_item(None);
}

/// Clica de verdade no widget `id` e exige que o Click chegue ao barramento.
fn click_reaches_bus(id: ph2d_a11y::NodeId, what: &str) {
    let mut host = MockPanelHost::with_panel::<VectorPanel>();
    let mut panel_state = VectorPanelState;
    let r = host
        .painted_rect::<VectorPanel>(&mut panel_state, VIEWPORT, id)
        .unwrap_or_else(|| panic!("{what} nao foi PINTADO com area clicavel"));
    let (cx, cy) = (r.x + r.w * 0.5, r.y + r.h * 0.5);
    host.dispatch_pointer_event(pointer(PointerKind::Down, cx, cy, SEC));
    let evs = host.dispatch_pointer_event(pointer(PointerKind::Up, cx, cy, SEC + SEC / 100));
    assert!(
        evs.iter()
            .any(|e| matches!(e, WidgetEvent::Click(c) if *c == id)),
        "o ponteiro sobre {what} nao virou Click — ele esta' desenhado e nao existe para o \
         dispatcher (falta o `register` no populate)"
    );
    for ev in evs {
        host.apply_panel_event::<VectorPanel>(&mut panel_state, ev);
    }
    assert!(
        host.drained_actions().into_iter().any(|a| matches!(
            a,
            EditorAction::ToolPanelEvent(PanelEvent::Click(c)) if c == id
        )),
        "o Click de {what} nao chegou ao bus — ele acende sob o mouse e nao faz nada (falta a \
         linha na allowlist do event_clicks)"
    );
}

/// O widget `id` foi pintado com área clicável?
fn painted(id: ph2d_a11y::NodeId) -> bool {
    rect(id).is_some()
}

/// O retângulo em que o painel pintou `id`.
fn rect(id: ph2d_a11y::NodeId) -> Option<Rect> {
    let mut host = MockPanelHost::with_panel::<VectorPanel>();
    let mut panel_state = VectorPanelState;
    host.painted_rect::<VectorPanel>(&mut panel_state, VIEWPORT, id)
}

fn rect_of(id: ph2d_a11y::NodeId, what: &str) -> Rect {
    rect(id).unwrap_or_else(|| panic!("{what} nao foi pintado"))
}

/// A calha de rótulo que a versão anterior dava a TODO campo: um caractere (`Spacing::Md`) mais
/// o vão até o campo (`Spacing::Xs`). Literal aqui de propósito — é o número do DEFEITO, e o
/// gate tem de continuar a falhar se alguém reintroduzir a constante com outro nome.
const OLD_FIXED_GUTTER_PX: f32 = 8.0 + 4.0;

/// **O REPRO da label sobreposta** (Enio 2026-08-02: *"caixas de input numérico grande e label
/// sobreposta"*).
///
/// A calha era fixa em oito pixels — um caractere —, então `paint_text` recortava "Gap" em "G"
/// e o campo era desenhado por cima do resto. Nasceu VERMELHO em exactamente `12.0`.
#[test]
fn a_multi_letter_label_gets_more_room_than_one_character() {
    clear();
    state::set_frame_clip(Some(true));
    state::set_layout_flow(Some(row_flow()));
    let inner_x = rect_of(ids::VECTOR_LAYOUT_DIR_OFF, "o chip Off").x;
    let gap = rect_of(ids::VECTOR_LAYOUT_GAP_MAIN, "o campo Gap");
    assert!(
        gap.x - inner_x > OLD_FIXED_GUTTER_PX,
        "a calha do rotulo 'Gap' e' de {:.1} px — o campo comeca em cima do proprio rotulo",
        gap.x - inner_x
    );
    clear();
}

/// **E a calha SEGUE o rótulo** — um rótulo mais largo empurra o campo mais para a direita.
///
/// ⚠️ É a metade que distingue *medir* de *escolher uma constante maior*: com um número fixo
/// (qualquer que seja) os dois campos começariam no MESMO x, e o próximo rótulo mais longo que
/// ele voltaria a ser recortado.
#[test]
fn a_wider_label_pushes_its_field_further_right() {
    clear();
    state::set_frame_clip(Some(true));
    state::set_layout_flow(Some(row_flow()));
    state::set_layout_item(Some(LayoutItem {
        grow: 0.0,
        shrink: 1.0,
    }));
    // "Gap" e "Grow" são os dois campos da coluna ESQUERDA — mesma origem de célula, então o x
    // deles é a calha e mais nada.
    let gap = rect_of(ids::VECTOR_LAYOUT_GAP_MAIN, "o campo Gap");
    let grow = rect_of(ids::VECTOR_LAYOUT_ITEM_GROW, "o campo Grow");
    assert!(
        grow.x > gap.x,
        "'Grow' e 'Gap' comecam no mesmo x ({:.1}) — a calha e' uma constante, nao a medida",
        gap.x
    );
    clear();
}

/// **Um campo SOZINHO ocupa meia largura, não a linha inteira** (a outra metade do report: o
/// *"input numérico grande"*).
///
/// O oráculo é a borda direita do painel, lida do último chip da fileira de direção — nenhum
/// número escrito à mão, então ele continua certo se a largura do painel mudar.
#[test]
fn a_lone_number_field_sits_in_half_the_row() {
    clear();
    state::set_frame_clip(Some(true));
    state::set_layout_flow(Some(row_flow()));
    let wrap = rect_of(ids::VECTOR_LAYOUT_DIR_WRAP, "o chip Wrap");
    let inner_right = wrap.x + wrap.w;
    let gap = rect_of(ids::VECTOR_LAYOUT_GAP_MAIN, "o campo Gap");
    let right = gap.x + gap.w;
    assert!(
        right < inner_right - gap.w * 0.5,
        "o campo Gap vai ate' {right:.1} e a linha acaba em {inner_right:.1} — ele tomou a \
         largura inteira do painel"
    );
    clear();
}

/// **Os quatro chips de DIREÇÃO estão vivos numa moldura**, incluindo o Off.
#[test]
fn the_four_direction_chips_are_reachable_and_reach_the_bus() {
    clear();
    state::set_frame_clip(Some(true));
    for (id, what) in [
        (ids::VECTOR_LAYOUT_DIR_OFF, "o chip Off"),
        (ids::VECTOR_LAYOUT_DIR_ROW, "o chip Row"),
        (ids::VECTOR_LAYOUT_DIR_COL, "o chip Column"),
        (ids::VECTOR_LAYOUT_DIR_WRAP, "o chip Wrap"),
    ] {
        click_reaches_bus(id, what);
    }
    clear();
}

/// **Os nove chips de alinhamento/distribuição estão vivos com a moldura FLUINDO.**
#[test]
fn the_alignment_chips_are_reachable_and_reach_the_bus() {
    clear();
    state::set_frame_clip(Some(true));
    state::set_layout_flow(Some(row_flow()));
    for id in [
        ids::VECTOR_LAYOUT_ALIGN_START,
        ids::VECTOR_LAYOUT_ALIGN_CENTER,
        ids::VECTOR_LAYOUT_ALIGN_END,
        ids::VECTOR_LAYOUT_ALIGN_STRETCH,
        ids::VECTOR_LAYOUT_JUSTIFY_START,
        ids::VECTOR_LAYOUT_JUSTIFY_CENTER,
        ids::VECTOR_LAYOUT_JUSTIFY_END,
        ids::VECTOR_LAYOUT_JUSTIFY_BETWEEN,
        ids::VECTOR_LAYOUT_JUSTIFY_AROUND,
    ] {
        click_reaches_bus(id, "um chip de alinhamento");
    }
    clear();
}

/// **Com a moldura PARADA só a fileira de direção é pintada.**
///
/// ⚠️ É a metade que impede cinco controles que não mudam um pixel: vão, recuo, alinhamento e
/// distribuição sobre uma moldura que não empilha não têm o que fazer.
#[test]
fn a_frame_that_does_not_flow_paints_only_the_direction_row() {
    clear();
    state::set_frame_clip(Some(true));
    assert!(
        painted(ids::VECTOR_LAYOUT_DIR_OFF),
        "a direcao e' oferecida"
    );
    for id in [
        ids::VECTOR_LAYOUT_GAP_MAIN,
        ids::VECTOR_LAYOUT_PAD_ALL,
        ids::VECTOR_LAYOUT_ALIGN_START,
        ids::VECTOR_LAYOUT_JUSTIFY_START,
    ] {
        assert!(
            !painted(id),
            "um controle de fluxo foi pintado numa moldura que nao flui"
        );
    }
    clear();
}

/// **O par All/Each TROCA os campos pintados** — nunca quatro campos espelhando um número.
#[test]
fn the_padding_mode_swaps_which_fields_are_painted() {
    clear();
    state::set_frame_clip(Some(true));
    state::set_layout_flow(Some(row_flow()));

    // Modo *All* (o default): um campo, e os quatro lados NÃO estão na tela.
    assert!(painted(ids::VECTOR_LAYOUT_PAD_ALL));
    assert!(!painted(ids::VECTOR_LAYOUT_PAD_T));

    // O chip Each é panel-local: ele muda o que é pintado sem passar pela shell.
    let mut host = MockPanelHost::with_panel::<VectorPanel>();
    let mut panel_state = VectorPanelState;
    let r = host
        .painted_rect::<VectorPanel>(&mut panel_state, VIEWPORT, ids::VECTOR_LAYOUT_PAD_EACH_MODE)
        .expect("o chip Each e' pintado");
    let (cx, cy) = (r.x + r.w * 0.5, r.y + r.h * 0.5);
    host.dispatch_pointer_event(pointer(PointerKind::Down, cx, cy, SEC));
    let evs = host.dispatch_pointer_event(pointer(PointerKind::Up, cx, cy, SEC + SEC / 100));
    assert!(
        evs.iter()
            .any(|e| matches!(e, WidgetEvent::Click(c) if *c == ids::VECTOR_LAYOUT_PAD_EACH_MODE)),
        "o chip Each esta' morto sob o mouse"
    );
    for ev in evs {
        host.apply_panel_event::<VectorPanel>(&mut panel_state, ev);
    }

    for id in [
        ids::VECTOR_LAYOUT_PAD_T,
        ids::VECTOR_LAYOUT_PAD_R,
        ids::VECTOR_LAYOUT_PAD_B,
        ids::VECTOR_LAYOUT_PAD_L,
    ] {
        assert!(painted(id), "os quatro lados tem de aparecer no modo Each");
    }
    assert!(
        !painted(ids::VECTOR_LAYOUT_PAD_ALL),
        "o campo unico tem de SAIR — dois campos para o mesmo numero nao dizem em qual se digita"
    );

    // Volta ao *All*, senão o modo vaza para os outros gates deste binário (é thread-local).
    let mut host = MockPanelHost::with_panel::<VectorPanel>();
    let r = host
        .painted_rect::<VectorPanel>(&mut panel_state, VIEWPORT, ids::VECTOR_LAYOUT_PAD_ALL_MODE)
        .expect("o chip All e' pintado");
    let (cx, cy) = (r.x + r.w * 0.5, r.y + r.h * 0.5);
    host.dispatch_pointer_event(pointer(PointerKind::Down, cx, cy, SEC));
    let evs = host.dispatch_pointer_event(pointer(PointerKind::Up, cx, cy, SEC + SEC / 100));
    for ev in evs {
        host.apply_panel_event::<VectorPanel>(&mut panel_state, ev);
    }
    clear();
}

/// **O vão TRANSVERSAL só existe no `Wrap`** — em linha ou coluna há uma faixa só, e não há entre
/// o que ele ficaria.
#[test]
fn the_cross_gap_is_born_with_the_mode_that_uses_it() {
    clear();
    state::set_frame_clip(Some(true));
    state::set_layout_flow(Some(row_flow()));
    assert!(painted(ids::VECTOR_LAYOUT_GAP_MAIN));
    assert!(
        !painted(ids::VECTOR_LAYOUT_GAP_CROSS),
        "o vao entre FAIXAS foi pintado numa linha unica"
    );

    state::set_layout_flow(Some(LayoutFlow {
        dir: ids::VECTOR_LAYOUT_DIR_WRAP,
        ..row_flow()
    }));
    assert!(painted(ids::VECTOR_LAYOUT_GAP_CROSS));
    clear();
}

/// **Grow/Shrink seguem o FILHO, não a moldura** — e os dois blocos COEXISTEM.
#[test]
fn the_item_rows_follow_the_child_and_coexist_with_the_frame_block() {
    clear();
    state::set_frame_clip(Some(true));
    state::set_layout_flow(Some(row_flow()));
    assert!(
        !painted(ids::VECTOR_LAYOUT_ITEM_GROW),
        "Grow apareceu sem o filho estar num fluxo"
    );

    state::set_layout_item(Some(LayoutItem {
        grow: 0.0,
        shrink: 0.0,
    }));
    assert!(painted(ids::VECTOR_LAYOUT_ITEM_GROW));
    assert!(painted(ids::VECTOR_LAYOUT_ITEM_SHRINK));
    // A moldura ANINHADA: os dois blocos ao mesmo tempo.
    assert!(painted(ids::VECTOR_LAYOUT_DIR_ROW));
    clear();
}

/// **Sem moldura E sem filho de fluxo a seção não existe.**
#[test]
fn the_layout_section_is_absent_without_a_subject() {
    clear();
    for id in [
        ids::VECTOR_LAYOUT_DIR_OFF,
        ids::VECTOR_LAYOUT_DIR_ROW,
        ids::VECTOR_LAYOUT_GAP_MAIN,
        ids::VECTOR_LAYOUT_ITEM_GROW,
    ] {
        assert!(
            !painted(id),
            "a secao Layout foi pintada sem moldura nem filho de fluxo"
        );
    }
}

/// **Só o filho selecionado: a seção existe com o bloco de ITEM sozinho.**
///
/// ⚠️ Sem esta metade o artista que seleciona uma forma dentro de uma moldura não teria onde
/// escrever o Grow — a moldura não está selecionada, então o outro bloco não aparece.
#[test]
fn a_selected_child_alone_still_gets_its_two_rows() {
    clear();
    state::set_layout_item(Some(LayoutItem {
        grow: 1.0,
        shrink: 0.0,
    }));
    assert!(painted(ids::VECTOR_LAYOUT_ITEM_GROW));
    assert!(
        !painted(ids::VECTOR_LAYOUT_DIR_OFF),
        "o bloco da MOLDURA nao pode aparecer sem moldura selecionada"
    );
    clear();
}
