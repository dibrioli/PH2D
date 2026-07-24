//! **The Dur(s) chip authors what the artist TYPES — driven by the real gesture.**
//!
//! Every prior gate on this chain was green while the product failed (Enio,
//! 2026-07-23): the ruler gate built the snapshot by hand, the clamp gate called
//! `set_clip_length_override` directly, the seam gate injected a synthetic
//! `ValueChanged`, and the commit-always gate began pre-focused. None contained
//! the GESTURE — click the chip, type, Enter — which is where the bug lived:
//! focus did not select the buffer, so typing "2" into a chip showing "2"
//! parsed 22 and authored a 22 s duration (veil off-screen, clamp pinning
//! nothing, box "not matching the real duration": all three symptoms at once).
//!
//! These gates paint the REAL panel (`with_panel` runs `populate`, which
//! registers the chip and sets its commit-always flag), put a real pointer on
//! the painted rect, type through `dispatch_text_input`, press Enter through
//! `dispatch_key`, route the resulting events through the panel, and read the
//! intent the shell would drain.

use bumpalo::Bump;
use ph2d_editor_core::interaction::WidgetEvent;
use ph2d_editor_core::interaction::dispatch::keymap::KEY_ENTER;
use ph2d_editor_core::interaction::{dispatch_key, dispatch_text_input};
use ph2d_editor_core::panel::PanelHostInternal;
use ph2d_editor_core::zones::Rect;
use ph2d_host::{KeyEvent, KeyKind, Modifiers};
use ph2d_panel_timeline::TimelinePanel;
use ph2d_panel_timeline::state::{TimelinePanelState, drain_intents, set_current_timeline};
use ph2d_timeline::{TimelineIntent, TimelineViewSnapshot};
use ph2d_ui_testkit::MockPanelHost;

const VIEWPORT: Rect = Rect::new(0.0, 0.0, 1600.0, 900.0);

fn enter() -> KeyEvent {
    KeyEvent {
        keycode: KEY_ENTER,
        modifiers: Modifiers {
            shift: false,
            ctrl: false,
            alt: false,
            meta: false,
        },
        kind: KeyKind::Down,
        timestamp_ns: 0,
    }
}

/// Paint a stacked Keys view (`keys_mode`) over a document showing `secs` and return
/// the chip's RECT. `explicit=false` mirrors the report's derived-end screenshot;
/// `keys_mode=true` is a Keys view soloing a clip, so the chip authors the CLIP scope
/// — the scope keys on `keys_mode`, not the tab (a no-stack Keys view edits the scene,
/// covered by `duration_drag_tests`).
fn painted_dur_rect(host: &mut MockPanelHost, state: &mut TimelinePanelState, secs: f64) -> Rect {
    set_current_timeline(Some(TimelineViewSnapshot {
        fps: 60.0,
        view_length_seconds: secs,
        view_length_explicit: false,
        keys_mode: true,
        ..TimelineViewSnapshot::default()
    }));
    let regs = host.paint::<TimelinePanel>(state, VIEWPORT);
    regs.iter()
        .find(|(w, _)| *w == ph2d_panel_timeline::ids::TIMELINE_LENGTH_NUM)
        .map(|(_, r)| *r)
        .expect("the Dur(s) chip was painted but never hit-registered")
}

/// Paint the Keys tab over a 2 s derived-end doc and return a click point on the
/// chip's LEFT body (the right column is the stepper zone, a value bump — not a
/// focus gesture).
fn painted_dur_chip(host: &mut MockPanelHost, state: &mut TimelinePanelState) -> (f32, f32) {
    let r = painted_dur_rect(host, state, 2.0);
    (r.x + r.w * 0.3, r.y + r.h * 0.5)
}

fn route_events(
    host: &mut MockPanelHost,
    state: &mut TimelinePanelState,
    evs: impl IntoIterator<Item = WidgetEvent>,
) {
    for ev in evs {
        host.apply_panel_event::<TimelinePanel>(state, ev);
    }
}

