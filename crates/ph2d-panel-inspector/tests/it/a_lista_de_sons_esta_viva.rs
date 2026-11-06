//! ⭐ **A lista de sons está pintada e viva sob o dedo** (SFX Lab W1), no modo cartões (o padrão) e no de antes.

use crate::o_inspector_em_cartoes_por_tipo::{abre_tudo, pinta};
use crate::the_audio_section_is_alive::{fonte, info};
use ph2d_a11y::NodeId;
use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::interaction::WidgetEvent;
use ph2d_editor_core::panel::PanelHostInternal;
use ph2d_editor_core::screens::hero::{
    AudioFieldEdit, InspectorAudioInfo, InspectorSoundRow, InspectorSoundSet,
};
use ph2d_editor_core::zones::Rect;
use ph2d_host::{PointerButton, PointerEvent, PointerKind, PointerSource};
use ph2d_panel_inspector::ids as i;
use ph2d_panel_inspector::{
    InspectorPanel, InspectorState, set_cards_on, set_current_inspector_audio,
};
use ph2d_ui_testkit::MockPanelHost;

const SEC: u128 = 1_000_000_000;

fn conjunto(n: usize, offer: &[&str]) -> InspectorSoundSet {
    InspectorSoundSet {
        rows: (0..n)
            .map(|k| InspectorSoundRow {
                name: format!("coin_{k}.wav"),
                weight: 1.0 + k as f32,
                enabled: k != 1,
                missing: false,
            })
            .collect(),
        decides: n > 0,
        strategy_tag: 2,
        sequence_tag: 1,
        no_repeat: true,
        pitch_jitter_st: 2.0,
        gain_jitter_db: 3.0,
        mode_labels: vec!["Random".into(), "Sequence".into(), "Shuffle".into()],
        offer: offer.iter().map(|s| (*s).to_owned()).collect(),
        project_sounds: n + offer.len(),
    }
}

fn com(set: InspectorSoundSet) -> InspectorAudioInfo {
    let mut f = fonte();
    f.set = set;
    info(Some(f), false, 1)
}

/// O mesmo Inspector nos dois modos: `set_cards_on` é por fio de teste.
fn palco(
    cartoes: bool,
    i: InspectorAudioInfo,
) -> (MockPanelHost, InspectorState, Vec<(NodeId, Rect)>) {
    set_cards_on(cartoes);
    set_current_inspector_audio(Some(i));
    let mut h = MockPanelHost::with_panel::<InspectorPanel>();
    let mut st = InspectorState::default();
    abre_tudo(&mut h);
    let _ = pinta(&mut h, &mut st);
    abre_tudo(&mut h);
    let rects = pinta(&mut h, &mut st);
    (h, st, rects)
}

fn fim() {
    set_current_inspector_audio(None);
    set_cards_on(false);
}

fn tem(rects: &[(NodeId, Rect)], id: NodeId) -> bool {
    rects
        .iter()
        .any(|(n, r)| *n == id && r.w > 0.0 && r.h > 0.0)
}

fn rect(rects: &[(NodeId, Rect)], id: NodeId) -> Rect {
    rects
        .iter()
        .find(|(n, _)| *n == id)
        .map(|(_, r)| *r)
        .unwrap_or_else(|| panic!("{id:?} não foi pintado"))
}

/// O gesto real: desce e sobe no rect pintado, e cada evento vai ao painel.
fn clica(h: &mut MockPanelHost, st: &mut InspectorState, r: Rect) -> Vec<AudioFieldEdit> {
    let (x, y) = (r.x + r.w * 0.5, r.y + r.h * 0.5);
    let p = |kind, t| PointerEvent {
        kind,
        x,
        y,
        button: PointerButton::Primary,
        source: PointerSource::Mouse,
        pressure: 1.0,
        timestamp_ns: t,
    };
    let _ = h.drained_actions();
    let mut evs = h.dispatch_pointer_event(p(PointerKind::Down, SEC));
    evs.extend(h.dispatch_pointer_event(p(PointerKind::Up, SEC + SEC / 100)));
    for ev in evs {
        h.apply_panel_event::<InspectorPanel>(st, ev);
    }
    edicoes(h)
}

