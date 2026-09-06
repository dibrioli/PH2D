//! ⭐⭐⭐ **SEAM DO ESQUELETO** (estudo 42 item 5) — o gesto REAL sobre cada controlo, e o que ele
//! faz chegar ao barramento.
//!
//! ⚠️ **Ele nasce com a wave, e não depois de um report**, que é a lição do [bug #29]: a pilha de
//! aparência shipou com o modelo, o transform, o renderer, o exportador e o painel gateados — e
//! **zero** gates no caminho do clique. Os controlos pintavam, hit-indexavam, registavam (logo
//! ACENDIAM sob o rato) e o `Click` morria no `apply_event` do painel.
//!
//! ⛔ **Nenhum gate existente vê isto**: o `hit_indexed_ids_are_registered` mede FOCABILIDADE, e os
//! testes da shell medem o que ela faz **depois** de receber o verbo. Entre os dois há um passo que
//! nenhum atravessa — *o clique sai do painel?*
//!
//! [bug #29]: ../../../docs/Vector%20Module/BUGS_vector.md

use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::interaction::WidgetEvent;
use ph2d_editor_core::tool::PanelEvent;
use ph2d_editor_core::zones::Rect;
use ph2d_host::{PointerButton, PointerEvent, PointerKind, PointerSource};
use ph2d_panel_vector::state::VectorPanelState;
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

/// A cena tem esqueleto, a selecção está presa e há um osso em foco — o estado em que **todos** os
/// controlos da seção são oferecidos.
fn publica_tudo() {
    state::set_current_has_skeleton(true);
    state::set_current_skinned(true);
    state::set_current_bone(Some((20.0, 1.0)));
}

fn limpa() {
    state::set_current_has_skeleton(false);
    state::set_current_skinned(false);
    state::set_current_bone(None);
}

/// **O gesto REAL sobre um retângulo pintado**, e o que ele deixa no barramento.
///
/// Down+Up de verdade, e não um `WidgetEvent::Click` sintético: o sintético prova a allowlist do
/// painel mas **pula a checagem de focabilidade no store** — as duas metades falham de maneiras
/// diferentes, e um gate que só faz uma fica verde sobre a outra.
fn clica(id: ph2d_a11y::NodeId, what: &str) -> Vec<EditorAction> {
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
        "{what}: o ponteiro sobre o retangulo nao virou Click — ele esta' desenhado e nao existe \
         para o dispatcher (falta o `register` no populate)"
    );
    for ev in evs {
        host.apply_panel_event::<VectorPanel>(&mut st, ev);
    }
    host.drained_actions()
}

/// ⭐⭐⭐ **OS TRÊS VERBOS DO ESQUELETO ATRAVESSAM O BARRAMENTO.**
///
/// ⚠️ **O oráculo é o `EditorAction`, nunca o `WidgetEvent`.** Um controlo que produz `Click` e não
/// produz `ToolPanelEvent` acende sob o rato, consome o gesto e não faz nada — que é literalmente o
/// report do bug #29 (*"o olho não funciona"*).
#[test]
fn every_verb_of_the_skeleton_reaches_the_bus() {
    publica_tudo();
    for (id, what) in [
        (ids::VECTOR_BONE_BIND, "Bind to Skeleton"),
        (ids::VECTOR_BONE_EXPAND, "Keep Pose"),
        (ids::VECTOR_BONE_RELEASE, "Release"),
    ] {
        let acoes = clica(id, what);
        assert!(
            acoes.iter().any(|a| matches!(
                a,
                EditorAction::ToolPanelEvent(PanelEvent::Click(c)) if *c == id
            )),
            "{what}: o Click nao chegou ao barramento — ele acende sob o rato e nao faz nada \
             (falta a familia na allowlist do `event_clicks::forwards_plain_click`)"
        );
    }
    limpa();
}

