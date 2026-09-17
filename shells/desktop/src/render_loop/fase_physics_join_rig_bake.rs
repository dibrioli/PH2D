//! **Fase do quadro: LIGAR, RIGAR E ASSAR** — as três rotas da física que agem sobre a SELECÇÃO: a corrente de
//! joints, o Rig (que lê a estrutura já desenhada) e o Bake do movimento simulado em curvas (OBRA 2 da `line/render-loop`, 2026-09-12).
//!
//! ⚠️ A ordem entre as três é a do quadro: as duas primeiras escrevem joints, e um corpo assado deixa de
//! ser simulado.

use super::*;
use ph2d_i18n::{tr, tr_with};

/// Os pedidos de ligar, rigar e assar que o dreno do barramento recolheu neste quadro.
pub(super) struct PhysicsCreateIntents {
    pub(super) bake_request: Option<Vec<u64>>,
    pub(super) join_chain: bool,
    pub(super) rig_now: bool,
    pub(super) inspector_selection: Vec<u64>,
}

impl crate::App {
    /// Ver o cabeçalho do módulo.
    pub(super) fn fase_physics_join_rig_bake(&mut self, intents: PhysicsCreateIntents) {
        // O `gfx` re-derivado; os guardas do quadro já correram na `fase_chrome_clock`.
        let Some(gfx) = self.gfx.as_mut() else {
            return;
        };
        let FrameGfx {
            sim,
            toasts,
            hero_screen,
            component_registry,
            editor_queue,
            physics,
            ..
        } = FrameGfx::of(gfx);
        // O bloco do quadro só chama esta fase com o `HeroScreen` vivo.
        let Some(hero) = hero_screen.as_mut() else {
            return;
        };
        let PhysicsCreateIntents {
            bake_request,
            join_chain,
            rig_now,
            inspector_selection,
        } = intents;
        if join_chain {
            let (made, last) = ph2d_app_physics::joint_draw::join_chain(
                sim,
                &inspector_selection,
                ph2d_app_physics::joint::kind_of(self.physics.join_kind),
            );
            // Select the LAST joint so §12 (Physics Joint) appears
            // immediately — the Kind selector and tuning are right there.
            // Otherwise the bodies stay selected (§11) and the joint is only
            // reachable by hunting for it in the Hierarchy by hand, which is
            // why creating anything but a Pin felt impossible. On a chain it
            // is the link that closes; showing one of the N is more honest
            // than showing none.
            if let Some(j) = last {
                hero.gizmo.selection = Some(j.to_bits());
                hero.gizmo.extra_selection.clear();
            }
            if made > 1 {
                toasts.push(ph2d_editor_core::Toast::info(tr_with(
                    "shell.fase_physics_join_rig_bake.chained_bodies_with",
                    &[("made2", &(made + 1)), ("made", &made)],
                )));
            }
        }
        // W-Rig: a TERCEIRA rota de criação — a única que não pede ao artista
        // que redescreva uma estrutura que ele já desenhou. Depois do
        // `join_chain` porque as duas escrevem joints, e a ordem entre elas
        // num mesmo frame só importaria se o artista tivesse clicado nas duas
        // (ele não pode: são dois botões).
        if rig_now {
            let roots: Vec<u64> = hero.gizmo.iter_selected().collect();
            let plan = ph2d_app_physics::joint_rig::plan(sim, &roots);
            let out = ph2d_app_physics::joint_rig::apply(
                sim,
                &plan,
                ph2d_app_physics::joint::kind_of(self.physics.join_kind),
                editor_queue,
                component_registry,
            );
            // ⚠️ O flush mora DENTRO do gerador, entre dar corpo e ligar: a
            // emenda mede o `Collider` que a primeira metade acabou de
            // enfileirar. Aqui fica só o deck de toasts, que é deste laço.
            if let Some(e) = out.error {
                toasts.push(ph2d_editor_core::Toast::error(tr_with(
                    "shell.fase_physics_join_rig_bake.rig_commit_failed",
                    &[("e", &e)],
                )));
            }
            let (bodies, joints, last) = (out.bodies, out.joints, out.last);
            // Seleciona o ÚLTIMO joint, pelo motivo que o `join_chain`
            // documenta: a §12 abre na hora, com o Kind e a afinação à mão —
            // e afinar UM e carimbar o resto é o gesto que a W-JointCopy
            // acabou de tornar barato.
            if let Some(j) = last {
                hero.gizmo.selection = Some(j.to_bits());
                hero.gizmo.extra_selection.clear();
            }
            if joints > 0 {
                toasts.push(ph2d_editor_core::Toast::info(tr_with(
                    "shell.fase_physics_join_rig_bake.rigged_new_bodies_with",
                    &[("bodies", &bodies), ("joints", &joints)],
                )));
            }
        }
        // W4 - bake the selection's simulated motion into curves. After the
        // joint work above because a baked body stops being simulated, and
        // the frame's other physics edits should land on the body as the
        // artist authored it.
        if let Some(bits) = bake_request {
            let entities: Vec<ph2d_ecs::Entity> = bits
                .iter()
                .map(|&b| ph2d_ecs::Entity::from_bits(b))
                .collect();
            let (start, end) =
                ph2d_app_physics::bake::bake_range(&self.timeline.doc, &self.playhead);
            let outcome = ph2d_app_physics::bake::bake_selection(
                &mut self.timeline,
                physics,
                sim,
                &entities,
                start,
                end,
                self.fixed_step.fixed_dt(),
                self.bake_channels,
                &mut self.player_tape,
                editor_queue,
                component_registry,
            );
            if outcome.unmappable {
                toasts.push(ph2d_editor_core::Toast::info(tr(
                    "shell.fase_physics_join_rig_bake.cannot_bake_here_this",
                )));
            } else if outcome.refused {
                toasts.push(ph2d_editor_core::Toast::info(tr(
                    "shell.fase_physics_join_rig_bake.finish_the_current",
                )));
            } else if outcome.already_baked {
                toasts.push(ph2d_editor_core::Toast::info(tr(
                    "shell.fase_physics_join_rig_bake.already_baked_the",
                )));
            } else if outcome.is_empty() {
                toasts.push(ph2d_editor_core::Toast::info(tr(
                    "shell.fase_physics_join_rig_bake.nothing_to_bake",
                )));
            } else {
                // Back to the top, because that is where the animation the
                // artist just made begins - and because the kind change only
                // reaches rapier at tick 0 (`reconcile_structure`
                // re-describes a body at rest), so this is also what makes
                // the hand-over take effect.
                //
                // ⚠️ PAUSE as well, and the rewind alone is not enough:
                // `Playhead::rewind` preserves the play state by design, and
                // `advance_ticks` runs EARLIER in the frame than
                // `ph2d_app_physics::bridge::dispatch::dispatch`. Still playing, the clock is
                // already past 0 by the time the bridge looks, `at_rest` is
                // never true again, and the flip never reaches rapier: the
                // body keeps falling as Dynamic and the curve is discarded -
                // exactly the "clicks Bake, nothing changes" failure the
                // hand-over exists to prevent.
                self.playhead.rewind();
                self.playhead.pause();
                // The window mirrors the button: a partial range (W-BakeRange)
                // reads "2.0-5.0s", the common full-range bake just "5.0s".
                let window = if start > 0.0 {
                    format!("{start:.1}-{end:.1}s")
                } else {
                    format!("{end:.1}s")
                };
                toasts.push(ph2d_editor_core::Toast::info(tr_with(
                    "shell.fase_physics_join_rig_bake.baked_bodies_tracks",
                    &[
                        ("window", &window),
                        ("bodies", &(outcome.bodies)),
                        ("tracks", &(outcome.tracks)),
                    ],
                )));
            }
        }
    }
}
