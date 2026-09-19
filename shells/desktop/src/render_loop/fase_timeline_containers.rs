//! **Fase do quadro: O RELÓGIO DOS CONTÊINERES** — entrar, trocar ou sair de um contêiner arma o relógio
//! dele com a volta dele, e o transporte segue o animador para dentro e para fora (OBRA 2 da
//! `line/render-loop`, 2026-09-12).
//!
//! ⚠️ Os dois parâmetros são os que a `fase_timeline_view` publicou para ESTE quadro (`Copy`), e o quadro
//! continua a lê-los no dreno das intents, a seguir.

use super::*;

impl crate::App {
    /// Ver o cabeçalho do módulo.
    pub(super) fn fase_timeline_containers(&mut self, container: Option<usize>, keys_mode: bool) {
        // O `gfx` re-derivado; os guardas do quadro já correram na `fase_chrome_clock`.
        let Some(gfx) = self.gfx.as_mut() else {
            return;
        };
        let FrameGfx {
            sim,
            toasts,
            hero_screen,
            ..
        } = FrameGfx::of(gfx);

        // Entering / switching / leaving a container arms the CONTAINER clock with its
        // OWN loop (edge-triggered, the nav mirror of the `keys_mode` sync below). The
        // scene clock is deliberately NOT touched — the Arrange loop survives the visit.
        if container != self.last_timeline_container {
            self.last_timeline_container = container;
            timeline_bridge::on_container_nav_change(
                &self.timeline.doc,
                container,
                &mut self.container_playhead,
            );
        }
        if keys_mode != self.last_timeline_keys_mode {
            self.last_timeline_keys_mode = keys_mode;
            if keys_mode {
                ph2d_timeline::sync_transport_loop(
                    &self.timeline.doc,
                    &mut self.clip_playhead,
                    true,
                );
            } else {
                ph2d_timeline::sync_transport_loop(&self.timeline.doc, &mut self.playhead, false);
            }
        }
        if self.timeline_insert_key {
            self.timeline_insert_key = false;
            // The K clock is the clock the animator is watching: the container's
            // inside one, the scene's on Arrange. Keys solos and reads the clip
            // directly (no scratch). The value/time helpers below read the same clock.
            let k_time = if container.is_some() {
                self.container_playhead.time()
            } else {
                self.playhead.time()
            };
            // The scratch must describe THIS instant before the Arrange/container K asks
            // it where a key lands or whether the pose is reachable — ROOTED at whatever
            // stack the animator is looking at (the scene's, or the container's interior;
            // Enio 2026-07-22). Skip it in Keys (solo has no scratch). Without the prime
            // the resolve runs against the PREVIOUS frame's strip state.
            if !keys_mode {
                self.timeline.doc.prime_rooted(container, k_time);
            }
            if let Some(entity) = hero_screen
                .as_ref()
                .and_then(|h| h.gizmo.iter_selected().next())
            {
                let mut props: Vec<_> = self
                    .timeline
                    .doc
                    .bindings()
                    .iter()
                    .filter(|b| b.entity == entity)
                    .map(|b| b.prop)
                    .collect();
                // If the entity has no position animation yet, K creates it in the
                // toggle's mode (ADR-0141) — the same choice auto-key makes for a
                // fresh object, so K and auto-key agree on Position vs separate X/Y.
                // An entity already in one mode keeps it (the branch below routes
                // Position → anchor, X/Y → scalar), so this only injects the FIRST
                // position channel; the existing loop then authors it uniformly.
                let has_position = props.iter().any(|p| {
                    matches!(
                        p,
                        ph2d_timeline::PropKind::Position
                            | ph2d_timeline::PropKind::TranslationX
                            | ph2d_timeline::PropKind::TranslationY
                    )
                });
                if !has_position {
                    // Fresh object → the After Effects default (motion path); the
                    // per-object toggle marks it Separate otherwise.
                    match self.timeline.doc.position_key_mode(entity, true) {
                        ph2d_timeline::PositionKeyMode::Path => {
                            props.push(ph2d_timeline::PropKind::Position);
                        }
                        ph2d_timeline::PositionKeyMode::Separate => {
                            props.push(ph2d_timeline::PropKind::TranslationX);
                            props.push(ph2d_timeline::PropKind::TranslationY);
                        }
                    }
                }
                for prop in props {
                    // ⚠️ **Position não amostra um escalar: ele acrescenta uma ÂNCORA**
                    // (ADR-0141). "Capturar a pose" numa trajetória é uma edição da
                    // GEOMETRIA — a âncora nova muda o percurso, que é o que as outras
                    // keys medem —, então não há valor a computar aqui e o
                    // `sample_prop_value` recusa este kind de propósito. O tempo é o
                    // mesmo `k_time` das outras (o relógio da vista), e o intent leva o
                    // LUGAR; quem aplica é que sabe quanto ele vale em distância.
                    if prop == ph2d_timeline::PropKind::Position {
                        if let Some(xf) = sim
                            .world()
                            .get::<ph2d_ecs::Transform>(ph2d_ecs::Entity::from_bits(entity))
                        {
                            // ⚠️ O tempo vem da porta que também decide SE pode: uma
                            // trajetória é geometria do CLIP, e só a aba Keys tem um clip
                            // escolhido (Enio, 2026-07-31). Em Keys é o relógio do clip
                            // cortado e remapeado (`solo_key_time`, a metade extraída do
                            // `key_authoring_solo` — uma composição própria aqui seria a
                            // pose a saltar de volta); fora dela é uma RECUSA com motivo,
                            // dita em voz alta. O `key_insert_time` respondia por este
                            // caso e responde a outra pergunta (*onde esta STRIP toca?*).
                            match timeline_bridge::path_key_time(
                                &self.timeline,
                                entity,
                                self.clip_playhead.time(),
                            ) {
                                Ok(t) => self.timeline_intents.push(
                                    ph2d_timeline::TimelineIntent::AddPathKey {
                                        entity,
                                        t,
                                        at: [xf.translation.x, xf.translation.y],
                                    },
                                ),
                                Err(r) => {
                                    // O `push` da fila de toasts devolve se coube; derrubar
                                    // um toast nunca derruba trabalho, e aqui não há
                                    // trabalho — a recusa É o resultado.
                                    let _ =
                                        toasts.push(Toast::warning(ph2d_i18n::tr(r.message_key())));
                                }
                            }
                        }
                        continue;
                    }
                    // In KEYS mode the animator sees the active clip soloed, so the
                    // pose IS the clip's value and the time is the clip playhead's —
                    // stored directly, and K never refuses (there is no stack to be
                    // overridden by or to play twice). In ARRANGE mode the pose is a
                    // blend, so `key_value_for` inverts it and `key_insert_time` can
                    // REFUSE (the clip has no influence, or plays zero/two times).
                    let authored = if keys_mode {
                        timeline_bridge::key_authoring_solo(
                            sim.world(),
                            &self.timeline,
                            entity,
                            prop,
                            self.clip_playhead.time(),
                        )
                    } else {
                        // Arrange OR inside a container — both read the stack (rooted at
                        // the scene or the container respectively, primed above) at
                        // `k_time`. The value/time helpers are root-aware, so a container
                        // key lands on the container's clock and refuses on the
                        // container's terms, never the scene's.
                        timeline_bridge::key_value_for(
                            sim.world(),
                            &self.timeline,
                            entity,
                            prop,
                            k_time,
                        )
                        .zip(timeline_bridge::key_insert_time(
                            &self.timeline,
                            entity,
                            prop,
                            k_time,
                        ))
                    };
                    if let Some((value, t)) = authored {
                        self.timeline_intents
                            .push(ph2d_timeline::TimelineIntent::AddKey {
                                entity,
                                prop,
                                t,
                                value,
                                interp: timeline_bridge::default_interp(),
                            });
                    }
                }
            }
        }
        // **O quadro da saída de sinais vira AQUI — antes do primeiro produtor.**
        //
        // ⚠️ A posição é load-bearing e tem gate (`the_shell_turns_the_signal_frame_before_it
        // _publishes`): virar o quadro no MEIO aposentaria sinais que consumidores deste mesmo
        // quadro ainda não leram, e virar duas vezes cortaria pela metade a janela de graça de
        // um quadro que existe para um consumidor futuro que rode CEDO demais.
        self.signals.advance_frame();
    }
}
