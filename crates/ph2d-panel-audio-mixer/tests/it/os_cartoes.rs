//! ⭐ **A mesa de som em cartões: faixa continua faixa.** Cada barra e cada botão de todas as faixas e
//! cada campo do rack, pelo ponteiro real, até ao retrato que a shell lê.

use ph2d_a11y::NodeId;
use ph2d_card_stack::metrics as m;
use ph2d_editor_core::interaction::{WidgetEvent, dispatch_key};
use ph2d_editor_core::panel::PanelHostInternal;
use ph2d_editor_core::panel::cards::{dicas, list_menu_of};
use ph2d_editor_core::zones::Rect;
use ph2d_host::{PointerButton, PointerEvent, PointerKind, PointerSource};
use ph2d_panel_audio_mixer as mx;
use ph2d_panel_audio_mixer::AudioMixerPanel;
use ph2d_panel_audio_mixer::cards::{chip_de, force_cards_on_this_thread};
use ph2d_panel_audio_mixer::state::AudioMixerState;
use ph2d_ui_testkit::MockPanelHost;

const COLUNA: Rect = Rect {
    x: 0.0,
    y: 0.0,
    w: 340.0,
    h: 2400.0,
};

const SECOES: [NodeId; 5] = [
    mx::AMIX_SEC_EQ,
    mx::AMIX_SEC_REVERB,
    mx::AMIX_SEC_DELAY,
    mx::AMIX_SEC_COMP,
    mx::AMIX_SEC_DUCK,
];

struct Mesa {
    host: MockPanelHost,
    st: AudioMixerState,
    rects: Vec<(NodeId, Rect)>,
}

impl Mesa {
    fn nova() -> Self {
        force_cards_on_this_thread(Some(true));
        let mut host = MockPanelHost::with_panel::<AudioMixerPanel>();
        for s in SECOES {
            host.set_collapsed(s, false);
        }
        let mut mesa = Self {
            host,
            st: AudioMixerState::default(),
            rects: Vec::new(),
        };
        mesa.assenta();
        mesa
    }

    fn assenta(&mut self) {
        for _ in 0..60 {
            let (h, st) = (&mut self.host, &mut self.st);
            self.rects = ph2d_editor_core::aquecimento::com_passo_fixo(
                ph2d_editor_core::aquecimento::PASSO_DE_QUADRO_S,
                || h.paint::<AudioMixerPanel>(st, COLUNA),
            );
        }
    }

    fn rect(&self, id: NodeId) -> Rect {
        self.rects
            .iter()
            .rev()
            .find(|(n, r)| *n == id && r.w > 0.0 && r.h > 0.0)
            .map_or_else(|| panic!("{id:?} não pintado"), |(_, r)| *r)
    }

    fn aplica(&mut self, eventos: Vec<WidgetEvent>) {
        for e in eventos {
            let _ = self
                .host
                .apply_panel_event::<AudioMixerPanel>(&mut self.st, e);
        }
    }

    fn clica(&mut self, id: NodeId) {
        let r = self.rect(id);
        let (cx, cy) = (r.x + r.w * 0.5, r.y + r.h * 0.5);
        assert_eq!(self.host.hit_at(cx, cy), Some(id), "outro id no centro");
        let ev = self.host.click_at(cx, cy);
        assert!(!ev.is_empty(), "{id:?} morto sob o dedo");
        self.aplica(ev);
    }

    fn ponteiro(&mut self, kind: PointerKind, x: f32, y: f32) -> Vec<WidgetEvent> {
        self.host.dispatch_pointer_event(PointerEvent {
            kind,
            x,
            y,
            button: PointerButton::Primary,
            source: PointerSource::Mouse,
            pressure: 1.0,
            timestamp_ns: 0,
        })
    }

    /// Arrasta uma barra da posição `de` à `a` (fracções do comprimento), e larga.
    fn arrasta_a_barra(&mut self, id: NodeId, vertical: bool, de: f32, a: f32) {
        let r = self.rect(id);
        let ponto = |t: f32| {
            if vertical {
                (r.x + r.w * 0.5, r.y + r.h * (1.0 - t))
            } else {
                (r.x + r.w * t, r.y + r.h * 0.5)
            }
        };
        let (x0, y0) = ponto(de);
        let (x1, y1) = ponto(a);
        let mut ev = self.ponteiro(PointerKind::Down, x0, y0);
        ev.extend(self.ponteiro(PointerKind::Move, x1, y1));
        ev.extend(self.ponteiro(PointerKind::Up, x1, y1));
        assert!(
            ev.contains(&WidgetEvent::ValueChanged(id)),
            "{id:?} não respondeu: {ev:?}"
        );
        self.aplica(ev);
    }
}

fn todas_as_barras_do_rack() -> Vec<NodeId> {
    let mut v = vec![
        mx::AMIX_EQ_LOW,
        mx::AMIX_EQ_MID,
        mx::AMIX_EQ_HIGH,
        mx::AMIX_REVERB_SIZE,
        mx::AMIX_REVERB_MIX,
        mx::AMIX_DELAY_TIME,
        mx::AMIX_DELAY_FEEDBACK,
        mx::AMIX_DELAY_MIX,
        mx::AMIX_DUCK_DEPTH,
    ];
    v.extend(mx::SUB_SEND);
    v.extend(mx::SUB_DELAY_SEND);
    v.extend(mx::SUB_COMP);
    v
}

