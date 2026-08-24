//! Timeline bridge — the per-frame glue between the app-general
//! [`TimelineState`] and the scene.
//!
//! Once per frame it drains the pending [`TimelineIntent`]s (from the panel /
//! auto-key) through `apply_intent` — which mutates the document + transport +
//! selection, one undo step per gesture — and then applies the document to the
//! world at the Playhead via `apply_from_doc`. Both halves are pure, tested
//! functions in `ph2d-timeline`; this module only composes them (mirrors how
//! `motion_bridge` / `vector_bridge` compose their crate logic).
//!
//! A no-op when the document is empty (no bindings) and no intents are pending,
//! so any programmatic `SpriteAnimation` bind is left untouched.

use ph2d_core::Playhead;
use ph2d_ecs::World;
use ph2d_editor::tool::PanelEvent;
use ph2d_timeline::{
    PropKind, TimelineIntent, TimelineSignal, TimelineState, apply_intent, apply_scene,
};

/// ⚠️ **A AUTORIA de chaves mudou-se para o irmão** ([`super::timeline_bridge_keys`]) quando este
/// ficheiro bateu no teto de LOC — e é re-exportada daqui para que nenhum chamador tenha de mudar
/// de `use`. *Um corte que obriga vinte ficheiros a mudar de endereço é um corte que ninguém
/// repete.*
pub(crate) use super::timeline_bridge_keys::*;

/// The timeline's signal outbox (ADR-0143). When forward SCENE play crosses a marker
/// that carries a signal, an event lands in `out`; the shell drains it after the apply
/// and hands it to consumers (the v1 consumer is a toast; audio/gameplay/Luau are the
/// deferred cross-line downstream). **Decoupled (ADR-0075):** [`Self::emit`] FILLS
/// `out` and never calls a consumer — the timeline emits an event, it does not make a
/// call. Scrub, reverse, pause and any non-Arrange view emit nothing and re-baseline
/// silently — the physics `hold`/`rewind` re-baseline, one module over.
#[derive(Debug, Default)]
pub(crate) struct SignalEmitter {
    /// The scene time the last forward tick reached — the `prev` of the crossing law.
    last_time: f64,
    /// Signals crossed this frame, drained by the shell. Cleared on every `emit`.
    pub out: Vec<TimelineSignal>,
    /// O `jumped` DESTE quadro — publicado aqui porque um SEGUNDO produtor precisa da
    /// mesma resposta, e duas cópias divergiriam no dia em que uma ganhasse um caso
    /// especial. Gravado em `run` (fora do braço Arrange), então ele descreve o quadro
    /// inteiro e não só a vista em que markers vivem.
    pub jumped: bool,
}

impl SignalEmitter {
    /// Refill `out` from the SCENE playhead's forward advance across the document's
    /// markers, then re-baseline. Called once per frame from [`run`]'s Arrange branch
    /// with the SCENE clock — the only view where the scene plays and markers live.
    ///
    /// Forward play only ([`Playhead::is_advancing_forward`]): a scrub, a reverse leg
    /// and a pause fire nothing but STILL re-baseline `last_time`, so the next play
    /// does not fire the gap the artist skipped over. This is the exact `hold`/`rewind`
    /// discipline the physics bridge uses for its contact events.
    ///
    /// `jumped` is `true` when a Scrub/SeekFrame intent moved the playhead this frame:
    /// a seek forward WHILE playing looks like a huge forward advance, but it is a
    /// discontinuity, not a crossing — so it re-baselines and fires nothing.
    fn emit(&mut self, doc: &ph2d_timeline::TimelineDoc, playhead: &Playhead, jumped: bool) {
        self.out.clear();
        let now = playhead.time();
        if super::clock_forward::clock_is_playing_forward(playhead, jumped) {
            for name in ph2d_timeline::signals_crossed(
                doc.markers(),
                self.last_time,
                now,
                playhead.loop_range(),
            ) {
                self.out.push(TimelineSignal {
                    name: name.to_string(),
                    t: now,
                });
            }
        }
        self.last_time = now;
    }
}

