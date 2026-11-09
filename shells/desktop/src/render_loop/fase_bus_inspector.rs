//! **O dreno do barramento — o Inspector.** Braços do `match` da [`fase_bus_drain`](super), movidos pela
//! ordem de sempre; a única troca no corpo deles é `pd.<pedido>` onde escreviam `<pedido>`, e o `gfx` de cada
//! sub-dreno é re-derivado (o dreno só corre com ele). Ver o cabeçalho de lá.

use super::*;
use ph2d_editor_core::action_bus::{ComponentEdit, EditorAction};
use ph2d_i18n::{tr, tr_with};

impl crate::App {
    /// A vista, o reimport, a precisão e a emissão da sprite, e as ferramentas de imagem de um disparo.
    pub(super) fn fase_bus_sprite_ops(
        &mut self,
        action: EditorAction,
        pd: &mut DrainOut,
    ) -> Option<EditorAction> {
        match action {
            EditorAction::SetViewFocus { kind } => {
                pd.view_focus_kind.get_or_insert(kind);
            }
            EditorAction::Reimport { entity_bits } => {
                pd.reimport_entity.get_or_insert(entity_bits);
            }
            EditorAction::InspectorSpritePrecisionChange {
                entity_bits,
                precision,
            } => {
                pd.precision_request.get_or_insert((entity_bits, precision));
            }
            // **A SPRITE COMO FONTE DE LUZ** (plano `docs/Sprite_projeto/18` W8).
            //
            // ⚠️ **Zero REMOVE o componente**, e é o que faz o quadro voltar a ser (…)
            EditorAction::InspectorSpriteEmissiveChange {
                entity_bits,
                intensity,
            } => {
                // BulkSelect fan-out, a mesma forma do `InspectorSpriteEdit` acima.
                if pd.inspector_selection.is_empty() {
                    pd.emissive_edits.push((entity_bits, intensity));
                } else {
                    for &t in &pd.inspector_selection {
                        pd.emissive_edits.push((t, intensity));
                    }
                }
            }
            // ADR-0040 TG-A: generic one-shot image-op dispatch.
            // Trim/MakeSquare/RealSize collect into per-tool Option<u64>
            // for the existing per-tool drain functions; bgremoval bake (…)
            oneshot @ EditorAction::OneShotImageOp {
                tool_id,
                entity_bits,
            } => match tool_id {
                "trim_transparency" => {
                    pd.trim_entities.push(entity_bits);
                }
                "make_square" => {
                    pd.make_square_entities.push(entity_bits);
                }
                "real_size" => {
                    pd.real_size_entities.push(entity_bits);
                }
                "rasterize" => {
                    pd.rasterize_entities.push(entity_bits);
                }
                "bgremoval" => {
                    pd.bgremoval_leftover.push(oneshot);
                }
                "painter" => {
                    pd.painter_leftover.push(oneshot);
                }
                _ => {}
            },
            other => return Some(other),
        }
        None
    }