/// Todos os ids do corpo de antes: registados e pintados; as barras do rack são a pega do campo.
#[test]
fn every_control_of_the_old_panel_is_painted_in_cards() {
    let mesa = Mesa::nova();
    let mut pintados = vec![
        mx::AMIX_CLOSE,
        mx::AMIX_FADER,
        mx::AMIX_PAN,
        mx::AMIX_CUTOFF,
        mx::AMIX_LOWCUT,
        mx::AMIX_MASTER_MUTE,
        mx::AMIX_MASTER_METER,
        mx::AMIX_PLAY,
        mx::AMIX_LIMITER,
        mx::AMIX_REVERB,
        mx::AMIX_DELAY,
        mx::AMIX_DUCK,
    ];
    pintados.extend(mx::AMIX_DUCK_KEY_BUS);
    for ids in [
        mx::SUB_FADER,
        mx::SUB_PAN,
        mx::SUB_TONE,
        mx::SUB_LOWCUT,
        mx::SUB_MUTE,
        mx::SUB_SOLO,
        mx::SUB_METER,
    ] {
        pintados.extend(ids);
    }
    for id in pintados {
        assert!(mesa.host.store().get(id).is_some(), "{id:?} sem registo");
        let _ = mesa.rect(id);
    }
    for b in todas_as_barras_do_rack() {
        assert!(
            !mesa.rects.iter().any(|(n, r)| *n == b && r.w > 0.0),
            "a barra {b:?} ainda se pinta"
        );
        let _ = mesa.rect(chip_de(b));
    }
}

/// ⭐ Cada faixa: o fader, a pan e os dois cortes respondem ao arrasto real, um de cada vez, e chegam ao
/// retrato. Só a faixa arrastada muda.
#[test]
fn every_strip_answers_real_drags_and_writes_only_itself() {
    let mut mesa = Mesa::nova();
    let antes = mx::sub_gain_target();
    mesa.arrasta_a_barra(mx::SUB_FADER[1], true, 0.5, 0.2);
    let depois = mx::sub_gain_target();
    assert_ne!(depois[1], antes[1], "o fader do SFX não escreveu");
    assert_eq!(depois[0], antes[0], "mexeu na faixa ao lado");
    mesa.assenta();
    assert!(mesa.host.store().slider(mx::SUB_FADER[1]).is_some());
    for id in [
        mx::AMIX_PAN,
        mx::AMIX_CUTOFF,
        mx::AMIX_LOWCUT,
        mx::SUB_PAN[0],
        mx::SUB_TONE[2],
        mx::SUB_LOWCUT[3],
    ] {
        mesa.arrasta_a_barra(id, false, 0.5, 0.8);
    }
    mesa.arrasta_a_barra(mx::AMIX_FADER, true, 0.5, 0.2);
}

/// Mute, Solo, medidor e Play Test pelo clique real.
#[test]
fn the_strip_buttons_answer_a_real_click() {
    let mut mesa = Mesa::nova();
    let m0 = mx::master_muted();
    mesa.clica(mx::AMIX_MASTER_MUTE);
    assert_ne!(mx::master_muted(), m0);
    mesa.clica(mx::AMIX_MASTER_MUTE);
    let s = mx::sub_muted()[2];
    mesa.clica(mx::SUB_MUTE[2]);
    assert_ne!(mx::sub_muted()[2], s);
    mesa.clica(mx::SUB_MUTE[2]);
    let so = mx::sub_soloed()[1];
    mesa.clica(mx::SUB_SOLO[1]);
    assert_ne!(mx::sub_soloed()[1], so);
    mesa.clica(mx::SUB_SOLO[1]);
    let p = mx::play_test();
    mesa.clica(mx::AMIX_PLAY);
    assert_ne!(mx::play_test(), p);
    mesa.clica(mx::AMIX_PLAY);
    mesa.clica(mx::AMIX_MASTER_METER);
    mesa.clica(mx::SUB_METER[0]);
}

