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

/// **Os documentos atravessam o restauro do undo** — que respawna o mundo inteiro com bits novos. Pela
/// identidade estável ([`ph2d_ecs::StableId`]): [`Self::antes`] lê quem cada documento é, [`Self::depois`]
/// passa-o aos bits novos da mesma identidade. O documento de uma imagem que o passo tirou fica à
/// espera dela aqui (o refazer devolve-a com outros bits).
#[derive(Default)]
pub struct Reata {
    /// Identidade → os bits sob que o Painter guarda o documento.
    quem: BTreeMap<u64, u64>,
}

impl Reata {
    /// `restaura` (o restauro do mundo) entre [`Self::antes`] e [`Self::depois`], com o Painter de `tools`.
    pub fn em_volta<R>(
        &mut self,
        sim: &mut SimWorld,
        tools: &mut ph2d_editor_core::ToolRegistry,
        restaura: impl FnOnce(&mut SimWorld) -> R,
    ) -> R {
        let p = crate::painter_bridge_queries::o_painter(tools);
        if let Some(p) = p.as_deref() {
            self.antes(sim, p);
        }
        let r = restaura(sim);
        if let Some(p) = p {
            self.depois(sim, p);
        }
        r
    }

    /// Antes do restauro: a identidade de cada documento cuja imagem está viva.
    pub fn antes(&mut self, sim: &SimWorld, painter: &PainterTool) {
        let w = sim.world();
        for bits in painter.document_entities() {
            let sid = Entity::try_from_bits(bits).and_then(|e| w.get::<ph2d_ecs::StableId>(e));
            if let Some(s) = sid {
                self.quem.insert(s.0, bits);
            }
        }
    }

    /// Depois do restauro: cada documento passa aos bits que a identidade dele tem agora.
    pub fn depois(&mut self, sim: &mut SimWorld, painter: &mut PainterTool) {
        let w = sim.world_mut();
        let agora: BTreeMap<u64, u64> = w
            .query::<(Entity, &ph2d_ecs::StableId)>()
            .iter(w)
            .map(|(e, s)| (s.0, e.to_bits()))
            .collect();
        let mut de_para = BTreeMap::new();
        self.quem.retain(|sid, de| match agora.get(sid) {
            Some(&para) => {
                if para != *de {
                    de_para.insert(*de, para);
                }
                false
            }
            None => true,
        });
        painter.reata_documentos(&de_para);
    }
}
