//! ⭐⭐⭐ **O ARRASTO DA BIBLIOTECA, ligado ao ponteiro** (plano `docs/Components/07`, etapa B).
//!
//! # Por que ele vive no SHELL e não no editor-core
//!
//! Porque **o alvo é a tela**. Todo arrasto do `ph2d-editor-core` resolve o alvo dentro do painel
//! que o começou — o `dispatch_up` fecha-os todos em sequência, cada um com o seu resolvedor. Este
//! começa no navegador de assets e acaba no canvas, e para saber *o que está debaixo do cursor*
//! precisa da **câmara** e do **pick**, que só existem aqui.
//!
//! ⇒ o `WidgetStore` guarda o estado (para o fantasma o poder pintar) e o shell conduz as três
//! metades: `Down` semeia, `Move` anda, `Up` resolve.
//!
//! ⚠️ **A lei da queda não está aqui** — ela é pura e vive no [`ph2d_app_components::asset_drop`]. Este ficheiro
//! é só a fiação: ele recolhe os factos (onde o cursor está, o que está por baixo) e executa o que
//! a lei devolver. *Uma decisão dentro da fiação seria uma decisão sem gate.*

use ph2d_app_components::asset_drop::{self, DropAction, DropOver, DropTarget};
use ph2d_app_components::object_drag;
use ph2d_editor_core::interaction::drag_payload::{DragPayload, DragVerdict};

impl crate::App {
    /// **`Down`** — se o botão caiu num cartão do navegador, semeia o arrasto.
    ///
    /// ⛔ Ele **não** consome o gesto: enquanto o limiar não for passado isto ainda é um clique, e
    /// o clique do cartão continua a chegar ao painel como sempre.
    pub(crate) fn asset_drag_down(&mut self, x: f32, y: f32) {
        let Some(gfx) = self.gfx.as_mut() else {
            return;
        };
        let Some(hero) = gfx.hero_screen.as_mut() else {
            return;
        };
        // ⛔ **Um `Down` novo apaga um arrasto preso.** Se o `Up` nunca chegou (alt-tab com o botão
        // em baixo, um grab do sistema), o arrasto ficava armado para sempre — o fantasma seguia o
        // cursor e o **clique seguinte, em qualquer sítio, executava a queda**. A casa já tinha o
        // precedente uma linha ao lado: o `Focused(false)` existe porque *«o `Up` de uma tecla
        // presa nunca chega quando a janela vai embora»*, e o mesmo é verdade de um botão.
        hero.store.end_asset_drag();
        if ph2d_app_components::property_drag::seed(hero, &gfx.sim, &gfx.component_registry, (x, y))
        {
            return;
        }
        let Some(id) = hero.chrome_hit(x, y) else {
            return;
        };
        if hero.store.is_hierarchy_row(id) {
            let row = gfx.hero_live.as_ref().and_then(|l| l.bridge.entity_for(id));
            return object_drag::seed(hero, gfx.sim.world(), row, (x, y));
        }
        if !hero.store.is_asset_cell(id) {
            return;
        }
        // ⚠️ **O índice vem do MAPA que o painel publicou**, e não de re-derivar 512 hashes por
        // cada clique do rato em qualquer sítio do app (o que a 1.ª versão fazia). Quem sabe o
        // índice é quem o pintou.
        let Some(index) = hero.store.asset_cell_index(id) else {
            return;
        };
        let Some(payload) = ph2d_panel_asset_browser::payload_at(index) else {
            return;
        };
        // ⚠️ O fantasma pinta-se no chrome (LÓGICO, `ui_scale`); a queda pergunta pela porta física.
        let (gx, gy) = hero.escala().to_logical((x, y));
        hero.store.begin_asset_drag(payload, gx, gy);
    }

    /// **`Move`** — anda com o arrasto (e arma-o depois do limiar).
    pub(crate) fn asset_drag_move(&mut self, x: f32, y: f32) {
        if let Some(gfx) = self.gfx.as_mut()
            && let Some(hero) = gfx.hero_screen.as_mut()
        {
            let (gx, gy) = hero.escala().to_logical((x, y));
            hero.store.update_asset_drag(gx, gy);
        }
        // ⭐⭐⭐ **A VOZ DO ARRASTO** (wave B4): o fantasma passa a dizer se o sítio aceita, ANTES
        // de a mão largar. ⚠️ Pela MESMA porta e pela MESMA lei que a queda usa — é isso que
        // impede o fantasma de prometer o que a queda não faz.
        let Some(payload) = self
            .gfx
            .as_ref()
            .and_then(|g| g.hero_screen.as_ref())
            .and_then(|h| h.store.asset_drag())
            .filter(|d| d.armed)
            .map(|d| d.payload)
        else {
            return;
        };
        let verdict = self
            .drop_target_at(x, y)
            .map_or(DragVerdict::Unknown, |t| asset_drop::verdict(payload, t));
        let under = self.chrome_id_under(x, y);
        if let Some(hero) = self.gfx.as_mut().and_then(|g| g.hero_screen.as_mut()) {
            hero.store.set_asset_drag_verdict(verdict);
            // GEF 20b — as caixas dos campos dizem se aceitam (o idioma do Godot).
            let ids: Vec<_> = hero
                .hit_index
                .iter_registrations()
                .map(|(id, _)| id)
                .collect();
            hero.store
                .set_asset_drop_fields(asset_drop::field_marks(ids, payload, under));
        }
    }

