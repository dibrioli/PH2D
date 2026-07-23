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
        end_offset: f64::from(spec.end_offset) * total,
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

/// **O par que o gesto de prender exige: dois caminhos, `(motivo, guia)`.**
///
/// Porta única — quem PUBLICA se o botão aparece (o painel) e quem HONRA o clique perguntam aqui.
///
/// ⚠️ **O guia é o caminho de maior EXTENSÃO (bbox); o motivo é o menor** (Enio, 2026-07-23: *"está
/// escolhendo como path a si mesmo e não a outra curva"*). A regra anterior — *"o primário (último
/// clicado) é o motivo"* — dependia da ORDEM de seleção: escolher a curva por último a tornava o
/// motivo, e o pattern ria a forma pequena (o "a si mesmo"). A extensão espacial é **independente
/// de ordem** e casa com o caso dominante — um motivo COMPACTO repetido ao longo de uma curva
/// GRANDE — e é a bbox **LOCAL** de propósito: o motivo é tilado na forma local (a pose é ignorada,
/// W2), então a extensão que decide é a MESMA que a que o artista vê repetida. Empate desempata
/// pela ordem; o caso raro (motivo maior que o guia) fica para um botão de *swap*.
///
/// (Comprimento de ARCO seria pior: uma forma FECHADA tem perímetro que passa fácil de uma reta
/// mais longa — um quadrado de lado 40 tem perímetro 160 contra os 100 da reta.)
#[must_use]
pub(crate) fn link_candidate(
    scene: &VecScene,
    selection: &[VecPathId],
) -> Option<(VecPathId, VecPathId)> {
    if selection.len() != 2 || selection[0] == selection[1] {
        return None;
    }
    let extent = |id: VecPathId| -> f64 {
        let Some(p) = scene.path(id) else { return 0.0 };
        let (mut lo, mut hi) = ([f64::MAX; 2], [f64::MIN; 2]);
        for v in p.verts_all() {
            for k in 0..2 {
                lo[k] = lo[k].min(v.anchor[k]);
                hi[k] = hi[k].max(v.anchor[k]);
            }
        }
        if lo[0] > hi[0] {
            0.0
        } else {
            (hi[0] - lo[0]).hypot(hi[1] - lo[1])
        }
    };
    let (a, b) = (selection[0], selection[1]);
    // guia = o de MAIOR extensão; motivo = o menor.
    if extent(a) >= extent(b) {
        Some((b, a))
    } else {
        Some((a, b))
    }
}

/// O caminho selecionado que É um pattern (carrega o `VecPatternPath`) — o MOTIVO vinculado, seja
/// ele o primário ou não. É por AQUI que o painel, os sliders e as alças acham o alvo, e não pelo
/// primário: depois de prender, o primário pode ser o GUIA (o último clicado), mas quem tem os
/// controles é o motivo.
#[must_use]
pub(crate) fn linked_motif(
    sim: &SimWorld,
    map: &VecEntityMap,
    selection: &[VecPathId],
) -> Option<VecPathId> {
    selection
        .iter()
        .copied()
        .find(|&id| spec_of(sim, map, id).is_some())
}

/// O vínculo VIVO do motivo em foco (o linkado na seleção), para o painel publicar os controles.
#[must_use]
pub(crate) fn current(
    sim: &SimWorld,
    map: &VecEntityMap,
    selection: &[VecPathId],
) -> Option<VecPatternPath> {
    spec_of(sim, map, linked_motif(sim, map, selection)?)
}

/// **Solta** o motivo do caminho (o caminho FICA — soltar é remover o componente). `true` se havia
/// vínculo.
pub(crate) fn detach(sim: &mut SimWorld, map: &VecEntityMap, motif: VecPathId) -> bool {
    let Some(&bits) = map.get(&motif) else {
        return false;
    };
    let e = Entity::from_bits(bits);
    if sim.world().get::<VecPatternPath>(e).is_none() {
        return false;
    }
    if let Ok(mut em) = sim.world_mut().get_entity_mut(e) {
        em.remove::<VecPatternPath>();
    }
    true
}

/// Edita o vínculo do motivo (spacing, start, offset, lado). `true` se havia vínculo.
///
/// A porta ÚNICA de escrita — o slider do painel e a alça de canvas (W4) passam por AQUI, então não
/// podem divergir sobre o mesmo número.
pub(crate) fn edit(
    sim: &mut SimWorld,
    map: &VecEntityMap,
    motif: VecPathId,
    f: impl FnOnce(&mut VecPatternPath),
) -> bool {
    let Some(&bits) = map.get(&motif) else {
        return false;
    };
    let e = Entity::from_bits(bits);
    let Some(mut link) = sim.world().get::<VecPatternPath>(e).copied() else {
        return false;
    };
    f(&mut link);
    if let Ok(mut em) = sim.world_mut().get_entity_mut(e) {
        em.insert(link);
    }
    true
}