/// Drain pending intents into `timeline`, then apply its document to `world` at
/// the current `playhead` time. The apply leaves untouched: `live_entity` (the
/// entity whose gizmo is being dragged this frame, if any) and every entity in
/// `ak.displaced` (a pose the user displaced while paused, waiting for a manual
/// K — see `autokey_pass`), so the document does not fight the manipulation.
/// Call each frame in the apply pass, after `apply_sprite_animations`.
///
/// Returns `true` when the identity upkeep RESET the document (the last animated
/// object was deleted — `timeline_persist::upkeep`); the caller rewinds the
/// clocks and drops the panel's container trail, which this function cannot do
/// because it only holds ONE of the two playheads.
// The frame's inputs, each load-bearing (the world, the two-way state, the active
// clock, the drag/displaced skips, and the two view discriminants `solo`/`container`).
// Bundling them into a struct would hide which the caller must supply per frame.
#[allow(clippy::too_many_arguments)]
pub(crate) fn run(
    world: &mut World,
    timeline: &mut TimelineState,
    playhead: &mut Playhead,
    intents: &mut Vec<TimelineIntent>,
    live_entity: Option<u64>,
    ak: &mut super::autokey_pass::AutokeyState,
    solo: bool,
    container: Option<usize>,
    signals: &mut SignalEmitter,
    drive: &mut crate::preview_drive::PreviewDrive,
) -> bool {
    // **The Containers list has no playback mode** (Enio, 2026-07-22). The two
    // refusal layers upstream (the play button paints dead + unhittable; the
    // TogglePlay intent maps to None) stop the GESTURES — this is the backstop
    // for the clock itself: switching to the list while playing, or any path
    // that reaches the playhead without asking the panel (spacebar), lands here
    // and the clock pauses. In the list `keys_mode` is false, so `playhead` IS
    // the scene clock the artist would otherwise watch run with dead controls.
    if timeline.containers_list && playhead.is_playing() {
        playhead.pause();
    }
    // A Scrub/SeekFrame this frame is a DISCONTINUITY, not a crossing — a seek
    // forward while playing must not fire every signal it skipped (ADR-0143 §3).
    let mut jumped = false;
    for intent in intents.drain(..) {
        jumped |= matches!(
            intent,
            TimelineIntent::Scrub(_) | TimelineIntent::SeekFrame(_)
        );
        apply_intent(timeline, playhead, intent);
    }
    // Publicado para o SEGUNDO produtor (a ponte de Motion, mais abaixo no quadro).
    signals.jumped = jumped;
    // **O PLAYHEAD É LIVRE — não há parede na duração autorada** (Enio, 2026-07-25). O transporte
    // também dirige a FÍSICA dinâmica, e um clamp aqui capava a simulação no fim da timeline; então
    // a duração de composição segue sendo a duração VISÍVEL (o véu) e o alvo de NAVEGAÇÃO (go-to-end,
    // o loop recém-armado — mecanismos à parte, intactos), mas NÃO um muro que para o relógio.
    //
    // ⚠️ **A AVALIAÇÃO de clips/strips/containers/Arrange é INTOCADA:** `clip_cut`/`container_cut`/
    // `cut_scene` clampam o RELÓGIO que o AVALIADOR lê (`apply_active_clip`/`apply_container` abaixo),
    // então a animação ainda CONGELA no fim autorado — só o playhead passa, e a física continua a
    // simular. Aqui havia um `pause() + seek(view_authored_end)` (a "parede" do AE comp end, 07-23);
    // foi removido DE PROPÓSITO — o gate `the_playhead_runs_free_past_the_authored_end` (nos
    // `timeline_bridge_container_tests`) impede re-introduzi-lo. `view_authored_end` segue vivo: é a
    // porta do VÉU (`snapshot`).
    // A playhead move (scrub, play, frame step) reclaims every displaced pose
    // for the animation — Blender semantics: the un-keyed pose is discarded.
    if playhead.time() != ak.displaced_t {
        ak.displaced.clear();
        ak.displaced_t = playhead.time();
    }
    let displaced = &ak.displaced;
    let skip = |bits: u64| live_entity == Some(bits) || displaced.contains(&bits);
    // **Three clocks, three views** (Enio, 2026-07-16 / 2026-07-22):
    // - Keys solos the active CLIP at its own clock (`apply_active_clip`) — pose the
    //   exact curves you edit, stack out of the way.
    // - Inside a container, solo the CONTAINER's interior at ITS clock
    //   (`apply_container`) — the playback the animator is editing IS the container's.
    // - Arrange blends the scene stack at the timeline clock — and an EMPTY Arrange
    //   plays NOTHING (`apply_scene`, not `apply_from_doc_except`): Arrange is a
    //   first-class scope independent of any clip, so a clip previews on Keys until
    //   it is arranged (Enio, 2026-07-27). `apply_scene` forces the stack path, so
    //   with nothing arranged every entity blends toward `rest`.
    // All honour `skip` (the gizmo-dragged entity, the displaced pin).
    // ⚠️ **A DIREÇÃO do relógio, estampada ANTES do apply** (Enio, 2026-08-01: *"a direção
    // do easing é invertida se a playhead está voltando a zero"*): um fade percorrido de trás
    // para frente — a perna de volta de um ping-pong — tem de SENTIR o easing que o artista
    // autorou, e sem o espelho um `Ease In` lido ao contrário lê como `Ease Out`.
    //
    // Quem sabe a direção é o `Playhead`, que não alcança o avaliador; o doc a carrega como
    // transiente (`#[serde(skip)]`, nenhum schema se move). **Pausado é para a frente**: um
    // scrub para trás é o artista LENDO a cena, não a animação rodando ao contrário.
    timeline
        .doc
        .set_reverse_play(playhead.is_playing() && !playhead.is_advancing_forward());
    // **O ESTADO DE ANTES** — o porquê inteiro vive no [`crate::timeline_preview`], que é o dono
    // do conceito; aqui fica só a ordem, que é o que pode estar errado neste ficheiro.
    let bound_before = crate::timeline_preview::state_of_bindings(world, &timeline.doc);
    if let Some(c) = container {
        ph2d_timeline::apply_container(world, &mut timeline.doc, c, playhead.time(), skip);
    } else if solo {
        ph2d_timeline::apply_active_clip(world, &mut timeline.doc, playhead.time(), skip);
    } else {
        apply_scene(world, &mut timeline.doc, playhead.time(), skip);
        // Arrange is the ONLY view where the scene plays and markers live, so it is
        // the only view that emits signals (ADR-0143). Here `playhead` IS the scene
        // clock (the caller passes `self.playhead` only in this branch). The other
        // two views freeze the scene clock, so nothing bursts on return.
        signals.emit(&timeline.doc, playhead, jumped);
    }
    // E o que os três ramos escreveram é declarado por UM sítio — pô-lo dentro de cada braço seria
    // três cópias, e a que ficasse de fora é a que ninguém repara (o Arrange é o ramo comum).
    crate::timeline_preview::declare_timeline_writes(world, &bound_before, drive);
    // Identity upkeep: heal (a project load's detached bindings recolam pelo
    // nome), then purge — a deleted object's tracks leave the document with it,
    // and deleting the LAST animated object resets the timeline whole
    // (`timeline_persist::upkeep`).
    crate::timeline_persist::upkeep(timeline, world)
}

