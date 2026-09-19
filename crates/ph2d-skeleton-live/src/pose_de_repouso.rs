//! ⭐⭐⭐ **VOLTAR À POSE DE REPOUSO** — os dois verbos, e a cura do *Reset Transform*.
//!
//! # ⛔⛔⛔ O defeito que esta wave fecha, MEDIDO
//!
//! O menu de contexto da Hierarquia oferecia *Reset Transform* a **qualquer** linha, e a tabela
//! daquele menu é **plana** — ela não sabe o que a linha é. Sobre um osso, `*t = Transform::IDENTITY`
//! manda a pose para a origem e **apaga a direcção** (num osso, a direcção *é* a `rotation` e a
//! posição *é* a `translation`). Medido em 2026-09-19 sobre a cena `PH2D_VEC_BONE_SMOKE`: repor a
//! transformação do osso do meio move a arte presa **20,000 unidades num desenho de 60** — um terço
//! do desenho — e o app responde com uma mensagem **verde** a dizer que correu bem.
//!
//! ⇒ *«repor a transformação» de um osso não é a identidade: é o REPOUSO dele.* E a única razão de
//! isto ter sido escrito assim é que o repouso **não existia** — o que esta wave traz primeiro.
//!
//! # ⚠️ Porque a decisão vive AQUI e não no `hierarchy.rs`
//!
//! Porque ela é a mesma pergunta em **três** superfícies — o menu de contexto, o botão do painel do
//! esqueleto, e amanhã um atalho — e *uma lei escrita em três sítios diverge no primeiro ajuste*.
//! A porta devolve o **veredito** ([`Reposicao`]) em vez de mexer na tela ou de imprimir: assim a
//! metade que interessa (*a razão certa para o facto certo*) fica dentro de um teste, e a decisão
//! não precisa de um mundo desenhado para ser medida.
//!
//! # ⚠️ O sujeito é o osso ESCOLHIDO e a descendência dele — não o esqueleto inteiro
//!
//! É estritamente mais expressivo e degenera no caso simples: escolher a **raiz** repõe o boneco
//! todo, escolher o **antebraço** repõe o antebraço e a mão. ⛔ Uma lei que subisse à raiz sozinha
//! tornaria *«repor só este braço»* inexprimível, e não há gesto que a contorne.

use ph2d_ecs::{Entity, SimWorld, Transform};
use ph2d_skeleton_ecs::{Bone, BoneRest};

/// **O que aconteceu a um pedido de repor a transformação.**
///
/// ⚠️ **Três respostas e não duas**, e a terceira é a que impede o defeito de voltar por outra
/// porta: um osso **sem** repouso guardado não é um osso que se possa mandar para a identidade —
/// é um osso sobre o qual não há resposta, e o artista tem de a ouvir.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Reposicao {
    /// Não é osso — quem manda é a lei de sempre da Hierarquia (a identidade).
    ///
    /// ⚠️ **É esta variante que mantém o gesto intacto para todo o resto do app:** uma sprite, um
    /// grupo, um corpo de física continuam a ir para a identidade, ao bit.
    NaoEOsso,
    /// É osso, e **nenhum** da sub-árvore tem repouso guardado.
    SemRepouso,
    /// Reposto — `ossos` é quantos voltaram ao repouso deles.
    Reposta {
        /// Quantos ossos da sub-árvore tinham repouso guardado e voltaram a ele.
        ossos: usize,
    },
}

/// **Os dois verbos do repouso**, tipados — é isto que o painel manda à shell.
///
/// ⚠️ **Um enum e não um `bool`:** o barramento carrega um valor que alguém lê três fases mais
/// tarde, e `true` ali não diz qual dos dois é. *Um valor que muda de significado e mantém a forma
/// não avisa ninguém* — a lição do anel da ponta, que esta casa já pagou.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Verbo {
    /// Devolve os ossos ao repouso guardado.
    Repor,
    /// Faz da pose de agora o repouso.
    Guardar,
}

/// ⭐ **A PORTA ÚNICA dos dois verbos.** Devolve quantos ossos foram tocados.
///
/// ⚠️ Ela existe para o dreno ter **uma** chamada: com um `match` na shell, o dia em que houver um
/// terceiro verbo ele nasce ligado num sítio e não no outro — a forma exacta do defeito que a
/// tabela `VECTOR_BONE_VERBS` existe para impedir, um nível acima.
pub fn aplica(sim: &mut SimWorld, alvo: Entity, verbo: Verbo) -> usize {
    match verbo {
        Verbo::Repor => repor(sim, alvo),
        Verbo::Guardar => guardar(sim, alvo),
    }
}

/// **Guarda a pose de AGORA como repouso** deste osso e da descendência dele. Devolve quantos.
///
/// `0` ⇒ o alvo não é osso.
///
/// ⚠️ **Ele sobrescreve sem perguntar, e é o que o artista quer:** *«a partir de agora, ESTA é a
/// pose de repouso»* é a frase inteira do gesto. O caminho de volta é o `Ctrl+Z`, que é o caminho
/// de volta de tudo nesta casa.
pub fn guardar(sim: &mut SimWorld, alvo: Entity) -> usize {
    let ossos = crate::esqueletos::ossos_desde(sim, alvo);
    let mut n = 0;
    for e in ossos {
        let Some(t) = sim.world().get::<Transform>(e).copied() else {
            continue;
        };
        sim.world_mut().entity_mut(e).insert(BoneRest::de(&t));
        n += 1;
    }
    n
}

/// **Devolve ao repouso** este osso e a descendência dele. Devolve quantos voltaram.
///
/// ⚠️ **Um osso da sub-árvore sem repouso guardado é SALTADO e os outros voltam** — é a mesma lei
/// do tendão cujo osso desapareceu: *uma peça em falta não pode apagar o gesto inteiro*. Quem conta
/// se houve alguma é o chamador, pela contagem.
pub fn repor(sim: &mut SimWorld, alvo: Entity) -> usize {
    let ossos = crate::esqueletos::ossos_desde(sim, alvo);
    let mut n = 0;
    for e in ossos {
        let Some(repouso) = sim.world().get::<BoneRest>(e).copied() else {
            continue;
        };
        let Some(mut t) = sim.world_mut().get_mut::<Transform>(e) else {
            continue;
        };
        repouso.aplica(&mut t);
        n += 1;
    }
    n
}

/// ⭐⭐⭐ **O QUE *Reset Transform* QUER DIZER NESTA LINHA** — a porta que o menu plano da
/// Hierarquia não tinha.
///
/// ⚠️ **Ela é `&mut` e faz o trabalho**, em vez de devolver um conselho que o chamador executa: com
/// um conselho haveria **duas** respostas à mesma pergunta (a desta porta e a do `if` que a
/// consome), e a segunda é a que envelhece. Quem lê o veredito lê-o para **falar**, nunca para
/// decidir.
pub fn repor_transformacao(sim: &mut SimWorld, alvo: Entity) -> Reposicao {
    if sim.world().get::<Bone>(alvo).is_none() {
        return Reposicao::NaoEOsso;
    }
    match repor(sim, alvo) {
        0 => Reposicao::SemRepouso,
        ossos => Reposicao::Reposta { ossos },
    }
}

#[cfg(test)]
#[path = "pose_de_repouso_tests.rs"]
mod tests;
