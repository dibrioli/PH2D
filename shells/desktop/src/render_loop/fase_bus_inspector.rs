//! **O dreno do barramento — o Inspector.** Braços do `match` da [`fase_bus_drain`](super), movidos pela
//! ordem de sempre; a única troca no corpo deles é `pd.<pedido>` onde escreviam `<pedido>`, e o `gfx` de cada
//! sub-dreno é re-derivado (o dreno só corre com ele). Ver o cabeçalho de lá.

use super::*;
use ph2d_editor_core::action_bus::EditorAction;
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
            // ⚠️ **Zero REMOVE o componente**, e é o que faz o quadro voltar a ser
            // byte-idêntico: uma sprite que não emite não tem por que carregar a linha no
            // ficheiro nem uma entrada na varredura do passe. Mesmo caminho do
            // `TextureFilter` — quem tem o `ComponentRegistry` é o shell.
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
            // for the existing per-tool drain functions; bgremoval bake
            // is deferred via leftover (must run AFTER ActivateTool
            // has switched the tool active, image_edit.rs:184 picks it up).
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
                // includes the primary first; single-select pushes
                // one. Fall back to the edit's own entity if the
                // selection snapshot is empty (stale dispatch).
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
            // carrega só significa alguma coisa na lista da entidade primária. Espalhar
            // por índice escreveria na âncora errada de todas as outras — pior que não
            // espalhar. Fan-out por nome é trabalho para quando houver quem o peça.
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
            // Fica de fora porque *marcar N objectos de uma vez* é um gesto que ninguém pediu e
            // que o painel não desenha: a secção mostra os chips da primária e diz quantos mais
            // estão escolhidos. ⛔ Ligá-lo sem o desenhar daria ao artista um efeito invisível.
            EditorAction::InspectorTagsEdit { entity_bits, edit } => {
                pd.tags_edits.push((entity_bits, edit));
            }
            // ⭐ **O `+` do Inspector** (ADR-0166 / F3) — o painel PEDE e a shell abre,
            // porque só ela sabe o tipo do objeto, o que ele já tem, e o que o registo
            // sabe construir.
            EditorAction::InspectorAddComponentRequested { entity_bits } => {
                pd.add_component_for = Some(entity_bits);
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
            // ⭐⭐⭐ **ABRIR a receita que o cartão NOMEIA** (2026-09-07) — a quarta e
            // última superfície da família. ⚠️ Pelo MESMO dreno dos outros três acessos,
            // que é onde vivem a resolução do sujeito (`master_subject`) e a voz da recusa.
            EditorAction::InspectorOpenPrefab { root_bits } => {
                // ⚠️ **Aplicado JÁ, como os irmãos deste bloco** — ele não precisa de nada
                // que este ponto não tenha: a lei de abrir é SELECCIONAR, e a porta
                // (`instance_open`) é a mesma que os outros três acessos usam. ⛔ Deferi-lo
                // para o dreno dos verbos pediria um terceiro canal (bits, a par de `row` e
                // `stable_id`) para um verbo que não toca no documento.
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
            // ⭐⭐⭐ **Devolver uma peça recusada** (F5.10). ⚠️ Ela só apaga a DECISÃO — quem
            // materializa a peça, lhe traz os bytes da receita e exuma a excepção que o
            // artista tinha nela é o passe estrutural, no quadro seguinte.
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
            // ⚠️ **ADIADO para depois do dreno**, ao contrário do irmão acima, e a razão é
            // uma só: a troca precisa do **eco** (`self.instance_echo`) para o esquecer, e
            // aqui dentro o `self` já está emprestado. *Um gesto que precisa de mais do que
            // o ponto de aplicação tem, adia-se — não se duplica o estado.*
            // ⭐⭐ **Mostrar a biblioteca** — o clique na ranhura da textura. ⚠️ Ele
            // **abre**, nunca alterna: o gesto é *«mostra-me o que cabe aqui»*, e
            // fechar um painel que o artista acabou de pedir seria responder ao
            // contrário.
            EditorAction::OpenAssetBrowser => {
                pd.open_asset_browser = true;
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

    /// O Inspector — a física (§11 corpo, §12 junta, §14 player e a roda). ⚠️ As interceptações que NÃO se espalham
    /// correm antes do fan-out de cada braço, e os quatro braços ficam juntos por isso.
    pub(super) fn fase_bus_physics_sections(
        &mut self,
        action: EditorAction,
        pd: &mut DrainOut,
    ) -> Option<EditorAction> {
        match action {
            // ⭐⭐⭐ **Renomear o VALOR de uma propriedade** (report do Enio, 2026-08-31).
            // ⚠️ O sujeito é a RECEITA; o gesto nasce sobre a cópia, que é onde o artista
            // está a olhar. Ver `ph2d-panel-inspector/src/event_value.rs`.
            // ⭐⭐⭐ **GRAVAR A VARIAÇÃO** (Enio, 2026-09-01) — o botão do cartão.
            // §11 Physics Body. Fans out over a BulkSelect like its
            // siblings — "make all of these physical" is the gesture
            // an artist actually performs.
            EditorAction::InspectorPhysicsEdit { entity_bits, edit } => {
                // ⚠️ **Join does NOT fan out.** Every other §11 edit is
                // per-entity ("make all of these static"), but joining
                // is one gesture over a PAIR — fanned out it would
                // create one joint per selected body, i.e. two joints
                // between the same two objects, on the very click that
                // is supposed to make one.
                if matches!(edit, ph2d_editor_core::PhysicsFieldEdit::Join) {
                    // ⚠️ **2 ou MAIS** (W-J4): três corpos marcados
                    // fazem uma CORRENTE de N−1 joints, na ordem da
                    // seleção. Não é fan-out (isso criaria um joint por
                    // corpo, entre os mesmos dois) — é UMA operação
                    // sobre a sequência, que a `join_selected_chain`
                    // executa depois do laço.
                    if pd.inspector_selection.len() >= 2 {
                        pd.join_chain = true;
                    }
                } else if matches!(edit, ph2d_editor_core::PhysicsFieldEdit::Rig) {
                    // ⚠️ **Nem o Rig faz fan-out** (W-Rig), e a razão é a
                    // do Bake mais que a do Join: cada corrida do gerador
                    // percorre a MESMA subárvore, então espalhado ele
                    // rodaria N vezes sobre o mesmo trabalho — a 2ª em
                    // diante achariam tudo já ligado e não fariam nada,
                    // mas o toast contaria a 1ª N vezes.
                    pd.rig_now = true;
                } else if matches!(edit, ph2d_editor_core::PhysicsFieldEdit::JoinDraw) {
                    // ARMA o gesto de canvas (sem operando, como os
                    // eyedroppers do §12): quem nomeia os dois corpos é
                    // o press e o release, não a seleção. Armado aqui e
                    // honrado no `input_dispatch`.
                    pd.join_draw_arm = true;
                } else if matches!(edit, ph2d_editor_core::PhysicsFieldEdit::Bake) {
                    // WARNING: **Bake does not fan out either**, and the
                    // cost of getting it wrong is bigger than Join's:
                    // ONE bake runs the whole simulation once and writes
                    // every selected body's curves from that single run.
                    // Fanned out it would re-simulate the entire scene
                    // once per selected body - same numbers, N times the
                    // work - and file a separate undo step for each, so
                    // undoing "the bake" would take as many Ctrl+Z
                    // presses as there were objects.
                    pd.bake_request = Some(if pd.inspector_selection.is_empty() {
                        vec![entity_bits]
                    } else {
                        pd.inspector_selection.clone()
                    });
                } else if let ph2d_editor_core::PhysicsFieldEdit::BakeChannels(tag) = edit {
                    // A GLOBAL bake option, not a per-body edit (like
                    // Bake itself): it says how the NEXT bake behaves.
                    // No fan-out, no Collider write — just the app state
                    // the Bake button reads.
                    self.bake_channels = ph2d_app_physics::bake::BakeChannels::from_tag(tag);
                } else if let ph2d_editor_core::PhysicsFieldEdit::JoinKind(tag) = edit {
                    // The pending join KIND, the same class as BakeChannels:
                    // an app-state option the Join gesture reads, not a
                    // per-body edit. No fan-out, no Collider write.
                    self.physics.join_kind = tag;
                } else if pd.inspector_selection.is_empty() {
                    pd.physics_edits.push((entity_bits, edit));
                } else {
                    for &t in &pd.inspector_selection {
                        pd.physics_edits.push((t, edit));
                    }
                }
            }
            // §12 Physics Joint. No fan-out either, and for a simpler
            // reason: the section only ever describes one joint object.
            EditorAction::InspectorJointEdit { entity_bits, edit } => {
                // The eyedropper ARMS a canvas pick (shell state), it is
                // not a component edit — handled here where `self` is
                // freely mutable, exactly like `Join` sets `join_request`.
                // The next canvas click resolves it (`input_dispatch`).
                match edit {
                    ph2d_editor_core::JointFieldEdit::PickBodyA => {
                        self.joint_body_pick = Some((entity_bits, false));
                    }
                    ph2d_editor_core::JointFieldEdit::PickBodyB => {
                        self.joint_body_pick = Some((entity_bits, true));
                    }
                    // ⚠️ **O ÚNICO fan-out da §12** (W-JointCopy). O
                    // resto da seção descreve UM joint e edita UM; um
                    // paste existe para carimbar o rig inteiro, e sem
                    // isto o gesto é *digitar quinze campos, dez vezes*.
                    // Espalhado sobre a seleção crua: quem não for joint
                    // cai no early-return de `paste_joint_properties`,
                    // do mesmo jeito que o fan-out do §11 atravessa
                    // entidades sem `Collider`.
                    ph2d_editor_core::JointFieldEdit::PasteProperties
                        if !pd.inspector_selection.is_empty() =>
                    {
                        for &t in &pd.inspector_selection {
                            pd.joint_edits.push((t, edit));
                        }
                    }
                    _ => pd.joint_edits.push((entity_bits, edit)),
                }
            }
            // §14 Platform Player. Sem fan-out, e pela razão da §12: a
            // seção descreve UM personagem, o que está selecionado.
            EditorAction::InspectorPlayerEdit { entity_bits, edit } => {
                // ⚠️ **O `ClearRun` é o único verbo da §14 que não é uma
                // escrita de componente** (W17): a fita de entrada mora na
                // shell, então ele é honrado AQUI, onde o `self` é
                // mutável — o lugar e a razão exatos do `Join` da §11 e do
                // eyedropper da §12.
                //
                // ⚠️ E interceptar não é higiene: descartar é idempotente,
                // então espalhá-lo pela seleção não corromperia nada HOJE.
                // É precisamente essa forma que apodrece — o Ctrl+V do
                // editor de nós colava duas vezes porque um dispatch
                // duplicado "nunca tinha importado enquanto todos os
                // verbos eram idempotentes".
                //
                // ⚠️ **E a troca em si mora numa PORTA** (`run_stash`,
                // W25), porque o painel de MUNDO é uma segunda VISTA da
                // mesma corrida: duas cópias do `mem::take` fariam a
                // mesma coisa hoje e divergiriam no dia em que o
                // descarte ganhar um caso especial.
                if matches!(edit, ph2d_editor_core::PlayerFieldEdit::ClearRun) {
                    // ⚠️ **Descartar GUARDA** (W24): a corrida sai do
                    // documento e fica na sessão, porque o clique era
                    // irreversível — a fita não é `ProjectState`, então
                    // sem isto o único caminho de volta era reabrir o
                    // arquivo.
                    ph2d_app_physics::run_stash::apply(
                        ph2d_app_physics::run_stash::RunVerb::Discard,
                        &mut self.player_tape,
                        &mut self.discarded_run,
                    );
                } else if matches!(edit, ph2d_editor_core::PlayerFieldEdit::RestoreRun) {
                    ph2d_app_physics::run_stash::apply(
                        ph2d_app_physics::run_stash::RunVerb::Restore,
                        &mut self.player_tape,
                        &mut self.discarded_run,
                    );
                } else {
                    pd.player_edits.push((entity_bits, edit));
                }
            }
            EditorAction::InspectorWheelEdit { entity_bits, edit } => {
                // W3: o eyedropper ARMA aqui (onde `self` é mutável), como
                // o do joint e pela mesma razão — o pick é estado da
                // shell, não uma escrita de componente.
                if matches!(edit, ph2d_editor_core::WheelFieldEdit::PickMountBody) {
                    self.wheel_body_pick = Some(entity_bits);
                } else if matches!(edit, ph2d_editor_core::WheelFieldEdit::PickRope) {
                    // W1: o mesmo lugar e a mesma razão — o pick é estado
                    // da shell. O alvo é a ROTA, resolvido no Down.
                    self.wheel_rope_pick = Some(entity_bits);
                } else {
                    pd.wheel_edits.push((entity_bits, edit));
                }
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
            other => return Some(other),
        }
        None
    }
}
