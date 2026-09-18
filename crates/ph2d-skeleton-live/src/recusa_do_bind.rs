//! ⭐⭐⭐ **O BIND DIZ PORQUE NÃO VAI FAZER NADA** — a porta que separa *«prendi ao esqueleto que
//! nomeaste»* de *«prendi ao que havia na cena»*.
//!
//! ⛔⛔⛔ **O defeito que ela cura está MEDIDO** (`sonda_do_rig_partilhado_tests.rs`): o botão *Bind*
//! passa `semente = osso_selecionado`, e com nada de osso aceso ele é `None`; o [`skeleton_of`]
//! responde a `None` com **todos os ossos da cena**. Com um esqueleto isso é a resposta certa e
//! conveniente — com dois, a pele guardou **6** ossos de duas cadeias que não se conhecem, a forma
//! passou a obedecer às duas, e o log do produto disse *«1 imagem presa»*.
//!
//! ⚠️ **A cerca é o que a torna aceitável:** com **um** esqueleto na cena o caminho é byte-idêntico
//! ao de sempre. *Exigir sempre o osso partiria o fluxo que o artista já aprendeu, para curar um
//! caso que só existe quando há ambiguidade.*
//!
//! ⚠️ **Ela devolve a RAZÃO, nunca a imprime** — senão a metade que interessa (*a razão certa para o
//! facto certo*) fica fora de qualquer teste, e a decisão precisaria de um `SimWorld` desenhado.
//!
//! ⏳ **DÍVIDA NOMEADA, e ela não é desta wave:** esta recusa sai no terminal, como as **duas** que
//! o mesmo botão já tinha (*«selecione ao menos UMA forma»* e a nota do que ficou preso). *Uma
//! recusa que só o terminal vê é um botão mudo* — e curar só a nova deixaria duas superfícies para
//! a mesma pergunta. As três sobem à tela juntas, numa wave com superfície própria.

use crate::esqueletos::bone_roots;
use ph2d_ecs::{Entity, SimWorld};

/// Porque é que este *Bind* não vai prender nada.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecusaDoBind {
    /// Há mais de um esqueleto na cena e nenhum osso escolhido — prender agora juntaria todos.
    VariosEsqueletos {
        /// Quantos esqueletos independentes a cena tem.
        quantos: usize,
    },
}

impl RecusaDoBind {
    /// A frase que o artista lê, **com a cura dentro**.
    ///
    /// ⚠️ Uma recusa que diz o problema e não diz o gesto manda o artista adivinhar, que é o mesmo
    /// que não dizer nada.
    #[must_use]
    pub fn frase(self) -> String {
        match self {
            Self::VariosEsqueletos { quantos } => format!(
                "osso: ha' {quantos} esqueletos na cena e nenhum osso escolhido -- escolha \
                 tambem UM osso do esqueleto a que quer prender (Ctrl+clique na Hierarquia), \
                 senao a forma ficaria presa aos {quantos} ao mesmo tempo"
            ),
        }
    }
}

/// **Este *Bind* pode correr?** — `None` quer dizer *sim*.
///
/// ⚠️ **A metade NEGATIVA é metade do valor:** com um esqueleto só, ou com o osso escolhido, ela
/// cala-se. *Um botão que se queixa sempre é ruído que o artista aprende a ignorar, exactamente
/// quando a queixa passar a ser verdade.*
#[must_use]
pub fn recusa_do_bind(sim: &SimWorld, semente: Option<Entity>) -> Option<RecusaDoBind> {
    if semente.is_some() {
        return None;
    }
    let quantos = bone_roots(sim).len();
    (quantos > 1).then_some(RecusaDoBind::VariosEsqueletos { quantos })
}

#[cfg(test)]
#[path = "recusa_do_bind_tests.rs"]
mod tests;