    /// O Inspector — as edições das secções (as que se espalham pela selecção, e as que dizem porque não) e o `+`.
    pub(super) fn fase_bus_inspector_sections(
        &mut self,
        action: EditorAction,
        pd: &mut DrainOut,
    ) -> Option<EditorAction> {
        match action {
            EditorAction::InspectorTransformEdit(info) => {
                pd.transform_edit.get_or_insert(info);
            }
            EditorAction::InspectorVisibilityEdit(info) => {
                // BulkSelect fan-out, a mesma forma do `InspectorVisibilitySectionEdit`.
                if pd.inspector_selection.is_empty() {
                    pd.visibility_edits.push((info.entity_bits, info.visible));
                } else {
                    for &t in &pd.inspector_selection {
                        pd.visibility_edits.push((t, info.visible));
                    }
                }
            }
            EditorAction::InspectorSpriteSourceChange {
                entity_bits,
                strategy,
            } => {
                pd.sprite_source_change
                    .get_or_insert((entity_bits, strategy));
            }
            EditorAction::InspectorSpriteEdit { entity_bits, edit } => {
                // BulkSelect: apply to EVERY selected sprite, not
                // just the dispatching (primary) entity. The Vec
                // includes the primary first; single-select pushes (…)
                if pd.inspector_selection.is_empty() {
                    pd.sprite_edits.push((entity_bits, edit));
                } else {
                    for &t in &pd.inspector_selection {
                        pd.sprite_edits.push((t, edit));
                    }
                }
            }
            EditorAction::InspectorOrderingEdit { entity_bits, edit } => {
                // BulkSelect fan-out, same shape as the sprite edit.
                if pd.inspector_selection.is_empty() {
                    pd.ordering_edits.push((entity_bits, edit));
                } else {
                    for &t in &pd.inspector_selection {
                        pd.ordering_edits.push((t, edit));
                    }
                }
            }
            EditorAction::InspectorSamplingEdit { entity_bits, edit } => {
                if pd.inspector_selection.is_empty() {
                    pd.sampling_edits.push((entity_bits, edit));
                } else {
                    for &t in &pd.inspector_selection {
                        pd.sampling_edits.push((t, edit));
                    }
                }
            }
            EditorAction::InspectorBlendEdit { entity_bits, edit } => {
                if pd.inspector_selection.is_empty() {
                    pd.blend_edits.push((entity_bits, edit));
                } else {
                    for &t in &pd.inspector_selection {
                        pd.blend_edits.push((t, edit));
                    }
                }
            }
            // §5 9-Slice. Espalha sobre a BulkSelect como as irmãs: uma caixa de diálogo
            // e as suas variantes partilham a mesma moldura, e ter de repetir a borda em
            // cada uma seria o gesto que esta seção existe para evitar.
            EditorAction::InspectorSliceEdit { entity_bits, edit } => {
                if pd.inspector_selection.is_empty() {
                    pd.slice_edits.push((entity_bits, edit));
                } else {
                    for &t in &pd.inspector_selection {
                        pd.slice_edits.push((t, edit));
                    }
                }
            }
            // §12 Sockets / Anchors. ⚠️ **NÃO espalha sobre a BulkSelect**, e isso é
            // uma decisão: uma âncora é identificada pelo NOME, e o índice que a edição
            // carrega só significa alguma coisa na lista da entidade primária. Espalhar (…)
            EditorAction::InspectorAnchorEdit { entity_bits, edit } => {
                pd.anchor_edits.push((entity_bits, edit));
            }
            // §11 Animation. ⚠️ **NÃO espalha sobre a BulkSelect**, e pela MESMA razão da
            // §12 acima: uma animação é identificada pelo NOME, e o índice que a edição
            // carrega só significa alguma coisa na biblioteca da entidade primária.
            EditorAction::InspectorAnimEdit { entity_bits, edit } => {
                pd.anim_edits.push((entity_bits, edit));
            }
            // ⭐ **A secção TIMERS.** ⚠️ **NÃO espalha sobre a BulkSelect**, pela MESMA
            // razão das duas acima: o índice que a edição carrega só significa alguma
            // coisa na lista da entidade primária, e espalhá-lo escreveria no timer
            // errado de todas as outras.
            EditorAction::InspectorTimerEdit { entity_bits, edit } => {
                pd.timer_edits.push((entity_bits, edit));
            }
            // ⭐ **A secção SIGNAL ACTIONS.** ⚠️ **NÃO espalha sobre a BulkSelect**,
            // pela MESMA razão das irmãs: o índice só significa alguma coisa na lista
            // da entidade primária.
            EditorAction::InspectorActionEdit { entity_bits, edit } => {
                pd.action_edits.push((entity_bits, edit));
            }
            // ⭐ **A secção AUDIO** (TOP-20 #4). ⚠️ **NÃO espalha sobre a BulkSelect**,
            // pela MESMA razão das irmãs — e aqui há uma segunda: duas das variantes
            // (`Preview`/`StopPreview`) TOCAM, e espalhá-las faria um clique em `Preview`
            // disparar N sons de uma vez.
            EditorAction::InspectorAudioEdit { entity_bits, edit } => {
                pd.audio_edits.push((entity_bits, edit));
            }
            // ⭐ **A secção CAMERA** (TOP-20 #7). ⚠️ **NÃO espalha sobre a BulkSelect**,
            // pela MESMA razão das irmãs — e aqui há uma segunda: o `Preview` é da VISTA,
            // e espalhá-lo faria N objectos disputarem um interruptor que é um só.
            EditorAction::InspectorCameraEdit { entity_bits, edit } => {
                pd.camera_edits.push((entity_bits, edit));
            }
            // ⭐ **A secção TAGS** (TOP-20 #9). ⚠️ **NÃO espalha sobre a BulkSelect**, e aqui a
            // razão é OUTRA que a das irmãs: o que a edição carrega não é um índice — é uma
            // IDENTIDADE, que significa o mesmo em toda a cena, logo espalhar seria exprimível.
            EditorAction::InspectorComponentEdit { entity_bits, edit } => match edit {
                ComponentEdit::Factory(e) => pd.factory_edits.push((entity_bits, e)),
                ComponentEdit::TopDown(e) => pd.topdown_edits.push((entity_bits, e)),
                ComponentEdit::Projectile(e) => pd.projectile_edits.push((entity_bits, e)),
                ComponentEdit::Ray(e) => pd.ray_edits.push((entity_bits, e)),
                ComponentEdit::Parallax(e) => pd.parallax_edits.push((entity_bits, e)),
                ComponentEdit::Weapon(e) => pd.weapon_edits.push((entity_bits, e)),
                ComponentEdit::Vida(e) => pd.vida_edits.push((entity_bits, e)),
                ComponentEdit::Nav(e) => pd.nav_edits.push((entity_bits, e)),
                ComponentEdit::Luz(e) => pd.luz_edits.push((entity_bits, e)),
                ComponentEdit::Constraint(e) => pd.constraint_edits.push((entity_bits, e)),
                ComponentEdit::MotionInput(e) => pd.motion_input_edits.push((entity_bits, e)),
                ComponentEdit::Macro(e) => pd.macro_edits.push((entity_bits, e)),
                ComponentEdit::Tween(e) => pd.tween_edits.push((entity_bits, e)),
                ComponentEdit::PathFollow(e) => pd.path_follow_edits.push((entity_bits, e)),
                ComponentEdit::StateMachine(e) => pd.statemachine_edits.push((entity_bits, e)),
                ComponentEdit::Script(e) => pd.script_edits.push((entity_bits, e)),
                ComponentEdit::Particles(e) => pd.particles_edits.push((entity_bits, e)),
                ComponentEdit::Hud(e) => pd.hud_edits.push((entity_bits, e)),
                ComponentEdit::Sequence(e) => pd.sequence_edits.push((entity_bits, e)),
                ComponentEdit::CounterWatch(e) => pd.counter_watch_edits.push((entity_bits, e)),
                ComponentEdit::ActionTrigger(e) => pd.action_trigger_edits.push((entity_bits, e)),
                ComponentEdit::Shake(e) => pd.shake_edits.push((entity_bits, e)),
                ComponentEdit::ShakeEmitter(e) => {
                    pd.shake_emitter_edits.push((entity_bits, e));
                }
                ComponentEdit::Tags(e) => pd.tags_edits.push((entity_bits, e)),
            },
            // ⭐⭐⭐ **O painel TAGS** (TOP-20 #9, W4). ⛔ **Sem `entity_bits`, e é isso que o
            // separa da irmã de cima:** o sujeito é a TAXONOMIA, não um objecto — e por isso a
            // pergunta da BulkSelect nem sequer se põe.
            EditorAction::TagTreeEdit { edit } => {
                pd.tag_tree_edits.push(edit);
            }
            // O painel MACROS e a linha de etapas (W5d): a família guarda o pedido e aplica-o na
            // `fase_macros`, depois de o Painter consumir a entrada do quadro.
            EditorAction::Macro(r) => ph2d_app_macros::com(|m| m.pede(r)),
            // O painel PIXEL (ADR-0196): a família aplica-o na `fase_pixel_panel`.
            EditorAction::Pixel(r) => self.pixel.pede(r),
            // ⭐ **O `+` do Inspector** (ADR-0166 / F3) — o painel PEDE e a shell abre,
            // porque só ela sabe o tipo do objeto, o que ele já tem, e o que o registo
            // sabe construir.
            EditorAction::InspectorAddComponentRequested { entity_bits } => {
                pd.add_component_for = Some(entity_bits);
            }
            // ⭐⭐⭐ **A CURA ao lado da QUEIXA** (suplente #24): a fileira do gatilho diz que a acção
            // não existe, e o botão dela cria-a — *uma queixa que nomeia a cura e não a alcança é
            // meia queixa*. ⛔ Ela NÃO é uma `ComponentEdit`: o que nasce é uma linha do Input Map,
            // que é estado do EDITOR.
            EditorAction::CreateInputAction { name } => {
                pd.create_input_actions.push(name);
            }
            other => return Some(other),
        }
        None
    }

