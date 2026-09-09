//! ⭐⭐⭐ **A secção AUDIO é PINTADA e está VIVA sob o dedo** (TOP-20 #4, W3).
//!
//! # Porque este ficheiro existe
//!
//! É a lição que o `Timers` custou, escrita como gate: ele shipou anexável e sem linha de edição, e
//! o report do dono foi *«timer sumiu do modal de componente»*. *Um componente anexável sem painel
//! é indistinguível de um que não foi anexado.*
//!
//! # ⚠️ E há um caso aqui que nenhuma outra secção tem: um componente SEM CAMPOS
//!
//! O `AudioListener2D` é um marcador — a presença é o valor. Anexá-lo não muda **nada** na tela a
//! menos que a secção diga o que ele faz, e um componente que se anexa sem efeito visível é
//! exactamente o report que esta wave existe para não repetir.

use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::ids;
use ph2d_editor_core::screens::hero::{AudioFieldEdit, InspectorAudioInfo, InspectorAudioSource};
use ph2d_editor_core::zones::Rect;
use ph2d_host::{PointerButton, PointerEvent, PointerKind, PointerSource};
use ph2d_panel_inspector::{InspectorPanel, InspectorState, set_current_inspector_audio};
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

fn fonte() -> InspectorAudioSource {
    InspectorAudioSource {
        sound: "/tmp/passo.wav".into(),
        volume_db: 0.0,
        pitch: 1.0,
        looping: false,
        autoplay: false,
        max_distance: 10.0,
        attenuation: 1.0,
        non_spatialized_radius: 0.0,
        panning_strength: 1.0,
        max_polyphony: 1,
        bus_tag: 0,
        file_missing: false,
        reachable_by_signal: true,
    }
}

/// ⚠️ **Os rótulos são escritos à mão de propósito** (o painel não conhece o enum), e por isso o
/// gate confere que a fixtura ainda cobre o modelo — senão um barramento novo dá um *«a entrada 4
/// não foi pintada»* que parece um defeito do painel.
fn info(source: Option<InspectorAudioSource>, is_listener: bool, n: usize) -> InspectorAudioInfo {
    let i = InspectorAudioInfo {
        entity_bits: 0xABCD_1234,
        source,
        is_listener,
        listener_count: n,
        is_active_listener: n > 0,
        bus_labels: vec![
            "SFX".into(),
            "Music".into(),
            "Voice".into(),
            "Master".into(),
        ],
        selected_count: 1,
    };
    assert_eq!(
        i.bus_labels.len(),
        ids::INSP_AUDIO_BUS_OPT.len(),
        "a FIXTURA e' que esta' velha: o modelo tem {} barramentos e ela escreve {} rotulos",
        ids::INSP_AUDIO_BUS_OPT.len(),
        i.bus_labels.len()
    );
    i
}

fn host(i: InspectorAudioInfo) -> (MockPanelHost, InspectorState) {
    let h = MockPanelHost::with_panel::<InspectorPanel>();
    set_current_inspector_audio(Some(i));
    (h, InspectorState::default())
}

/// ⭐⭐⭐ **TODO campo da fonte é pintado com área** — a metade que o report do `Timers` cobrou.
///
/// ⚠️ **A lista dos ids é varrida, não amostrada:** um campo novo que alguém acrescente ao
/// componente e esqueça de pintar cai aqui, em vez de ficar inalcançável em silêncio.
///
/// **Mutação que deve sangrar:** tirar qualquer linha da tabela de campos da secção.
#[test]
fn every_field_of_the_source_is_painted() {
    let (mut h, mut st) = host(info(Some(fonte()), false, 1));
    let rects = h.paint::<InspectorPanel>(&mut st, VIEWPORT);
    for id in [
        ids::INSP_AUDIO_SOUND,
        ids::INSP_AUDIO_BROWSE,
        ids::INSP_AUDIO_PREVIEW,
        ids::INSP_AUDIO_STOP,
        ids::INSP_AUDIO_VOLUME,
        ids::INSP_AUDIO_PITCH,
        ids::INSP_AUDIO_MAX_DIST,
        ids::INSP_AUDIO_ATTENUATION,
        ids::INSP_AUDIO_RADIUS,
        ids::INSP_AUDIO_PANNING,
        ids::INSP_AUDIO_POLYPHONY,
        ids::INSP_AUDIO_BUS_PICK,
        ids::INSP_AUDIO_LOOP,
        ids::INSP_AUDIO_AUTOPLAY,
    ] {
        let r = rects
            .iter()
            .find(|(n, _)| *n == id)
            .map(|(_, r)| *r)
            .unwrap_or_else(|| panic!("o campo {id:?} nao foi PINTADO com area clicavel"));
        assert!(r.w > 0.0 && r.h > 0.0, "campo {id:?} sem area: {r:?}");
    }
    set_current_inspector_audio(None);
}

