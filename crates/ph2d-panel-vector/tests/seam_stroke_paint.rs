//! Seam das duas fileiras da **wave D** do plano 35 — a tinta do TRAÇO (`Solid | Pattern`) e o
//! ALVO da secção *Pattern* (`Fill | Stroke`).
//!
//! O gesto é REAL (Down+Up sobre o rectângulo que o painel pintou), e não um `WidgetEvent::Click`
//! sintético: ⚠️ o sintético prova a allowlist do painel mas **pula a checagem de focabilidade no
//! store** — a lacuna que já deixou 36 células da matriz de física, dez chips do Painter e os
//! quatro chips da booleana *pintados, hit-registrados e mortos sob o ponteiro*.
//!
//! ⚠️⚠️ **As duas metades de cada gate são independentes**: sair do `populate` mata a primeira (o
//! ponteiro não vira Click), sair do `event_clicks` mata a segunda (o Click não chega ao bus). E a
//! terceira metade — a AUSÊNCIA — tem gate próprio: uma fileira que descreve o que não está lá é
//! pior que fileira nenhuma.

use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::interaction::WidgetEvent;
use ph2d_editor_core::tool::PanelEvent;
use ph2d_editor_core::zones::Rect;
use ph2d_host::{PointerButton, PointerEvent, PointerKind, PointerSource};
use ph2d_panel_vector::state::VectorPanelState;
use ph2d_panel_vector::{StrokePaintKind, VectorPanel, ids, state};
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

fn rect(id: ph2d_a11y::NodeId) -> Option<Rect> {
    let mut host = MockPanelHost::with_panel::<VectorPanel>();
    let mut st = VectorPanelState;
    host.painted_rect::<VectorPanel>(&mut st, VIEWPORT, id)
}

fn click_reaches_bus(id: ph2d_a11y::NodeId, what: &str) {
    let mut host = MockPanelHost::with_panel::<VectorPanel>();
    let mut st = VectorPanelState;
    let r = host
        .painted_rect::<VectorPanel>(&mut st, VIEWPORT, id)
        .unwrap_or_else(|| panic!("{what} nao foi PINTADO com area clicavel"));
    let (cx, cy) = (r.x + r.w * 0.5, r.y + r.h * 0.5);
    host.dispatch_pointer_event(pointer(PointerKind::Down, cx, cy, SEC));
    let evs = host.dispatch_pointer_event(pointer(PointerKind::Up, cx, cy, SEC + SEC / 100));
    assert!(
        evs.iter()
            .any(|e| matches!(e, WidgetEvent::Click(c) if *c == id)),
        "o ponteiro sobre {what} nao virou Click - ele esta' desenhado e nao existe para o \
         dispatcher (falta o `register` no populate)"
    );
    for ev in evs {
        host.apply_panel_event::<VectorPanel>(&mut st, ev);
    }
    assert!(
        host.drained_actions().into_iter().any(|a| matches!(
            a,
            EditorAction::ToolPanelEvent(PanelEvent::Click(c)) if c == id
        )),
        "o Click de {what} nao chegou ao bus - ele acende sob o rato e nao faz nada (falta a \
         linha na allowlist do event_clicks)"
    );
}

/// Repõe tudo o que estes gates publicam — o estado é `thread_local` e os testes partilham a thread
/// dentro de um binário.
fn limpa() {
    state::set_stroke_paint_kind(None);
    state::set_texpat_target_is_stroke(None);
    state::set_current_texture_pattern(None);
}

/// **Os dois chips da tinta do traço estão vivos, nos DOIS estados.**
///
/// ⚠️ Os dois valores são exercitados de propósito: o grupo segmentado ramifica no chip aceso, e um
/// gate que só publicasse `Solid` ficaria verde sobre um `Pattern` que nunca é pintado.
#[test]
fn both_stroke_paint_chips_are_reachable_and_reach_the_bus() {
    for k in [StrokePaintKind::Solid, StrokePaintKind::Pattern] {
        state::set_stroke_paint_kind(Some(k));
        click_reaches_bus(ids::VECTOR_STROKE_KIND_SOLID, "o chip Solid do traco");
        click_reaches_bus(ids::VECTOR_STROKE_KIND_PATTERN, "o chip Pattern do traco");
    }
    limpa();
}

