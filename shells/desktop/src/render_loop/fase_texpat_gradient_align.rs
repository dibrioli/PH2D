//! **Fase do quadro: O PADRÃO DE TEXTURA, OS GRADIENTES, O ALINHAMENTO E O PIVÔ** — a arte e a lei do padrão de textura, os gradientes e as suas paradas, alinhar e distribuir,
//! o pivô e o arme da ferramenta do osso (OBRA 2 da `line/render-loop`, 2026-09-13).

use super::*;

/// Os pedidos que o dreno do barramento recolheu neste quadro para esta fase.
pub(super) struct TexpatGradientAlignIntents {
    pub(super) pending_vec_pivot_edit: bool,
    pub(super) pending_texpat: Option<(
        ph2d_vec_render::PatternSlot,
        crate::texture_pattern_edit::TexPatCmd,
    )>,
    pub(super) pending_texpat_source: Option<ph2d_vec_render::PatternSlot>,
    pub(super) pending_vec_grad_angle: Option<f64>,
    pub(super) pending_vec_grad_add: bool,
    pub(super) pending_vec_grad_remove: bool,
    pub(super) pending_vec_grad_influence: Option<f64>,
    pub(super) pending_vec_grad_jitter: Option<f64>,
    pub(super) pending_vec_grad_add_stop: bool,
    pub(super) pending_vec_grad_remove_stop: bool,
    pub(super) pending_vec_align: Option<crate::input_dispatch::VecAlign>,
    pub(super) pending_vec_distribute: Option<crate::input_dispatch::VecDistribute>,
}