/// ⭐⭐ **Um objecto que só tem as ORELHAS ainda tem secção.**
///
/// ⚠️ **É o caso que nenhuma outra secção deste painel tem:** o `AudioListener2D` é um marcador sem
/// campos, então a única coisa que o distingue de *«não anexei nada»* é a secção existir e dizer o
/// que ele faz. ⛔ Sem este gate, o componente podia ficar mudo na tela com a suíte inteira verde —
/// que é exactamente o report que o `Timers` custou.
///
/// **Mutação que deve sangrar (corrida):** a secção devolver cedo quando `info.source` é `None`.
#[test]
fn an_object_that_only_has_the_ears_still_has_a_section() {
    let (mut h, mut st) = host(info(None, true, 1));
    let rects = h.paint::<InspectorPanel>(&mut st, VIEWPORT);
    let cabecalho = rects
        .iter()
        .find(|(n, _)| *n == ids::INSP_LIVE_AUDIO_SECTION)
        .map(|(_, r)| *r)
        .expect("a seccao AUDIO nao foi pintada para um objecto que so' tem as orelhas");
    assert!(
        cabecalho.h > 0.0,
        "o cabecalho nao tem banda: {cabecalho:?}"
    );
    // ⚠️ **E os campos da FONTE não aparecem** — ele não tem fonte nenhuma.
    assert!(
        !rects.iter().any(|(n, _)| *n == ids::INSP_AUDIO_SOUND),
        "o campo do ficheiro foi pintado para um objecto SEM fonte de som"
    );
    set_current_inspector_audio(None);
}

/// ⭐⭐⭐ **Carregar em `Preview` chega ao barramento** — com o gesto REAL.
///
/// ⚠️ **Um `WidgetEvent::Click` sintético passa sobre um botão ausente do `populate`**, que é a
/// família dos dez chips do impasto. Aqui o `Down`+`Up` cai no rectângulo que a pintura registou.
///
/// **Mutação que deve sangrar:** tirar o `INSP_AUDIO_PREVIEW` do `register_button_ids` do
/// `populate_audio`.
#[test]
fn pressing_preview_reaches_the_bus_with_a_real_pointer() {
    let (mut h, mut st) = host(info(Some(fonte()), false, 1));
    let rects = h.paint::<InspectorPanel>(&mut st, VIEWPORT);
    let (_, r) = rects
        .iter()
        .find(|(n, _)| *n == ids::INSP_AUDIO_PREVIEW)
        .copied()
        .expect("o botao Preview nao foi pintado");
    let (cx, cy) = (r.x + r.w * 0.5, r.y + r.h * 0.5);
    let _ = h.drained_actions();
    let mut evs = h.dispatch_pointer_event(pointer(PointerKind::Down, cx, cy, SEC));
    evs.extend(h.dispatch_pointer_event(pointer(PointerKind::Up, cx, cy, SEC + SEC / 100)));
    assert!(
        !evs.is_empty(),
        "o ponteiro sobre `Preview` nao virou evento nenhum — ele esta' desenhado e nao existe \
         para o dispatcher (falta o registo no populate)"
    );
    for ev in evs {
        h.apply_panel_event::<InspectorPanel>(&mut st, ev);
    }
    let acoes = h.drained_actions();
    assert!(
        acoes.iter().any(|a| matches!(
            a,
            EditorAction::InspectorAudioEdit {
                edit: AudioFieldEdit::Preview,
                ..
            }
        )),
        "carregar em Preview nao chegou ao barramento; o que chegou foi {acoes:?}"
    );
    set_current_inspector_audio(None);
}

/// ⭐⭐ **O seletor do BARRAMENTO abre e escolhe** — o mesmo mecanismo do verbo, medido aqui porque
/// é um chip diferente com o seu próprio slot.
///
/// **Mutações que devem sangrar:** apagar o `set_pending_audio_dd` · apagar o bloco do barramento
/// no passe diferido dos popovers.
#[test]
fn the_bus_picker_opens_and_picks() {
    let (mut h, mut st) = host(info(Some(fonte()), false, 1));
    let _ = h.paint::<InspectorPanel>(&mut st, VIEWPORT);
    h.set_dropdown_open(ids::INSP_AUDIO_BUS_PICK, true);
    let rects = h.paint::<InspectorPanel>(&mut st, VIEWPORT);
    for (i, &id) in ids::INSP_AUDIO_BUS_OPT.iter().enumerate() {
        assert!(
            rects.iter().any(|(n, _)| *n == id),
            "a entrada {i} do seletor de barramento nao chegou ao indice de acerto"
        );
    }
    // Escolher `Music` (a posição 1) chega ao barramento e fecha a lista.
    let alvo = ids::INSP_AUDIO_BUS_OPT[1];
    let _ = h.drained_actions();
    h.apply_panel_event::<InspectorPanel>(
        &mut st,
        ph2d_editor_core::interaction::WidgetEvent::Click(alvo),
    );
    let acoes = h.drained_actions();
    assert!(
        acoes.iter().any(|a| matches!(
            a,
            EditorAction::InspectorAudioEdit {
                edit: AudioFieldEdit::Bus(1),
                ..
            }
        )),
        "escolher `Music` nao chegou ao barramento; o que chegou foi {acoes:?}"
    );
    assert_eq!(
        h.dropdown_is_open(ids::INSP_AUDIO_BUS_PICK),
        Some(false),
        "o seletor ficou ABERTO depois da escolha"
    );
    set_current_inspector_audio(None);
}
