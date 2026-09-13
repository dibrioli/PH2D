//! **Fase do quadro: O VALOR DO FILTRO E A COR DO PICKER** — o slider de uma linha da pilha de filtros e a cor que o picker OKLCH partilhado escolhe (OBRA 2 da `line/render-loop`, 2026-09-13).

use super::*;
use crate::fx_live::FilterHit;

/// Os pedidos que o dreno do barramento recolheu neste quadro para esta fase.
pub(super) struct FilterValuesAndColourIntents {
    pub(super) pending_filter_val: Option<(crate::fx_live::FilterHit, f64)>,
}

impl crate::App {
    /// Ver o cabeçalho do módulo.
    pub(super) fn fase_filter_values_and_colour(
        &mut self,
        intents: FilterValuesAndColourIntents,
        sel: Vec<ph2d_vec_scene::VecPathId>,
    ) {
        // O `gfx` re-derivado; os guardas do quadro já correram na `fase_chrome_clock`.
        let Some(gfx) = self.gfx.as_mut() else {
            return;
        };
        let FrameGfx {
            sim, hero_screen, ..
        } = FrameGfx::of(gfx);
        // O bloco do quadro só chama esta fase com o `HeroScreen` vivo.
        let Some(hero) = hero_screen.as_mut() else {
            return;
        };
        let FilterValuesAndColourIntents { pending_filter_val } = intents;
        if let Some((hit, v)) = pending_filter_val {
            #[allow(clippy::cast_possible_truncation)]
            let x = v as f32;
            crate::fx_live::edit(sim, &self.vec.entities, &sel, |f| {
                let row = match hit {
                    FilterHit::Radius(r)
                    | FilterHit::OffX(r)
                    | FilterHit::OffY(r)
                    | FilterHit::Opacity(r)
                    | FilterHit::Scale(r)
                    | FilterHit::Detail(r)
                    | FilterHit::Seed(r)
                    | FilterHit::Grow(r)
                    | FilterHit::Hue(r)
                    | FilterHit::Sat(r)
                    | FilterHit::Bright(r) => r,
                    _ => return,
                };
                let Some(op) = f.ops.get_mut(row) else { return };
                match hit {
                    FilterHit::Radius(_) => op.radius = x,
                    FilterHit::OffX(_) => op.offset[0] = x,
                    FilterHit::OffY(_) => op.offset[1] = x,
                    FilterHit::Opacity(_) => op.opacity = x,
                    FilterHit::Scale(_) => op.scale = x,
                    // As duas CONTAGENS chegam já arredondadas da fronteira do painel; o
                    // clamp aqui é a recusa de um número órfão (um arquivo, um teste),
                    // não uma segunda régua.
                    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
                    FilterHit::Detail(_) => {
                        op.detail = (x.round() as u8).clamp(1, ph2d_ecs::FxOp::MAX_DETAIL);
                    }
                    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
                    FilterHit::Seed(_) => op.seed = x.clamp(0.0, 255.0).round() as u8,
                    FilterHit::Grow(_) => op.grow = x,
                    // ⚠️ **GRAUS -> VOLTAS**, o inverso exato da linha que publica o
                    // snapshot. As duas conversões são as ÚNICAS do eixo, e é por isso
                    // que ficam nomeadas uma na outra.
                    FilterHit::Hue(_) => op.hue = x / 360.0,
                    FilterHit::Sat(_) => op.sat = x,
                    FilterHit::Bright(_) => op.bright = x,
                    _ => {}
                }
            });
        }
        // A cor do halo vem do MESMO picker OKLCH partilhado (lido como o Contour, aqui,
        // porque o alvo é um componente ECS). Qual LINHA? A que o alvo do picker nomeia.
        if let Some(target) = hero.store.picker_target()
            && let Some((row, slot)) = crate::fx_live::colour_target(target)
            && let Some((value, _, _, _)) = hero
                .store
                .blender_picker(ph2d_editor_core::ids::INSP_BLENDER_PICKER)
        {
            let c = value.rgba;
            let col = [
                f32::from(c[0]) / 255.0,
                f32::from(c[1]) / 255.0,
                f32::from(c[2]) / 255.0,
                f32::from(c[3]) / 255.0,
            ];
            // ⚠️ **A rota mora numa função PURA** (`apply_picked_colour`), e não aqui: a
            // decisão de QUAL cor recebe a escolha é o que um arch-gate sobre o fonte NÃO
            // consegue provar — a mutação que dobrava o stop na ponta escura manteve o nome
            // do slot num braço inalcançável e passou verde. Lá ela é observável.
            let sel_stop = usize::from(ph2d_panel_vector::selected_stop(row));
            crate::fx_live::edit(sim, &self.vec.entities, &sel, |f| {
                if let Some(op) = f.ops.get_mut(row) {
                    crate::fx_live::apply_picked_colour(op, slot, sel_stop, col);
                }
            });
        }
    }
}
