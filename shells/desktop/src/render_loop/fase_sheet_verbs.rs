//! **Fase do quadro: OS VERBOS DA FOLHA DE SPRITES** — empacotar, arrumar, assar, exportar a folha, exportar
//! uma sprite e o Create do modal de tamanho (OBRA 2 da `line/render-loop`, 2026-09-12).

use super::*;

/// Os pedidos de folha de sprites que o dreno do barramento recolheu neste quadro.
pub(super) struct SheetIntents {
    pub(super) pack_sheet_row: Option<NodeId>,
    pub(super) arrange_sheet_row: Option<NodeId>,
    pub(super) bake_sheet_row: Option<NodeId>,
    pub(super) export_sheet_row: Option<NodeId>,
    pub(super) export_image_row: Option<NodeId>,
}

impl crate::App {
    /// Ver o cabeçalho do módulo.
    pub(super) fn fase_sheet_verbs(&mut self, intents: SheetIntents) {
        // O `gfx` re-derivado; os guardas do quadro já correram na `fase_chrome_clock`.
        let Some(gfx) = self.gfx.as_mut() else {
            return;
        };
        let FrameGfx {
            renderer,
            sim,
            asset_db,
            toasts,
            vec_scene,
            hero_screen,
            hero_live,
            sheets,
            sheet_textures,
            next_sheet_id,
            atlas_asset_map,
            imageio_exporters,
            ..
        } = FrameGfx::of(gfx);
        // O bloco do quadro só chama esta fase com o `HeroScreen` vivo.
        let Some(hero) = hero_screen.as_mut() else {
            return;
        };
        let SheetIntents {
            pack_sheet_row,
            arrange_sheet_row,
            bake_sheet_row,
            export_sheet_row,
            export_image_row,
        } = intents;
        let mut sheet_targets: Vec<u64> = Vec::new();
        if let Some(row) = pack_sheet_row
            && let Some(live) = hero_live.as_ref()
            && let Some(anchor) = live.bridge.entity_for(row)
        {
            if hero.gizmo.is_selected(anchor) {
                sheet_targets.extend(hero.gizmo.iter_selected());
            } else {
                sheet_targets.push(anchor);
            }
        }
        if !sheet_targets.is_empty() {
            let ppm = hero.project.pixels_per_meter;
            // ⚠️ **Um item, um verbo — e este item CRIA.** Ele fazia as duas coisas conforme o
            // alvo (com sprites criava, com uma folha re-arranjava), e a economia era falsa:
            // um verbo que só se descobre por ter selecionado a coisa certa não está no menu,
            // está escondido nele. O Enio pediu o segundo **pelo nome** (2026-08-19), que é a
            // prova de que ele não o encontrava. Agora recusar aponta para onde ele mora.
            if !crate::sheet_frame::sheets_among(sim, &sheet_targets).is_empty() {
                toasts.push(Toast::warning(
                    "Pack into Sheet: that is already a sheet - use Auto-Arrange Pieces",
                ));
            } else {
                // **CRIAR pergunta primeiro** (Enio 2026-08-19: *"Ao criar uma sheet um modal
                // com a resolução deve aparecer antes da criação"*). Os alvos ficam
                // RESERVADOS até o Create — o modal é modal, mas o mundo continua a andar, e
                // recalcular a seleção no Create leria o que ela for ENTÃO, não o que era
                // quando ele pediu. *A pergunta e a resposta têm de falar do mesmo conjunto.*
                match crate::sheet_frame::suggested_size(sim, &sheet_targets, ppm) {
                    Some(px) => {
                        self.pending_sheet_targets = sheet_targets.clone();
                        hero.store.open_sheet_size_dialog(px);
                    }
                    // Sem peça nenhuma não há o que perguntar: um modal a pedir a resolução de
                    // uma folha vazia é a caixa de diálogo que não devia ter aberto.
                    None => {
                        toasts.push(Toast::warning("Sheet: select at least one sprite first"));
                    }
                }
            }
            self.title_dirty = true;
        }
        // **ARRUMAR AS PEÇAS AUTOMATICAMENTE** — o item próprio (Enio 2026-08-19: *"uma opção
        // no menu do botão direito da sheet: arrumar as sprites filhas automaticamente"*).
        //
        // ⚠️ Ele não pergunta resolução nenhuma: ela foi escolhida quando a folha nasceu, e o
        // gesto aqui é *arrume*, não *redimensione*. O que não couber acende a moldura.
        //
        // ⚠️ E aceita a SELEÇÃO inteira, como os irmãos: com três folhas selecionadas, arruma
        // as três. A lei de alvo é a mesma do "Merge Sprites" — a seleção quando a linha
        // clicada faz parte dela, só ela quando não faz — porque duas leis de alvo no mesmo
        // menu seriam adivinhação.
        if let Some(row) = arrange_sheet_row
            && let Some(live) = hero_live.as_ref()
            && let Some(anchor) = live.bridge.entity_for(row)
        {
            let mut targets: Vec<u64> = Vec::new();
            if hero.gizmo.is_selected(anchor) {
                targets.extend(hero.gizmo.iter_selected());
            } else {
                targets.push(anchor);
            }
            let sheets = crate::sheet_frame::sheets_among(sim, &targets);
            if sheets.is_empty() {
                toasts.push(Toast::warning(
                    "Auto-Arrange: select a sheet - to make one, use Pack into Sheet",
                ));
            } else {
                crate::sheet_frame::repack_all(sim, vec_scene, &sheets, toasts);
            }
            self.title_dirty = true;
        }
        // **ASSAR** — a folha vira UMA textura, e cada peça uma janela nela (plano §7.3).
        //
        // ⚠️ Age sobre a linha clicada e só sobre ela, ao contrário dos irmãos: assar é caro
        // (lê N texturas da GPU) e produz um ficheiro por folha, então difundi-lo pela seleção
        // faria um clique distraído reamostrar meia cena. *O custo do gesto decide o alcance
        // dele.*
        if let Some(row) = bake_sheet_row
            && let Some(live) = hero_live.as_ref()
            && let Some(bits) = live.bridge.entity_for(row)
        {
            crate::sheet_bake::bake(
                sim,
                renderer,
                asset_db,
                atlas_asset_map,
                sheets,
                sheet_textures,
                next_sheet_id,
                bits,
                toasts,
            );
            self.title_dirty = true;
        }
        // **EXPORTAR** — os mesmos pixels, mas para disco, e **sem tocar na cena**.
        if let Some(row) = export_sheet_row
            && let Some(live) = hero_live.as_ref()
            && let Some(bits) = live.bridge.entity_for(row)
            && let Some((authored, _)) = crate::sheet_bake::compose_sheet(
                sim,
                renderer,
                asset_db,
                atlas_asset_map,
                next_sheet_id,
                bits,
                toasts,
            )
        {
            crate::sheet_export::export(&authored, toasts);
        }
        // **EXPORTAR UMA SPRITE** (plano `docs/Sprite_projeto/18` W9, Enio 2026-08-21). Os 16
        // exportadores da engine já estavam registados e nenhum gesto os alcançava; esta é a
        // porta. ⚠️ Uma sprite de 16 bits é oferecida em ALTA PRECISÃO primeiro, e só cai para
        // 8 bits quando o formato escolhido a recusa (e aí diz-se).
        if let Some(row) = export_image_row
            && let Some(live) = hero_live.as_ref()
            && let Some(bits) = live.bridge.entity_for(row)
        {
            crate::image_export::export_with_dialog(
                ph2d_ecs::Entity::from_bits(bits),
                sim,
                renderer,
                asset_db,
                atlas_asset_map,
                imageio_exporters,
                toasts,
            );
            self.title_dirty = true;
        }
        // O Create do modal — a criação de facto, com os alvos que ficaram reservados.
        if let Some(size_px) = hero.store.take_sheet_size_request() {
            let targets = std::mem::take(&mut self.pending_sheet_targets);
            let ppm = hero.project.pixels_per_meter;
            if let Some(sheet) = crate::sheet_frame::create_at(
                sim,
                vec_scene,
                &mut self.vec.entities,
                &targets,
                ppm,
                size_px,
                toasts,
            ) {
                // A folha fica selecionada: é ela que o artista vai querer mover,
                // redimensionar ou nomear a seguir — e é o convite a verificar que tudo
                // isso funciona sem uma linha de código próprio.
                hero.gizmo.replace_selection(Some(sheet));
            }
            self.title_dirty = true;
        }
    }
}
