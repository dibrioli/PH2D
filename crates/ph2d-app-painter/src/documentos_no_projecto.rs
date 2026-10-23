//! **Os documentos do Painter no ficheiro de projecto** — a metade sem placa da porta de gravar e
//! abrir (a shell chama-as de `project_painter.rs`; a textura que o sprite mostra é dela).
//!
//! A identidade estável é o [`PaintedDoc`] carimbado no sprite: os bits de entidade morrem com o
//! processo, e o load reata cada documento ao sprite de mesmo id.

use ph2d_ecs::{Entity, PaintedDoc, SimWorld};
use ph2d_tool_painter::PainterTool;
use ph2d_tool_painter::tool::persist::PaintedDocument;
use std::collections::BTreeMap;

/// **Gravar**: carimba um [`PaintedDoc`] em cada sprite de que o Painter tem documento (ids novos a
/// partir de `proximo`) e devolve os documentos. Feito só no save: um documento aberto e fechado sem
/// pintar não deixa componente nenhum no mundo.
pub fn carimba_e_colhe(
    sim: &mut SimWorld,
    painter: &mut PainterTool,
    proximo: &mut u32,
) -> Vec<PaintedDocument> {
    let mut ids = BTreeMap::new();
    for bits in painter.document_entities() {
        let entity = Entity::from_bits(bits);
        let world = sim.world_mut();
        if world.get_entity(entity).is_err() {
            continue; // o sprite já não existe (deletado); o documento morre com ele
        }
        let id = if let Some(d) = world.get::<PaintedDoc>(entity) {
            d.0
        } else {
            let id = *proximo;
            *proximo = proximo.saturating_add(1);
            world.entity_mut(entity).insert(PaintedDoc(id));
            id
        };
        ids.insert(bits, id);
    }
    if ids.is_empty() {
        return Vec::new();
    }
    painter.collect_documents(&ids)
}

/// **Abrir**: depois do mundo restaurado (bits novos), devolve cada documento ao sprite com o mesmo
/// [`PaintedDoc`] e diz os píxeis compostos que cada um deve mostrar `(bits, rgba, w, h)`. `proximo`
/// passa os ids do ficheiro (os da sessão não colidem com eles).
pub fn devolve(
    sim: &mut SimWorld,
    painter: &mut PainterTool,
    proximo: &mut u32,
    docs: Vec<PaintedDocument>,
) -> Vec<(u64, Vec<u8>, u32, u32)> {
    if let Some(max) = docs.iter().map(|d| d.id).max() {
        *proximo = (*proximo).max(max.saturating_add(1));
    }
    let world = sim.world_mut();
    let targets: Vec<(u64, u32)> = world
        .query::<(Entity, &PaintedDoc)>()
        .iter(world)
        .map(|(e, d)| (e.to_bits(), d.0))
        .collect();
    let by_id: BTreeMap<u32, PaintedDocument> = docs.into_iter().map(|d| (d.id, d)).collect();
    let mut baked = Vec::new();
    for (bits, id) in targets {
        let Some(doc) = by_id.get(&id).cloned() else {
            continue; // sprite carimbado sem documento no arquivo — nada a devolver
        };
        painter.install_document(bits, doc);
        if let Some((px, w, h)) = painter.baked_document_pixels(bits) {
            baked.push((bits, px, w, h));
        }
    }
    baked
}
