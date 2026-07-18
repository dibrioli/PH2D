//! **A pilha de Live Path Effects** (ADR-0132) — efeitos não-destrutivos e empilháveis
//! sobre um caminho.
//!
//! O caminho GUARDA a fonte autorada + a lista de efeitos; o mundo CONSOME o resultado
//! ([`crate::VecPath::cooked`]). É o `inkscape:original-d` + `d`, um nível acima: o
//! ADR-0121 fez a costura para o raio de quina, e esta é a mesma costura generalizada.
//!
//! ```text
//! verts autorados → [estágio 0: quina] → efeito₁ → efeito₂ → … → mundo
//! ```
//!
//! # Três invariantes, e nenhum é decoração
//!
//! 1. **Um efeito é `VecPath -> VecPath`, puro.** É *por isso* que a pilha compõe — a saída
//!    de um é entrada legítima do seguinte, sem caso especial no meio.
//! 2. **O ponto neutro é um no-op byte-idêntico**, e a pilha SALTA efeitos neutros. Sem
//!    isso, ligar a seção "Effects" no painel custaria uma alocação por frame a todo
//!    documento que a abrisse e nunca a usasse. (É o invariante que a rack de áudio provou
//!    valer a pena: 42 efeitos, gate por-efeito.)
//! 3. **`Cow::Borrowed` sobrevive.** Sem raio e sem efeito ativo, `cooked()` devolve o
//!    mesmo ponteiro — foi essa propriedade que permitiu ligar o `cooked()` em TODO
//!    consumidor sem mudar comportamento, e ela não pode morrer aqui.
//!
//! # A quina é o estágio ZERO, e não entra na pilha
//!
//! O raio mora no vértice **autorado**, e arredondar **divide** um vértice em dois. Um
//! efeito a jusante resampleia — a contagem de vértices é *saída* dele. Não há para onde
//! levar o raio da quina que deixou de existir, e por isso a quina corre primeiro, sempre,
//! e o que segue é geometria **plana** (ADR-0132 §3).
//!
//! # Acrescentar um efeito
//!
//! Um `variant` novo em [`PathEffect`] (**no fim** — postcard é posicional), um módulo
//! `fx_*.rs` irmão com a matemática pura, um braço em [`PathEffect::apply`] e um em
//! [`PathEffect::is_neutral`]. A matemática mora num módulo próprio e não conhece a pilha;
//! é isso que mantém aberto o caminho de um dia embrulhá-la num nó (ADR-0132 §4).

use crate::fx_trim::{self, TrimSpec};
use crate::{Contour, VecPath};

/// **Um efeito da pilha.** Dado de documento — viaja no save e no undo como qualquer
/// geometria.
///
/// ⚠️ **Append-only**: o postcard serializa o índice do variant. Um variant inserido no meio
/// relê saves antigos como o efeito errado, em silêncio.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum PathEffect {
    /// Revela só um trecho do caminho — o *draw-on*. Ver [`crate::fx_trim`].
    Trim(TrimSpec),
}

impl PathEffect {
    /// Este efeito está no ponto neutro — ou seja, é um no-op byte-idêntico?
    ///
    /// A pilha usa isto para **saltá-lo por inteiro**, e é o que mantém o `Cow::Borrowed`
    /// vivo num documento que tem a seção aberta mas nada configurado.
    #[must_use]
    pub fn is_neutral(&self) -> bool {
        match self {
            Self::Trim(t) => t.is_neutral(),
        }
    }

    /// O nome que o painel mostra. Mora aqui (e não numa tabela no painel) porque uma
    /// segunda lista dos efeitos divergiria da primeira assim que alguém acrescentasse um.
    #[must_use]
    pub fn label(&self) -> &'static str {
        match self {
            Self::Trim(_) => "Trim Path",
        }
    }

    /// Aplica este efeito a um caminho inteiro — o contorno primário **e** cada subpath.
    ///
    /// Cada contorno é tratado **independentemente** (o modo "Individually" do After
    /// Effects). Um contorno que o efeito esvazie é DESCARTADO em vez de virar um contorno
    /// de zero vértices: um buraco vazio num compound não é geometria, é ruído para a
    /// booleana e para o preenchimento.
    #[must_use]
    pub fn apply(&self, path: &VecPath) -> VecPath {
        let mut out = path.clone();
        match self {
            Self::Trim(spec) => {
                let (verts, closed) = fx_trim::trim_contour(&path.verts, path.closed, spec);
                out.verts = verts;
                out.closed = closed;
                out.subpaths = path
                    .subpaths
                    .iter()
                    .filter_map(|c| {
                        let (v, cl) = fx_trim::trim_contour(&c.verts, c.closed, spec);
                        (!v.is_empty()).then_some(Contour {
                            verts: v,
                            closed: cl,
                        })
                    })
                    .collect();
            }
        }
        out
    }
}

/// **Roda a pilha** sobre `path`, na ordem em que ela está.
///
/// `None` quando não há nada a fazer (pilha vazia, ou toda ela neutra) — e é esse `None`
/// que permite ao `cooked()` devolver `Cow::Borrowed`.
///
/// **A saída sai com a pilha VAZIA**, e isso não é higiene: é o que faz cozinhar duas vezes
/// ser igual a cozinhar uma. O `corner_live` garante o mesmo zerando o `corner_radius` do
/// que emite. Sem isso, qualquer consumidor que chamasse `cooked()` sobre um resultado já
/// cozido aplicaria a pilha outra vez — e o sintoma seria uma forma que encolhe a cada
/// passagem, sem erro nenhum.
#[must_use]
pub fn run_stack(path: &VecPath, stack: &[PathEffect]) -> Option<VecPath> {
    let mut active = stack.iter().filter(|e| !e.is_neutral()).peekable();
    active.peek()?;
    let mut cur: Option<VecPath> = None;
    for fx in active {
        cur = Some(fx.apply(cur.as_ref().unwrap_or(path)));
    }
    let mut out = cur?;
    out.effects.clear();
    Some(out)
}

#[cfg(test)]
#[path = "effect_tests.rs"]
mod tests;
