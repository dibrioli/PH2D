//! ⭐⭐⭐ **POR QUE LEI CADA DESENHO ESCOLHIDO SE DEFORMA** — a porta da escolha que o dono mandou
//! construir (2026-09-19: *«como se escolhe se os envelopes vão ou não influenciar?»* ⇒ *«construa.
//! por desenho»*).
//!
//! # ⚠️ Ela existe porque o SUJEITO atravessa duas famílias
//!
//! Uma forma vectorial é escolhida pela lista de caminhos do pen; uma imagem é escolhida pelo
//! **gizmo**. São dois selectores, e a pergunta *«o que está escolhido e tem pele?»* precisa dos
//! dois. ⛔ Escrita duas vezes — uma no espelho que PINTA o chip e outra no dreno que o APLICA —,
//! ela divergiria no primeiro ajuste, e o sintoma seria o mais caro desta família: *o chip acende e
//! o desenho não muda*.
//!
//! ⇒ **o encadeamento mora aqui**, e quem chama passa os dois selectores que já tem na mão.

use ph2d_ecs::{Entity, SimWorld};
use ph2d_skeleton_ecs::{SkinBind, SkinLaw};

/// **As peles ESCOLHIDAS** — as duas mídias, por uma porta só.
///
/// ⚠️ **Filtra por [`SkinBind`] e não por mídia:** uma coisa sem pele não tem lei de pele, e
/// oferecer-lhe uma seria o selector sem sujeito que o `CLAUDE.md` §5.0 nomeia.
#[must_use]
pub fn escolhidas(
    sim: &SimWorld,
    caminhos: impl IntoIterator<Item = u64>,
    gizmo: impl IntoIterator<Item = u64>,
) -> Vec<Entity> {
    let mut out: Vec<Entity> = caminhos
        .into_iter()
        .chain(gizmo)
        .filter_map(Entity::try_from_bits)
        .filter(|&e| sim.world().get::<SkinBind>(e).is_some())
        .collect();
    // ⚠️ **Sem duplicados:** uma forma pode estar nos DOIS selectores, e escrever-lhe a lei duas
    // vezes é inofensivo hoje e deixa de o ser no dia em que a escrita contar alguma coisa.
    out.sort_by_key(|e| e.to_bits());
    out.dedup();
    out
}

/// **Alguma das escolhidas corre POR ALCANCE?** — o que o chip acende.
///
/// ⚠️ **`any` e não `all`, e a escolha é DECLARADA:** com metade da selecção em cada lei, um chip
/// que só acendesse com unanimidade deixaria o artista sem saber que a outra metade está noutra
/// lei. *Mostrar que há uma divergência é mais honesto do que mostrar a maioria.*
#[must_use]
pub fn alguma_por_alcance(
    sim: &SimWorld,
    caminhos: impl IntoIterator<Item = u64>,
    gizmo: impl IntoIterator<Item = u64>,
) -> bool {
    escolhidas(sim, caminhos, gizmo).into_iter().any(|e| {
        sim.world()
            .get::<SkinBind>(e)
            .is_some_and(|s| s.law == SkinLaw::Envelope)
    })
}

/// **Escreve a lei em todas as escolhidas.** Devolve quantas mudaram de facto.
///
/// ⚠️ **Conta as que MUDARAM, não as que foram tocadas** — é essa contagem que decide se o artista
/// merece um aviso, e escrever a lei que já lá estava não é um acontecimento.
///
/// ⭐ **Nada mais é tocado:** a tabela do padrão-ouro fica guardada onde está, e voltar ao
/// [`SkinLaw::Auto`] devolve a deformação de antes **ao bit**, sem re-resolver nada.
pub fn escreve(
    sim: &mut SimWorld,
    caminhos: impl IntoIterator<Item = u64>,
    gizmo: impl IntoIterator<Item = u64>,
    lei: SkinLaw,
) -> usize {
    let alvos = escolhidas(sim, caminhos, gizmo);
    let mut n = 0;
    for e in alvos {
        if let Some(mut skin) = sim.world_mut().get_mut::<SkinBind>(e)
            && skin.law != lei
        {
            skin.law = lei;
            n += 1;
        }
    }
    n
}

#[cfg(test)]
#[path = "skin_law_tests.rs"]
mod tests;
