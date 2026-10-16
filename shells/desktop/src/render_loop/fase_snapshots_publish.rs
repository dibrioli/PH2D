//! **Fase do quadro: A PUBLICAÇÃO DOS INSTANTÂNEOS** — o `snapshots::publish` que escreve no `HeroScreen` o que a
//! cena é neste quadro: hierarquia, grelha, estatísticas, gizmo e inspector (OBRA 2 da `line/render-loop`, 2026-09-13).
//!
//! ⚠️ É UM statement: a chamada e a lista de argumentos, com os blocos que os calculam — menos as leituras da física
//! (a contagem do Paste, o player, as alças, o encaixe), que a fase-filha `fase_snapshot_readouts` calcula ANTES.
//! Nenhuma delas escreve nada, logo a ordem nova de avaliação não se observa (OBRA 3 da `line/render-bodies`).

use super::*;

/// As leituras da física que esta fase passa ao `publish` — fase-filha, num ficheiro irmão (por `#[path]`, para o
/// `render_loop/mod.rs` não crescer acima do tecto dele).
#[path = "fase_snapshot_readouts.rs"]
mod readouts;
use readouts::SnapshotReadouts;

/// ⭐ **Os instantâneos que o `publish` não pode calcular** (SCRIPT e PARTICLES) — fase-filha, num
/// ficheiro irmão, pelo tecto de LOC desta função. ⚠️ O nome TEM de começar por `fase_`: o texto
/// emendado do quadro colhe só esses.
#[path = "fase_snapshots_tardios.rs"]
mod tardios;

/// ⭐ **O instantâneo da CUTSCENE** (TOP-20 #19) — fase-filha própria, pelo tecto de ARGUMENTOS da
/// irmã: ela é a única que lê o documento da ANIMAÇÃO. ⚠️ O nome TEM de começar por `fase_`.
#[path = "fase_snapshots_sequence.rs"]
mod sequencia;

/// ⭐ **O instantâneo do painel TAGS** (TOP-20 #9) — fase-filha própria, pelo tecto de LOC desta
/// função: ele é de OUTRO painel, e o custo dele depende de aquele painel estar ABERTO.
/// ⚠️ O nome TEM de começar por `fase_`.
#[cfg(feature = "panel-tags")]
#[path = "fase_snapshots_tags.rs"]
mod tags_snapshot;