/// Um comando de vínculo vindo do painel — um clique, um comando.
#[derive(Clone, Copy, Debug)]
pub(crate) enum PatternPathCmd {
    /// Prender o motivo (primário) ao outro selecionado.
    Link,
    /// Soltar (o caminho fica).
    Detach,
    /// Trocar o lado.
    Flip(bool),
}

/// **Qual das DUAS alças de canvas (W4)** está sob o dedo — a de início ou a de fim do trecho.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum PatternHandle {
    /// A ficha em `start_offset` — onde a tilagem começa.
    Start,
    /// A ficha em `end_offset` — onde termina.
    End,
}

/// **As duas alças de canvas** (W4): fichas nos pontos de Start e End do trecho, no modo Select.
///
/// Espelham a alça do texto ([`crate::vec_text_ride::handle`]) — a versão de manipulação direta dos
/// sliders Start/End, na tela. Cada uma escreve o MESMO número que o slider correspondente pela
/// MESMA porta ([`edit`]), então não podem divergir. As fichas ficam SOBRE a curva (marcam as
/// posições de arco); o desvio perpendicular é o slider Offset, à parte.
pub(crate) mod handle {
    use super::{PatternHandle, VecEntityMap, VecScene, edit, linked_motif, spec_of};
    use ph2d_ecs::SimWorld;
    use ph2d_vec_scene::VecPathId;

    /// Os pontos de MUNDO das fichas de Start e End do motivo vinculado na seleção. `None` sem um
    /// pattern na seleção (ou com o guia apagado/degenerado).
    #[must_use]
    pub(crate) fn world(
        sim: &SimWorld,
        scene: &VecScene,
        map: &VecEntityMap,
        selection: &[VecPathId],
    ) -> Option<([f64; 2], [f64; 2])> {
        let spec = spec_of(sim, map, linked_motif(sim, map, selection)?)?;
        let arc = crate::vec_guide::guide_arc(sim, scene, map, spec.path)?;
        let total = arc.total();
        let at = |frac: f32| arc.frame_at(f64::from(frac).clamp(0.0, 1.0) * total).0;
        Some((at(spec.start_offset), at(spec.end_offset)))
    }

    /// **Pressão sobre uma ficha (modo Select):** se uma está sob o cursor, arma o arrasto DELA e
    /// devolve `true` (o host então PULA o picking/gizmo). O Start tem prioridade quando as duas
    /// coincidem — arrastar desempata na primeira mão.
    #[must_use]
    pub(crate) fn press(
        sim: &SimWorld,
        scene: &VecScene,
        map: &VecEntityMap,
        selection: &[VecPathId],
        world_pt: [f64; 2],
        radius: f64,
        armed: &mut Option<PatternHandle>,
    ) -> bool {
        let Some((s, e)) = world(sim, scene, map, selection) else {
            return false;
        };
        let hit = |p: [f64; 2]| (p[0] - world_pt[0]).hypot(p[1] - world_pt[1]) <= radius;
        let which = if hit(s) {
            Some(PatternHandle::Start)
        } else if hit(e) {
            Some(PatternHandle::End)
        } else {
            None
        };
        if which.is_some() {
            *armed = which;
        }
        which.is_some()
    }

    /// Arrasta a ficha armada para o cursor: projeta no caminho, converte em fração e escreve
    /// `start_offset`/`end_offset` pela porta única [`edit`] (a MESMA do slider). No-op sem arrasto.
    pub(crate) fn drag(
        sim: &mut SimWorld,
        scene: &VecScene,
        map: &VecEntityMap,
        selection: &[VecPathId],
        world_pt: [f64; 2],
        armed: Option<PatternHandle>,
    ) -> bool {
        let (Some(which), Some(motif)) = (armed, linked_motif(sim, map, selection)) else {
            return false;
        };
        let Some(spec) = spec_of(sim, map, motif) else {
            return false;
        };
        let Some(arc) = crate::vec_guide::guide_arc(sim, scene, map, spec.path) else {
            return false;
        };
        #[allow(clippy::cast_possible_truncation)]
        let frac = (arc.closest_arc(world_pt) / arc.total()) as f32;
        edit(sim, map, motif, |l| match which {
            PatternHandle::Start => l.start_offset = frac,
            PatternHandle::End => l.end_offset = frac,
        })
    }
}

#[cfg(test)]
#[path = "pattern_live_tests.rs"]
mod tests;