    /// O Inspector — o cartão da instância: abrir a receita, as excepções sem alvo, a peça recusada e a acrescentada, a
    /// biblioteca, a variante e o degrau do *Aplicar*.
    pub(super) fn fase_bus_inspector_instance(
        &mut self,
        action: EditorAction,
        pd: &mut DrainOut,
    ) -> Option<EditorAction> {
        let gfx = self.gfx.as_mut()?;
        let FrameGfx {
            sim,
            toasts,
            hero_screen,
            ..
        } = FrameGfx::of(gfx);
        let hero = hero_screen.as_mut()?;
        match action {
            // ⭐ **Limpar as excepções SEM ALVO** (ADR-0164 / F5.3). Aplicado JÁ, e não
            // adiado para um local: ele não precisa de nada que este ponto não tenha, e o
            // `post_frame_undo` (que corre no fim) vê a mudança e regista o passo.
            EditorAction::InspectorOpenPrefab { root_bits } => {
                // ⚠️ **Aplicado JÁ, como os irmãos deste bloco** — ele não precisa de nada
                // que este ponto não tenha: a lei de abrir é SELECCIONAR, e a porta
                // (`instance_open`) é a mesma que os outros três acessos usam. ⛔ Deferi-lo (…)
                let mut select_out = None;
                ph2d_app_components::instance_open::open_prefab(
                    sim,
                    ph2d_ecs::Entity::from_bits(root_bits),
                    toasts,
                    &mut select_out,
                );
                if let Some(bits) = select_out {
                    hero.gizmo.replace_selection(Some(bits));
                }
            }
            EditorAction::InspectorClearUnusedOverrides { root_bits } => {
                let n = inspector_instance::clear_orphans(sim, root_bits);
                if n > 0 {
                    toasts.push(ph2d_editor_core::Toast::success(tr_with(
                        "shell.fase_bus_inspector.cleared_unused",
                        &[("n", &n)],
                    )));
                }
            }
            // ⭐⭐⭐ **Largar UMA** (F5.3-ter) — o `✕` da linha. ⚠️ Aplicado JÁ, como o irmão
            // acima e pela mesma razão: ele não precisa de nada que este ponto não tenha, e
            // o `post_frame_undo` vê a mudança e regista o passo.
            EditorAction::InspectorRestoreRemovedPiece { root_bits, piece } => {
                if ph2d_app_components::instance_structure::restore_piece(sim, root_bits, piece) {
                    toasts.push(ph2d_editor_core::Toast::success(tr(
                        "shell.fase_bus_inspector.put_the_piece_back_it",
                    )));
                }
            }
            EditorAction::InspectorDropUnusedOverride {
                root_bits,
                piece,
                type_id,
            } => {
                if inspector_instance::drop_orphan(sim, root_bits, piece, type_id) {
                    toasts.push(ph2d_editor_core::Toast::success(tr(
                        "shell.fase_bus_inspector.dropped_1_unused",
                    )));
                }
            }
            // ⭐⭐⭐ **Trocar a VARIANTE** (ADR-0164 / F5, critério 2).
            //
            // ⚠️ **ADIADO para depois do dreno**, ao contrário do irmão acima, e a razão é (…)
            EditorAction::OpenImagePicker { entity_bits } => {
                pd.open_image_picker = Some(entity_bits);
            }
            EditorAction::ChooseImage { entity_bits, asset } => {
                pd.choose_image = Some((entity_bits, asset));
            }
            EditorAction::ImageUseEdit { entity_bits, edit } => {
                pd.image_use_edit = Some((entity_bits, edit));
            }
            // ⭐⭐⭐ **Aplicar uma peça ACRESCENTADA** (F5.11). ⚠️ **ADIADO como o irmão
            // abaixo, e pela mesma família de razões:** ela precisa do registo de
            // componentes e dos documentos possuídos (a peça pode ser uma forma vetorial),
            // e aqui dentro o `self` já está emprestado.
            EditorAction::InspectorApplyAddedPiece { piece } => {
                pd.apply_added = Some(piece);
            }
            EditorAction::InspectorSwapVariant { root_bits, master } => {
                pd.swap_variant = Some((root_bits, master));
            }
            EditorAction::InspectorApplyToLevel {
                entity_bits,
                master,
            } => {
                pd.apply_to_level = Some((entity_bits, master));
            }
            other => return Some(other),
        }
        None
    }

