//! **Convert to Curves** — o converter UNIFICADO: uma pergunta, dois consumidores.
//!
//! Congelar geometria VIVA em curva crua tinha respostas ESPALHADAS que se contradiziam: o
//! botão perguntava uma coisa para se oferecer (`convertible`), o conversor fazia outra, e as
//! ferramentas de quina uma terceira. Cada uma **enumerava** as fontes de "isto não é curva
//! crua" e apodrecia sozinha quando nascia a fonte seguinte — o Enio bateu nas três de uma vez
//! (o botão desligado num caminho com quina viva; *"nao funciona com quase nada"*; o Fillet
//! recusando o vértice de uma Shape). [[feedback_a_condition_that_enumerates_its_readers_rots]]
//!
//! Agora [`is_convertible`] é a porta ÚNICA — o botão a consulta para SE OFERECER, o
//! [`to_curves`] a honra ao rodar, e as ferramentas de quina usam a [`freeze_shape_recipe`] para
//! congelar a forma viva antes de escrever um raio nela. Um "sim" que o conversor não honrasse
//! seria um botão morto; um "não" sobre algo que ele congela seria a feature inalcançável.
//!
//! # As fontes de geometria viva, e o que "congelar" significa em cada uma
//!
//! - **Texto** (`VecShape::Text`) — explode num grupo por-letra (glyph-paths individuais).
//! - **Forma paramétrica** (`VecShape::Param`) — descarta a RECEITA; a geometria já é a forma.
//! - **Conector / Morph** (`VecConnector` / `VecMorph`) — a geometria deles já está em `verts`
//!   (o `*_live::recook` a escreve todo frame), então congelar é **soltar a relação**: a linha
//!   deixa de seguir as formas, a morfada para no `t` atual.
//! - **Pilha de efeitos** (ADR-0132) e **quinas vivas** (ADR-0121) — assadas no cozido pelo
//!   `bake_cooked`, que é a MESMA porta do botão "Apply" da seção Effects.
//! - **Offset vivo** (`VecOffset`) — MATERIALIZADO pela porta do `Apply Offset`
//!   ([`crate::offset_live::materialise`]). ⚠️ Não basta soltar o componente, como no
//!   conector/morph: a geometria do offset **não está no `verts`** (ela é DESENHO, cozida por
//!   frame), então soltá-lo devolveria a curva autorada — o artista clicaria "Convert to
//!   Curves" e veria a forma ENCOLHER de volta ao que era antes do offset. E o offset pode
//!   produzir VÁRIOS caminhos de um só (o donut do smoke devolve oito a `Side=Inner`), que é a
//!   outra razão de ele passar pelo `expand_selection` e não por um `bake_cooked`.
//!
//! ⚠️ **Blend e Envelope ficam de FORA, de propósito.** Os dois têm botão **Expand** próprio, e
//! o que eles fazem não é congelar um path: o blend MATERIALIZA passos que são *virtuais* (não
//! estão na cena — descartar o componente os apagaria), e o envelope DISSOLVE um container que
//! não é um path da seleção. São operações com undo e seleção próprios; chamá-las daqui seria
//! uma 2ª porta para elas.

use crate::vec_entities::VecEntityMap;
use ph2d_ecs::{Entity, SimWorld, VecConnector, VecMorph, VecShape};
use ph2d_vec_edit::{History, PenTool};
use ph2d_vec_scene::{VecPath, VecPathId, VecScene, VecXforms};

/// Um **HOST VIVO** reescreve o `verts` deste path por trás do artista (a receita da forma, a
/// rota do conector, o `t` do morph). Congelar é REMOVER o host — o `verts` já carrega o cozido.
fn has_live_host(sim: &SimWorld, map: &VecEntityMap, id: VecPathId) -> bool {
    let Some(&bits) = map.get(&id) else {
        return false;
    };
    let e = Entity::from_bits(bits);
    let w = sim.world();
    w.get::<VecShape>(e).is_some()
        || w.get::<VecConnector>(e).is_some()
        || w.get::<VecMorph>(e).is_some()
}