impl crate::App {
    /// Ver o cabeçalho do módulo.
    pub(super) fn fase_snapshots_publish(
        &mut self,
        diag_input_events: u32,
        diag_paint_stamps: u32,
        tool_preview_bits: [Option<u64>; 3],
        window_size: ph2d_host::WindowSize,
    ) -> Option<[Option<u64>; 3]> {
        // As leituras da física que a chamada passa — ver a fase-filha.
        let SnapshotReadouts {
            joint_paste_targets,
            player_live,
            player_law,
            joint_anchor_handles,
            joint_anchor_snap,
        } = self.fase_snapshot_readouts(window_size)?;
        // ⭐ **Copiado ANTES do empréstimo do `gfx`** (TOP-20 #14): o `publish` recebe `&mut` de
        // metade da `App`, e ler `self.physics` lá dentro emprestaria `self` duas vezes. É o mesmo
        // molde do `audio_ready` do `components_ctx`.
        let projectile_over = self.physics.projectile_over.clone();
        let ray_hits = self.physics.ray_hits.clone();
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
            tags,
            tags_problem,
            script,
            particles,
            luz2d,
            toasts,
            motion,
            ..
        } = FrameGfx::of(gfx);
        // O bloco do quadro só chama esta fase com o `HeroScreen` vivo.
        let hero = hero_screen.as_mut()?;
        // F10: a caixa da ferramenta vetorial — inteira no Select, só ALÇAS no modo Forma (a ferramenta
        // fica NOMEADA aqui: o gate `the_object_box_condition_names_every_authoring_tool` lê-a).
        let vector =
            tools.active().map(|t| t.id()) == Some(ph2d_editor_core::ToolId::new("vector"));
        let caixa = ph2d_app_vec::overlay::object_box(vector, self.vec.draw_config.mode);
        hero.gizmo.handles_only = caixa == ph2d_app_vec::overlay::ObjectBox::HandlesOnly;
        // A publicação dos snapshots (`snapshots.rs`, HR-8 / ADR-0021: o paint só lê o HeroScreen).
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
            // ⭐ O ledger — a secção PARALLAX pergunta se OUTRO motor conduz o objecto (auditoria 26).
            &self.preview_drive,
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
            // ⭐⭐⭐ **A caixa de objecto só existe quando NENHUMA ferramenta autora no canvas**
            // (ADR-0112) — a vectorial fora do Select e do modo Forma (lá, só alças: F10), **e a do Flip sempre**.
            // As alças registam hit-rects, e os dois ramos de canvas (`ramo_ferramenta_vetorial` e
            // `ramo_flip_premidos`) exigem o MESMO `on_canvas`: uma caixa sobre um canvas de
            // autoria é um ladrão de cliques, seja qual for a família do objecto que a publica.
            //
            // ⚠️ **E nunca durante o modo de PREVIEW** (W7r): a caixa é derivada da pose
            // AUTORADA, então enquanto a máquina move a forma ela fica para trás e passa a
            // descrever um lugar que a forma já não ocupa — é a razão pela qual o ADR-0128
            // recusou cinco vezes um gizmo sobre geometria que se move.
            caixa != ph2d_app_vec::overlay::ObjectBox::Off
                && !tools
                    .active()
                    .is_some_and(|t| t.id() == ph2d_editor_core::ToolId::new("flip"))
                // ⭐ A ferramenta de osso (A14): o canvas inteiro é o gesto dela.
                && !tools
                    .active()
                    .is_some_and(|t| t.id() == ph2d_editor_core::ToolId::new("bone"))
                && !self.ui_preview.is_on(),
            // As poses que o último desenho derivou — sem elas a caixa do gizmo de um filho
            // colocado aparece onde a forma foi AUTORADA.
            &self.vec.view_derived,
            flip,
            // ⭐ **O relógio anda?** — a secção FACTORY di-lo, e é o que separa *«a fábrica está
            // avariada»* de *«a fábrica está à espera»*. Resolvido AQUI porque é a shell que tem o
            // relógio, como o `bake_range` logo abaixo.
            self.playhead.is_playing(),
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
            joint_paste_targets,
            // W17: quantos tiques de corrida gravada o documento carrega — e
            // é o zero que tira o *Clear Recorded Run* da tela, pelo mesmo
            // desenho do Paste acima.
            self.player_tape.len(),
            // W24: e quantos esperam por um desfazer. O par decide QUAL
            // botão a §14 pinta, e os dois nunca são não-zero ao mesmo tempo
            // (descartar esvazia a fita viva).
            self.discarded_run.len(),
            self.fixed_step.fixed_dt(),
            player_live,
            player_law,
            // W-Pulley W3: o eyedropper de montagem da §13, pelo mesmo motivo.
            self.wheel_body_pick,
            self.wheel_rope_pick,
            // W-J4: o gesto de desenhar está armado?
            self.physics.joint_draw_armed,
            joint_anchor_handles,
            joint_anchor_snap,
            // **O SELO do papel booleano de cada linha** (2026-08-22). ⚠️ Ele lê o plano do
            // quadro ANTERIOR: a hierarquia publica aqui, e a booleana cozinha lá em baixo
            // no mesmo `run_render_frame`. O atraso é de um quadro e o `vec_bool_shape` o
            // documenta — mover qualquer das duas metades na ordem do frame é mudança com
            // gates próprios e sem nada a ganhar.
            &crate::vec_bool_shape::badges(sim, &self.vec.entities, &self.bool_live),
            // O registo — ver o parâmetro na assinatura do `publish`.
            component_registry,
            // ⭐ A árvore de tags do projecto — ver o parâmetro na assinatura do `publish`.
            tags,
            // ⭐ Os projécteis cujo voo acabou (TOP-20 #14) — o readout que a fase da física
            // deixou. ⚠️ Ele vem do `PhysicsState` e não da ponte: o Inspector não a alcança.
            &projectile_over,
            &ray_hits,
            // ⭐ A Animação por objecto (doc ObjectsAndComponents 01 §6): o 🎞 e os clips na Hierarquia.
            &self.timeline,
        );
        // ⭐⭐⭐ **O painel TAGS** (TOP-20 #9) — fase-filha própria; ver o `mod tags_snapshot`.
        #[cfg(feature = "panel-tags")]
        tags_snapshot::publica(sim, hero, tags, tags_problem.as_ref());
        // ⭐⭐ **As secções que o `publish` NÃO pode calcular** (leem a VM, o relógio, as partículas
        // vivas — nada disso está no mundo) — na fase-filha, num ficheiro irmão (tecto de função).
        tardios::publica(
            sim,
            tags,
            script.as_ref(),
            particles,
            &tardios::Sessao {
                escolhido: hero.gizmo.selection,
                quantos: hero.gizmo.selected_len(),
                a_correr: self.playhead.is_playing(),
                mapa: &hero.input_map,
                motion,
            },
        );
        // ⭐ O EDITOR DE SCRIPTS (a família); `filed` = o projecto tem ficheiro (Ctrl+S grava).
        let filed = self.project_path.is_some();
        ph2d_app_code::frame(sim, script.as_mut(), hero, toasts, filed);
        // ⭐ **A LUZ** (Luz 2D): a secção do Inspector e a recusa do solver, que sobe como aviso.
        let sel = (hero.gizmo.selection, hero.gizmo.selected_len());
        let (info, corpo, recusa) = luz2d.publicar(hero.view.luz2d, sim.world(), sel);
        ph2d_panel_inspector::set_current_inspector_luz(info);
        ph2d_panel_inspector::set_current_inspector_corpo_luz(corpo);
        if let Some(m) = recusa {
            toasts.push(Toast::warning(m));
        }
        // ⭐⭐⭐ **A CUTSCENE** (TOP-20 #19) — fase-filha própria; ver o `mod sequencia`.
        sequencia::publica(
            sim,
            &self.timeline,
            &self.playhead,
            hero.gizmo.selection,
            hero.gizmo.selected_len(),
        );
        Some(tool_preview_bits)
    }
}
