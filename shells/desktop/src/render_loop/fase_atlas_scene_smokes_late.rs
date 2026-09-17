//! **Fase do quadro: AS CENAS DE SMOKE QUE PRECISAM DO ATLAS — a 2.ª metade** — a máscara do Painter, a
//! folha como objeto, a família `PH2D_GPU_COOK_DEMO` a tomar a ferramenta Motion e a sujidade na lente
//! (OBRA 2 da `line/render-loop`, 2026-09-12).
//!
//! ⚠️ **A cura do `PH2D_GPU_COOK_DEMO` mora AQUI** (o `if` que dá a ferramenta Motion às cenas da família):
//! o gate `ph2d_app_motion::motion_state_demo_router_tests` lê este ficheiro por `include_str!`.

use super::*;

impl crate::App {
    /// Ver o cabeçalho do módulo.
    pub(super) fn fase_atlas_scene_smokes_late(&mut self) {
        // O `gfx` re-derivado; os guardas do quadro já correram na `fase_chrome_clock`.
        let Some(gfx) = self.gfx.as_mut() else {
            return;
        };
        let FrameGfx {
            renderer,
            sim,
            asset_db,
            toasts,
            tools,
            vec_scene,
            hero_screen,
            next_import_cell,
            atlas_asset_map,
            motion,
            ..
        } = FrameGfx::of(gfx);

        // Mask smoke (`PH2D_MASK_SMOKE=1`): the same dance for the mask coverage law (doc 25 §13.9).
        // Nothing but the canvas is staged — the artist picks the rail chip, so the scene shows the
        // shipped default mask brush rather than a rigged one.
        if let Some(hero) = hero_screen.as_mut()
            && ph2d_app_painter::mask_smoke::enabled()
            && !std::mem::replace(&mut self.mask_smoke_done, true)
        {
            let ppm = hero.project.pixels_per_meter;
            let cell = *next_import_cell;
            if let Some(bits) = ph2d_app_painter::mask_smoke::spawn_if_enabled(
                sim,
                renderer,
                asset_db,
                cell,
                ppm,
                atlas_asset_map,
            ) {
                *next_import_cell = next_import_cell.saturating_add(1);
                hero.gizmo.replace_selection(Some(bits));
                hero.bus
                    .push(ph2d_editor_core::action_bus::EditorAction::SetViewFocus {
                        kind: ph2d_editor_core::ViewFocusKind::Selected,
                    });
                toasts.push(Toast::success(
                    "Mask smoke: paint some art, then the MASK chip — and SCRUB".to_string(),
                ));
            }
        }

        // ⭐⭐⭐ **O CANVAS DO PAINTER PRESO A OSSOS** (`PH2D_VEC_BONE_PAINT_SMOKE=1`) — a cena que
        // faltava à cura das guias chatas (item 4 do dono): sem ela a correcção estava gateada e
        // **invisível**, e uma cura que ninguém pode ver é uma cura que ninguém julga.
        //
        // ⚠️ Mesma dança da máscara ao lado, e pela mesma razão: o canvas fica SELECCIONADO (é ele o
        // sujeito do Painter) e nada mais é armado — a ferramenta, o pincel e a grelha são do artista.
        if let Some(hero) = hero_screen.as_mut()
            && ph2d_app_vec::smoke_bone_paint::armed()
            && !std::mem::replace(&mut self.vec.bone_paint_smoke_done, true)
        {
            let ppm = hero.project.pixels_per_meter;
            let cell = *next_import_cell;
            // ⚠️ **O `n` é do ROTEADOR e não desta fase**: a cena monta `n` canvas e diz quantos
            // de facto montou — as células do atlas avançam por esse número, senão a cena seguinte
            // sobrescreve a tinta desta.
            if let Some((bits, quantos)) = ph2d_app_vec::smoke_bone_paint::build(
                sim,
                renderer,
                asset_db,
                cell,
                ppm,
                atlas_asset_map,
            ) {
                *next_import_cell = next_import_cell.saturating_add(quantos);
                hero.gizmo.replace_selection(Some(bits));
                // ⛔⛔ **`Selected` mesmo na cena LOTADA, e o `All` foi construído, FOTOGRAFADO e
                // REVERTIDO:** ele ajusta-se às CAIXAS das sprites e a arte dobrada varre para fora
                // delas, logo os canvas das pontas saíam cortados de qualquer maneira — e, mais
                // importante, *enquadrar a fileira inteira é a pergunta errada*: os outros canvas
                // existem para ENCHER o orçamento do quadro, não para serem vistos ao mesmo tempo.
                // O que o dono julga é a junta do canvas à frente dele.
                hero.bus
                    .push(ph2d_editor_core::action_bus::EditorAction::SetViewFocus {
                        kind: ph2d_editor_core::ViewFocusKind::Selected,
                    });
                toasts.push(Toast::success(if quantos > 1 {
                    "Bone-paint smoke: Window > Bones, e na linha Deform compare Fast com Smooth"
                        .to_string()
                } else {
                    "Bone-paint smoke: pegue o Painter e desenhe uma forma sobre o canvas dobrado"
                        .to_string()
                }));
            }
        }

        // **A FOLHA COMO OBJETO** (`PH2D_SHEET_SMOKE=1`, plano `docs/Sprite_projeto/17` §7): cinco
        // peças de tamanhos diferentes entram, e sai UM objeto — um retângulo na hierarquia, com
        // as peças arranjadas dentro como filhos.
        //
        // ⚠️ A folha fica SELECIONADA de propósito: é ela que o artista tem de conseguir mover,
        // redimensionar, esconder e duplicar, e nenhuma dessas coisas tem código próprio — a
        // seleção é o convite a verificá-lo.
        if let Some(hero) = hero_screen.as_mut()
            && crate::sheet_smoke::enabled()
            && !std::mem::replace(&mut self.sheet_smoke_done, true)
        {
            let ppm = hero.project.pixels_per_meter;
            if let Some((sheet, n)) = crate::sheet_smoke::spawn_if_enabled(
                sim,
                renderer,
                asset_db,
                vec_scene,
                &mut self.vec.entities,
                next_import_cell,
                ppm,
                atlas_asset_map,
            ) {
                hero.gizmo.replace_selection(Some(sheet));
                hero.bus
                    .push(ph2d_editor_core::action_bus::EditorAction::SetViewFocus {
                        kind: ph2d_editor_core::ViewFocusKind::Selected,
                    });
                toasts.push(Toast::success(format!(
                    "Sheet smoke: {n} pieces packed into one object — move it, resize it, hide it"
                )));
                self.title_dirty = true;
            }
        }

        // **A FAMÍLIA `PH2D_GPU_COOK_DEMO` PRECISA DA FERRAMENTA MOTION** — ver
        // `motion_state_demo_router::demo_wants_the_motion_tool`, onde está a medição que o
        // expôs. Sem isto a cena monta, a legenda imprime, e a tela fica VAZIA.
        if ph2d_app_motion::motion_state::demo_router::demo_wants_the_motion_tool(
            motion.sinks.len(),
        ) && !std::mem::replace(&mut self.demo_tool_forced, true)
        {
            // ⚠️ **O resultado é GUARDADO e o latch só queima se a troca deu certo.** O
            // `set_active` devolve `false` quando o id não está registado, e um `let _ =` com o
            // latch já queimado seria uma falha silenciosa e DEFINITIVA na sessão. Os irmãos do
            // Flip já o guardavam (`flip_hardness_smoke.rs`); este não.
            let ok = tools.set_active(&ph2d_editor_core::ToolId::new("motion"));
            if ok {
                self.title_dirty = true;
            } else {
                self.demo_tool_forced = false;
                eprintln!(
                    "[demo] a ferramenta `motion` nao esta' registada — a cena nao vai desenhar"
                );
            }
        }

        // **A SUJIDADE NA LENTE** (`PH2D_GLOW_DIRT_SMOKE=1`, doc 89 folha 11): uma sprite com
        // uma imagem de pó e riscos, um campo de peças a brilhar, e o nó `Glow` já a ler a
        // primeira. ⚠️ Ela mora AQUI e não entre os demos de grafo porque precisa de uma
        // textura a sério — a mesma razão que já está escrita para o `PH2D_MOTION_OBJ_SMOKE=9`.
        if ph2d_app_motion::glow_dirt_smoke::enabled()
            && !std::mem::replace(&mut self.glow_dirt_smoke_done, true)
            && ph2d_app_motion::glow_dirt_smoke::spawn_if_enabled(
                sim,
                renderer,
                asset_db,
                next_import_cell,
                atlas_asset_map,
                motion,
            )
        {
            let _ = tools.set_active(&ph2d_editor_core::ToolId::new("motion"));
            self.title_dirty = true;
        }
    }
}
