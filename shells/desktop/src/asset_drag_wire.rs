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
        let Some(id) = hero.hit_index.hit(x, y) else {
            return;
        };
        if !hero.store.is_asset_cell(id) {
            return;
        }
        // O índice do cartão dentro da grade — a mesma tabela que o pintou.
        let Some(index) = (0..ph2d_editor::ids::MAX_ASSET_CELLS)
            .find(|i| ph2d_editor::ids::asset_cell_id(*i) == id)
        else {
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
        // ⚠️ **A pergunta «estou sobre chrome?» é a MESMA que o clique faz** (`panel_at`), e não
        // uma segunda: um alvo que discordasse do clique aceitaria quedas em cima de um painel.
        // ⭐ **Voltar ao painel de onde saiu é DESISTIR**, e desistir é calado.
        let panel = hero.store.panel_at(x, y);
        let target = if panel == Some(ph2d_editor::ids::ASSET_PANEL) {
            DropTarget::Source
        } else if panel.is_some() {
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
        }
    }
}