/// The reported gesture end to end: the chip shows the derived 2.00, the artist
/// clicks it, types "2", presses Enter — the intent must carry 2.0. Before the
/// select-all-on-focus fix this authored `Some(22.0)`.
#[test]
fn typing_the_shown_duration_authors_that_duration() {
    let mut host = MockPanelHost::with_panel::<TimelinePanel>();
    let mut state = TimelinePanelState::default(); // tab: Keys
    let (cx, cy) = painted_dur_chip(&mut host, &mut state);

    let clicked = host.click_at(cx, cy);
    route_events(&mut host, &mut state, clicked);
    assert_eq!(
        drain_intents(),
        vec![],
        "the focusing click alone must author NOTHING (a stepper hit here \
         would silently author derived ± one frame)"
    );

    let arena = Bump::new();
    let _ = dispatch_text_input(host.store_mut(), '2', &arena);
    let evs: Vec<WidgetEvent> = dispatch_key(host.store_mut(), enter(), &arena).to_vec();
    route_events(&mut host, &mut state, evs);

    assert_eq!(
        drain_intents(),
        vec![TimelineIntent::SetClipLength { len: Some(2.0) }],
        "typing '2' into the chip showing 2.00 must author 2.0 — not 22 \
         (append), not nothing"
    );
    set_current_timeline(None);
}

/// The no-typing half (the commit-always chain, through the REAL populate):
/// click the chip, change nothing, Enter — the shown derived value becomes the
/// authored one. This is the derived→authored transition the chip exists for.
#[test]
fn enter_on_the_untouched_chip_authors_the_shown_value() {
    let mut host = MockPanelHost::with_panel::<TimelinePanel>();
    let mut state = TimelinePanelState::default();
    let (cx, cy) = painted_dur_chip(&mut host, &mut state);

    let clicked = host.click_at(cx, cy);
    route_events(&mut host, &mut state, clicked);
    let _ = drain_intents();

    let arena = Bump::new();
    let evs: Vec<WidgetEvent> = dispatch_key(host.store_mut(), enter(), &arena).to_vec();
    route_events(&mut host, &mut state, evs);

    assert_eq!(
        drain_intents(),
        vec![TimelineIntent::SetClipLength { len: Some(2.0) }],
        "Enter on the untouched chip must author the shown value \
         (`set_number_commit_always`, wired by the real populate)"
    );
    set_current_timeline(None);
}

/// Typing a DIFFERENT value replaces the readout (the general case): the chip
/// shows 2.00, the artist types 5, Enter — the intent carries 5.0, never 25.
#[test]
fn typing_a_new_duration_replaces_the_readout() {
    let mut host = MockPanelHost::with_panel::<TimelinePanel>();
    let mut state = TimelinePanelState::default();
    let (cx, cy) = painted_dur_chip(&mut host, &mut state);

    let clicked = host.click_at(cx, cy);
    route_events(&mut host, &mut state, clicked);
    let _ = drain_intents();

    let arena = Bump::new();
    let _ = dispatch_text_input(host.store_mut(), '5', &arena);
    let evs: Vec<WidgetEvent> = dispatch_key(host.store_mut(), enter(), &arena).to_vec();
    route_events(&mut host, &mut state, evs);

    assert_eq!(
        drain_intents(),
        vec![TimelineIntent::SetClipLength { len: Some(5.0) }],
        "typing '5' must author 5.0 — the focus click selects the readout"
    );
    set_current_timeline(None);
}

// ── the stepper arrows: +/- 0.2 s per click (Enio, 2026-07-23) ───────────────
//
// The Dur chip's right column is a stepper. It stepped by `1/fps` (~0.04 s), so a
// click produced the fiddly "wrong" values the report named (2.04, 2.08, …). A
// duration is coarser: each click is 0.2 s. `stepper_up_rect`/`_down_rect` mark the
// upper/lower half of the stepper column (`stepper_width` = clamp(h*0.6, 16, 22)).

use ph2d_editor_core::widget::NumberInput;

/// A click on the UP arrow raises the authored duration by exactly 0.2 s.
#[test]
fn the_up_stepper_adds_a_fifth_of_a_second() {
    let mut host = MockPanelHost::with_panel::<TimelinePanel>();
    let mut state = TimelinePanelState::default();
    let r = painted_dur_rect(&mut host, &mut state, 2.0);
    let up = NumberInput::new(ph2d_panel_timeline::ids::TIMELINE_LENGTH_NUM, "", 2.0).up_rect(r);
    let evs = host.click_at(up.x + up.w * 0.5, up.y + up.h * 0.5);
    route_events(&mut host, &mut state, evs);
    assert_eq!(
        drain_intents(),
        vec![TimelineIntent::SetClipLength { len: Some(2.2) }],
        "one up-click authors +0.2 s (was 1/fps ~0.04)"
    );
    set_current_timeline(None);
}

