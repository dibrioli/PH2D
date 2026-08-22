//! **A moldura da SELEÇÃO** — a projeção que o painel lê, e o chip que a edita.
//!
//! Duas metades da mesma pergunta (*a seleção é uma moldura, e ela recorta?*), respondidas UMA vez
//! e no lugar onde o mundo mora. O painel não alcança o ECS — e não deve: se alcançasse, haveria
//! duas respostas para *"esta seleção é uma moldura?"*, e a que desenha a seção divergiria da que
//! honra o clique.
//!
//! # A moldura é a que CONTÉM a seleção
//!
//! ⚠️ **Um GRUPO puro ainda expande** — o [`crate::vec_selection`] entrega ao pen o que
//! `selection_paths` diz, e um grupo não tem geometria própria, logo empresta os filhos. Então
//! uma moldura dentro de um grupo chega aqui junto com os irmãos, e uma regra de *"exactamente
//! um selecionado"* responderia `None` sobre a moldura que interessa. Foi o defeito reportado —
//! *"não vejo em lugar nenhum a seção Frame"*.
//!
//! A pergunta certa é o **INVERSO da expansão**, pela mesma relação de parentesco que a produziu:
//! *qual moldura selecionada contém tudo o que está selecionado?* Ela responde igual pelas duas
//! rotas de selecção, e é isso que impede a resposta de depender de POR ONDE o artista selecionou.
//!
//! ⚠️ E ela ficou MAIS simples em 2026-08-02, sem mudar de forma: uma **moldura** deixou de
//! emprestar os filhos (ela é uma forma, e restilizá-la restilizava seis), então hoje a moldura
//! costuma chegar aqui sozinha — e contém-se a si própria. Continua a ser o inverso da expansão
//! porque a expansão continua a existir *para grupos*.
//!
//! O que ela recusa, e é o desenho:
//!
//! - **um filho sozinho** não oferece a seção — o artista selecionou a forma, não o contêiner;
//! - **duas molduras irmãs** não têm resposta única (nenhuma contém a outra);
//! - **moldura + forma solta** também não: o chip editaria o que o artista não está a olhar.
//!
//! Aninhadas, a de FORA contém as de dentro ⇒ ela é a resposta, que é a moldura em que o artista
//! de facto clicou.

use ph2d_ecs::{ChildOf, Entity, SimWorld, VecFrame};
use ph2d_vec_scene::VecPathId;

use crate::vec_entities::VecEntityMap;

/// Teto de profundidade da caminhada de ancestral — defesa contra árvore corrompida, o mesmo
/// número e pelo mesmo motivo do [`crate::vec_selection`].
const MAX_DEPTH: usize = 64;

/// `who` é `root`, ou descende dela?
///
/// ⚠️ `pub(crate)` porque o RECORTE (`crate::vec_clip_edit`) faz a mesma pergunta de parentesco
/// sobre um conjunto de candidatos MAIOR (qualquer forma fechada, não só a moldura). Duas
/// caminhadas de ancestral divergiriam no dia em que o teto de profundidade ou a regra de
/// expansão mudasse, e as duas seções apareceriam em seleções diferentes.
pub(crate) fn is_within(sim: &SimWorld, root: Entity, who: Entity) -> bool {
    let w = sim.world();
    let mut cur = who;
    for _ in 0..MAX_DEPTH {
        if cur == root {
            return true;
        }
        let Some(parent) = w.get::<ChildOf>(cur).map(|c| c.parent()) else {
            return false;
        };
        cur = parent;
    }
    false
}

/// A entidade VIVA de um caminho.
///
/// ⚠️ `pub(crate)` pela mesma razão do [`is_within`]: o recorte resolve o mesmo par
/// caminho → entidade, e uma segunda resolução seria uma segunda resposta a *"esta seleção
/// descreve um mundo que ainda existe?"*.
pub(crate) fn entity_of(sim: &SimWorld, map: &VecEntityMap, id: VecPathId) -> Option<Entity> {
    let &bits = map.get(&id)?;
    let e = Entity::from_bits(bits);
    sim.world().get_entity(e).ok().map(|_| e)
}

/// A moldura que CONTÉM a seleção inteira — `None` se não houver.
///
/// ⚠️ **`pub(crate)` porque o AUTO LAYOUT (W2) faz a MESMA pergunta** — *qual moldura o artista
/// está a olhar?* — e uma segunda resolução divergiria desta no dia em que a regra de expansão
/// mudasse: a seção Frame e a seção Layout apareceriam em seleções diferentes.
pub(crate) fn frame_of_selection(
    sim: &SimWorld,
    map: &VecEntityMap,
    selected: &[VecPathId],
) -> Option<Entity> {
    if selected.is_empty() {
        return None;
    }
    let members: Vec<Entity> = selected
        .iter()
        .filter_map(|id| entity_of(sim, map, *id))
        .collect();
    // ⚠️ Um caminho selecionado sem entidade viva descreve um mundo que já não existe, e ali
    // *"contém tudo"* deixaria de significar o que diz — a moldura pareceria conter uma seleção
    // cuja metade ausente ninguém conferiu.
    if members.len() != selected.len() {
        return None;
    }
    members.iter().copied().find(|&f| {
        sim.world().get::<VecFrame>(f).is_some() && members.iter().all(|&m| is_within(sim, f, m))
    })
}

// ⚠️ **O RECORTE mudou-se para [`crate::vec_clip_edit`]** (2026-08-21). Ele deixou de ser um
// campo da moldura e passou a valer para qualquer forma FECHADA, então a pergunta *"quem recorta
// nesta seleção?"* tem um conjunto de candidatos maior do que *"qual é a moldura desta seleção?"*
// — que é o que este módulo continua a responder, para a seção Frame, o auto layout e as âncoras.

#[cfg(test)]
#[path = "vec_frame_edit_tests.rs"]
mod tests;
