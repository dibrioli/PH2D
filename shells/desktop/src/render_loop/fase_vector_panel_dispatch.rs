//! **Fase do quadro: O DESPACHO DO PAINEL VECTORIAL E A TINTA DO TRAÇO** — o `vector_bridge`, o traço presente, o registo da selecção, o pincel e a tinta do traço (OBRA 2 da `line/render-loop`, 2026-09-13).

use super::*;

/// Os pedidos que o dreno do barramento recolheu neste quadro para esta fase.
pub(super) struct VectorPanelDispatchIntents {
    pub(super) pending_stroke_present: bool,
}

impl crate::App {
    /// Ver o cabeçalho do módulo.
    pub(super) fn fase_vector_panel_dispatch(
        &mut self,
        intents: VectorPanelDispatchIntents,
        vec_px_to_world: f64,
        vec_xf_ops: ph2d_vec_scene::VecXforms,
    ) -> Option<(
        ph2d_tool_vector::VectorDrawConfig,
        ph2d_vec_scene::VecXforms,
    )> {
        // O `gfx` re-derivado; os guardas do quadro já correram na `fase_chrome_clock`.
        let gfx = self.gfx.as_mut()?;
        let FrameGfx {
            sim,
            tools,
            vec_scene,
            hero_screen,
            ..
        } = FrameGfx::of(gfx);
        // O bloco do quadro só chama esta fase com o `HeroScreen` vivo.
        let hero = hero_screen.as_mut()?;
        let VectorPanelDispatchIntents {
            pending_stroke_present,
        } = intents;
        let vec_cfg = vector_bridge::dispatch(
            hero,
            tools,
            vec_scene,
            &mut self.vec.pen,
            &mut self.vec.shape,
            &mut self.vec.pencil,
            vec_px_to_world,
            self.vec.grad_selected,
            &vec_xf_ops,
            sim,
            &self.vec.entities,
            self.vec.pivot_edit,
            self.vec.snap,
            self.texpat_lock_aspect,
            self.texpat_gap_link,
            self.texture_pattern_live.tiles(),
        );
        // ⭐ **Stroke** (plano 34): dar/tirar o traço da forma selecionada. **Honrar e só depois
        // publicar**, a mesma ordem do `resize_box` — publicar antes deixaria a caixa a mostrar
        // o estado ANTERIOR por um quadro, e o artista veria o clique *"não pegar"*.
        //
        // ⚠️ **Aqui, e não no dreno de baixo:** a ficha do traço novo sai do `vec_pen`, que o
        // `dispatch` acabou de sincronizar com a ferramenta (`pen.set_style`). Um sítio mais
        // tarde leria a ficha do quadro anterior.
        if pending_stroke_present {
            crate::vec_stroke_present::toggle(vec_scene, &self.vec.pen, vec_px_to_world);
        }
        ph2d_panel_vector::state::set_stroke_present(
            crate::vec_stroke_present::selected_stroke_present(vec_scene, &self.vec.pen),
        );
        // ⭐ **A TINTA do traço** (plano 35, wave D) — publicada AQUI, ao lado da irmã, e não no
        // `vector_bridge`: o dreno acima pode ter acabado de dar ou tirar o traço, e uma
        // publicação anterior a ele mostraria a fileira de um traço que já não existe.
        // ⭐ **O DIAGNÓSTICO da selecção** (`PH2D_PATTERN_LOG=1`) — aqui, depois dos drenos,
        // porque é aqui que a cena é o que o artista vê. Por EVENTO: só quando a selecção muda.
        crate::texture_pattern_edit::log_selection(vec_scene, &self.vec.pen);
        // ⭐ A lei do PINCEL da selecção (plano 36, W4) — `None` esconde a secção *Brush*.
        ph2d_panel_vector::set_current_brush(
            // ⚠️ O `sel` fica ATADO até ao fim: a pergunta *"tem arte?"* precisa do
            // ANFITRIÃO (a recusa é sobre pertença), e a redacção anterior consumia-o no
            // primeiro `and_then`.
            self.vec.pen.selected().and_then(|sel| {
                vec_scene
                    .path(sel)
                    .and_then(|p| p.stroke.as_ref())
                    .and_then(ph2d_vec_scene::StrokeSpec::brush)
                    .map(|b| ph2d_panel_vector::BrushRow {
                        // ⚠️ *"Tem arte?"* é uma pergunta sobre a CENA, não sobre o campo: um id que
                        // aponta para uma forma apagada é um pincel sem arte, e o rótulo do botão
                        // tem de o dizer.
                        //
                        // ⛔⛔ **E ela vai pela porta que RESOLVE** (auditoria de 2026-08-30). Um
                        // `scene.path(a).is_some()` escrito aqui é uma SEGUNDA resposta: desde que
                        // a arte pode ser um grupo, a recusa é sobre **pertença** — o caminho pode
                        // existir e a arte ser recusada na mesma, e o botão dizia *"Change
                        // Shape…"* sobre um traço que pinta a cor de recurso, sem mensagem.
                        has_art: b.art.is_some_and(|a| {
                            !ph2d_vec_art_live::pattern::art_members(sel, a, &|id| {
                                ph2d_vec_entities::entities::object_selection_for(
                                    sim,
                                    vec_scene,
                                    &self.vec.entities,
                                    id,
                                )
                            })
                            .is_empty()
                        }),
                        spacing: b.spacing,
                        scale: b.scale,
                        offset: b.offset,
                        rotation_deg: b.rotation_deg,
                        flip: b.flip,
                    })
            }),
        );
        ph2d_panel_vector::state::set_stroke_paint_kind(
            crate::vec_stroke_paint::selected_stroke_paint_kind(vec_scene, &self.vec.pen),
        );
        Some((vec_cfg, vec_xf_ops))
    }
}
