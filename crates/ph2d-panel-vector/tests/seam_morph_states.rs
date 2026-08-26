//! ⭐⭐ **A seção MORPH STATES é PRÓPRIA — e este é o gate que faltava** (plano 32 W7/W8).
//!
//! ⛔⛔ **A regressão que este ficheiro existe para impedir.** A W4 pôs as transições do Morph
//! dentro da seção **States** (as poses de UI + Smart Animate). Nenhum dos gates dela olhava para o
//! que era **PINTADO** — todos mediam o mapa e o estado publicado —, então doze verdes conviveram
//! com um cabeçalho de uma feature já entregue a aparecer por causa de outra. Enio, 2026-08-25:
//! *"vc contaminou ou até mesmo estragou a feature states previamente implementada? Os states de
//! morph deveriam ter sessão exclusiva."*
//!
//! ⇒ **os dois primeiros gates medem a AUSÊNCIA nos dois sentidos**, que é a única forma de a
//! afirmação ser mais do que uma promessa: nenhuma das duas seções pode fazer a outra aparecer.
//!
//! O gesto dos dois últimos é **REAL** (Down+Up sobre o rect que o painel pintou), e não um
//! `WidgetEvent::Click` sintético: o sintético prova a allowlist do painel mas **pula a checagem de
//! focabilidade no store** — a lacuna que já deixou os quatro chips da booleana *pintados,
//! hit-registrados e mortos sob o ponteiro*.

use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::interaction::WidgetEvent;
use ph2d_editor_core::tool::PanelEvent;
use ph2d_editor_core::zones::Rect;
use ph2d_host::{PointerButton, PointerEvent, PointerKind, PointerSource};
use ph2d_panel_vector::state::{
    MorphShapeRow, MorphStatesState, UiStatesState, VectorPanelState, set_morph_states_state,
    set_ui_states_state,
};
use ph2d_panel_vector::{VectorPanel, ids};
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

fn clear() {
    set_morph_states_state(None);
    set_ui_states_state(None);
}

/// Uma máquina com duas transições.
fn machine() -> MorphStatesState {
    MorphStatesState {
        rows: vec![
            MorphShapeRow {
                to: "Wide".into(),
                when: String::new(),
                live: true,
            },
            MorphShapeRow {
                to: "Tall".into(),
                when: "jump".into(),
                live: false,
            },
        ],
        actions: vec!["jump".into(), "dash".into()],
        current: Some("Wide".into()),
        can_make: 0,
        preview: false,
    }
}

/// A mesma máquina, com a pré-visualização LIGADA.
fn machine_previewing() -> MorphStatesState {
    MorphStatesState {
        preview: true,
        ..machine()
    }
}

/// Uma seleção de N formas, ainda sem conjunto — a face que traz o botão.
fn can_make(n: usize) -> MorphStatesState {
    MorphStatesState {
        actions: vec!["jump".into()],
        can_make: n,
        ..Default::default()
    }
}

/// Uma forma-hospedeiro com poses de UI (a feature de que a W4 se apropriou).
fn poses() -> UiStatesState {
    UiStatesState {
        host: Some("Host".into()),
        recorded: [true, false, false, false],
        role_labels: [
            "Default".into(),
            "Hover".into(),
            "Pressed".into(),
            "Disabled".into(),
        ],
        live: None,
        duration_s: 0.15,
        spring: None,
        preview: None,
        move_all: None,
        easing: ph2d_anim::Easing::new(ph2d_anim::EasingFamily::Cubic, ph2d_anim::EasingMode::Out),
        bindings: Vec::new(),
    }
}

fn painted(id: ph2d_a11y::NodeId) -> Option<Rect> {
    let mut host = MockPanelHost::with_panel::<VectorPanel>();
    let mut ps = VectorPanelState;
    host.painted_rect::<VectorPanel>(&mut ps, VIEWPORT, id)
}

/// ⛔⛔ **UM MORPH NÃO FAZ A SEÇÃO `States` APARECER.** É a regressão, medida no pixel.
///
/// **Mutação que deve sangrar:** a `ui_states_section` voltar a consultar o `morph_states_state` —
/// o cabeçalho **States** volta a ser pintado sobre uma seleção que não tem pose nenhuma, que é
/// exactamente a foto de que o dono se queixou.
#[test]
fn a_morph_machine_never_makes_the_ui_states_section_appear() {
    clear();
    set_morph_states_state(Some(machine()));
    assert!(
        painted(ids::VECTOR_SECTION_MORPH_STATES).is_some(),
        "a seccao PROPRIA do Morph tem de ser pintada -- senao a feature nao tem porta nenhuma"
    );
    assert!(
        painted(ids::VECTOR_SECTION_STATES).is_none(),
        "⛔ o cabecalho STATES (poses de UI) foi pintado por causa de um MORPH -- e' a \
         contaminacao de 2026-08-25 de volta"
    );
    clear();
}