/// A click on the DOWN arrow lowers it by exactly 0.2 s.
#[test]
fn the_down_stepper_subtracts_a_fifth_of_a_second() {
    let mut host = MockPanelHost::with_panel::<TimelinePanel>();
    let mut state = TimelinePanelState::default();
    let r = painted_dur_rect(&mut host, &mut state, 2.0);
    let down =
        NumberInput::new(ph2d_panel_timeline::ids::TIMELINE_LENGTH_NUM, "", 2.0).down_rect(r);
    let evs = host.click_at(down.x + down.w * 0.5, down.y + down.h * 0.5);
    route_events(&mut host, &mut state, evs);
    assert_eq!(
        drain_intents(),
        vec![TimelineIntent::SetClipLength { len: Some(1.8) }],
        "one down-click authors -0.2 s"
    );
    set_current_timeline(None);
}

// ── the veil DRAG handle is painted + hit-registered (Enio, 2026-07-23) ───────
//
// A grip on the ruler at the veil's left edge — drag it to resize the composition.
// The handle must be REGISTERED by the paint, or it is dead under the mouse (the
// trap this project keeps hitting). It exists iff the view has an authored
// duration, exactly like the veil it edges.

/// With an authored duration on screen, the ruler paints and hit-registers the
/// duration handle. With none, the handle does not exist (no veil, no grip).
#[test]
fn the_duration_handle_is_painted_only_with_an_authored_duration() {
    let mut host = MockPanelHost::with_panel::<TimelinePanel>();
    let mut state = TimelinePanelState::default();

    // Authored 2 s duration → the handle is registered.
    set_current_timeline(Some(TimelineViewSnapshot {
        fps: 60.0,
        view_length_seconds: 2.0,
        view_length_explicit: true,
        ..TimelineViewSnapshot::default()
    }));
    let regs = host.paint::<TimelinePanel>(&mut state, VIEWPORT);
    assert!(
        regs.iter()
            .any(|(w, _)| *w == ph2d_panel_timeline::ids::TIMELINE_DUR_HANDLE),
        "an authored duration must paint a grabbable resize handle on the ruler"
    );

    // Derived end (nothing authored) → no veil, no handle.
    set_current_timeline(Some(TimelineViewSnapshot {
        fps: 60.0,
        view_length_seconds: 2.0,
        view_length_explicit: false,
        ..TimelineViewSnapshot::default()
    }));
    let regs = host.paint::<TimelinePanel>(&mut state, VIEWPORT);
    assert!(
        !regs
            .iter()
            .any(|(w, _)| *w == ph2d_panel_timeline::ids::TIMELINE_DUR_HANDLE),
        "no authored duration → no veil → no handle"
    );
    set_current_timeline(None);
}

