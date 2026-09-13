//! **Fase do quadro: A PUBLICAÇÃO DOS INSTANTÂNEOS** — o `snapshots::publish` que escreve no `HeroScreen` o que a
//! cena é neste quadro: hierarquia, grelha, estatísticas, gizmo e inspector (OBRA 2 da `line/render-loop`, 2026-09-13).
//!
//! ⚠️ É UM statement: a chamada e a lista de argumentos, com os blocos que os calculam. Partir os argumentos em
//! `let`s mudaria a ORDEM em que são avaliados — por isso a fase leva uma entrada NUMERADA no tecto por função.

use super::*;

impl crate::App {
    /// Ver o cabeçalho do módulo.
    pub(super) fn fase_snapshots_publish(
        &mut self,
        diag_input_events: u32,
        diag_paint_stamps: u32,
        tool_preview_bits: [Option<u64>; 3],
        window_size: ph2d_host::WindowSize,
    ) -> Option<[Option<u64>; 3]> {
        // O `gfx` re-derivado; os guardas do quadro já correram na `fase_chrome_clock`.
        let gfx = self.gfx.as_mut()?;
        let FrameGfx {
            renderer,
            sim,
            present,
            camera,
            asset_db,
            tools,
            vec_scene,
            flip,
            hero_screen,
            hero_live,
            sheets,
            atlas_asset_map,
            asset_catalogs,
            component_registry,
            physics,
            ..
        } = FrameGfx::of(gfx);
        // O bloco do quadro só chama esta fase com o `HeroScreen` vivo.
        let hero = hero_screen.as_mut()?;
        // Snapshot publication phase — extracted to sibling
        // `snapshots.rs` as a free fn taking explicit refs (Wave
        // 3.2 stage A). Reads PresentWorld + SimWorld + AssetDb,
        // writes onto the HeroScreen (live_hierarchy, grid_view,
        // stats, gizmo_view, inspector_*) so the paint pass
        // honors the HR-8 / ADR-0021 boundary.
        snapshots::publish(
            hero,
            hero_live,
            // A resposta da porta única, computada no início deste quadro.
            self.hovered_object,
            sim,
            present,
            camera,
            asset_db,
            atlas_asset_map,
            asset_catalogs,
            sheets,
            renderer,
            window_size,
            self.game_camera_preview,
            self.last_pointer,
            self.frame_ms_ewma,
            self.frame_cpu_ms_ewma,
            diag_input_events,
            diag_paint_stamps,
            self.paint_ms_ewma,
            // Deform Transform live ⇒ the sprite gizmo is suppressed for the frame (its corner
            // handles share the deform gizmo's screen corners on a whole-image transform).
            ph2d_app_painter::painter_bridge_queries::deform_transform_gizmo_active(tools),
            // Em que disposição a folha aberta está — a caixa do gizmo envolve-a inteira.
            &tool_preview_bits,
            vec_scene,
            // O gizmo da forma só existe fora da ferramenta vetorial, ou no modo
            // Select dela (ADR-0112).
            //
            // ⚠️ **E nunca durante o modo de PREVIEW** (W7r): a caixa é derivada da pose
            // AUTORADA, então enquanto a máquina move a forma ela fica para trás e passa a
            // descrever um lugar que a forma já não ocupa — é a razão pela qual o ADR-0128
            // recusou cinco vezes um gizmo sobre geometria que se move. E as alças dela
            // registram hit-rects, que é o mesmo motivo pelo qual o ADR-0112 já a suprime
            // nos modos de nó: uma caixa sobre a apresentação é um ladrão de cliques.
            (!tools
                .active()
                .is_some_and(|t| t.id() == ph2d_editor_core::ToolId::new("vector"))
                || self.vec.draw_config.mode == ph2d_tool_vector::DrawMode::Select)
                && !self.ui_preview.is_on(),
            // As poses que o último desenho derivou — sem elas a caixa do gizmo de um filho
            // colocado aparece onde a forma foi AUTORADA.
            &self.vec.view_derived,
            flip,
            // Idem para o objeto Flip: gizmo fora da tool Flip, ou no modo Select
            // dela — em Draw/Erase ele comeria o clique do canvas (ADR-0112 parity).
            !tools
                .active()
                .is_some_and(|t| t.id() == ph2d_editor_core::ToolId::new("flip"))
                || matches!(
                    self.flip_state.style.map(|s| s.mode),
                    Some(ph2d_tool_flip::FlipMode::Select)
                ),
            // W4: the range the §11 Bake button covers, resolved HERE
            // because the shell owns both the document and the clock, and
            // shown on the button so the artist never has to guess it.
            // Two numbers now — the loop's start is honoured (W-BakeRange),
            // so a `[2s, 5s]` loop bakes `[2s, 5s]` and the button says so.
            {
                let (bs, be) =
                    ph2d_app_physics::bake::bake_range(&self.timeline.doc, &self.playhead);
                (bs as f32, be as f32)
            },
            // Which pose channels the Bake selector shows as chosen.
            self.bake_channels.tag(),
            // The pending join KIND the §11 selector shows as chosen.
            self.physics.join_kind,
            // The armed §12 joint-body eyedropper, so the waiting slot's
            // picker paints pressed.
            self.joint_body_pick,
            // W-JointCopy: quantos joints um Paste atingiria. `0` sem nada
            // copiado — e é o zero que tira o botão da tela.
            //
            // ⚠️ Contado sobre a SELEÇÃO, porque o Paste é a única edição da
            // §12 que faz fan-out; e contando só quem de fato carrega um
            // `PhysicsJoint`, senão o rótulo prometeria dez alvos numa
            // seleção de nove sprites e um joint.
            if self.joint_clipboard.is_some() {
                hero.gizmo
                    .iter_selected()
                    .filter(|&b| {
                        sim.world()
                            .get::<ph2d_physics_ecs::PhysicsJoint>(ph2d_ecs::Entity::from_bits(b))
                            .is_some()
                    })
                    .count()
            } else {
                0
            },
            // W17: quantos tiques de corrida gravada o documento carrega — e
            // é o zero que tira o *Clear Recorded Run* da tela, pelo mesmo
            // desenho do Paste acima.
            self.player_tape.len(),
            // W24: e quantos esperam por um desfazer. O par decide QUAL
            // botão a §14 pinta, e os dois nunca são não-zero ao mesmo tempo
            // (descartar esvazia a fita viva).
            self.discarded_run.len(),
            self.fixed_step.fixed_dt(),
            // `W-PlayerOut` A3: o readout do player SELECIONADO. Resolvido
            // aqui porque `publish` não recebe a ponte, e pela porta única —
            // `None` fora de um player, e também com a física desarmada, que
            // é o que faz a §14 dizer *"not simulating"* em vez de mostrar
            // números de uma corrida que acabou.
            hero.gizmo
                .selection
                .and_then(|b| physics.player_view(ph2d_ecs::Entity::from_bits(b)))
                .copied(),
            // **O que a LEI de facto lê deste personagem** — resolvido aqui
            // pela mesma razão do readout acima (`publish` não recebe a
            // ponte) e pela MESMA porta que decide quem escreve a pose. A
            // shell re-derivá-lo do `PlayerMode` era a segunda cópia que
            // fazia a §14 pintar doze cards vivos sobre um player ASSADO,
            // que a lei não dirige.
            hero.gizmo
                .selection
                .map_or(ph2d_physics_ecs::PlayerLiveness::INERT, |b| {
                    physics.player_liveness(sim.world(), ph2d_ecs::Entity::from_bits(b))
                }),
            // W-Pulley W3: o eyedropper de montagem da §13, pelo mesmo motivo.
            self.wheel_body_pick,
            self.wheel_rope_pick,
            // W-J4: o gesto de desenhar está armado?
            self.physics.joint_draw_armed,
            // W-J2/W-J2b: every grabbable joint anchor. Resolved HERE
            // because `publish` does not take the bridge, and through the
            // SAME door `sync_joint_pivots` uses for the A pivot — two
            // derivations of "where is this anchor" is how two dots would
            // come to disagree. Rest-only (the rule lives in the callee):
            // during play the overlay draws the SOLVER's anchors, and these
            // authored ones would describe a pose the artist is not editing.
            {
                // As DUAS famílias numa lista só: as âncoras (sempre) e os
                // grips de parâmetro (só com o overlay de joints na tela —
                // eles agarram a geometria DELE).
                let at_rest = !self.playhead.is_playing();
                let mut hs = point_gizmo::joint_anchor_handles(sim, physics, at_rest);
                hs.extend(point_gizmo::joint_param_handles(
                    physics,
                    camera,
                    window_size,
                    self.show_colliders,
                    at_rest,
                ));
                // E as alças da RODA selecionada (W-Pulley W1). Terceira
                // família, e a única que lê a SELEÇÃO: uma corda com seis
                // roldanas publicaria doze alças sobrepostas.
                hs.extend(point_gizmo::wheel_handles(
                    sim,
                    hero.gizmo.selection,
                    self.show_colliders,
                    at_rest,
                ));
                // E a QUARTA: os limitadores da corda (W-RopeStop). De toda
                // polia, como as âncoras — a marca É a feature, e escondê-la
                // atrás de uma seleção faria o artista ter de descobrir que
                // ela existe antes de poder descobri-la.
                hs.extend(point_gizmo::rope_stop_handles(
                    sim,
                    physics,
                    self.show_colliders,
                    at_rest,
                ));
                hs
            },
            // The candidate a live anchor drag has caught (the crosshair).
            self.physics.joint_anchor_drag.and_then(|d| d.snap),
            // **O SELO do papel booleano de cada linha** (2026-08-22). ⚠️ Ele lê o plano do
            // quadro ANTERIOR: a hierarquia publica aqui, e a booleana cozinha lá em baixo
            // no mesmo `run_render_frame`. O atraso é de um quadro e o `vec_bool_shape` o
            // documenta — mover qualquer das duas metades na ordem do frame é mudança com
            // gates próprios e sem nada a ganhar.
            // ⭐⭐ **E o selo de quem SEGUE UM DESENHO** (W57), fundido no mesmo mapa: o
            // campo é um selo por linha, e as duas famílias nunca caem na mesma entidade (uma
            // é forma vetorial, a outra é nó do modelador). ⚠️ Fundir aqui, e não somar dois
            // mapas lá dentro, é o que mantém *um produtor, um campo* — a lei que o comentário
            // do `hovered` já escreve dez linhas acima.
            &{
                let mut b = crate::vec_bool_shape::badges(sim, &self.vec.entities, &self.bool_live);
                b.extend(ph2d_app_field3d::scene::link_badges());
                b
            },
            // O registo — ver o parâmetro na assinatura do `publish`.
            component_registry,
        );
        Some(tool_preview_bits)
    }
}