/// ⛔ **E O CONTRÁRIO: poses de UI não fazem a seção do Morph aparecer.**
///
/// ⚠️ Sem esta metade o gate acima ficaria verde sobre uma seção do Morph pintada **sempre** — que
/// é a mesma doença, com o cabeçalho do outro lado.
#[test]
fn ui_poses_never_make_the_morph_states_section_appear() {
    clear();
    set_ui_states_state(Some(poses()));
    assert!(
        painted(ids::VECTOR_SECTION_STATES).is_some(),
        "o CONTROLE: a seccao das poses tem de continuar a aparecer para quem a tem"
    );
    assert!(
        painted(ids::VECTOR_SECTION_MORPH_STATES).is_none(),
        "⛔ a seccao do Morph foi pintada sobre uma selecao que so' tem poses"
    );
    clear();
}

/// ⭐ **O botão que FAZ o conjunto está vivo sob o rato e o clique chega ao barramento.**
///
/// ⚠️ **É a única porta para a feature inteira.** Ele pintado-e-morto e a máquina de estados fica
/// inalcançável por gesto nenhum — com a lista, os gates e o motor todos verdes.
#[test]
fn the_make_button_is_alive_and_reaches_the_bus() {
    clear();
    set_morph_states_state(Some(can_make(3)));
    let mut host = MockPanelHost::with_panel::<VectorPanel>();
    let mut ps = VectorPanelState;
    let r = host
        .painted_rect::<VectorPanel>(&mut ps, VIEWPORT, ids::VECTOR_MORPH_STATES_MAKE)
        .expect("o botao «Make Morph States» nao foi PINTADO com area clicavel");
    let (cx, cy) = (r.x + r.w * 0.5, r.y + r.h * 0.5);
    host.dispatch_pointer_event(pointer(PointerKind::Down, cx, cy, SEC));
    let evs = host.dispatch_pointer_event(pointer(PointerKind::Up, cx, cy, SEC + SEC / 100));
    assert!(
        evs.iter()
            .any(|e| matches!(e, WidgetEvent::Click(c) if *c == ids::VECTOR_MORPH_STATES_MAKE)),
        "o ponteiro sobre o botao nao virou Click -- ele esta' desenhado e nao existe para o \
         dispatcher (falta o `register` no populate)"
    );
    for ev in evs {
        host.apply_panel_event::<VectorPanel>(&mut ps, ev);
    }
    assert!(
        host.drained_actions().into_iter().any(|a| matches!(
            a,
            EditorAction::ToolPanelEvent(PanelEvent::Click(c)) if c == ids::VECTOR_MORPH_STATES_MAKE
        )),
        "o Click nao chegou ao bus -- o botao acende sob o rato e nao faz nada (falta a linha na \
         allowlist do event_clicks)"
    );
    clear();
}

/// ⛔ **Com menos de duas formas o botão NÃO existe** — e a frase que fica diz o que falta.
///
/// ⚠️ A metade da ausência: sem ela o gate acima passaria sobre um botão pintado sempre, que
/// recusaria em silêncio quando premido. *Um botão que recusa sem dizer porquê ensina o artista a
/// desconfiar dos outros.*
#[test]
fn one_shape_offers_no_button_at_all() {
    clear();
    set_morph_states_state(Some(can_make(1)));
    assert!(
        painted(ids::VECTOR_MORPH_STATES_MAKE).is_none(),
        "o botao foi pintado com UMA forma escolhida -- ele so' pode recusar em silencio"
    );
    // E acima do tecto medido também não: ali a frase diz o número.
    set_morph_states_state(Some(can_make(ids::MAX_MORPH_STATES + 1)));
    assert!(
        painted(ids::VECTOR_MORPH_STATES_MAKE).is_none(),
        "o botao foi pintado acima do tecto de {} formas",
        ids::MAX_MORPH_STATES
    );
    clear();
}