fn edicoes(h: &mut MockPanelHost) -> Vec<AudioFieldEdit> {
    h.drained_actions()
        .into_iter()
        .filter_map(|a| match a {
            EditorAction::InspectorAudioEdit { edit, .. } => Some(edit),
            _ => None,
        })
        .collect()
}

/// *Mutação: tirar `INSP_AUDIO_SET_DEL` do `populate` ⇒ o clique real vira nada.*
#[test]
fn each_row_turns_off_and_removes_with_a_real_pointer_on_cards() {
    for cartoes in [true, false] {
        let (mut h, mut st, rects) = palco(cartoes, com(conjunto(2, &["x.wav"])));
        for k in 0..2 {
            assert!(
                tem(&rects, i::INSP_AUDIO_SET_ON[k]),
                "caixa {k} (cartões {cartoes})"
            );
            assert!(tem(&rects, i::INSP_AUDIO_SET_WEIGHT[k]), "peso {k}");
            assert!(tem(&rects, i::INSP_AUDIO_SET_DEL[k]), "× {k}");
        }
        assert!(!tem(&rects, i::INSP_AUDIO_SET_ON[2]), "só há duas linhas");
        // A linha 1 está desligada: o clique afirma o contrário do snapshot.
        assert_eq!(
            clica(&mut h, &mut st, rect(&rects, i::INSP_AUDIO_SET_ON[1])),
            [AudioFieldEdit::SetEnabled(1, true)],
            "cartões {cartoes}"
        );
        assert_eq!(
            clica(&mut h, &mut st, rect(&rects, i::INSP_AUDIO_SET_DEL[0])),
            [AudioFieldEdit::SetRemove(0)],
            "cartões {cartoes}"
        );
        fim();
    }
}

/// Lei da compactação: nome, peso e × dividem a linha, e o «Weight» é da coluna, pintado por cima da caixa.
#[test]
fn name_weight_and_remove_share_one_row_on_a_card() {
    let (_h, _st, rects) = palco(true, com(conjunto(2, &[])));
    let (a, b, c) = (
        rect(&rects, i::INSP_AUDIO_SET_ON[0]),
        rect(&rects, i::INSP_AUDIO_SET_WEIGHT[0]),
        rect(&rects, i::INSP_AUDIO_SET_DEL[0]),
    );
    assert!(
        (a.y - b.y).abs() < 0.5 && (b.y - c.y).abs() < 0.5,
        "{a:?} {b:?} {c:?}"
    );
    assert!(a.x + a.w <= b.x && b.x + b.w <= c.x, "{a:?} {b:?} {c:?}");
    // E as duas variações dividem a linha.
    let (p, v) = (
        rect(&rects, i::INSP_AUDIO_PITCH_JITTER),
        rect(&rects, i::INSP_AUDIO_GAIN_JITTER),
    );
    assert!((p.y - v.y).abs() < 0.5 && p.x + p.w <= v.x, "{p:?} {v:?}");
    let abaixo = rect(&rects, i::INSP_AUDIO_SET_ON[1]);
    assert!(abaixo.y >= a.y + a.h, "linhas grudadas: {a:?} {abaixo:?}");
    fim();
}

