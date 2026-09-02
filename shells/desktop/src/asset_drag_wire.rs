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
use ph2d_editor::interaction::drag_payload::{DragPayload, DragVerdict};

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
        let verdict = match self.drop_target_at(x, y) {
            // ⚠️ **Desistir não é recusa.** Voltar ao painel de origem é o gesto universal de
            // largar o assunto, e ele é silencioso em todo o software que o tem — pintá-lo de
            // aviso acusaria o artista de um erro que ele não cometeu.
            Some(t) => match crate::asset_drop::resolve(payload, t) {
                crate::asset_drop::DropAction::Refuse => DragVerdict::Refuse,
                crate::asset_drop::DropAction::Cancel => DragVerdict::Unknown,
                _ => DragVerdict::Accept,
            },
            None => DragVerdict::Unknown,
        };
        if let Some(hero) = self.gfx.as_mut().and_then(|g| g.hero_screen.as_mut()) {
            hero.store.set_asset_drag_verdict(verdict);
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
        let Some(target) = target else {
            return true;
        };
        let action = crate::asset_drop::resolve(drag.payload, target);
        self.perform_drop(action, drag.payload);
        true
    }

    /// ⭐⭐⭐ **O que está debaixo do cursor, na linguagem da lei da queda** — a porta ÚNICA de
    /// *«o que aconteceria se a mão largasse aqui?»* (waves B2/B3/B4).
    ///
    /// ⚠️ **Ela tem DOIS chamadores e isso é o ponto:** o `Move` para o fantasma dizer se o sítio
    /// aceita, e o `Up` para agir. Escrever a resolução duas vezes seria o fantasma a prometer uma
    /// coisa e a queda a fazer outra — e a discordância só se veria no dia em que uma das duas
    /// fosse corrigida.
    ///
    /// ⚠️ **A ORDEM é a lei**: um alvo que sabe receber pergunta-se ANTES dos que só desistem ou
    /// recusam. A ranhura da textura e a linha de catálogo ganham ao *«de volta ao painel»* e ao
    /// chrome — sem isso, largar numa gaveta seria indistinguível de não fazer nada.
    fn drop_target_at(&mut self, x: f32, y: f32) -> Option<DropTarget> {
        // ⚠️ Resolvidos ANTES de pegar o `gfx` mutável — são leituras, e adiá-las para dentro do
        // bloco mutável seria pedir dois empréstimos do mesmo `self`.
        let inspector_slot = self.inspector_slot_under(x, y);
        let catalog_row = self.catalog_row_under(x, y);
        let (panel, on_canvas, world) = {
            let gfx = self.gfx.as_ref()?;
            let hero = gfx.hero_screen.as_ref()?;
            let panel = hero.store.panel_at(x, y);
            // ⛔⛔ **A pergunta «estou sobre a TELA?» tem DUAS metades, e a 1.ª versão só fazia
            // uma.** O `panel_at` só conhece os 28 painéis que publicam rect; o **rail de
            // ferramentas**, a **barra de cima**, o HUD e os menus registam só rect de acerto. ⇒
            // largar sobre a barra de cima caía no ramo do canvas e **re-texturava a sprite
            // escondida por trás dela**, em silêncio.
            let on_canvas = panel.is_none() && hero.hit_index.hit(x, y).is_none();
            let world = gfx.camera.screen_to_world((x, y), gfx.surface.size());
            (panel, on_canvas, world)
        };
        Some(if let Some(entity_bits) = inspector_slot {
            DropTarget::InspectorTexture { entity_bits }
        } else if let Some(catalog) = catalog_row {
            DropTarget::CatalogRow { catalog }
        } else if panel == Some(ph2d_editor::ids::ASSET_PANEL) {
            DropTarget::Source
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
        let id = hero.hit_index.hit(x, y)?;
        match ph2d_panel_asset_browser::catalog_row_pick(id)? {
            ph2d_panel_asset_browser::CatalogPick::One(c) => Some(Some(c.0)),
            ph2d_panel_asset_browser::CatalogPick::Unassigned => Some(None),
            // ⛔ A linha **All** não é uma gaveta — largar nela não significa nada, e tratá-la como
            // *«tira do catálogo»* seria inventar um gesto a partir de um alvo que só filtra.
            ph2d_panel_asset_browser::CatalogPick::All => None,
        }
    }

    /// ⭐⭐ **A sprite cuja RANHURA DE TEXTURA está debaixo do cursor** (wave B3).
    ///
    /// ⚠️ **Duas perguntas, e cada uma ao seu dono:** o `HitIndex` responde *«que widget está
    /// aqui»* — a porta única, e é dela que vêm de graça o recorte do corpo do painel e a oclusão
    /// —, e o **Inspector** responde *«aquele id é a minha ranhura, e de que sprite»*. A shell
    /// conhecer o literal do id seria conhecer a tabela de outro painel.
    fn inspector_slot_under(&self, x: f32, y: f32) -> Option<u64> {
        let hero = self.gfx.as_ref()?.hero_screen.as_ref()?;
        ph2d_panel_inspector::texture_slot_pick(hero.hit_index.hit(x, y)?)
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
