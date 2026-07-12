//! Os **documentos do Painter** dentro do arquivo de projeto (irmão de [`crate::project`]).
//!
//! O que estava quebrado: o projeto salvava o mundo e os pixels dos sprites *importados*, mas nada do
//! que o Painter pintava. Um sprite pintado é `SpriteSource::Individual { texture_id }`, e esse
//! `texture_id` é um id de runtime da GPU — noutra sessão ele aponta para um slot vazio. Pintar,
//! `Ctrl+S`, reabrir: o quadro voltava em branco. **Nem o bake achatado sobrevivia**, quanto mais as
//! camadas e o relevo.
//!
//! A correção tem três peças, e a do meio é a que faltava:
//!
//! 1. **Uma identidade estável** — [`ph2d_ecs::PaintedDoc`], carimbada no sprite. Os bits de entidade
//!    (a chave do `doc_cache` do Painter) são id de ALOCAÇÃO: o restore despawna tudo e recria, então
//!    eles morrem. A componente viaja no `WorldSnapshot`, logo sobrevive ao arquivo *e* ao undo.
//! 2. **O DOCUMENTO no arquivo**, não o bake: camadas, pixels de cada uma, relevo e cobertura
//!    ([`PaintedDocument`]). Salvar só o sprite achatado devolveria uma *fotografia* de tinta grossa —
//!    sem camadas, sem espessura, sem como continuar esculpindo. É a diferença entre reabrir um
//!    trabalho e reabrir a impressão dele.
//! 3. **A textura re-materializada no load**: o documento é composto (com a luz do impasto assada, pelo
//!    caminho normal do preview) e sobe para um slot novo do `IndividualTextureStore`; o `Sprite.source`
//!    passa a apontar para ele. Sem isto o sprite carregado referenciaria a textura morta do save.

use ph2d_ecs::{Entity, PaintedDoc};
use ph2d_editor::ToolId;
use ph2d_render::{Sprite, SpriteSource};
use ph2d_tool_painter::PainterTool;
use ph2d_tool_painter::tool::persist::PaintedDocument;
use std::collections::BTreeMap;

/// O id do Painter no registry (o mesmo que o `vector_bridge` usa para o dele).
fn painter_id() -> ToolId {
    ToolId::new("painter")
}

impl crate::App {
    /// Carimba um [`PaintedDoc`] em cada sprite que o Painter tem documento, e devolve o mapa
    /// `entity bits → id estável` que a coleta usa.
    ///
    /// Feito no SAVE (e não a cada bind) de propósito: é o único momento em que a identidade precisa
    /// existir, e assim um documento que o artista abriu e fechou sem pintar não deixa componente
    /// nenhum no mundo.
    fn stamp_painted_docs(&mut self) -> BTreeMap<u64, u32> {
        let Some(gfx) = self.gfx.as_mut() else {
            return BTreeMap::new();
        };
        let Some(painter) = gfx
            .tools
            .tool_by_id_mut(&painter_id())
            .and_then(|t| t.as_any_mut().downcast_mut::<PainterTool>())
        else {
            return BTreeMap::new();
        };
        let entities = painter.document_entities();
        let mut ids = BTreeMap::new();
        for bits in entities {
            let entity = Entity::from_bits(bits);
            let world = gfx.sim.world_mut();
            if world.get_entity(entity).is_err() {
                continue; // o sprite já não existe (deletado); o documento morre com ele
            }
            let id = match world.get::<PaintedDoc>(entity) {
                Some(d) => d.0,
                None => {
                    let id = gfx.next_painted_doc;
                    gfx.next_painted_doc = gfx.next_painted_doc.saturating_add(1);
                    gfx.sim
                        .world_mut()
                        .entity_mut(entity)
                        .insert(PaintedDoc(id));
                    id
                }
            };
            ids.insert(bits, id);
        }
        ids
    }

    /// Os documentos do Painter para embutir no arquivo. Vazio quando nada foi pintado.
    pub(crate) fn collect_painted_docs(&mut self) -> Vec<PaintedDocument> {
        let ids = self.stamp_painted_docs();
        if ids.is_empty() {
            return Vec::new();
        }
        let Some(gfx) = self.gfx.as_mut() else {
            return Vec::new();
        };
        gfx.tools
            .tool_by_id_mut(&painter_id())
            .and_then(|t| t.as_any_mut().downcast_mut::<PainterTool>())
            .map(|p| p.collect_documents(&ids))
            .unwrap_or_default()
    }

    /// Devolve cada documento ao seu sprite e re-materializa a textura que o sprite mostra.
    ///
    /// Roda DEPOIS de `apply_project` (o mundo já foi restaurado, com bits novos): varre os sprites
    /// procurando o [`PaintedDoc`] e reata cada um ao documento de mesmo id. É o mesmo padrão do
    /// `cooked_texture_bridge` — o snapshot referencia simbolicamente, um bridge materializa.
    pub(crate) fn restore_painted_docs(&mut self, docs: Vec<PaintedDocument>) {
        if docs.is_empty() {
            return;
        }
        let Some(gfx) = self.gfx.as_mut() else {
            return;
        };
        // Ids futuros nesta sessão não podem colidir com os do projeto (o contrato do `next_import_cell`).
        if let Some(max) = docs.iter().map(|d| d.id).max() {
            gfx.next_painted_doc = gfx.next_painted_doc.max(max.saturating_add(1));
        }
        // 1. Que sprite (bits NOVOS) tem que documento (id estável)?
        let mut targets: Vec<(u64, u32)> = Vec::new();
        {
            let world = gfx.sim.world_mut();
            let mut q = world.query::<(Entity, &PaintedDoc)>();
            for (e, d) in q.iter(world) {
                targets.push((e.to_bits(), d.0));
            }
        }
        if targets.is_empty() {
            return;
        }
        let by_id: BTreeMap<u32, PaintedDocument> = docs.into_iter().map(|d| (d.id, d)).collect();

        // 2. Instala no Painter e compõe os pixels que o sprite deve mostrar. O composite sai do
        //    caminho NORMAL do preview (que já assa a luz do impasto) — não há um segundo bake aqui.
        let mut baked: Vec<(u64, Vec<u8>, u32, u32)> = Vec::new();
        if let Some(painter) = gfx
            .tools
            .tool_by_id_mut(&painter_id())
            .and_then(|t| t.as_any_mut().downcast_mut::<PainterTool>())
        {
            for (bits, id) in &targets {
                let Some(doc) = by_id.get(id).cloned() else {
                    continue; // sprite carimbado sem documento no arquivo — nada a devolver
                };
                painter.install_document(*bits, doc);
                if let Some((px, w, h)) = painter.baked_document_pixels(*bits) {
                    baked.push((*bits, px, w, h));
                }
            }
        }

        // 3. Sobe cada composite para uma textura individual NOVA e re-aponta o sprite: o `texture_id`
        //    do save morreu com o processo que o criou.
        for (bits, mut px, w, h) in baked {
            // O sprite amostra PREMULTIPLICADO — o mesmo passo que o bridge do preview e o Apply dão.
            ph2d_render::premultiply_rgba8(&mut px);
            let Ok(texture_id) = gfx.renderer.acquire_individual(w, h, &px) else {
                eprintln!("[proj] textura do documento pintado (entity {bits}) falhou");
                continue;
            };
            let entity = Entity::from_bits(bits);
            if let Some(mut sprite) = gfx.sim.world_mut().get_mut::<Sprite>(entity) {
                sprite.source = SpriteSource::Individual { texture_id };
                sprite.size = [w as f32, h as f32];
                sprite.premultiplied = true;
            }
        }
    }
}
