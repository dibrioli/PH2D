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
//! # ⛔⛔⛔ E há uma SEGUNDA pergunta, que o dono reportou em 2026-09-19
//!
//! *«Num vector linkado aos ossos não consigo mudar a espessura do stroke.»*
//!
//! A [`VecPath::replace_cooked`] responde *«o que um re-cozimento produz»* para quem re-gera a
//! forma **a partir dos parâmetros dela** — o texto a cada tecla, o objecto de texto a cada knob.
//! Ali o estilo É produto do re-cozimento, e tem de vir do `next`.
//!
//! ⚠️ **A PELE não é desse tipo:** a fonte dela é uma FOTOGRAFIA tirada no instante do `Bind`
//! ([`ph2d_skeleton_ecs::SkinBind::source`]), congelada em bytes opacos, e o que ela re-gera a cada
//! quadro é a **posição de cada ponto**. Mandar o estilo daquela fotografia para o path vivo faz o
//! quadro seguinte **desfazer** toda edição de traço, preenchimento ou regra de preenchimento —
//! medido: o artista põe `width = 0,2` e o recook devolve `None`, sem um erro e sem um pixel de
//! aviso. *Um controlo que o produto desfaz no quadro seguinte lê-se exactamente como um controlo
//! morto.*
//!
//! ⇒ a [`VecPath::replace_geometry`] é a porta de *«re-gerei ONDE os pontos estão»*, e ela preserva
//! todo o estilo. ⛔ **As duas portas destruturam a struct de forma EXAUSTIVA**, e é deliberado: um
//! campo novo obriga a responder **as duas** perguntas — *é produto de um re-cozimento?* e *é
//! GEOMETRIA?* — no commit em que ele nasce, que é a única hora em que as duas respostas são
//! conhecidas.
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
            // ⭐ **A opacidade e a mistura SOBREVIVEM ao re-cozimento** (v19), como a pilha de
            // efeitos e pela mesma razão: elas são autoria do OBJECTO, não produto dos parâmetros
            // que o re-cozimento consome. `next` nasce de um `VecPath::default()` com a geometria
            // por cima, então lê-las dali repunha o neutro — e o sintoma seria **reescrever o
            // texto de uma forma a 50% devolvê-la opaca**, sem erro nenhum.
            opacity: _,
            blend: _,
            // ⭐ **A PILHA DE APARÊNCIA SOBREVIVE ao re-cozimento** (v20), pela MESMA razão que a
            // opacidade e a mistura: ela é autoria do OBJECTO e não produto dos parâmetros que o
            // re-cozimento consome. `next` nasce de um `VecPath::default()`, então lê-la dali
            // apagaria a pilha — e o sintoma seria **reescrever o texto de uma forma com dois
            // contornos devolvê-la com um só**, sem erro nenhum.
            paints: _,
        } = next;
        self.verts = verts;
        self.closed = closed;
        self.fill = fill;
        self.stroke = stroke;
        self.subpaths = subpaths;
        self.fill_rule = fill_rule;
        // `self.id`, `self.effects`, `self.opacity` e `self.blend` sobrevivem — a lei do módulo.
    }

    /// ⭐⭐⭐ **Substitui em lugar só a GEOMETRIA** — onde os pontos estão —, preservando **todo** o
    /// estilo do objecto: `fill`, `stroke`, `fill_rule`, a pilha de aparência, a de efeitos, a
    /// opacidade, a mistura e a identidade.
    ///
    /// É a porta de quem re-gera POSIÇÕES a partir de uma fonte que ele guardou: a pele a cada
    /// quadro ([`ph2d_skeleton_live::skin_live`]) e o `Release` que devolve o desenho autorado.
    /// ⚠️ **Quem re-gera a forma a partir dos PARÂMETROS dela** — o texto, o objecto de texto, o
    /// envelope — continua a ir pela [`Self::replace_cooked`]: ali o estilo é produto do
    /// re-cozimento. Ver o cabeçalho do módulo para o report que separou as duas.
    ///
    /// `next.id` é descartado de propósito, pela mesma razão da irmã.
    pub fn replace_geometry(&mut self, next: Self) {
        // ⚠️ Destructuring EXAUSTIVO, como na irmã: é isto que faz o compilador barrar um campo
        // novo esquecido. Não troque por `..`.
        let Self {
            id: _,
            verts,
            closed,
            subpaths,
            // ⭐ **Tudo o que segue é ESTILO e sobrevive** — é o que o artista autora no painel
            // sobre a forma, e a fotografia do `Bind` não tem nada a dizer sobre isso.
            fill: _,
            stroke: _,
            fill_rule: _,
            effects: _,
            opacity: _,
            blend: _,
            paints: _,
        } = next;
        self.verts = verts;
        self.closed = closed;
        self.subpaths = subpaths;
    }
}

#[cfg(test)]
#[path = "recook_tests.rs"]
mod tests;
