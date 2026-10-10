//! ⭐ **O parâmetro do *Play Clip* escolhe-se num ▾ com os clipes da Animação do alvo** (onda 5,
//! item c) — e o ▾ está VIVO, com o gesto real (`Down`+`Up` no rect que a pintura registou), pela
//! mesma razão do `action_verb_is_a_dropdown`: um `Click` sintético passaria sobre um chip pintado
//! e ausente do `populate`.

use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::screens::hero::{
    ActionArgHint, ActionFieldEdit, InspectorActionInfo, InspectorActionRow,
};
use ph2d_editor_core::zones::Rect;
use ph2d_host::{PointerButton, PointerEvent, PointerKind, PointerSource};
use ph2d_panel_inspector::ids::{INSP_ACTION_ARG, INSP_ACTION_CLIP, INSP_ACTION_CLIP_PICK};
use ph2d_panel_inspector::{InspectorPanel, InspectorState, set_current_inspector_action};
use ph2d_ui_testkit::MockPanelHost;

const VIEWPORT: Rect = Rect {
    x: 0.0,
    y: 0.0,
    w: 1600.0,
    h: 900.0,
};
const SEC: u128 = 1_000_000_000;
/// A posição do `Play Clip` em `SignalVerb::ALL`.
const TAG_PLAY_CLIP: u8 = 14;

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

/// Uma linha *Play Clip* com `opcoes` no retrato e `andar` escrito. ⚠️ O alvo do clique (`correr`)
/// é outro nome: um alvo igual ao de partida não distingue «a escolha chegou» de «nada aconteceu».
fn tocar(opcoes: Option<Vec<String>>) -> InspectorActionInfo {
    InspectorActionInfo {
        entity_bits: 0xABCD_1234,
        rows: vec![InspectorActionRow {
            on: "abre".into(),
            target: String::new(),
            verb_tag: TAG_PLAY_CLIP,
            arg: "andar".into(),
            uses_arg: true,
            arg_hint: ActionArgHint::ClipName,
            arg_options: opcoes,
            uses_target: true,
            target_mode: 0,
            from_tag: 0,
            target_tag: None,
            target_tag_path: String::new(),
        }],
        verb_labels: (0..=TAG_PLAY_CLIP).map(|i| format!("verbo {i}")).collect(),
        selected_count: 1,
    }
}

fn host(info: InspectorActionInfo) -> (MockPanelHost, InspectorState) {
    let h = MockPanelHost::with_panel::<InspectorPanel>();
    set_current_inspector_action(Some(info));
    (h, InspectorState::default())
}

fn nomes(n: &[&str]) -> Option<Vec<String>> {
    Some(n.iter().map(|s| (*s).to_string()).collect())
}

/// ⭐⭐⭐ **Abrir o ▾ e clicar a 2.ª opção escreve o NOME dela no parâmetro — e fecha a lista.**
///
/// **Mutações que devem sangrar:** o índice→nome do despacho (ler a posição de outra lista, ou
/// `i + 1`) · apagar o `register_button_ids(INSP_ACTION_CLIP)` do `populate_action` · apagar o
/// `action_clip_popover` do passe diferido.
#[test]
fn escolher_a_segunda_opcao_escreve_o_nome_dela() {
    let (mut h, mut st) = host(tocar(nomes(&["andar", "correr", "pular"])));
    let r = h
        .painted_rect::<InspectorPanel>(&mut st, VIEWPORT, INSP_ACTION_CLIP_PICK)
        .expect("o chip do clipe nao foi PINTADO com area clicavel");
    assert!(r.w > 0.0 && r.h > 0.0);
    assert_eq!(
        h.dropdown_is_open(INSP_ACTION_CLIP_PICK),
        Some(false),
        "o chip nao esta' registado como Dropdown"
    );
    assert!(
        h.painted_rect::<InspectorPanel>(&mut st, VIEWPORT, INSP_ACTION_ARG)
            .is_none(),
        "com escolhas no retrato o texto livre nao pode ficar ao lado do chip"
    );

    h.set_dropdown_open(INSP_ACTION_CLIP_PICK, true);
    let rects = h.paint::<InspectorPanel>(&mut st, VIEWPORT);
    for (i, id) in INSP_ACTION_CLIP.iter().enumerate() {
        let pintada = rects.iter().any(|(n, _)| n == id);
        assert_eq!(pintada, i < 3, "a opcao {i}: pintada={pintada}");
    }
    let (_, r) = rects
        .iter()
        .find(|(n, _)| *n == INSP_ACTION_CLIP[1])
        .copied()
        .expect("a 2.a opcao nao foi pintada");
    let (cx, cy) = (r.x + r.w * 0.5, r.y + r.h * 0.5);
    let _ = h.drained_actions();
    let mut evs = h.dispatch_pointer_event(pointer(PointerKind::Down, cx, cy, SEC));
    evs.extend(h.dispatch_pointer_event(pointer(PointerKind::Up, cx, cy, SEC + SEC / 100)));
    assert!(!evs.is_empty(), "o ponteiro sobre a opcao nao virou evento");
    for ev in evs {
        h.apply_panel_event::<InspectorPanel>(&mut st, ev);
    }
    let acoes = h.drained_actions();
    assert!(
        acoes.iter().any(|a| matches!(
            a,
            EditorAction::InspectorActionEdit {
                edit: ActionFieldEdit::Arg(0, nome),
                ..
            } if nome == "correr"
        )),
        "escolher `correr` nao chegou ao barramento; chegou {acoes:?}"
    );
    assert_eq!(
        h.dropdown_is_open(INSP_ACTION_CLIP_PICK),
        Some(false),
        "o ▾ ficou ABERTO depois da escolha"
    );
    set_current_inspector_action(None);
}

/// ⛔ **Lista vazia (o alvo não tem Animação) ⇒ nenhuma opção clicável** — o chip nem é alvo do
/// ponteiro, e mesmo forçado aberto o passe diferido não pinta opção nenhuma.
#[test]
fn com_a_lista_vazia_nao_ha_opcao_clicavel() {
    let (mut h, mut st) = host(tocar(Some(vec![])));
    assert!(
        h.painted_rect::<InspectorPanel>(&mut st, VIEWPORT, INSP_ACTION_CLIP_PICK)
            .is_none(),
        "um ▾ sem nada a oferecer nao pode ser alvo do ponteiro"
    );
    h.set_dropdown_open(INSP_ACTION_CLIP_PICK, true);
    let rects = h.paint::<InspectorPanel>(&mut st, VIEWPORT);
    assert!(
        !rects.iter().any(|(n, _)| INSP_ACTION_CLIP.contains(n)),
        "uma opcao de clipe foi pintada com a lista vazia"
    );
    set_current_inspector_action(None);
}

/// Sem escolhas no retrato (`None`: alvo por tag, quem bateu, outros verbos) fica o texto livre.
#[test]
fn sem_escolhas_fica_o_texto_livre() {
    let (mut h, mut st) = host(tocar(None));
    assert!(
        h.painted_rect::<InspectorPanel>(&mut st, VIEWPORT, INSP_ACTION_ARG)
            .is_some(),
        "o campo de texto do parametro sumiu"
    );
    assert!(
        h.painted_rect::<InspectorPanel>(&mut st, VIEWPORT, INSP_ACTION_CLIP_PICK)
            .is_none(),
        "um ▾ apareceu sem escolhas no retrato"
    );
    set_current_inspector_action(None);
}