/// ⭐ **O PILL do modo Osso troca a ferramenta.** É a metade que o bug #29 mediu no lado dos
/// modos: um pill fora da allowlist pinta, acende e o modo nunca muda.
#[test]
fn the_bone_pill_reaches_the_tool() {
    let acoes = clica(ids::VECTOR_MODE_BONE, "o pill Bone");
    assert!(
        acoes.iter().any(|a| matches!(
            a,
            EditorAction::ToolPanelEvent(PanelEvent::Click(c)) if *c == ids::VECTOR_MODE_BONE
        )),
        "o pill Bone nao chegou a' ferramenta"
    );
}

/// ⛔ **AS DUAS SAÍDAS SÓ EXISTEM COM UMA FORMA PRESA** — e o `Bind` existe sempre que a seção
/// existe. *Um botão que só sabe recusar é pior que um botão ausente.*
///
/// ⚠️ **As duas metades**, porque um gate que só afirmasse a presença ficaria verde sobre um painel
/// que as mostra sempre.
#[test]
fn the_two_exits_appear_only_when_something_is_bound() {
    let sem = |id| {
        let mut host = MockPanelHost::with_panel::<VectorPanel>();
        let mut st = VectorPanelState;
        host.painted_rect::<VectorPanel>(&mut st, VIEWPORT, id)
            .is_some()
    };
    state::set_current_has_skeleton(true);
    state::set_current_skinned(false);
    state::set_current_bone(None);
    assert!(sem(ids::VECTOR_BONE_BIND), "o Bind tem de estar la' sempre");
    assert!(
        !sem(ids::VECTOR_BONE_EXPAND) && !sem(ids::VECTOR_BONE_RELEASE),
        "as saidas foram pintadas sem nada preso"
    );
    state::set_current_skinned(true);
    assert!(
        sem(ids::VECTOR_BONE_EXPAND) && sem(ids::VECTOR_BONE_RELEASE),
        "as saidas sumiram com uma forma presa"
    );
    limpa();
}

/// ⛔ **A SEÇÃO INTEIRA SOME num app sem esqueleto nenhum** (fora da ferramenta que faz ossos).
///
/// É a lei da tabela de escopo, aplicada ao caso que ela não cobre: aqui o sujeito não é a
/// ferramenta na mão nem a selecção, é *a cena ter ou não ossos*.
#[test]
fn the_whole_section_is_absent_from_an_app_that_has_no_bones() {
    limpa();
    let mut host = MockPanelHost::with_panel::<VectorPanel>();
    let mut st = VectorPanelState;
    for (id, what) in [
        (ids::VECTOR_SECTION_BONE, "o cabecalho"),
        (ids::VECTOR_BONE_BIND, "o Bind"),
    ] {
        assert!(
            host.painted_rect::<VectorPanel>(&mut st, VIEWPORT, id)
                .is_none(),
            "{what} foi pintado numa cena sem osso nenhum"
        );
    }
}

/// ⭐⭐ **OS DOIS NÚMEROS DO OSSO SÓ EXISTEM COM UM OSSO EM FOCO** — sem sujeito, um campo é a
/// espécie de controlo morto que o `CLAUDE.md` §5.0 nomeia.
#[test]
fn the_two_bone_numbers_need_a_bone_in_focus() {
    state::set_current_has_skeleton(true);
    state::set_current_skinned(false);
    state::set_current_bone(None);
    let pintado = |id| {
        let mut host = MockPanelHost::with_panel::<VectorPanel>();
        let mut st = VectorPanelState;
        host.painted_rect::<VectorPanel>(&mut st, VIEWPORT, id)
            .is_some()
    };
    assert!(
        !pintado(ids::VECTOR_BONE_LENGTH) && !pintado(ids::VECTOR_BONE_STRENGTH),
        "os campos foram pintados sem osso nenhum em foco"
    );
    state::set_current_bone(Some((20.0, 1.0)));
    assert!(
        pintado(ids::VECTOR_BONE_LENGTH) && pintado(ids::VECTOR_BONE_STRENGTH),
        "os campos sumiram com um osso em foco"
    );
    limpa();
}