/// **Há algo a congelar neste path?** A porta única (ver o módulo): um host vivo na entidade, ou
/// estado vivo no próprio path (pilha de efeitos / quina com raio).
#[must_use]
pub(crate) fn is_convertible(
    sim: &SimWorld,
    map: &VecEntityMap,
    scene: &VecScene,
    id: VecPathId,
) -> bool {
    has_live_host(sim, map, id)
        || crate::offset_live::spec_of(sim, map, id).is_some()
        || scene.path(id).is_some_and(VecPath::has_live_geometry)
}

/// Solta os hosts de RELAÇÃO (conector, morph) das entidades da seleção. A forma paramétrica tem
/// porta própria (`vec_shape_live::drop_shape_params`, que pula o TEXTO — já explodido).
fn drop_relation_hosts(sim: &mut SimWorld, map: &VecEntityMap, selection: &[VecPathId]) -> usize {
    let mut n = 0;
    for id in selection {
        let Some(&bits) = map.get(id) else { continue };
        let e = Entity::from_bits(bits);
        let live = {
            let w = sim.world();
            w.get::<VecConnector>(e).is_some() || w.get::<VecMorph>(e).is_some()
        };
        if live && let Ok(mut em) = sim.world_mut().get_entity_mut(e) {
            em.remove::<VecConnector>();
            em.remove::<VecMorph>();
            n += 1;
        }
    }
    n
}

/// Converte a seleção em paths crus e devolve a nova seleção (catálogo de fontes no módulo).
///
/// A ORDEM importa: o texto explode ANTES (cria os glyph-paths que os passos seguintes veem), e
/// o bake é por ÚLTIMO — sobre a seleção final, incluindo os glyphs recém-criados (que não têm
/// efeito nem quina, então ali é um no-op barato).
pub(crate) fn to_curves(
    sim: &mut SimWorld,
    scene: &mut VecScene,
    map: &mut VecEntityMap,
    pen: &mut PenTool,
    history: &mut History,
    xforms: &VecXforms,
    selection: &[VecPathId],
) -> Vec<VecPathId> {
    let new_sel = crate::vec_text::convert_text_selection_to_curves(sim, scene, map, selection);
    crate::vec_shape_live::drop_shape_params(sim, map, &new_sel);
    drop_relation_hosts(sim, map, &new_sel);
    // O Offset vivo materializa ANTES do bake: ele CONSOME o caminho (remove+insere), então
    // assar o cozido de um path que vai deixar de existir seria trabalho jogado fora — e a
    // seleção que sai daqui tem de ser a que o `expand_selection` produziu.
    let new_sel =
        if crate::offset_live::materialise(scene, sim, pen, history, map, xforms, &new_sel) {
            pen.selected_paths().to_vec()
        } else {
            new_sel
        };
    for id in &new_sel {
        scene.bake_cooked(*id);
    }
    new_sel
}

/// **Congela a RECEITA de uma forma viva**, e só ela — a porta que as ferramentas de quina
/// (Fillet / Chamfer) usam antes de escrever um `corner_radius` num vértice de Shape.
///
/// Sem isto o raio seria varrido pelo `recook_into` no frame seguinte (o *"funciona e depois
/// esquece"*), e por isso as ferramentas RECUSAVAM a forma viva — o que o Enio leu, com razão,
/// como *"nao funciona nos vertex das shapes"*. Congelar é o mesmo que o artista faria à mão com
/// "Convert to Curves"; fazê-lo dentro do gesto poupa a viagem.
///
/// **Só a forma paramétrica.** Conector/morph/blend/envelope também são derivados, mas ali a
/// geometria é uma RELAÇÃO — soltá-la sem o artista pedir destruiria o que ele construiu, então
/// a quina segue recusada neles. Devolve `true` se congelou algo.
pub(crate) fn freeze_shape_recipe(sim: &mut SimWorld, map: &VecEntityMap, id: VecPathId) -> bool {
    let Some(&bits) = map.get(&id) else {
        return false;
    };
    let e = Entity::from_bits(bits);
    // O TEXTO não: congelá-lo é explodir em glyphs (outra operação, com seleção nova).
    if !matches!(sim.world().get::<VecShape>(e), Some(VecShape::Param { .. })) {
        return false;
    }
    if let Ok(mut em) = sim.world_mut().get_entity_mut(e) {
        em.remove::<VecShape>();
        return true;
    }
    false
}

#[cfg(test)]
#[path = "vec_convert_tests.rs"]
mod tests;
