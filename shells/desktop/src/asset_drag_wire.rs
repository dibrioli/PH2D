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
//! ⚠️ **A lei da queda não está aqui** — ela é pura e vive no [`crate::asset_drop`]. Este ficheiro
//! é só a fiação: ele recolhe os factos (onde o cursor está, o que está por baixo) e executa o que
//! a lei devolver. *Uma decisão dentro da fiação seria uma decisão sem gate.*

use crate::asset_drop::{DropAction, DropOver, DropTarget};
use ph2d_editor::interaction::drag_payload::DragPayload;

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
        let Some(id) = hero.hit_index.hit(x, y) else {
            return;
        };
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
        hero.store.begin_asset_drag(payload, x, y);
    }

    /// **`Move`** — anda com o arrasto (e arma-o depois do limiar).
    pub(crate) fn asset_drag_move(&mut self, x: f32, y: f32) {
        if let Some(gfx) = self.gfx.as_mut()
            && let Some(hero) = gfx.hero_screen.as_mut()
        {
            hero.store.update_asset_drag(x, y);
        }
    }

    /// **`Up`** — resolve a queda. Devolve `true` se o gesto foi um ARRASTO (e portanto o clique
    /// tem de ser suprimido).
    ///
    /// ⚠️ **`false` para um gesto que não passou o limiar** — ele foi um clique, e o cartão tem de
    /// o receber (escolher; e o duplo-clique, instanciar).
    pub(crate) fn asset_drag_up(&mut self, x: f32, y: f32) -> bool {
        // ⚠️ Resolvido ANTES de pegar o `gfx` mutável — o alvo é uma leitura, e adiá-la para
        // dentro do bloco mutável seria pedir dois empréstimos do mesmo `self`.
        let catalog_row = self.catalog_row_under(x, y);
        let Some(gfx) = self.gfx.as_mut() else {
            return false;
        };
        let Some(hero) = gfx.hero_screen.as_mut() else {
            return false;
        };
        let Some(drag) = hero.store.end_asset_drag() else {
            return false;
        };
        if !drag.armed {
            return false;
        }
        // ⭐ **Voltar ao painel de onde saiu é DESISTIR**, e desistir é calado.
        let panel = hero.store.panel_at(x, y);
        // ⛔⛔ **A pergunta «estou sobre a TELA?» tem DUAS metades, e a 1.ª versão só fazia uma.**
        // O doc que aqui estava dizia *«é a MESMA que o clique faz»* e citava `panel_at` — mas o
        // clique pergunta `panel_at(..).is_none() && hit_index.hit(..).is_none()`. O `panel_at` só
        // conhece os 28 painéis que publicam rect; o **rail de ferramentas**, a **barra de cima**,
        // o HUD e os menus registam só rect de acerto. ⇒ largar sobre a barra de cima caía no ramo
        // do canvas e **re-texturava a sprite escondida por trás dela**, em silêncio.
        let on_canvas = panel.is_none() && hero.hit_index.hit(x, y).is_none();
        // ⭐⭐ **Uma LINHA DE CATÁLOGO ganha ao «de volta ao painel»** (wave A3), e a ordem é a
        // lei: sem ela, largar numa gaveta caía no `Source` — *desistir* — e o gesto que o plano
        // nomeia seria silenciosamente indistinguível de não fazer nada.
        let target = if let Some(catalog) = catalog_row {
            DropTarget::CatalogRow { catalog }
        } else if panel == Some(ph2d_editor::ids::ASSET_PANEL) {
            DropTarget::Source
        } else if !on_canvas {
            DropTarget::Chrome
        } else {
            let world = gfx.camera.screen_to_world((x, y), gfx.surface.size());
            let over = self.pick_drop_target(x, y);
            DropTarget::Canvas { world, over }
        };
        let action = crate::asset_drop::resolve(drag.payload, target);
        self.perform_drop(action, drag.payload);
        true
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
        let id = hero.hit_index.hit(x, y)?;
        match ph2d_panel_asset_browser::catalog_row_pick(id)? {
            ph2d_panel_asset_browser::CatalogPick::One(c) => Some(Some(c.0)),
            ph2d_panel_asset_browser::CatalogPick::Unassigned => Some(None),
            // ⛔ A linha **All** não é uma gaveta — largar nela não significa nada, e tratá-la como
            // *«tira do catálogo»* seria inventar um gesto a partir de um alvo que só filtra.
            ph2d_panel_asset_browser::CatalogPick::All => None,
        }
    }

    /// O objecto debaixo do cursor, no resumo de que a lei precisa.
    ///
    /// ⚠️ Ela chega **resolvida** à lei: um segundo pick, com outras entradas, é a segunda resposta
    /// que este repo já pagou noutros sítios.
    fn pick_drop_target(&mut self, x: f32, y: f32) -> Option<DropOver> {
        // ⚠️ **A porta ÚNICA de pick** — a mesma que o realce de proveniência e o clique usam.
        let bits = self.pick_hovered_object((x, y))?;
        let gfx = self.gfx.as_ref()?;
        let e = ph2d_ecs::Entity::from_bits(bits);
        // *«Isto mostra pixels?»* — a pergunta que decide se ele pode receber uma imagem.
        let is_sprite = gfx.sim.world().get::<ph2d_render::Sprite>(e).is_some();
        Some(DropOver {
            entity_bits: bits,
            is_sprite,
        })
    }

    /// Executa o que a lei devolveu. ⛔ Sem `_` — uma acção nova tem de vir aqui escolher o que faz.
    fn perform_drop(&mut self, action: DropAction, payload: DragPayload) {
        match action {
            // ⭐ Desistir não diz nada — ver a lei.
            DropAction::Cancel => {}
            DropAction::Refuse => {
                // ⛔ **A recusa VÊ-SE** — largar num sítio que não sabe receber nunca é silêncio.
                if let Some(gfx) = self.gfx.as_mut() {
                    gfx.toasts
                        .push(ph2d_editor::Toast::warning(DropAction::refusal_line(
                            payload,
                        )));
                }
            }
            DropAction::PlacePrefab { stable_id, world } => {
                self.drop_place_prefab(stable_id, world);
            }
            DropAction::RetextureSprite { entity_bits, asset } => {
                self.drop_retexture(entity_bits, asset);
            }
            DropAction::SpawnImage { asset, world } => {
                self.drop_spawn_image(asset, world);
            }
            DropAction::Filecatalog { payload, catalog } => {
                // ⚠️ Pelo BARRAMENTO, como todo verbo de catálogo: quem muta a taxonomia é o dreno,
                // num sítio só. Mutá-la aqui seria a segunda porta.
                if let Some(gfx) = self.gfx.as_mut()
                    && let Some(hero) = gfx.hero_screen.as_mut()
                {
                    hero.bus
                        .push(ph2d_editor::action_bus::EditorAction::AssetCatalogVerb(
                            ph2d_editor::action_bus::CatalogVerb::Assign {
                                asset: payload,
                                catalog,
                            },
                        ));
                }
            }
        }
    }
}
