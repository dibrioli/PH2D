//! **Os toggles do Onion existem NA TELA, CLICAM e empurram `SetOnion`** (ADR-0142 W3).
//!
//! O onion é estado de vista GLOBAL (não per-objeto como o Motion Path), então — como o
//! Dur(s) — ele não sai como `PanelEvent` para a shell: o handler lê o onion autoritativo
//! do snapshot, vira o bit/modo, e empurra um `TimelineIntent::SetOnion` que o
//! `apply_intent` grava em `TimelineState::onion`. O gate pinta de verdade (o `paint`
//! devolve os rects que registrou) e dirige um ponteiro real, porque três omissões
//! diferentes leem como "o botão não faz nada" e só uma é pega por um `WidgetEvent`
//! sintético: pintado-sem-registro, registrado-fora-do-`populate` (morto sob o mouse), e
//! roteado-para-lugar-nenhum.

use ph2d_editor_core::ids;
use ph2d_editor_core::interaction::WidgetEvent;
use ph2d_editor_core::zones::Rect;
use ph2d_panel_timeline::TimelinePanel;
use ph2d_panel_timeline::state::{TimelinePanelState, drain_intents, set_current_timeline};
use ph2d_timeline::{OnionMode, OnionSettings, TimelineIntent, TimelineViewSnapshot};
use ph2d_ui_testkit::MockPanelHost;

const VIEWPORT: Rect = Rect::new(0.0, 0.0, 1600.0, 900.0);

fn transport(onion: OnionSettings) -> TimelineViewSnapshot {
    TimelineViewSnapshot {
        fps: 60.0,
        onion,
        ..TimelineViewSnapshot::default()
    }
}

fn paint(
    host: &mut MockPanelHost,
    state: &mut TimelinePanelState,
    snap: TimelineViewSnapshot,
) -> Vec<(ph2d_editor_core::NodeId, Rect)> {
    set_current_timeline(Some(snap));
    host.paint::<TimelinePanel>(state, VIEWPORT)
}

/// Clica o toggle e devolve o único `SetOnion` que ele empurrou.
fn click_onion(id: ph2d_editor_core::NodeId, snap: TimelineViewSnapshot) -> OnionSettings {
    let mut host = MockPanelHost::with_panel::<TimelinePanel>();
    let mut state = TimelinePanelState::default();
    let regs = paint(&mut host, &mut state, snap);
    let r = regs
        .iter()
        .find(|(w, _)| *w == id)
        .map(|(_, r)| *r)
        .unwrap_or_else(|| panic!("o toggle {id:?} foi pintado mas nunca hit-registrado"));

    let _ = drain_intents(); // limpa o canal antes de agir
    let (cx, cy) = (r.x + r.w * 0.5, r.y + r.h * 0.5);
    let evs = host.click_at(cx, cy);
    let toggled = evs
        .iter()
        .find(|e| matches!(e, WidgetEvent::Toggled(t) if *t == id))
        .copied()
        .unwrap_or_else(|| panic!("o ponteiro caiu em {id:?} mas nenhum Toggled saiu — {evs:?}"));

    host.apply_panel_event::<TimelinePanel>(&mut state, toggled);
    match drain_intents().into_iter().find_map(|i| match i {
        TimelineIntent::SetOnion(o) => Some(o),
        _ => None,
    }) {
        Some(o) => o,
        None => panic!("clicar {id:?} não empurrou um SetOnion"),
    }
}

#[test]
fn the_onion_toggle_paints_clicks_and_pushes_set_onion() {
    // Off no snapshot ⇒ o clique empurra SetOnion com enabled = true.
    let out = click_onion(ids::TIMELINE_ONION, transport(OnionSettings::default()));
    assert!(out.enabled, "clicar Onion (off) tem de armar o onion");

    // On no snapshot ⇒ o clique o desliga.
    let on = OnionSettings { enabled: true, ..OnionSettings::default() };
    let out = click_onion(ids::TIMELINE_ONION, transport(on));
    assert!(!out.enabled, "clicar Onion (on) tem de desligar");
}

#[test]
fn the_onion_mode_toggle_flips_keys_and_frames() {
    let keys = OnionSettings { mode: OnionMode::Keys, ..OnionSettings::default() };
    let out = click_onion(ids::TIMELINE_ONION_MODE, transport(keys));
    assert_eq!(out.mode, OnionMode::Frames, "Keys -> Frames");

    let frames = OnionSettings { mode: OnionMode::Frames, ..OnionSettings::default() };
    let out = click_onion(ids::TIMELINE_ONION_MODE, transport(frames));
    assert_eq!(out.mode, OnionMode::Keys, "Frames -> Keys");
}

#[test]
fn the_painted_onion_switches_show_the_snapshot() {
    // O switch pintado segue o snapshot em AMBAS as direções — senão o botão pode
    // discordar do que o passe de fantasmas de fato desenha.
    for enabled in [false, true] {
        for mode in [OnionMode::Frames, OnionMode::Keys] {
            let mut host = MockPanelHost::with_panel::<TimelinePanel>();
            let mut state = TimelinePanelState::default();
            let onion = OnionSettings { enabled, mode, ..OnionSettings::default() };
            paint(&mut host, &mut state, transport(onion));

            let (_, on) = host
                .store()
                .toggle(ids::TIMELINE_ONION)
                .expect("o toggle Onion não está registrado — veja populate.rs");
            assert_eq!(on, enabled, "o switch Onion discorda do snapshot");

            let (_, keys_on) = host
                .store()
                .toggle(ids::TIMELINE_ONION_MODE)
                .expect("o toggle Onion Keys não está registrado — veja populate.rs");
            assert_eq!(keys_on, mode == OnionMode::Keys, "o switch Keys discorda do snapshot");
        }
    }
}