/// O «Pick» oferece o que o snapshot traz e a escolha chega ao barramento; a lista fecha.
#[test]
fn pick_opens_offers_and_adds() {
    for cartoes in [true, false] {
        let (mut h, mut st, _) = palco(cartoes, com(conjunto(1, &["x.wav", "y.wav"])));
        h.set_dropdown_open(i::INSP_AUDIO_PICK, true);
        let rects = pinta(&mut h, &mut st);
        assert!(
            tem(&rects, i::INSP_AUDIO_PICK_OPT[1]),
            "a oferta não pintou (cartões {cartoes})"
        );
        assert!(
            !tem(&rects, i::INSP_AUDIO_PICK_OPT[2]),
            "só há duas ofertas"
        );
        let _ = h.drained_actions();
        h.apply_panel_event::<InspectorPanel>(
            &mut st,
            WidgetEvent::Click(i::INSP_AUDIO_PICK_OPT[1]),
        );
        assert_eq!(edicoes(&mut h), [AudioFieldEdit::SetAdd("y.wav".into())]);
        assert_eq!(h.dropdown_is_open(i::INSP_AUDIO_PICK), Some(false));
        fim();
    }
}

/// Projeto sem som: o controle aparece parado, sem alvo, e a primeira linha que o painel pinta fala a razão.
#[test]
fn pick_without_sounds_is_disabled_and_says_why() {
    let (_h, _st, rects) = palco(true, com(InspectorSoundSet::default()));
    assert!(
        !tem(&rects, i::INSP_AUDIO_PICK),
        "um chip parado não tem alvo"
    );
    assert!(
        tem(&rects, i::INSP_AUDIO_BROWSE),
        "o Browse continua a importar"
    );
    assert!(
        !tem(&rects, i::INSP_AUDIO_MODE),
        "sem som o Mode não decide nada"
    );
    assert!(!tem(&rects, i::INSP_AUDIO_PITCH_JITTER), "nem a variação");
    fim();
}

/// Mode e No repeat. No Sequence a caixa não tem alvo; nos outros, escreve.
#[test]
fn mode_picks_and_no_repeat_dies_only_in_sequence() {
    for cartoes in [true, false] {
        let (mut h, mut st, rects) = palco(cartoes, com(conjunto(2, &[])));
        assert_eq!(
            clica(&mut h, &mut st, rect(&rects, i::INSP_AUDIO_NO_REPEAT)),
            [AudioFieldEdit::SetNoRepeat(false)],
            "cartões {cartoes}"
        );
        h.set_dropdown_open(i::INSP_AUDIO_MODE, true);
        let rects = pinta(&mut h, &mut st);
        for k in 0..3 {
            assert!(
                tem(&rects, i::INSP_AUDIO_MODE_OPT[k]),
                "modo {k} (cartões {cartoes})"
            );
        }
        h.apply_panel_event::<InspectorPanel>(
            &mut st,
            WidgetEvent::Click(i::INSP_AUDIO_MODE_OPT[1]),
        );
        assert_eq!(edicoes(&mut h), [AudioFieldEdit::SetStrategy(1)]);
        assert_eq!(h.dropdown_is_open(i::INSP_AUDIO_MODE), Some(false));
        fim();

        let mut seq = conjunto(2, &[]);
        seq.strategy_tag = seq.sequence_tag;
        let (mut h, mut st, rects) = palco(cartoes, com(seq));
        assert!(
            !tem(&rects, i::INSP_AUDIO_NO_REPEAT),
            "controle morto (cartões {cartoes})"
        );
        h.apply_panel_event::<InspectorPanel>(
            &mut st,
            WidgetEvent::Toggled(i::INSP_AUDIO_NO_REPEAT),
        );
        assert!(edicoes(&mut h).is_empty(), "No repeat escreveu no Sequence");
        fim();
    }
}