/// ⭐ **O menu da CONDIÇÃO de cada transição está vivo e o clique chega ao barramento.**
///
/// ⚠️ **É o único verbo que age sobre uma seta** desde a W8 (não há lixeira): se ele morrer sob o
/// ponteiro, o grafo completo fica inteiramente inerte e nada na tela o diz.
#[test]
fn the_key_button_of_every_row_opens_the_event_list() {
    clear();
    set_morph_states_state(Some(machine()));
    for row in 0..machine().rows.len() {
        assert!(
            painted(ids::morph_shape_key_button_id(row)).is_some(),
            "o botao da tecla da forma {row} nao foi pintado"
        );
    }
    // E a opção dentro do menu chega ao bus — ela é o que de facto escreve no mundo.
    let mut host = MockPanelHost::with_panel::<VectorPanel>();
    let mut ps = VectorPanelState;
    let opt = ids::morph_shape_key_option_id(0, 1);
    let chip = host
        .painted_rect::<VectorPanel>(&mut ps, VIEWPORT, ids::morph_shape_key_button_id(0))
        .expect("o chip existe");
    host.dispatch_pointer_event(pointer(PointerKind::Down, chip.x + 2.0, chip.y + 2.0, SEC));
    let evs = host.dispatch_pointer_event(pointer(
        PointerKind::Up,
        chip.x + 2.0,
        chip.y + 2.0,
        SEC + SEC / 100,
    ));
    for ev in evs {
        host.apply_panel_event::<VectorPanel>(&mut ps, ev);
    }
    let r = host
        .painted_rect::<VectorPanel>(&mut ps, VIEWPORT, opt)
        .expect("com o menu ABERTO a opcao tem de ser pintada -- senao ela nao e' escolhivel");
    let (cx, cy) = (r.x + r.w * 0.5, r.y + r.h * 0.5);
    host.dispatch_pointer_event(pointer(PointerKind::Down, cx, cy, 2 * SEC));
    let evs = host.dispatch_pointer_event(pointer(PointerKind::Up, cx, cy, 2 * SEC + SEC / 100));
    assert!(
        evs.iter()
            .any(|e| matches!(e, WidgetEvent::Click(c) if *c == opt)),
        "a opcao do menu esta' desenhada e morta sob o ponteiro"
    );
    for ev in evs {
        host.apply_panel_event::<VectorPanel>(&mut ps, ev);
    }
    assert!(
        host.drained_actions().into_iter().any(|a| matches!(
            a,
            EditorAction::ToolPanelEvent(PanelEvent::Click(c)) if c == opt
        )),
        "o Click da opcao nao chegou ao bus -- escolher a accao nao escreveria no mundo"
    );
    clear();
}

/// ⭐⭐ **O INTERRUPTOR DA PRÉ-VISUALIZAÇÃO está vivo e o clique chega ao barramento.**
///
/// ⚠️ **É a única porta de entrada E de saída do modo que toma o teclado.** Se ele morrer sob o
/// ponteiro, o artista que entrar (por outro caminho) fica sem botão para sair, e o modo consome
/// exactamente as teclas com que ele tentaria escapar.
#[test]
fn the_preview_toggle_is_alive_and_reaches_the_bus() {
    clear();
    set_morph_states_state(Some(machine()));
    let mut host = MockPanelHost::with_panel::<VectorPanel>();
    let mut ps = VectorPanelState;
    let r = host
        .painted_rect::<VectorPanel>(&mut ps, VIEWPORT, ids::VECTOR_MORPH_PREVIEW)
        .expect("o botao «Preview» nao foi PINTADO com area clicavel");
    let (cx, cy) = (r.x + r.w * 0.5, r.y + r.h * 0.5);
    host.dispatch_pointer_event(pointer(PointerKind::Down, cx, cy, SEC));
    let evs = host.dispatch_pointer_event(pointer(PointerKind::Up, cx, cy, SEC + SEC / 100));
    assert!(
        evs.iter()
            .any(|e| matches!(e, WidgetEvent::Click(c) if *c == ids::VECTOR_MORPH_PREVIEW)),
        "o ponteiro sobre o interruptor nao virou Click -- falta o `register` no populate"
    );
    for ev in evs {
        host.apply_panel_event::<VectorPanel>(&mut ps, ev);
    }
    assert!(
        host.drained_actions().into_iter().any(|a| matches!(
            a,
            EditorAction::ToolPanelEvent(PanelEvent::Click(c)) if c == ids::VECTOR_MORPH_PREVIEW
        )),
        "o Click nao chegou ao bus -- o interruptor acende sob o rato e nao liga modo nenhum"
    );
    clear();
}

/// ⭐ **O interruptor CONTINUA clicável com a preview LIGADA** — ele é a porta de saída.
///
/// ⚠️ **Mutação que deve sangrar:** o `morph_preview_row` deixar de registar o hit-rect quando
/// `on` — o artista entra no modo, o botão fica aceso e **morto**, e as teclas com que ele tentaria
/// sair são precisamente as que o modo consome.
#[test]
fn the_way_out_stays_clickable_while_the_mode_runs() {
    clear();
    set_morph_states_state(Some(machine_previewing()));
    assert!(
        painted(ids::VECTOR_MORPH_PREVIEW).is_some(),
        "com a preview LIGADA o interruptor tem de continuar pintado e clicavel"
    );
    clear();
}

/// ⛔ **Sem máquina não há interruptor** — um modo de pré-visualização sobre um objecto sem
/// transições é um modo que não faz nada, e o artista não teria como o saber.
#[test]
fn a_selection_without_a_machine_offers_no_preview_toggle() {
    clear();
    set_morph_states_state(Some(can_make(3)));
    assert!(
        painted(ids::VECTOR_MORPH_PREVIEW).is_none(),
        "o interruptor foi pintado sobre uma seleccao que ainda nao tem maquina nenhuma"
    );
    clear();
}