/// ⭐ Cada campo do rack pelo scrub real: avisa pela barra, escreve o retrato e o Esc devolve.
#[test]
fn every_rack_field_scrubs_through_its_bar_and_escape_gives_it_back() {
    for barra in todas_as_barras_do_rack() {
        let mut mesa = Mesa::nova();
        let chip = chip_de(barra);
        let antes = mesa.host.store().slider(barra).map_or(0.0, |(_, v)| v);
        let r = mesa.rect(chip);
        let (x, y) = (r.x + m::ICON_CELL * 0.5, r.y + r.h * 0.5);
        let dx = if antes > 0.5 { -60.0 } else { 60.0 };
        let mut ev = mesa.ponteiro(PointerKind::Down, x, y);
        ev.extend(mesa.ponteiro(PointerKind::Move, x + dx * 0.5, y));
        ev.extend(mesa.ponteiro(PointerKind::Move, x + dx, y));
        assert!(
            ev.contains(&WidgetEvent::ValueChanged(barra)),
            "{barra:?} não avisou pela barra: {ev:?}"
        );
        let durante = mesa.host.store().slider(barra).map_or(0.0, |(_, v)| v);
        assert!((durante - antes).abs() > 0.01, "{barra:?} não se moveu");
        mesa.aplica(ev);
        let arena = bumpalo::Bump::new();
        let esc = ph2d_host::KeyEvent {
            keycode: 0x1B,
            modifiers: ph2d_host::Modifiers::default(),
            kind: ph2d_host::KeyKind::Down,
            timestamp_ns: 0,
        };
        let ev = dispatch_key(mesa.host.store_mut(), esc, &arena).to_vec();
        mesa.aplica(ev);
        let fim = mesa.host.store().slider(barra).map_or(0.0, |(_, v)| v);
        assert!((fim - antes).abs() < 1e-3, "{barra:?}: o Esc não devolveu");
    }
}

/// As quatro caixas de efeito e a chave do ducking.
#[test]
fn effect_checks_and_the_key_choice_answer_a_real_click() {
    let mut mesa = Mesa::nova();
    let l = mx::limiter();
    mesa.clica(mx::AMIX_LIMITER);
    assert_ne!(mx::limiter(), l);
    mesa.clica(mx::AMIX_LIMITER);
    let r = mx::reverb_on();
    mesa.clica(mx::AMIX_REVERB);
    assert_ne!(mx::reverb_on(), r);
    mesa.clica(mx::AMIX_REVERB);
    let d = mx::delay_on();
    mesa.clica(mx::AMIX_DELAY);
    assert_ne!(mx::delay_on(), d);
    mesa.clica(mx::AMIX_DELAY);
    let du = mx::ducking();
    mesa.clica(mx::AMIX_DUCK);
    assert_ne!(mx::ducking(), du);
    mesa.clica(mx::AMIX_DUCK);
    mesa.clica(mx::AMIX_DUCK_KEY_BUS[2]);
    assert_eq!(mx::ducking_key(), 2);
    mesa.clica(mx::AMIX_DUCK_KEY_BUS[0]);
}

/// Os efeitos pesados nascem dobrados e a escolha do artista ganha à semente; os cartões movem-se.
#[test]
fn the_heavy_effects_are_born_folded_and_cards_reorder() {
    force_cards_on_this_thread(Some(true));
    let mut host = MockPanelHost::with_panel::<AudioMixerPanel>();
    let mut st = AudioMixerState::default();
    let _ = host.paint::<AudioMixerPanel>(&mut st, COLUNA);
    assert!(host.store().is_collapsed(mx::AMIX_SEC_REVERB));
    assert!(!host.store().is_collapsed(mx::AMIX_SEC_EQ));
    let mut mesa = Mesa::nova();
    let a = mesa.rect(mx::AMIX_SEC_OUTPUT);
    let b = mesa.rect(mx::AMIX_SEC_CHANNELS);
    let (x, y0) = (a.x + a.w * 0.4, a.y + m::CARD_HEAD_H * 0.5);
    let _ = mesa.ponteiro(PointerKind::Down, x, y0);
    let _ = mesa.ponteiro(PointerKind::Move, x, y0 + m::CARD_HEAD_H);
    mesa.assenta();
    let alvo = b.y + b.h;
    let _ = mesa.ponteiro(PointerKind::Move, x, alvo);
    mesa.assenta();
    let _ = mesa.ponteiro(PointerKind::Up, x, alvo);
    mesa.assenta();
    let ordem = mesa.host.store().section_order().to_vec();
    let pos = |id| ordem.iter().position(|&k| k == id);
    assert!(
        pos(mx::AMIX_SEC_OUTPUT) > pos(mx::AMIX_SEC_CHANNELS),
        "{ordem:?}"
    );
}

/// Todo ícone sem texto tem dica.
#[test]
fn every_mute_control_has_a_tooltip() {
    let mesa = Mesa::nova();
    let store = mesa.host.store();
    let mudos: Vec<NodeId> = dicas::icon_only_ids(mx::AMIX_PANEL)
        .into_iter()
        .chain([mx::AMIX_CLOSE, list_menu_of(mx::AMIX_PANEL)])
        .filter(|id| store.tooltip_for(*id).is_none())
        .collect();
    assert!(mudos.is_empty(), "sem dica: {mudos:?}");
}

/// `PH2D_CARDS=0`: a mesa de antes, com as barras do rack à vista.
#[test]
fn cards_off_paints_the_old_body() {
    force_cards_on_this_thread(Some(false));
    let mut host = MockPanelHost::with_panel::<AudioMixerPanel>();
    let mut st = AudioMixerState::default();
    let rects = host.paint::<AudioMixerPanel>(&mut st, COLUNA);
    assert!(rects.iter().any(|(id, _)| *id == mx::AMIX_EQ_LOW));
    assert!(!rects.iter().any(|(id, _)| *id == chip_de(mx::AMIX_EQ_LOW)));
}
