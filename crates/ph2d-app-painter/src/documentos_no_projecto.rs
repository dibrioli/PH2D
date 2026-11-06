//! **Os documentos do Painter no ficheiro de projecto** — a metade sem placa da porta de gravar e
//! abrir (a shell chama-as de `project_painter.rs`; a textura que o sprite mostra é dela).
//!
//! A identidade estável é o [`PaintedDoc`] carimbado no sprite: os bits de entidade morrem com o
//! processo, e o load reata cada documento ao sprite de mesmo id.

use ph2d_asset::AssetId;
use ph2d_ecs::{Entity, PaintedDoc, SimWorld, SpritePixels, StableId};
use ph2d_render::{Sprite, texture_of};
use ph2d_tool_painter::PainterTool;
use ph2d_tool_painter::tool::persist::PaintedDocument;
use std::cell::RefCell;
use std::collections::BTreeMap;

/// O documento de um objecto que recebeu píxeis por fora: fora do Painter enquanto o objecto mostra `depois`, de volta
/// assim que o Undo o devolve a `antes`.
struct Achatado {
    id: StableId,
    antes: AssetId,
    depois: AssetId,
    guardado: Option<PaintedDocument>,
}

thread_local! {
    static ACHATADOS: RefCell<Vec<Achatado>> = const { RefCell::new(Vec::new()) };
}

/// ⭐ **Píxeis vindos de fora vencem as camadas** (decisão de 05/11, ficha da Sprite): cada um de `quem` que tinha
/// documento do Painter passa de `antes` a `depois` sem ele, senão gravar e abrir re-assaria as camadas por cima. O Undo
/// devolve-as ([`Reata::depois`]). `true` = alguma camada saiu: quem chama avisa.
pub fn achata(
    sim: &mut SimWorld,
    painter: &mut PainterTool,
    quem: &[Entity],
    antes: AssetId,
    depois: AssetId,
) -> bool {
    let mut algum = false;
    for e in quem {
        let Some(guardado) = painter.tira_documento(e.to_bits()) else {
            continue;
        };
        algum = true;
        let world = sim.world_mut();
        let id = world.get::<StableId>(*e).copied();
        world.entity_mut(*e).remove::<PaintedDoc>();
        if let Some(id) = id {
            ACHATADOS.with(|t| {
                t.borrow_mut().push(Achatado {
                    id,
                    antes,
                    depois,
                    guardado: Some(guardado),
                });
            });
        }
    }
    algum
}

/// Um projecto novo: o que estava à espera do Undo era do que saiu.
pub fn esquece_achatados() {
    ACHATADOS.with(|t| t.borrow_mut().clear());
}

/// Depois de um restauro: o objecto que voltou a `antes` retoma as camadas; o que voltou a `depois` larga-as outra vez.
fn reconcilia_achatados(
    sim: &mut SimWorld,
    painter: &mut PainterTool,
    atlas: &BTreeMap<u32, AssetId>,
) {
    let world = sim.world_mut();
    let quem: BTreeMap<StableId, Entity> = world
        .query::<(Entity, &StableId)>()
        .iter(world)
        .map(|(e, s)| (*s, e))
        .collect();
    ACHATADOS.with(|t| {
        for a in t.borrow_mut().iter_mut() {
            let Some(&e) = quem.get(&a.id) else {
                continue;
            };
            let mostra = texture_of(world.get::<SpritePixels>(e), world.get::<Sprite>(e), atlas);
            if mostra == Some(a.antes) {
                if let Some(doc) = a.guardado.take() {
                    painter.install_document(e.to_bits(), doc);
                }
            } else if mostra == Some(a.depois) && a.guardado.is_none() {
                a.guardado = painter.tira_documento(e.to_bits());
            }
        }
    });
}

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
/// identidade estável ([`ph2d_ecs::ReataPorIdentidade`]): [`Self::antes`] lê quem cada documento é,
/// [`Self::depois`] passa-o aos bits novos da mesma identidade. O documento de uma imagem que o passo
/// tirou fica à espera dela aqui (o refazer devolve-a com outros bits).
#[derive(Default)]
pub struct Reata(ph2d_ecs::ReataPorIdentidade);

impl Reata {
    /// `restaura` (o restauro do mundo) entre [`Self::antes`] e [`Self::depois`], com o Painter de `tools`.
    pub fn em_volta<R>(
        &mut self,
        sim: &mut SimWorld,
        tools: &mut ph2d_editor_core::ToolRegistry,
        atlas: &BTreeMap<u32, AssetId>,
        restaura: impl FnOnce(&mut SimWorld) -> R,
    ) -> R {
        let p = crate::painter_bridge_queries::o_painter(tools);
        if let Some(p) = p.as_deref() {
            self.antes(sim, p);
        }
        let r = restaura(sim);
        if let Some(p) = p {
            self.depois(sim, p);
            reconcilia_achatados(sim, p, atlas);
        }
        r
    }

    /// Antes do restauro: a identidade de cada documento cuja imagem está viva.
    pub fn antes(&mut self, sim: &SimWorld, painter: &PainterTool) {
        self.0.antes(sim.world(), painter.document_entities());
    }

    /// Depois do restauro: cada documento passa aos bits que a identidade dele tem agora.
    pub fn depois(&mut self, sim: &SimWorld, painter: &mut PainterTool) {
        painter.reata_documentos(&self.0.depois(sim.world()));
    }
}

#[cfg(test)]
#[path = "documentos_no_projecto_tests.rs"]
mod tests;