impl crate::App {
    /// Ver o cabeçalho do módulo.
    pub(super) fn fase_texpat_gradient_align(
        &mut self,
        intents: TexpatGradientAlignIntents,
        vec_xf_ops: ph2d_vec_scene::VecXforms,
    ) -> Option<ph2d_vec_scene::VecXforms> {
        // O `gfx` re-derivado; os guardas do quadro já correram na `fase_chrome_clock`.
        let gfx = self.gfx.as_mut()?;
        let FrameGfx {
            sim,
            asset_db,
            tools,
            vec_scene,
            ..
        } = FrameGfx::of(gfx);
        let TexpatGradientAlignIntents {
            pending_vec_pivot_edit,
            mut pending_texpat,
            pending_texpat_source,
            pending_vec_grad_angle,
            pending_vec_grad_add,
            pending_vec_grad_remove,
            pending_vec_grad_influence,
            pending_vec_grad_jitter,
            pending_vec_grad_add_stop,
            pending_vec_grad_remove_stop,
            pending_vec_align,
            pending_vec_distribute,
        } = intents;
        // ⭐ **A secção PATTERN** (plano 33 W5) — a arte primeiro (ela abre um diálogo, que
        // congela o laço), depois a lei. As duas desaguam na MESMA porta.
        if let Some(slot) = pending_texpat_source
            && let Some(sel) = self.vec.pen.selected()
        {
            // ⚠️ O `source_for` devolve a fonte que a forma JÁ tem quando ela tem uma — e aqui o
            // artista pediu para TROCAR. O diálogo abre sempre, então o caminho é o directo.
            if let Some(source) = crate::texture_pattern_pick::pick_source(asset_db) {
                // ⭐ **O tamanho a adoptar SE a forma ainda não tinha arte** — quem decide é o
                // `apply`; aqui só se MEDE, porque é aqui que o `AssetDb` está. Ver a variante.
                // ⭐ **A ARTE mede-se pela porta que ASSA** — e o mapa de afins e a geometria
                // viva são os do quadro anterior, que é o que os seis sítios de pick desta
                // shell já consomem: a forma que o artista acabou de apontar já lá está.
                //
                // ⏳ **MEDIDO e não curado** (auditoria de 2026-08-30): quando a forma JÁ tem
                // padrão, o `apply_vec_set_fill_kind` preserva-o e descarta este `size` inteiro
                // — é a forma do *«consumidor que projecta o valor fora»*. ⛔ Um guarda aqui
                // codificaria um facto sobre **outra** função (*"o `source_for` só devolve
                // não-`None` quando o slot já era padrão"*), e é essa acoplagem que diverge em
                // silêncio. O custo é **por clique**, não por quadro.
                let xf = ph2d_vec_entities::transform::build(sim, &self.vec.entities);
                let arte = crate::texture_pattern_pick::art_dims(
                    asset_db,
                    vec_scene,
                    &xf,
                    &self.vec.live_drawn,
                    sel,
                    &source,
                    &|id| {
                        ph2d_vec_entities::entities::object_selection_for(
                            sim,
                            vec_scene,
                            &self.vec.entities,
                            id,
                        )
                    },
                );
                let (size, _) =
                    crate::texture_pattern_pick::default_placement(vec_scene, sel, arte);
                pending_texpat = Some((
                    slot,
                    crate::texture_pattern_edit::TexPatCmd::Source(source, size),
                ));
            }
        }
        // ⭐⭐ **O SUJEITO VEIO NO ID DO CONTROLO** (plano 35, wave F): cada secção tem os seus,
        // então não há preferência a coagir nem alvo a resolver — *o gesto diz em quem escreve*.
        if let Some((slot, cmd)) = pending_texpat {
            crate::texture_pattern_edit::apply(vec_scene, &self.vec.pen, slot, cmd);
        }
        if let Some(deg) = pending_vec_grad_angle {
            crate::input_dispatch::apply_vec_set_grad_angle(vec_scene, &self.vec.pen, deg);
        }
        if pending_vec_grad_add {
            crate::input_dispatch::apply_vec_grad_add_point(vec_scene, &self.vec.pen);
        }
        if pending_vec_grad_remove {
            self.vec.grad_selected = crate::input_dispatch::apply_vec_grad_remove_point(
                vec_scene,
                &self.vec.pen,
                self.vec
                    .grad_selected
                    .and_then(ph2d_vec_render::GradHandle::point),
            )
            .map(ph2d_vec_render::GradHandle::Point);
        }
        if let Some(v) = pending_vec_grad_influence {
            crate::input_dispatch::apply_vec_grad_influence(
                vec_scene,
                &self.vec.pen,
                self.vec
                    .grad_selected
                    .and_then(ph2d_vec_render::GradHandle::point),
                v,
            );
        }
        if let Some(v) = pending_vec_grad_jitter {
            crate::input_dispatch::apply_vec_grad_jitter(
                vec_scene,
                &self.vec.pen,
                self.vec
                    .grad_selected
                    .and_then(ph2d_vec_render::GradHandle::point),
                v,
            );
        }
        if pending_vec_grad_add_stop {
            self.vec.grad_selected =
                crate::input_dispatch::apply_vec_grad_add_stop(vec_scene, &self.vec.pen)
                    .map(ph2d_vec_render::GradHandle::Stop)
                    .or(self.vec.grad_selected);
        }
        if let Some(a) = pending_vec_align {
            crate::input_dispatch::apply_vec_align(vec_scene, &self.vec.pen, &vec_xf_ops, a);
        }
        if let Some(d) = pending_vec_distribute {
            crate::input_dispatch::apply_vec_distribute(vec_scene, &self.vec.pen, &vec_xf_ops, d);
        }
        if pending_vec_grad_remove_stop
            && let Some(si) = self
                .vec
                .grad_selected
                .and_then(ph2d_vec_render::GradHandle::stop)
        {
            // Only an interior stop can be removed; a no-op otherwise keeps the
            // current selection (endpoint handles aren't removable stops).
            self.vec.grad_selected = crate::input_dispatch::apply_vec_grad_remove_stop(
                vec_scene,
                &self.vec.pen,
                Some(si),
            )
            .map(ph2d_vec_render::GradHandle::Stop);
        }
        if pending_vec_pivot_edit {
            // Arma "Set Center": a próxima pressão no canvas põe a ORIGEM ali.
            self.vec.pivot_edit = true;
        }
        // ⭐⭐⭐ **A FERRAMENTA ARMA-SE AQUI** — antes de ela republicar o espelho (`vec_cfg`
        // abaixo), senão a escrita da aresta do foco seria revertida no mesmo quadro.
        //
        // ⛔⛔ Ordem do dono (2026-09-09): *«ao seleccionar o osso … o botão Transform é
        // seleccionado»*. Quem o pede é a aresta lá em baixo, que não pode tocar em `gfx.tools`
        // (ele está emprestado a `sim`/`hero`).
        if let Some(acao) = self.skeleton.bone_arm_pending.take() {
            vector_bridge::arm_bone(tools, acao);
        }
        Some(vec_xf_ops)
    }
}