/// **A Dur(s) aceita QUALQUER duração — o teto de `u16::MAX` era um palpite** (Enio,
/// 2026-07-23: *"a timeline está limitando a simulação … permita que eu possa colocar
/// qualquer valor em Dur"*).
///
/// Uma composição não tem um comprimento máximo que o app conheça, e o `u16::MAX` que o
/// chip registrava não era medido: era um número redondo servindo de cerca (CLAUDE.md §0 —
/// *um limite que só diz "por segurança" é um palpite esperando um smoke*). Ele mordia por
/// **duas** vias, e as duas estão aqui:
///
/// 1. **O stepper PARAVA nele.** Segurar a setinha a partir de uma duração longa pinava em
///    65535 s — e quem simula física precisa justamente da corrida longa.
/// 2. **O ARRASTO ficava inútil**, que é a via que o artista de fato usa: o modelo de
///    scrub deste app é *proporcional ao alcance* (`DRAG_RANGE_PX_H` = 250 px varrem
///    `[min, max]` inteiro), então com teto 65535 **um pixel valia 262 segundos**. Não dava
///    para pousar num número; a caixa parecia recusar edição.
///
/// ⚠️ O `step` e o piso `0` FICAM (o stepper de 0,2 s que o Enio pediu, e uma duração
/// negativa não é uma coisa — `0` é o gesto de LIMPAR). O que sai é só o teto.
#[test]
fn the_duration_box_takes_any_length() {
    let mut host = MockPanelHost::with_panel::<TimelinePanel>();
    let mut state = TimelinePanelState::default();
    let _ = painted_dur_rect(&mut host, &mut state, 2.0);
    let id = ph2d_panel_timeline::ids::TIMELINE_LENGTH_NUM;

    let (min, max, step) = host.store().number_range(id).expect(
        "a Dur(s) precisa de um `step` registrado — sem ele o stepper cai na \
                 heurística do buffer (0.01 num valor com ponto) e o incremento de 0,2 s \
                 que o Enio pediu some",
    );
    assert_eq!(
        min, 0.0,
        "o piso fica: uma duração negativa não é uma coisa"
    );
    assert_eq!(
        step, 0.2,
        "o stepper da Dur(s) é 0,2 s por clique (Enio, 2026-07-23) — este é o número que \
         se perderia ao simplesmente apagar o alcance"
    );
    assert!(
        max.is_infinite(),
        "a Dur(s) não pode ter TETO: {max} é uma cerca que ninguém mediu, e o stepper para \
         nela. Uma composição não tem comprimento máximo que este app conheça"
    );

    // E a outra metade: o ARRASTO tem de ser calibrado, não proporcional a um alcance.
    // Sem isto o scrub varre `[min, max]` em 250 px — com teto finito são centenas de
    // segundos por pixel, e com teto infinito seria pior ainda (delta não-finito).
    let rate = host.store().number_drag_rate(id).expect(
        "a Dur(s) precisa de um drag-rate calibrado: é ele que VENCE o modelo proporcional \
         (`pointer_move`) e dá ao arrasto uma escala em que dá para pousar num número",
    );
    assert!(
        rate.is_finite() && rate > 0.0 && rate <= 1.0,
        "um pixel de arrasto tem de valer uma fração de segundo, não centenas: rate = {rate}"
    );
    set_current_timeline(None);
}

/// **E o ARRASTO pousa num número** — a metade que o artista sente.
///
/// O gate acima afirma que existe um rate calibrado; este mede o que ele PRODUZ, que é a
/// pergunta de verdade: arrastar N pixels move a duração ~`N · step`, não `N · 262 s`.
/// Red-first — com o alcance `[0, 65535]` proporcional (250 px varrem tudo), este mesmo
/// arrasto de 20 px movia a caixa **5242 segundos**.
///
/// ⚠️ O oráculo é uma FAIXA, não um número exato: o dispatch tem limiar de arrasto
/// (`NUMBER_INPUT_DRAG_THRESHOLD_PX`) e acumula por Move, então o que importa é a ORDEM DE
/// GRANDEZA em que o gesto aterrissa — que é exatamente o que estava quebrado.
#[test]
fn dragging_the_duration_box_lands_on_a_number() {
    use ph2d_host::{PointerButton, PointerEvent, PointerKind, PointerSource};

    let mut host = MockPanelHost::with_panel::<TimelinePanel>();
    let mut state = TimelinePanelState::default();
    let r = painted_dur_rect(&mut host, &mut state, 2.0);
    let id = ph2d_panel_timeline::ids::TIMELINE_LENGTH_NUM;

    let (cx, cy) = (r.x + r.w * 0.3, r.y + r.h * 0.5);
    let ev = |x: f32, kind: PointerKind, t: u128| PointerEvent {
        x,
        y: cy,
        pressure: 1.0,
        kind,
        source: PointerSource::Mouse,
        button: PointerButton::Primary,
        timestamp_ns: t,
    };
    let _ = host.dispatch_pointer_event(ev(cx, PointerKind::Down, 0));
    // 20 px para a direita, em passos de 4 px (um mouse real entrega vários Moves).
    for i in 1..=5 {
        let _ = host.dispatch_pointer_event(ev(cx + (i * 4) as f32, PointerKind::Move, i as u128));
    }
    let _ = host.dispatch_pointer_event(ev(cx + 20.0, PointerKind::Up, 99));

    let v = host
        .store()
        .number_value(id)
        .expect("o chip guarda um número");
    let moved = v - 2.0;
    assert!(
        (0.0..=20.0).contains(&moved),
        "20 px de arrasto têm de mover a duração alguns segundos — mediu {moved} s. Sob o \
         modelo proporcional do teto de 65535 este mesmo gesto valia milhares de segundos, \
         e era isso que tornava a caixa impossível de ajustar"
    );
    set_current_timeline(None);
}