/// Translate a transport [`PanelEvent`] (by widget id) into a [`TimelineIntent`].
/// The timeline semantics live here so editor-core stays timeline-agnostic;
/// frame-relative and duration-relative commands read the current
/// `playhead`/`timeline`. Returns `None` for ids this panel does not own.
pub(crate) fn intent_for_transport(
    ev: &PanelEvent,
    timeline: &TimelineState,
    playhead: &Playhead,
) -> Option<TimelineIntent> {
    use TimelineIntent as I;
    use ph2d_editor::ids;
    let fps = timeline.doc.fps_display;
    // **"The end" is the end of what THIS VIEW shows** (`TimelineDoc::view_end_seconds`):
    // the active clip on Keys, the last strip on Arrange. Both go-to-end and a freshly
    // armed loop read it, which is the point — they are the same question, and asking
    // the clip while looking at the stack bracketed one strip out of the set.
    //
    // Within a clip it is the last keyframe when that runs past the authored duration:
    // a fresh clip's duration is 0, which would pin both at t = 0 for every hand-keyed
    // animation.
    let duration = || timeline.doc.view_end_seconds(timeline.keys_mode);
    // **Inside a container the transport is the CONTAINER's own clock** (Enio, 2026-07-22:
    // *"o playback deve ser relativo ao container aberto em edição"*), so every command
    // here speaks the container's LOCAL time: go-to-start is 0, go-to-end is its length,
    // and Loop/PingPong write the CONTAINER's own loop ([`TimelineIntent::SetContainerLoop`],
    // persisted on the `NamedContainer`) — independent of the scene's and of every clip's.
    // `None` on Keys / at the scene root, where the arms below keep their old behaviour.
    let container: Option<usize> = (!timeline.keys_mode)
        .then(|| timeline.edit_path.last().map(|s| s.container))
        .flatten();
    let container_len = |c: usize| timeline.doc.container_length_seconds(c);
    match *ev {
        // No playback mode on the Containers LIST (Enio, 2026-07-22): the panel
        // already paints the button dead and unhittable — this layer keeps a
        // synthetic/stale click from starting a clock the view says cannot run
        // ([[feedback_layered_defenses_need_per_layer_gates]]).
        PanelEvent::Click(id) if id == ids::TIMELINE_PLAY && timeline.containers_list => None,
        PanelEvent::Click(id) if id == ids::TIMELINE_PLAY => Some(I::TogglePlay),
        // Go-to-start is 0 in every mode — the local start of whatever clock the shell
        // hands us (scene, clip, or container). (In container mode the clock is the
        // container's own, so 0 is its interior start, not the scene's.)
        PanelEvent::Click(id) if id == ids::TIMELINE_GO_START => Some(I::Scrub(0.0)),
        PanelEvent::Click(id) if id == ids::TIMELINE_ADD_MARKER => Some(I::AddMarker {
            t_seconds: playhead.time(),
            label: format!("M{}", timeline.doc.markers().len() + 1),
        }),
        PanelEvent::Click(id) if id == ids::TIMELINE_GO_END => {
            Some(I::Scrub(container.map_or_else(duration, container_len)))
        }
        PanelEvent::Click(id) if id == ids::TIMELINE_PREV_FRAME => {
            Some(I::SeekFrame(playhead.frame(fps) - 1))
        }
        PanelEvent::Click(id) if id == ids::TIMELINE_NEXT_FRAME => {
            Some(I::SeekFrame(playhead.frame(fps) + 1))
        }
        PanelEvent::SetValue(id, v) if id == ids::TIMELINE_TIME_NUM => Some(I::Scrub(v)),
        PanelEvent::SetValue(id, v) if id == ids::TIMELINE_RULER => Some(I::Scrub(v)),
        PanelEvent::SetValue(id, v) if id == ids::TIMELINE_FRAME_NUM => {
            Some(I::SeekFrame(v as i64))
        }
        // Loop and PingPong are ONE loop seen two ways — a range plus what happens
        // at its end. Each toggle sends the whole value, so arming one necessarily
        // disarms the other: there is no state where both are on, and no rule
        // anyone has to remember to enforce.
        PanelEvent::Toggle(id, on) if id == ids::TIMELINE_LOOP || id == ids::TIMELINE_PINGPONG => {
            let ping_pong = id == ids::TIMELINE_PINGPONG;
            // Inside a container the toggle writes the CONTAINER's OWN loop, over its
            // local `[0, length)` — persisted on the `NamedContainer`, independent of
            // the scene's and every clip's (Enio, 2026-07-22). The leak this replaces
            // was a container toggle reaching the SCENE's loop.
            if let Some(c) = container {
                let len = container_len(c).max(1.0 / fps.max(1.0));
                return Some(I::SetContainerLoop {
                    container: c,
                    range: on.then_some((0.0, len)),
                    ping_pong: on && ping_pong,
                });
            }
            Some(if on {
                I::SetLoop {
                    range: Some((0.0, duration().max(1.0 / fps.max(1.0)))),
                    ping_pong,
                }
            } else {
                I::SetLoop {
                    range: None,
                    ping_pong: false,
                }
            })
        }
        PanelEvent::Toggle(id, on) if id == ids::TIMELINE_PHYSICS => {
            Some(I::SetSimulatePhysics(on))
        }
        PanelEvent::Toggle(id, on) if id == ids::TIMELINE_AUTOKEY => Some(I::SetAutoKey(on)),
        PanelEvent::Toggle(id, on) if id == ids::TIMELINE_RECORD => Some(I::SetPerforming(on)),
        // TIMELINE_MOTION_PATH is NOT translated here: it is per-object and needs the
        // selection, which this pure translator does not have — the shell resolves the
        // entity and emits `ConvertPositionMode` (mod.rs, mirror of the +Track path).
        PanelEvent::Toggle(id, on) if id == ids::TIMELINE_SNAP => Some(I::SetFrameSnap(on)),
        _ => None,
    }
}

