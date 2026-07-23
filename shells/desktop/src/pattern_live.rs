//! **O Pattern Along Path VIVO na shell** — o cozimento do [`ph2d_ecs::VecPatternPath`] (plano 23).
//!
//! Espelho do [`crate::offset_live`]: o componente guarda a **relação** (a que caminho, com que
//! espaçamento, de que lado) e as cópias são uma **função pura** dela, re-cozidas aqui por frame
//! numa [`LiveGeometry`] e desenhadas por [`ph2d_vec_render::dispatch`] **no z do motivo**. A curva
//! do motivo nunca é tocada — é ela que o modo Node edita; o que se vê são as cópias.
//!
//! # Duas entradas, e onde cada uma vem
//!
//! Um pattern precisa de DUAS geometrias: o **motivo** (a forma a estampar) e o **caminho-guia**.
//!
//! - O **guia** entra pela porta única [`crate::vec_guide::guide_arc`] — cozido, assado em MUNDO,
//!   virado em [`ph2d_vec_scene::arc_path::ArcPath`]. É a MESMA porta do texto em caminho.
//! - O **motivo** é a `cooked()` do próprio path, em espaço **LOCAL** (a forma, não a pose). O
//!   motor a recentra pelo bbox e a estampa nos pontos do guia. ⚠️ **A pose do motivo é ignorada
//!   de propósito** (v1, plano 23 §3): as cópias substituem o desenho do motivo, então mexer no
//!   gizmo dele não teria o que mostrar — é o mesmo *"mover o objeto vinculado não quer dizer
//!   nada"* do texto e do conector. Para mudar o tamanho das cópias, edite os nós do motivo (a
//!   fonte) ou o Spacing; escalar por gizmo é decisão de produto adiada.
//!
//! # Sem memo (v1), e é MEDIDO
//!
//! O [`crate::offset_live`] memoiza porque `offset_path` mede 0,4–1 ms. O [`pattern_along`] mede
//! **0,597 ms para 200 cópias** (plano 23 §0), e uma cena típica tem muito menos. Uma cena PARADA
//! com um pattern pagaria isso por frame — dentro do orçamento (3,6% de um frame de 60 fps), e não
//! medido como problema. Memoizar exige detectar a mudança do guia (cozido + assado), e adicioná-lo
//! sem uma medição que o justifique é otimização prematura ([[project_m5_perf_validated]]).

use ph2d_ecs::{Entity, SimWorld, VecPatternPath};
use ph2d_vec_render::LiveGeometry;
use ph2d_vec_scene::pattern_path::{PatternSpec, pattern_along};
use ph2d_vec_scene::{VecPathId, VecScene};

use crate::vec_entities::VecEntityMap;

/// O cozimento vivo de todos os patterns da cena.
#[derive(Default)]
pub(crate) struct PatternLive {
    live: LiveGeometry,
}

impl PatternLive {
    /// A geometria derivada deste frame — o que o [`ph2d_vec_render::dispatch`] desenha no lugar do
    /// motivo. Vazia = nenhum pattern vivo, e o desenho é o de sempre.
    pub(crate) fn live(&self) -> &LiveGeometry {
        &self.live
    }

    /// Re-coze todos os patterns. Chamado uma vez por frame, DEPOIS do `sync` (senão um motivo/guia
    /// recém-criado ainda não teria entidade e o componente não seria encontrado).
    pub(crate) fn recook(&mut self, scene: &VecScene, sim: &SimWorld, map: &VecEntityMap) {
        self.live.clear();
        for path in scene.paths() {
            let Some(spec) = spec_of(sim, map, path.id) else {
                continue;
            };
            let Some(arc) = crate::vec_guide::guide_arc(sim, scene, map, spec.path) else {
                continue; // guia apagado / degenerado → o motivo volta a ser desenhado (fonte)
            };
            let motif = path.cooked();
            let copies = pattern_along(&motif, &arc, &spec_to_motor(&spec, arc.total()));
            // ⚠️ Só insere se HÁ cópia. Vazio (o guia é curto demais para caber uma) fica AUSENTE
            // de propósito — e ausente faz o `dispatch` desenhar a FONTE (o motivo). Mostrar o
            // motivo *"não coube"* é mais honesto que sumir com ele, ao contrário do offset (onde
            // vazio É a aniquilação e desenhar nada é o certo).
            if !copies.is_empty() {
                self.live.insert(path.id, copies);
            }
        }
    }

    /// Esquece tudo — o load de projeto e o restore de undo trocam a cena inteira, e os
    /// `VecPathId` são reciclados entre documentos.
    pub(crate) fn forget(&mut self) {
        self.live.clear();
    }
}

/// O `PatternSpec` do motor a partir do componente: a fração vira comprimento AQUI (a mesma porta,
/// pela mesma razão do texto — um número que metade lê como fração e a outra como distância é o bug
/// que não dá erro em lado nenhum).
fn spec_to_motor(spec: &VecPatternPath, total: f64) -> PatternSpec {
    PatternSpec {
        start_offset: f64::from(spec.start_offset) * total,
        spacing: f64::from(spec.spacing),
        offset: f64::from(spec.offset),
        flip: spec.flip,
    }
}

/// O pattern vivo de `id`, se houver. Porta única: o cozimento, o painel e o `Apply` perguntam AQUI.
#[must_use]
pub(crate) fn spec_of(sim: &SimWorld, map: &VecEntityMap, id: VecPathId) -> Option<VecPatternPath> {
    let &bits = map.get(&id)?;
    sim.world()
        .get::<VecPatternPath>(Entity::from_bits(bits))
        .copied()
}

/// **Prende** o motivo ao caminho-guia (ids EXPLÍCITOS). `true` se prendeu.
///
/// Recusa prender uma forma a ela mesma (um motivo que cavalga a si próprio não quer dizer nada) e
/// exige que o motivo exista no mapa. A disambiguação *"qual dos selecionados é o motivo, qual é o
/// guia?"* é decisão de UX do painel (W3) — esta porta recebe os dois já resolvidos.
pub(crate) fn link(
    sim: &mut SimWorld,
    map: &VecEntityMap,
    motif: VecPathId,
    guide: VecPathId,
) -> bool {
    if motif == guide {
        return false;
    }
    let Some(&bits) = map.get(&motif) else {
        return false;
    };
    let Ok(mut e) = sim.world_mut().get_entity_mut(Entity::from_bits(bits)) else {
        return false;
    };
    e.insert(VecPatternPath {
        path: guide,
        ..VecPatternPath::default()
    });
    true
}

// As portas de ESCRITA — `detach` (soltar) e `edit` (mudar spacing/offset/lado) — nascem na W3, com
// o painel que as consome (o slider e a alça passarão por elas, e a regra das duas-portas exige que
// concordem). Adicioná-las agora seria código sem chamador vivo (YAGNI); o gate abaixo já prova a
// BEHAVIOR de soltar (componente ausente ⇒ sem cópias), que é o invariante que importa.

#[cfg(test)]
#[path = "pattern_live_tests.rs"]
mod tests;