    /// **`Up`** — resolve a queda. Devolve `true` se o gesto foi um ARRASTO (e portanto o clique
    /// tem de ser suprimido).
    ///
    /// ⚠️ **`false` para um gesto que não passou o limiar** — ele foi um clique, e o cartão tem de
    /// o receber (escolher; e o duplo-clique, instanciar).
    pub(crate) fn asset_drag_up(&mut self, x: f32, y: f32) -> bool {
        // ⚠️ **A MESMA porta que o `Move` usa** — ver [`Self::drop_target_at`]. Duas respostas a
        // *«o que aconteceria se eu largasse aqui?»* fariam o fantasma prometer uma coisa e a
        // queda fazer outra.
        let target = self.drop_target_at(x, y);
        let mods = Self::convert_modifiers(self.modifiers);
        let Some(gfx) = self.gfx.as_mut() else {
            return false;
        };
        let Some(hero) = gfx.hero_screen.as_mut() else {
            return false;
        };
        let Some((drag, label)) = hero.store.end_named_drag() else {
            return false;
        };
        if !drag.armed {
            return false;
        }
        let Some(target) = target else {
            return true;
        };
        let action = ph2d_app_components::asset_drop::resolve(drag.payload, target);
        let libs = object_drag::Libraries {
            sim: &mut gfx.sim,
            db: &gfx.asset_db,
            atlas: &gfx.atlas_asset_map,
        };
        let carga = (drag.payload, label.as_deref());
        if object_drag::write_code(action, carga, hero, libs, (x, y), mods) {
            return true;
        }
        self.perform_drop(action, drag.payload);
        true
    }

    /// ⭐⭐⭐ **O que está debaixo do cursor, na linguagem da lei da queda** — a porta ÚNICA de
    /// *«o que aconteceria se a mão largasse aqui?»* (waves B2/B3/B4).
    ///
    /// ⚠️ **Dois chamadores, UMA resolução:** o `Move` (o fantasma e as caixas dizem se aceita) e o
    /// `Up` (age) — resolver duas vezes seria o fantasma a prometer o que a queda não faz.
    ///
    /// ⚠️ **A ORDEM é a lei**: quem sabe receber (campo, gaveta) ganha a quem desiste ou recusa; um
    /// Quadro aberto tapa a cena, então fora dos painéis o alvo é ele.
    fn drop_target_at(&mut self, x: f32, y: f32) -> Option<DropTarget> {
        // ⚠️ Leituras ANTES do `gfx` mutável (senão, dois empréstimos do mesmo `self`).
        let campo = self
            .chrome_id_under(x, y)
            .and_then(asset_drop::field_target);
        let catalog_row = self.catalog_row_under(x, y);
        let mods = Self::convert_modifiers(self.modifiers);
        let (on_canvas, world, code, home, board) = {
            let gfx = self.gfx.as_mut()?;
            let window = gfx.scene_window();
            let hero = gfx.hero_screen.as_mut()?;
            let panel = hero.chrome_panel_at(x, y);
            let home = panel == Some(ph2d_editor_core::ids::ASSET_PANEL)
                || hero.store.hierarchy_drag_in_own_panel();
            // ⛔⛔ **A pergunta «estou sobre a TELA?» tem DUAS metades, e a 1.ª versão só fazia
            // uma.** O `panel_at` só conhece os 28 painéis que publicam rect; o **rail de
            // ferramentas**, a **barra de cima**, o HUD e os menus registam só rect de acerto. ⇒
            // largar sobre a barra de cima caía no ramo do canvas e **re-texturava a sprite
            // escondida por trás dela**, em silêncio.
            let on_canvas = panel.is_none() && hero.chrome_hit(x, y).is_none();
            let board = on_canvas && hero.documents.active().is_some();
            let world = gfx.camera.screen_to_world((x, y), window);
            let libs = object_drag::Libraries {
                sim: &mut gfx.sim,
                db: &gfx.asset_db,
                atlas: &gfx.atlas_asset_map,
            };
            let code = object_drag::hover_code(hero, libs, (x, y), on_canvas, mods);
            (on_canvas, world, code, home, board)
        };
        Some(if let Some(campo) = campo {
            campo
        } else if let Some(catalog) = catalog_row {
            DropTarget::CatalogRow { catalog }
        } else if home {
            DropTarget::Source
        } else if let Some(on_text) = code {
            DropTarget::Code { on_text }
        } else if board {
            DropTarget::BoardCanvas { at: [x, y] }
        } else if !on_canvas {
            DropTarget::Chrome
        } else {
            DropTarget::Canvas {
                world,
                over: self.pick_drop_target(x, y),
            }
        })
    }