/// **Selecionar um objeto NOVO leva a timeline à aba Keys** (Enio, 2026-07-22) — a
/// decisão de BORDA: dispara quando a seleção primária muda para um objeto (qualquer
/// um), e só então. Deselecionar não puxa aba nenhuma, e re-observar a MESMA seleção
/// frame após frame também não — senão o animador nunca conseguiria ficar em
/// Containers/Arrange com um objeto selecionado.
pub(crate) fn selection_jumps_to_keys(prev: Option<u64>, now: Option<u64>) -> bool {
    now.is_some() && now != prev
}

/// Whether this transport event **jumps** the playhead to a time the current
/// view may not show — go-to-start/end, a frame step off the edge, a typed time
/// or frame. The shell asks the panel to pan the playhead back into view after
/// applying one (the panel page-follows only while playing).
///
/// Deliberately excludes the ruler scrub (its value is a fraction of the visible
/// span, so it can never land off-screen) and the flag toggles.
pub(crate) fn jumps_the_playhead(ev: &PanelEvent) -> bool {
    use ph2d_editor::ids;
    match *ev {
        PanelEvent::Click(id) => {
            id == ids::TIMELINE_GO_START
                || id == ids::TIMELINE_GO_END
                || id == ids::TIMELINE_PREV_FRAME
                || id == ids::TIMELINE_NEXT_FRAME
        }
        PanelEvent::SetValue(id, _) => {
            id == ids::TIMELINE_TIME_NUM || id == ids::TIMELINE_FRAME_NUM
        }
        _ => false,
    }
}

