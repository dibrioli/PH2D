//! **A porta única do re-cozimento em lugar** — [`VecPath::replace_cooked`].
//!
//! # Por que existe
//!
//! Várias features re-geram a geometria de um path que **já está na cena** e a escrevem em lugar,
//! porque um id novo por frame faria entidade / seleção / gizmo **piscarem**: o texto vivo a cada
//! tecla ([`crate`]'s consumidores no shell), o objeto de texto quando o painel muda um knob, e o
//! envelope a cada frame. Todos precisam responder à mesma pergunta — *o que um re-cozimento
//! produz, e o que ele não conhece?* — e todos a respondiam **por conta própria**.
//!
//! Havia três respostas e duas estavam erradas:
//!
//! | Sítio | Como respondia | Resultado |
//! |---|---|---|
//! | envelope (`write_shape`) | seis campos escritos à mão | **certo**, por enumeração |
//! | texto vivo (`regen_into`) | `*p = np` | apagava [`VecPath::effects`] |
//! | objeto de texto (`recook_text_object`) | `*p = np` | apagava [`VecPath::effects`] |
//!
//! O `np` de um re-cozimento nasce de um construtor com `..Default::default()`, então `*p = np`
//! não "atualiza a geometria": **substitui o path inteiro**, e leva junto tudo o que o
//! re-cozimento não sabe montar. O sintoma medido era a **pilha de Live Path Effects** (ADR-0132)
//! de um texto **desaparecer ao digitar a letra seguinte** — e como o re-cook do texto é
//! event-driven, o defeito só dispara quando o artista volta a escrever, que é exatamente quando
//! ele não está olhando para o efeito.
//!
//! # A lei
//!
//! Um re-cozimento produz **geometria e estilo**. Ele **não** conhece:
//!
//! - a **identidade** (`id`) — quem re-cozinha é o path que já existe, não um path novo;
//! - a **pilha de efeitos** (`effects`) — é dado AUTORADO sobre a forma, e a forma cozida é a
//!   entrada dela, não a saída (ADR-0121: fonte autorada ≠ geometria cozida).
//!
//! # Por que o compilador é o guarda, e não um comentário
//!
//! A enumeração do envelope estava **certa e frágil**: acertava os seis campos de hoje e ficaria
//! errada em silêncio no sétimo. O `let Self { .. }` exaustivo abaixo **não compila** quando
//! [`VecPath`] ganha um campo, e obriga quem o acrescentar a responder *"isto é produzido pelo
//! re-cozimento, ou sobrevive a ele?"* — no mesmo commit em que o campo nasce, que é a única hora
//! em que a resposta é conhecida.

use crate::VecPath;

impl VecPath {
    /// Substitui em lugar tudo o que um **re-cozimento** produz (geometria + estilo), preservando
    /// o que ele não conhece: a **identidade** (`id`) e a **pilha de efeitos** (`effects`).
    ///
    /// É a porta única de "esta forma foi re-gerada a partir dos próprios parâmetros" — o texto a
    /// cada tecla, o objeto de texto a cada knob do painel, o filho de um envelope a cada frame.
    /// Escrever os campos à mão no sítio de chamada **funciona hoje e apodrece no campo seguinte**;
    /// ver o cabeçalho do módulo.
    ///
    /// `next.id` é **descartado de propósito**: quem manda na identidade é o path que já está na
    /// cena. Isso apaga o `np.id = id` que cada chamador repetia antes de escrever.
    pub fn replace_cooked(&mut self, next: Self) {
        // ⚠️ Destructuring EXAUSTIVO: é isto que faz o compilador barrar um campo novo esquecido.
        // Não troque por `..` — seria devolver a enumeração frágil com outra sintaxe.
        let Self {
            id: _,
            verts,
            closed,
            fill,
            stroke,
            subpaths,
            fill_rule,
            effects: _,
        } = next;
        self.verts = verts;
        self.closed = closed;
        self.fill = fill;
        self.stroke = stroke;
        self.subpaths = subpaths;
        self.fill_rule = fill_rule;
        // `self.id` e `self.effects` sobrevivem — a lei do módulo.
    }
}

#[cfg(test)]
#[path = "recook_tests.rs"]
mod tests;