    /// ⭐ **A linha de catálogo debaixo do cursor**, se houver.
    ///
    /// ⚠️ **Pelo HIT-INDEX, que é a única prova de que ela é alcançável** — e pela porta do painel,
    /// que é quem sabe que linha o índice `i` desenhou neste quadro. Uma tabela de ids aqui seria a
    /// segunda resposta a *«que catálogo é esta linha?»*.
    ///
    /// O `Option` de fora é *«é uma linha de catálogo?»*; o de dentro é *«qual gaveta»* (`None` =
    /// a linha *Unassigned*).
    fn catalog_row_under(&self, x: f32, y: f32) -> Option<Option<u128>> {
        let hero = self.gfx.as_ref()?.hero_screen.as_ref()?;
        let id = hero.chrome_hit(x, y)?;
        match ph2d_panel_asset_browser::catalog_row_pick(id)? {
            ph2d_panel_asset_browser::CatalogPick::One(c) => Some(Some(c.0)),
            ph2d_panel_asset_browser::CatalogPick::Unassigned => Some(None),
            // ⛔ A linha **All** não é uma gaveta — largar nela não significa nada, e tratá-la como
            // *«tira do catálogo»* seria inventar um gesto a partir de um alvo que só filtra.
            ph2d_panel_asset_browser::CatalogPick::All => None,
        }
    }

    fn chrome_id_under(&self, x: f32, y: f32) -> Option<ph2d_editor_core::NodeId> {
        self.gfx.as_ref()?.hero_screen.as_ref()?.chrome_hit(x, y)
    }

    /// O objecto debaixo do cursor, no resumo de que a lei precisa.
    ///
    /// ⚠️ Ela chega **resolvida** à lei: um segundo pick, com outras entradas, é a segunda resposta
    /// que este repo já pagou noutros sítios.
    fn pick_drop_target(&mut self, x: f32, y: f32) -> Option<DropOver> {
        // ⚠️ **A porta ÚNICA de pick** — a mesma que o realce de proveniência e o clique usam.
        let bits = self.pick_hovered_object((x, y))?;
        Some(DropOver { entity_bits: bits })
    }

    /// Executa o que a lei devolveu. ⛔ Sem `_` — uma acção nova tem de vir aqui escolher o que faz.
    pub(crate) fn perform_drop(&mut self, action: DropAction, payload: DragPayload) {
        match action {
            // ⭐ Desistir não diz nada — ver a lei.
            // `WriteCode`, `PlaceKanban` e os campos de objecto e de som: `object_drag::write_code` já os fez.
            DropAction::Cancel
            | DropAction::WriteCode
            | DropAction::PlaceKanban { .. }
            | DropAction::SetScriptObject { .. }
            | DropAction::SetObjectRef { .. }
            | DropAction::SetSound { .. } => {}
            DropAction::Refuse => {
                // ⛔ **A recusa VÊ-SE** — largar num sítio que não sabe receber nunca é silêncio.
                if let Some(gfx) = self.gfx.as_mut() {
                    gfx.toasts
                        .push(ph2d_editor_core::Toast::warning(DropAction::refusal_line(
                            payload,
                        )));
                }
            }
            DropAction::PlacePrefab {
                stable_id,
                world,
                over,
            } => {
                self.drop_place_prefab(stable_id, world, over);
            }
            DropAction::RetextureSprite { entity_bits, asset } => {
                self.choose_image(entity_bits, asset);
            }
            DropAction::SetNormalMap { entity_bits, asset } => self.image_use_edit(
                entity_bits,
                ph2d_editor_core::ImageUseEdit::SetNormalMap(Some(asset)),
            ),
            DropAction::SpawnImage { asset, world } => {
                self.drop_spawn_image(asset, world);
            }
            DropAction::PlaceImage { asset, at } => {
                if let Some(gfx) = self.gfx.as_mut()
                    && let Some(hero) = gfx.hero_screen.as_mut()
                    && let Some(t) =
                        ph2d_app_board::images::drop_asset(hero, &gfx.asset_db, asset, at)
                {
                    gfx.toasts.push(t);
                }
            }
            DropAction::Filecatalog { payload, catalog } => {
                // ⚠️ Pelo BARRAMENTO, como todo verbo de catálogo: quem muta a taxonomia é o dreno,
                // num sítio só. Mutá-la aqui seria a segunda porta.
                if let Some(gfx) = self.gfx.as_mut()
                    && let Some(hero) = gfx.hero_screen.as_mut()
                {
                    hero.bus.push(
                        ph2d_editor_core::action_bus::EditorAction::AssetCatalogVerb(
                            ph2d_editor_core::action_bus::CatalogVerb::Assign {
                                asset: payload,
                                catalog,
                            },
                        ),
                    );
                }
            }
        }
    }
}
