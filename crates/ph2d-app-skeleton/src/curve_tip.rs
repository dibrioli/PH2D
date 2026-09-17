//! ⭐⭐⭐ **AS OPÇÕES DE «QUEM MANDA NA PONTA DA CURVA»** — a lista que o painel MOSTRA e a escolha
//! que a shell ESCREVE, derivadas da mesma porta.
//!
//! # ⛔⛔ Porque é UMA função e não duas listas
//!
//! O selector é um índice: o painel pinta o `n`-ésimo nome e a shell aplica a `n`-ésima escolha. Com
//! duas derivações — uma a montar nomes, outra a montar `CurveTip` — bastava um filho nascer entre
//! os dois quadros para o artista escolher *«Bone 21»* e o osso obedecer ao *«Bone 22»*. *Duas
//! listas escritas à mão ao lado uma da outra são duas respostas à mesma pergunta, e a que o artista
//! vê é a que envelhece* (a mesma lei do selector de acções, ao lado).
//!
//! # A ordem, e porque ela é a da VARREDURA
//!
//! As duas fixas primeiro (`Chain`, `Straight`) e depois um filho-osso por linha, na ordem em que o
//! mundo os devolve. ⚠️ **A ordem não decide nada** — quem manda é o [`ph2d_ecs::StableId`] que a
//! opção carrega, não a posição dela; a posição só tem de ser a MESMA nas duas leituras do mesmo
//! quadro, e é o que esta porta garante.

use ph2d_ecs::{Entity, SimWorld};
use ph2d_skeleton_ecs::{Bone, CurveTip};

/// ⭐ **Quantas opções o selector alcança** — o pool de ids do chrome é fixo, porque ele **não cunha
/// um id em tempo de execução** (a mesma cerca do selector de acções).
///
/// ⚠️ **O recurso é a MÃO**: uma mão abre em cinco dedos, logo um pulso tem cinco filhos-osso, e
/// `14` deixa quase três vezes essa folga. ⛔ Um osso com mais filhos que isto **não** vê a lista
/// truncada em silêncio — a porta devolve quantos ficaram de fora ([`Options::escondidos`]) e o
/// painel di-lo.
pub const MAX_TIP_CHILDREN: usize = 14;

/// O que o selector mostra: um rótulo por opção, a escolha ligada, e quantos filhos não couberam.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Options {
    /// Os rótulos, na ordem em que o painel os pinta.
    pub rotulos: Vec<String>,
    /// O índice do que está ligado — sempre válido: uma escolha que já não resolve cai no `Chain`.
    pub ligado: usize,
    /// Quantos filhos-osso ficaram fora do pool. `0` no caso normal.
    pub escondidos: usize,
}

/// ⭐⭐⭐ **A LISTA, e a escolha que cada linha dela representa.**
///
/// `None` ⇒ este osso não tem a pergunta: ou não é osso, ou as alças dele são **autoradas** (ali
/// ninguém deriva tangente de vizinho nenhum, e um selector seria um controlo morto).
#[must_use]
pub fn options(sim: &SimWorld, e: Entity) -> Option<(Options, Vec<CurveTip>)> {
    let osso = sim.world().get::<Bone>(e)?;
    if osso.handles != ph2d_skeleton::bend::Handles::Auto {
        return None;
    }
    let escolhido = osso.curve_tip;
    let mut rotulos = vec![
        ph2d_i18n::tr("panel.vector.bone.tip.chain").to_string(),
        ph2d_i18n::tr("panel.vector.bone.tip.straight").to_string(),
    ];
    let mut escolhas = vec![CurveTip::Chain, CurveTip::Straight];
    let mut escondidos = 0usize;
    for er in sim.world().iter_entities() {
        let meu_filho = er
            .get::<ph2d_ecs::ChildOf>()
            .is_some_and(|c| c.parent() == e);
        if !meu_filho || er.get::<Bone>().is_none() {
            continue;
        }
        // ⚠️ **Sem id durável não há escolha possível** — e é um estado real: o
        // `assign_missing_stable_ids` corre no quadro, então um osso acabado de nascer pode chegar
        // aqui sem ele. Mostrá-lo daria uma linha que não se pode escrever.
        let Some(id) = er.get::<ph2d_ecs::StableId>().copied() else {
            continue;
        };
        if escolhas.len() - 2 >= MAX_TIP_CHILDREN {
            escondidos += 1;
            continue;
        }
        rotulos.push(
            er.get::<ph2d_ecs::Name>()
                .map_or_else(|| format!("#{}", id.0), |n| n.0.clone()),
        );
        escolhas.push(CurveTip::Bone(id));
    }
    // ⚠️ **A escolha ligada resolve-se contra a lista de AGORA**: um filho apagado deixa de ter
    // linha, e o selector mostra `Chain` — que é exactamente o que a lei faz nesse caso.
    let ligado = escolhas.iter().position(|c| *c == escolhido).unwrap_or(0);
    Some((
        Options {
            rotulos,
            ligado,
            escondidos,
        },
        escolhas,
    ))
}

/// ⭐ **A escolha da `n`-ésima linha** — a outra metade da porta, para quem aplica o clique.
///
/// `None` ⇒ o índice não existe nesta lista (a lista mudou entre o clique e o dreno), e então
/// **não se escreve nada**: uma escolha inventada mudaria a curva por um clique que o artista não
/// deu.
#[must_use]
pub fn choice_at(sim: &SimWorld, e: Entity, i: usize) -> Option<CurveTip> {
    options(sim, e).and_then(|(_, escolhas)| escolhas.get(i).copied())
}

#[cfg(test)]
#[path = "curve_tip_tests.rs"]
mod tests;