/// Map a "+Track" property-button id to its [`PropKind`] (the shell binds the
/// selected sprite's matching property). `None` for non-"+Track" ids.
///
/// ⚠️ **Lê a tabela que o painel PINTA** (`ADDPROP_BUTTONS`), em vez de repetir o
/// mapeamento aqui. Isto era uma segunda cópia escrita à mão, e ela apodreceu na
/// primeira oportunidade: o `+Track → Position` (ADR-0141) foi para a tabela do painel,
/// nasceu pintado, registrado e clicável, o clique chegou até aqui — e caiu no `_ =>
/// None`, **sem erro e sem nada na tela**. O gate que existia amostrava três das sete
/// entradas e nunca poderia ter pego.
///
/// A tabela já pareia id ↔ `PropKind` porque o painel precisa do rótulo; usar o mesmo
/// par aqui faz uma linha nova nascer roteada, e não roteável.
pub(crate) fn prop_for_addprop_id(id: ph2d_editor::NodeId) -> Option<PropKind> {
    ph2d_panel_timeline::ids::ADDPROP_BUTTONS
        .iter()
        .find(|(bid, _)| *bid == id)
        .map(|(_, prop)| *prop)
}

/// **What entering (or leaving) a container does to the transport loop** — called on the
/// navigation EDGE only, from the shell's per-frame `edit_path` stamp (Enio, 2026-07-20:
/// *"dentro do container o loop não se ajustou automaticamente ao tempo das lanes"*).
///
/// Inside a container the thing being authored is the INSTANCE, so play cycles over the
/// timeline window that instance occupies — leads included ([`ph2d_timeline::entry_reach`]:
/// bracketing only the moving window would cut the very fades the artist is tuning, the
/// `stack_end_seconds` bug one level down). A playhead standing outside that window is
/// moved to its start: the alternative is a marker-less ruler that reads as broken.
///
/// Leaving (path empty) hands the transport back to the DOCUMENT's Arrange loop via the
/// same [`ph2d_timeline::sync_transport_loop`] a tab switch uses — navigation is not an
/// edit, so nothing here writes the document; the authored loop is merely re-installed.
///
/// Edge-triggered, not stamped every frame, so the artist can still re-arm or clear the
/// loop while inside — their gesture wins over the convenience.
/// **Entering / switching a container arms the CONTAINER clock with the container's
/// OWN loop** (Enio, 2026-07-22) — `container` is the innermost the animator has walked
/// into (`None` when they left back to the scene). It touches ONLY `container_ph`.
///
/// This replaces the old scene-side bracketing: entering a container no longer reaches
/// the scene playhead at all, so the Arrange loop the artist authored survives every
/// visit untouched. Leaving (`None`) does nothing — there is nothing on the scene clock
/// to restore, because nothing on it was ever moved.
pub(crate) fn on_container_nav_change(
    doc: &ph2d_timeline::TimelineDoc,
    container: Option<usize>,
    container_ph: &mut ph2d_core::Playhead,
) {
    let Some(c) = container else {
        return; // left to the scene: the scene clock was never ours to touch
    };
    ph2d_timeline::sync_container_loop(doc, c, container_ph);
    // Land inside the loop if one is set and the clock is parked outside it — the same
    // courtesy the scene-side bracketing used to do, now on the container's own clock.
    if let (Some((lo, hi)), _) = doc.container_loop(c)
        && (container_ph.time() < lo || container_ph.time() > hi)
    {
        container_ph.seek(lo);
    }
}

#[cfg(test)]
#[path = "timeline_bridge_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "timeline_bridge_k_tests.rs"]
mod k_tests;

#[cfg(test)]
#[path = "timeline_bridge_container_tests.rs"]
mod container_tests;

#[cfg(test)]
#[path = "timeline_bridge_signal_tests.rs"]
mod signal_tests;