/// As variações e o peso: o valor do snapshot aparece na caixa e a edição leva o número e o índice certos.
#[test]
fn weight_and_variations_show_the_snapshot_and_write_their_field() {
    let (mut h, mut st, _) = palco(true, com(conjunto(2, &[])));
    let v = |h: &MockPanelHost, id| h.store().number_value(id).unwrap_or(f64::NAN);
    assert!((v(&h, i::INSP_AUDIO_SET_WEIGHT[1]) - 2.0).abs() < 1e-6);
    assert!((v(&h, i::INSP_AUDIO_PITCH_JITTER) - 2.0).abs() < 1e-6);
    assert!((v(&h, i::INSP_AUDIO_GAIN_JITTER) - 3.0).abs() < 1e-6);
    let _ = h.drained_actions();
    h.store_mut()
        .set_number_value(i::INSP_AUDIO_SET_WEIGHT[1], 4.0);
    h.apply_panel_event::<InspectorPanel>(
        &mut st,
        WidgetEvent::ValueChanged(i::INSP_AUDIO_SET_WEIGHT[1]),
    );
    h.store_mut()
        .set_number_value(i::INSP_AUDIO_PITCH_JITTER, 5.0);
    h.apply_panel_event::<InspectorPanel>(
        &mut st,
        WidgetEvent::ValueChanged(i::INSP_AUDIO_PITCH_JITTER),
    );
    h.store_mut()
        .set_number_value(i::INSP_AUDIO_GAIN_JITTER, 6.0);
    h.apply_panel_event::<InspectorPanel>(
        &mut st,
        WidgetEvent::ValueChanged(i::INSP_AUDIO_GAIN_JITTER),
    );
    assert_eq!(
        edicoes(&mut h),
        [
            AudioFieldEdit::SetWeight(1, 4.0),
            AudioFieldEdit::SetPitchJitter(5.0),
            AudioFieldEdit::SetGainJitter(6.0),
        ]
    );
    fim();
}

/// Com a lista a mandar o campo Sound some; sem som ligado, ele volta.
#[test]
fn the_sound_field_gives_way_only_while_the_list_decides() {
    let (_h, _st, rects) = palco(true, com(conjunto(1, &[])));
    assert!(!tem(&rects, i::INSP_AUDIO_SOUND));
    assert!(tem(&rects, i::INSP_AUDIO_PREVIEW));
    fim();
    let mut desligada = conjunto(1, &[]);
    desligada.rows[0].enabled = false;
    desligada.decides = false;
    let (_h, _st, rects) = palco(true, com(desligada));
    assert!(tem(&rects, i::INSP_AUDIO_SOUND));
    fim();
}

/// Mais linhas do que há ids: o que cabe pinta, e não entra em pânico.
#[test]
fn more_rows_than_ids_still_paints() {
    let n = i::MAX_SOUND_SET_ROWS + 3;
    let (_h, _st, rects) = palco(false, com(conjunto(n, &[])));
    assert!(tem(&rects, i::INSP_AUDIO_SET_ON[0]));
    fim();
}

/// Sonda do limite (`MAX_SOUND_SET_ROWS`): o custo de pintar a lista cheia e de registar as tabelas.
#[test]
#[ignore = "sonda: cargo-test-narrow ... -- --ignored --nocapture"]
fn diag_o_custo_da_lista_cheia() {
    use std::time::Instant;
    let t = Instant::now();
    let _h = MockPanelHost::with_panel::<InspectorPanel>();
    println!("com o painel inteiro registado: {:?}", t.elapsed());
    for n in [0, 8, i::MAX_SOUND_SET_ROWS] {
        let (mut h, mut st, rects) = palco(true, com(conjunto(n, &[])));
        let mut melhor = u128::MAX;
        for _ in 0..30 {
            let t = Instant::now();
            let _ = h.paint::<InspectorPanel>(
                &mut st,
                Rect {
                    x: 0.0,
                    y: 0.0,
                    w: 1600.0,
                    h: 900.0,
                },
            );
            melhor = melhor.min(t.elapsed().as_micros());
        }
        let a = rects
            .iter()
            .find(|(id, _)| *id == i::INSP_AUDIO_SET_ON[0])
            .map(|(_, r)| *r);
        let b = rects
            .iter()
            .find(|(id, _)| *id == i::INSP_AUDIO_SET_ON[1])
            .map(|(_, r)| *r);
        println!(
            "{n} linhas: {melhor} us por pintura (mínimo de 30); passo {:?} px",
            a.zip(b).map(|(a, b)| b.y - a.y)
        );
        fim();
    }
}
