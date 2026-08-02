//! **O BINDING DE TOKEN** — uma propriedade da forma deixa de ser um literal e passa a
//! REFERENCIAR um token, resolvido por MODO.
//!
//! É a feature de maior alavancagem do plano de UI/UX (§4/W4), e aqui ela tem um segundo consumidor
//! que nenhuma outra ferramenta de design tem: **o próprio editor**. A mesma tabela
//! (`docs/design/tokens.json` → `ph2d-tokens`) que veste os 44 widgets do app passa a vestir a arte
//! do artista, então trocar de modo re-veste **o card que ele desenhou E o app inteiro**.
//!
//! # A tabela é LATERAL, e essa é a decisão inteira
//!
//! ⚠️ **Nenhum campo é apendado a `Paint`, a `StrokeSpec` ou a `VecShape`.** Se o binding morasse
//! dentro do `Paint`, **todo** save de vetor mudaria de forma e o `VEC_SCENE_SCHEMA` bumparia por
//! uma feature que 90% dos documentos não usa — e um bump **recusa todo projeto já salvo**. Um
//! componente NOVO cunha `stable_type_id = blake3(NOME)[..8]` e não move nada.
//!
//! É a mesma lei que o repo já aplica em quatro lugares (*"todo canal novo é side-metadata no
//! registry, nunca contrato"* — os 6 canais que o `KernelResolver` ganhou sem mover
//! `NodeOp`/`OpResolver`/`NodeManifest`), e o precedente direto é o [`crate::VecStrokeProfile`]
//! (ADR-0148): *o perfil é um componente ECS, não um campo do `StrokeSpec`*.
//!
//! # A chave é o NOME do token, nunca o índice
//!
//! ⚠️ [`TokenRef`] guarda a chave kebab-case (`"accent"`), que é a identidade estável de um sistema
//! de tokens — a que o `tokens.json` usa, a que o DTCG fala, e a que
//! [`ph2d_tokens::ColorToken::from_key`] resolve. Guardar o índice do variant amarraria todo
//! projeto salvo à ORDEM da lista, e inserir um token no meio dela re-pintaria arte que ninguém
//! tocou.
//!
//! Corolário que vale para a wave seguinte: quando a tabela virar autorável (tokens do ARTISTA,
//! aliases, math), eles entram por aqui **sem migração** — a chave já é o endereço.
//!
//! # O literal SOBREVIVE
//!
//! Bindar não apaga a cor autorada: o `Paint` do documento fica onde está, e o binding é uma
//! camada por cima na hora de DESENHAR (a costura fonte ≠ cozido do ADR-0121, agora na TINTA).
//! Desbindar devolve exatamente a cor que estava lá — sem isso, experimentar um token custaria a
//! escolha anterior.

use bevy_ecs::component::Component;
use serde::{Deserialize, Serialize};

use crate::SimComponent;

/// **Que propriedade desta forma está bindada.**
///
/// ⚠️ Os discriminantes são valores de ARQUIVO e a lista é **append-only**: um variant novo entra
/// no fim, nunca no meio. Inserir um no meio re-interpretaria todo binding já salvo — o `Fill` de
/// ontem viraria o `StrokeColor` de hoje, em silêncio e com o projeto abrindo normalmente.
///
/// A lista de hoje é a das propriedades de COR, que são as que o `ph2d-tokens` sabe resolver. As
/// que o plano nomeia e que ainda não estão aqui (`CornerRadius`, `StrokeWidth`, `LayoutGap`) são
/// tokens de ESCALA, e cada uma espera o canal que a resolve — acrescentá-las agora seria oferecer
/// um alvo que nada preenche.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[repr(u16)]
pub enum BoundProp {
    /// O preenchimento (`VecPath::fill`).
    Fill = 0,
    /// A cor do traço (`VecPath::stroke.paint`).
    StrokeColor = 1,
}

impl BoundProp {
    /// As propriedades que se podem bindar hoje — a lista que o painel OFERECE.
    ///
    /// Ela é DADO pelo mesmo motivo que o `ColorToken::ALL`: uma segunda lista escrita à mão na UI
    /// nasce desatualizada no dia em que esta ganhar um membro.
    pub const ALL: &'static [Self] = &[Self::Fill, Self::StrokeColor];

    /// Rótulo curto, para a UI dizer QUAL propriedade está presa.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Fill => "Fill",
            Self::StrokeColor => "Stroke",
        }
    }
}

/// A chave kebab-case de um token (`"accent"`, `"bg-2"`, `"text-1"`).
pub type TokenRef = String;

/// **As propriedades desta forma que seguem um token.**
///
/// Ausência do componente = nada bindado, e o desenho é **byte-idêntico** ao mundo pré-token — que
/// é o caso de todo documento que já existe.
///
/// ⚠️ **Uma propriedade tem no máximo UM token.** As entradas são mantidas ordenadas por
/// [`BoundProp`] e `set` SUBSTITUI: duas entradas para o mesmo alvo seriam duas respostas a *"de
/// que cor é este preenchimento?"*, e qual vence dependeria da ordem de inserção — um fato que o
/// artista não vê e não controla.
#[derive(Component, Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct VecBindings {
    /// Os pares `(propriedade, token)`, ordenados pela propriedade.
    pub entries: Vec<(BoundProp, TokenRef)>,
}

impl VecBindings {
    /// O token que dirige esta propriedade, se houver.
    #[must_use]
    pub fn get(&self, prop: BoundProp) -> Option<&str> {
        self.entries
            .iter()
            .find(|(p, _)| *p == prop)
            .map(|(_, t)| t.as_str())
    }

    /// Prende a propriedade a um token, SUBSTITUINDO o que lá estava.
    pub fn set(&mut self, prop: BoundProp, token: impl Into<TokenRef>) {
        let token = token.into();
        match self.entries.iter_mut().find(|(p, _)| *p == prop) {
            Some(slot) => slot.1 = token,
            None => {
                self.entries.push((prop, token));
                self.entries.sort_by_key(|(p, _)| *p);
            }
        }
    }

    /// Solta a propriedade — ela volta a valer o literal que o documento sempre guardou.
    pub fn clear(&mut self, prop: BoundProp) {
        self.entries.retain(|(p, _)| *p != prop);
    }

    /// Nada preso ⇒ o componente não tem razão de existir.
    ///
    /// Quem edita usa isto para DESANEXAR em vez de deixar um componente vazio: um vazio viaja no
    /// save, entra no diff do undo e faz duas cenas logicamente iguais compararem diferente.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

impl SimComponent for VecBindings {}

#[cfg(test)]
#[path = "vec_bindings_tests.rs"]
mod tests;
