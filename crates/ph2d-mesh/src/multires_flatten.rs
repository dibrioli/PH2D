//! **O ACHATAR** — a pilha de níveis vira UMA malha, com todo o detalhe.
//!
//! Módulo FILHO do [`super`] pelo mesmo motivo do [`super::multires_reverse`]:
//! ele mexe nos `levels`, nos `details` e no `sel`, que são privados da pilha de
//! propósito. Um irmão precisaria que eles virassem `pub(crate)`, e aí a próxima
//! wave escreveria neles sem passar pelas leis que o `lower`/`higher` mantêm.
//!
//! # Por que ele existe: três verbos RECUSAM com a pilha montada
//!
//! Reconstruir por voxelização troca a topologia inteira, e todo nível acima é
//! `subdivide` da base — o detalhe deles passaria a descrever uma malha que não
//! existe mais. A recusa é certa. O que faltava era a **saída**: o artista que
//! subdividiu não tinha gesto nenhum que o devolvesse a um nível só sem perder
//! trabalho (descartar o topo é jogar fora o detalhe; reverter é o oposto — ela
//! constrói uma base ABAIXO e a pilha fica mais alta).
//!
//! ⚠️ **E a alternativa que este verbo NÃO é: achatar em silêncio dentro do
//! remesh.** O botão que o artista aperta diz *reconstruir*; colapsar a pilha
//! dele de carona seria destruir estrutura autorada num gesto que não a nomeia —
//! literalmente o que a recusa existe para impedir. Aqui ele PEDE, e o gesto tem
//! inverso.
//!
//! # A malha que fica é a do TOPO, e é a única resposta que não perde trabalho
//!
//! ⚠️ **`levels[k]` acima do selecionado está OBSOLETO** — é a `higher` quem o
//! sintetiza de `(base, detalhe)`. Ficar com o nível em que o artista está de pé
//! jogaria fora todo detalhe acima dele, em silêncio. Subir primeiro é exato de
//! graça (o doc da [`super::Multires::higher`]: subir só escreve no topo valores
//! que a base e o detalhe já determinam), e o que sobra é exatamente a malha que
//! ele veria lá em cima.

use super::{Mesh, Multires};

impl Multires {
    /// **ACHATA a pilha** para um nível só, com todo o detalhe, e devolve o
    /// estado anterior INTEIRO — a entrada de desfazer.
    ///
    /// `None` quando já há um nível só: não há o que achatar, e a recusa é do
    /// chamador, que é quem sabe o que dizer ao artista.
    ///
    /// ⚠️ **O custo é UM clone da malha do topo, e ele é ESTRUTURAL.** Desfazer
    /// instala a pilha de antes e refazer instala a achatada, então as duas
    /// existem ao mesmo tempo e as duas precisam dessa malha. A alternativa
    /// (deixar um marcador vazio no topo da pilha guardada) é o tipo que *se diz
    /// um nível e devolve metade dele*, exatamente o que o doc do
    /// [`super::DetachedLevel`] recusa. É o mesmo preço que o desfazer de uma
    /// reconstrução já paga, e quem o limita é o teto em BYTES da história.
    #[must_use]
    pub fn flatten(&mut self) -> Option<Multires> {
        if self.levels.len() < 2 {
            return None;
        }
        let was = self.sel;
        // Subir até o topo é o que traz o detalhe de TODOS os níveis para dentro
        // de uma malha só. `select` só ascende aqui — nunca desce —, então ela
        // não carimba nada na base.
        self.select(self.levels.len() - 1);
        let top: Mesh = self.levels[self.sel].clone();
        let mut previous = core::mem::replace(self, Multires::new(top));
        // ⚠️ **A subida foi um MEIO, não uma escolha do artista**, e sem esta
        // linha ela vazava para fora: a pilha que volta apontava para o TOPO, e
        // o Ctrl+Z de quem achatou estando no nível 0 o teleportava para cima.
        //
        // ⚠️ **E ela escreve o campo em vez de chamar `select`** — de propósito.
        // Descer passa por `lower`, que re-encoda o detalhe e reconstrói a base;
        // aqui isso seria trabalho para reproduzir o que já está lá (nada foi
        // esculpido entre a subida e este ponto), com o round-trip de frame
        // custando o ulp que o gate de ida-e-volta do módulo mede. Re-apontar a
        // seleção é EXATO, e os níveis acima ficam sintetizados, que é um estado
        // que a pilha sempre admite.
        previous.sel = was;
        Some(previous)
    }
}
