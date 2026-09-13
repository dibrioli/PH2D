//! **Fase do quadro: A FORMA DO CAMINHO E AS TINTAS** — as operações de forma do caminho, abrir e fechar, e a tinta do preenchimento e do traço (OBRA 2 da `line/render-loop`, 2026-09-13).

use super::*;

/// Os pedidos que o dreno do barramento recolheu neste quadro para esta fase.
pub(super) struct PathShapeAndPaintIntents {
    pub(super) pending_vec_path_shape: Option<crate::input_dispatch::VecPathShapeOp>,
    pub(super) pending_vec_toggle_closed: bool,
    pub(super) pending_vec_fill_kind: Option<crate::input_dispatch::VecFillKind>,
    pub(super) pending_vec_stroke_kind: Option<ph2d_panel_vector::StrokePaintKind>,
}

impl crate::App {
    /// Ver o cabeçalho do módulo.
    pub(super) fn fase_path_shape_and_paint(&mut self, intents: PathShapeAndPaintIntents) {
        // O `gfx` re-derivado; os guardas do quadro já correram na `fase_chrome_clock`.
        let Some(gfx) = self.gfx.as_mut() else {
            return;
        };
        let FrameGfx {
            sim,
            asset_db,
            vec_scene,
            ..
        } = FrameGfx::of(gfx);
        let PathShapeAndPaintIntents {
            pending_vec_path_shape,
            pending_vec_toggle_closed,
            pending_vec_fill_kind,
            pending_vec_stroke_kind,
        } = intents;
        if let Some(op) = pending_vec_path_shape {
            crate::input_dispatch::apply_vec_path_shape(vec_scene, &self.vec.pen, op);
        }
        if pending_vec_toggle_closed {
            crate::input_dispatch::apply_vec_toggle_closed(vec_scene, &mut self.vec.pen);
        }
        if let Some(kind) = pending_vec_fill_kind {
            crate::texture_pattern_edit::log_shape(
                &format!("ANTES de mudar para {kind:?}"),
                vec_scene,
                &self.vec.pen,
            );
            // ⭐ **A 4ª condição da costura: o chip tem de LEVAR A ALGUM LUGAR.** Escolher
            // *Tile* numa forma sem padrão abre o diálogo da arte — um chip que muda o tipo de
            // preenchimento para algo invisível é o defeito que esta linha já recebeu três
            // vezes. ⚠️ Desistir devolve `None`, e o `apply` **não muda nada**.
            // ⚠️ Sem closure: `self.vec.pen.selected()` e `self.texture_pattern_source_for`
            // (que é `&mut self`) não cabem no mesmo `and_then`.
            let mut pattern = None;
            if kind == crate::input_dispatch::VecFillKind::Pattern
                && let Some(sel) = self.vec.pen.selected()
                && let Some(source) = crate::texture_pattern_pick::source_for(
                    vec_scene,
                    sel,
                    ph2d_vec_render::PatternSlot::Fill,
                )
            {
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
                let (size, origin) =
                    crate::texture_pattern_pick::default_placement(vec_scene, sel, arte);
                pattern = Some((source, size, origin));
            }
            crate::input_dispatch::apply_vec_set_fill_kind(vec_scene, &self.vec.pen, kind, pattern);
            crate::texture_pattern_edit::log_shape("DEPOIS", vec_scene, &self.vec.pen);
            // The old handle no longer addresses the new fill kind — reset the
            // gradient selection so the overlay highlight + panel don't cling to it.
            self.vec.grad_selected = None;
            self.vec.grad_drag = None;
        }
        // ⭐⭐ **A TINTA DO TRAÇO** (plano 35, wave D) — a 4ª condição da costura, outra vez:
        // escolher *Pattern* num traço que ainda não tem padrão **abre o diálogo da arte**.
        // ⚠️ Desistir devolve `None`, e o `set_kind` **não muda nada** — apagar-lhe a cor do
        // traço por ter fechado um diálogo seria o pior dos dois mundos.
        if let Some(kind) = pending_vec_stroke_kind {
            let mut pattern = None;
            // ⭐⭐⭐ **E O TRAÇO PELA MESMA PORTA** (report do Enio, 2026-08-30: *"e para
            // Stroke?"*). Ele ia direto ao `pick_source`, que abre SEMPRE o diálogo de imagem —
            // o mesmo defeito do preenchimento, e ainda mais cru, porque nem passava pela porta
            // que decide.
            //
            // ⚠️⚠️ **A lei já estava escrita neste ficheiro, um braço acima, para o PINCEL:**
            // *"`art: None` é legítimo por TIPO — um pincel sem arte escolhida desenha a
            // `fallback`, e a arte entra depois pelo gesto de duas mãos. Exigi-la aqui faria o
            // chip abrir um diálogo de ficheiro, que é precisamente o que o plano 36 recusa."*
            // O braço `Pattern` não a herdou porque o `PatternSource` não tinha variante vazia —
            // e desde 30/08 tem. *A regra certa estava no mesmo `match`, para o vizinho.*
            if kind == ph2d_panel_vector::StrokePaintKind::Pattern
                && let Some(sel) = self.vec.pen.selected()
                && let Some(source) = crate::texture_pattern_pick::source_for(
                    vec_scene,
                    sel,
                    ph2d_vec_render::PatternSlot::Stroke,
                )
            {
                // ⚠️ **A colocação sai da MESMA porta do preenchimento** — o tamanho preserva o
                // aspecto da arte e o canto é o da FORMA, nunca a origem do mundo. Uma segunda
                // lei de nascimento aqui reabriria o report do `Clamp` em branco.
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
                let (size, origin) =
                    crate::texture_pattern_pick::default_placement(vec_scene, sel, arte);
                pattern = Some((source, size, origin));
            }
            crate::vec_stroke_paint::set_kind(vec_scene, &self.vec.pen, kind, pattern);
        }
    }
}
