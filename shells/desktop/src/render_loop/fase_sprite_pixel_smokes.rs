//! **Fase do quadro: AS CENAS DOS PIXELS DA SPRITE** — o import do `.ase` a acontecer, as faixas e a
//! cura delas (dither) e a sprite como fonte de luz (OBRA 2 da `line/render-loop`, 2026-09-12).

use super::*;

impl crate::App {
    /// Ver o cabeçalho do módulo.
    pub(super) fn fase_sprite_pixel_smokes(&mut self) {
        // O `gfx` re-derivado; os guardas do quadro já correram na `fase_chrome_clock`.
        let Some(gfx) = self.gfx.as_mut() else {
            return;
        };
        let FrameGfx {
            renderer,
            sim,
            asset_db,
            toasts,
            hero_screen,
            next_import_cell,
            atlas_asset_map,
            ..
        } = FrameGfx::of(gfx);

        // **O IMPORT DO `.ase` A ACONTECER** (`PH2D_ASE_SMOKE=1`): ele ESCREVE um `.ase` numa pasta
        // temporária e larga-o pela porta do produto — o mesmo `import_ase` que o drag & drop
        // chama. ⚠️ Não há caminho paralelo: se este smoke funciona, largar um ficheiro do artista
        // funciona. E o que a conversão perdeu sai numa linha por assunto, nomeada.
        if let Some(hero) = hero_screen.as_mut()
            && crate::ase_smoke::enabled()
            && !std::mem::replace(&mut self.ase_smoke_done, true)
            && let Some((bits, lines)) = crate::ase_smoke::spawn_if_enabled(
                sim,
                renderer,
                asset_db,
                next_import_cell,
                atlas_asset_map,
                hero.project.pixels_per_meter,
            )
        {
            if bits != 0 {
                hero.gizmo.replace_selection(Some(bits));
                hero.bus
                    .push(ph2d_editor_core::action_bus::EditorAction::SetViewFocus {
                        kind: ph2d_editor_core::ViewFocusKind::Selected,
                    });
            }
            for (i, line) in lines.into_iter().enumerate() {
                toasts.push(if i == 0 && bits != 0 {
                    Toast::success(line)
                } else {
                    Toast::warning(line)
                });
            }
            self.title_dirty = true;
        }

        // **AS FAIXAS, E A CURA** (`PH2D_DITHER_SMOKE=1`, plano `docs/Sprite_projeto/18` W6.1):
        // UMA sprite partida ao meio — cima a descida fiel (faixas duras), baixo a descida com
        // dither (liso). As duas metades partem do mesmo degradê de 16 bits, coluna a coluna, e as
        // arestas de cima param na costura.
        //
        // ⚠️ **Foram duas sprites lado a lado, e o Enio não viu diferença nenhuma.** Duas metades
        // adjacentes valem muito mais que duas imagens vizinhas: a diferença é de UM código, que é
        // perto do limiar do olho, e o olho compara muito melhor através de uma fronteira
        // partilhada. A sprite traz também o seu `TextureFilter(Nearest)` — o filtro `Smooth` do
        // projeto interpolava o degrau todo ao ampliar, e era isso que lavava a cena.
        if let Some(hero) = hero_screen.as_mut()
            && crate::dither_smoke::enabled()
            && !std::mem::replace(&mut self.dither_smoke_done, true)
        {
            let ppm = hero.project.pixels_per_meter;
            if let Some(bits) = crate::dither_smoke::spawn_if_enabled(sim, renderer, asset_db, ppm)
            {
                hero.gizmo.replace_selection(Some(bits));
                hero.bus
                    .push(ph2d_editor_core::action_bus::EditorAction::SetViewFocus {
                        kind: ph2d_editor_core::ViewFocusKind::All,
                    });
                toasts.push(Toast::success(
                    "Dither smoke: one gradient, two descents — the top half has bands, \
                     the bottom half does not"
                        .to_string(),
                ));
                self.title_dirty = true;
            }
        }

        // **A SPRITE COMO FONTE DE LUZ** (`PH2D_EMISSIVE_SMOKE=1`, plano `docs/Sprite_projeto/18`
        // W8): duas lâmpadas iguais, e só a da direita carrega `SpriteEmissive`. A da esquerda
        // existe para o «antes» estar no ecrã — um halo sozinho parece só uma sprite clara.
        if let Some(hero) = hero_screen.as_mut()
            && crate::emissive_smoke::enabled()
            && !std::mem::replace(&mut self.emissive_smoke_done, true)
        {
            let ppm = hero.project.pixels_per_meter;
            if let Some(bits) =
                crate::emissive_smoke::spawn_if_enabled(sim, renderer, asset_db, ppm)
            {
                hero.gizmo.replace_selection(Some(bits));
                hero.bus
                    .push(ph2d_editor_core::action_bus::EditorAction::SetViewFocus {
                        kind: ph2d_editor_core::ViewFocusKind::All,
                    });
                toasts.push(Toast::success(
                    "Emissive smoke: same lamp twice — only the right one emits. \
                     Drag `Emissive` in the Inspector to dial it"
                        .to_string(),
                ));
                self.title_dirty = true;
            }
        }
    }
}