/// ⚠️⚠️ **Sem traço a fileira NÃO existe** — e é isto que a distingue da caixa *Stroke* logo acima,
/// que tem uma resposta (`Some(false)`) para a MESMA forma.
///
/// *Sem traço não há tinta de traço a escolher.*
#[test]
fn the_stroke_type_row_is_absent_without_a_stroke() {
    limpa();
    for id in [
        ids::VECTOR_STROKE_KIND_SOLID,
        ids::VECTOR_STROKE_KIND_PATTERN,
    ] {
        assert!(
            rect(id).is_none(),
            "a fileira de tipo foi pintada sem tinta publicada - ela descreve o que nao esta' la'"
        );
    }
    // CONTROLO: a caixa *Stroke* continua a ter resposta para a mesma selecção, senão este gate
    // ficaria verde sobre um painel que perdeu as duas.
    state::set_stroke_present(Some(false));
    assert!(
        rect(ids::VECTOR_STROKE_PRESENT).is_some(),
        "a caixa Stroke desapareceu junto - sao duas perguntas diferentes"
    );
    state::set_stroke_present(None);
    limpa();
}

/// Uma lei de padrão qualquer — o conteúdo não importa aqui, só que a secção suba.
fn lei() -> ph2d_panel_vector::TexturePatternRow {
    ph2d_panel_vector::TexturePatternRow {
        kind: 0,
        offset_denom: 1.0,
        size: [1.0, 1.0],
        lock_aspect: true,
        gap: 0.0,
        angle_deg: 0.0,
        shift_pct: [0.0, 0.0],
        mode: 0,
    }
}

/// **Os dois chips do ALVO estão vivos, nos DOIS estados.**
#[test]
fn both_target_chips_are_reachable_and_reach_the_bus() {
    state::set_current_texture_pattern(Some(lei()));
    for no_traco in [false, true] {
        state::set_texpat_target_is_stroke(Some(no_traco));
        click_reaches_bus(ids::VECTOR_TEXPAT_TARGET_FILL, "o chip Fill do alvo");
        click_reaches_bus(ids::VECTOR_TEXPAT_TARGET_STROKE, "o chip Stroke do alvo");
    }
    limpa();
}

/// ⛔ **Com UM alvo só, a fileira não é pintada** — não há escolha a oferecer, e a secção edita o
/// que houver.
///
/// ⚠️ **O controle é a metade que importa**: as fileiras ABAIXO continuam lá. Sem ele, este gate
/// ficaria verde sobre uma secção que desapareceu inteira.
#[test]
fn the_target_row_is_absent_when_there_is_no_choice_to_offer() {
    state::set_current_texture_pattern(Some(lei()));
    state::set_texpat_target_is_stroke(None);
    for id in [
        ids::VECTOR_TEXPAT_TARGET_FILL,
        ids::VECTOR_TEXPAT_TARGET_STROKE,
    ] {
        assert!(
            rect(id).is_none(),
            "o chip do alvo foi pintado com um sujeito so' - ele nao tem escolha a oferecer"
        );
    }
    assert!(
        rect(ids::VECTOR_TEXPAT_SOURCE).is_some(),
        "a seccao inteira sumiu - o que devia sumir era so' a fileira do alvo"
    );
    limpa();
}

/// ⚠️ **E sem padrão nenhum a secção inteira não sobe** — inclusive a fileira do alvo, que de outro
/// modo seria um chip a escolher entre dois sujeitos inexistentes.
#[test]
fn no_pattern_means_no_section_at_all() {
    limpa();
    state::set_texpat_target_is_stroke(Some(true));
    for id in [
        ids::VECTOR_TEXPAT_TARGET_FILL,
        ids::VECTOR_TEXPAT_SOURCE,
        ids::VECTOR_TEXPAT_W,
    ] {
        assert!(
            rect(id).is_none(),
            "a seccao Pattern subiu sem padrao nenhum publicado"
        );
    }
    limpa();
}