    /// O Inspector — a secção Visibility e os campos de nome e de sinal (a última tecla do quadro vence).
    pub(super) fn fase_bus_inspector_identity(
        &mut self,
        action: EditorAction,
        pd: &mut DrainOut,
    ) -> Option<EditorAction> {
        match action {
            EditorAction::InspectorVisibilitySectionEdit { entity_bits, edit } => {
                // BulkSelect fan-out, same shape as the sampling edit.
                if pd.inspector_selection.is_empty() {
                    pd.visibility_section_edits.push((entity_bits, edit));
                } else {
                    for &t in &pd.inspector_selection {
                        pd.visibility_section_edits.push((t, edit));
                    }
                }
            }
            EditorAction::InspectorNameEdit(info) => {
                // Latest-wins (Option-coalesce parity).
                pd.name_edit = Some(info);
            }
            EditorAction::InspectorSignalEdit(info) => {
                // Mesma coalescência: um `TextChanged` por tecla, e só a
                // última do quadro vira comando (W-Signal).
                pd.signal_edit = Some(info);
            }
            EditorAction::InspectorSignalLeaveEdit(info) => {
                // O gêmeo (W-SignalLeave), com slot PRÓPRIO: coalescer os
                // dois no mesmo faria a última tecla de uma row apagar o
                // que a outra tinha acabado de dizer.
                pd.signal_leave_edit = Some(info);
            }
            EditorAction::LayerPairToggle { a, b } => {
                ph2d_panel_physics::alterna_par(usize::from(a), usize::from(b));
            }
            acao if acao.e_das_camadas() => {
                if let Some(g) = self.gfx.as_mut() {
                    ph2d_app_physics::panel_bridge::acao_das_camadas(
                        &mut g.sim,
                        &mut g.hero_screen,
                        acao,
                    );
                }
            }
            other => return Some(other),
        }
        None
    }
}

#[cfg(test)]
#[path = "fase_bus_inspector_luz_tests.rs"]
mod luz_tests;

#[cfg(test)]
#[path = "fase_bus_inspector_preso_tests.rs"]
mod preso_tests;

#[cfg(test)]
#[path = "fase_bus_outline_tests.rs"]
mod outline_tests;
