//! ⭐⭐⭐ **A ESCULTURA REIVINDICA ESTE `Delete`?** — a lei, pura, e a razão quando não.
//!
//! # O report, e a segunda volta dele
//!
//! Enio, 2026-09-04: *«corrija o deletar com a tecla del»*, sobre o canvas com a escultura na
//! tela. A tecla morria num de três guardas e **nenhum deles dizia nada**. Posta a dizer, ela
//! nomeou o culpado à primeira: *«a ferramenta Motion/Vector está EM MÃOS e reivindica as
//! teclas nuas»*.
//!
//! # ⛔⛔ E o guarda estava a responder à pergunta ERRADA
//!
//! O [`crate::App::a_tool_owns_the_bare_keys`] existe para as teclas **NUAS** — as letras e os
//! dígitos que o Motion (`F`/`A`/`H`/`K`/`P`) e o Vector reclamam, e que a escultura também usa
//! como verbos (report de 2026-08-17: *«nem mesmo digitar um número no painel motion»*). ⛔ **O
//! `Delete` não é uma tecla nua**: nenhum dos dois o reclama por ter a ferramenta em mãos —
//! o Motion apaga um NÓ (e isso é no painel do grafo, que esta lei já cede pela área) e o Vector
//! apaga a forma **SELECIONADA** (e isso é uma condição, não um modo).
//!
//! ⇒ o `Delete` deixa de morrer por «ferramenta em mãos», e passa a ceder por **selecção**: se a
//! ferramenta vetorial tem um vértice ou um caminho escolhido, o `Delete` é dela. *Uma tecla com
//! vários donos reparte-se pelo que cada um TEM para apagar, não pelo que cada um É.*
//!
//! # ⚠️ Por que ela é uma função e não um `if` no teclado
//!
//! O teclado da escultura é um `impl App` — pede janela, superfície e device, e por isso só se
//! podia gatear por TEXTO. *Um gate textual não distingue a chamada viva da chamada atrás de um
//! `if false`.* Aqui a decisão é um punhado de factos → um veredito, e tem gate a sério.

/// **Os factos de que a decisão precisa** — e nada do `App`.
#[derive(Debug, Clone, Copy)]
pub struct DeleteFacts {
    /// O barro está na tela? (`FormRole::draws_clay`)
    pub clay_on_screen: bool,
    /// Um campo de texto tem o foco do teclado?
    pub text_focused: bool,
    /// O ponteiro está sobre um painel do chrome?
    pub over_panel: bool,
    /// A ferramenta vetorial tem um vértice ou um caminho **selecionado**?
    pub vector_has_selection: bool,
}

/// O veredito sobre um `Delete` que chegou ao teclado da escultura.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum DeleteClaim {
    /// A escultura apaga a peça activa.
    Ours,
    /// Não é dela — e a razão vai para o log, porque *uma tecla que não faz nada e não diz nada
    /// é indistinguível de uma tecla que não chegou*.
    NotOurs(&'static str),
}

/// ⭐⭐⭐ **A LEI**, na ordem em que as perguntas se fazem — e a ordem é a feature.
///
/// | # | quando | por que ANTES das seguintes |
/// |---|---|---|
/// | 1 | não há barro na tela | sem barro não há peça: nada a apagar, e nada a explicar |
/// | 2 | um campo de texto está focado | é a explicação que o artista precisa (*clique fora*), e ela ganha da área |
/// | 3 | o ponteiro está sobre um painel | ali quem manda é a linha da Hierarquia — onde o mesh agora TEM linha |
/// | 4 | o vetor tem uma selecção | o `Delete` dele apaga o que está escolhido; a escultura não o rouba |
///
/// ⛔ **Este teclado corre ANTES de toda a cadeia** (é o 2.º ramo do `key_input`): sem as
/// perguntas 3 e 4 ele comia o `Delete` da Hierarquia, do Flip, da timeline, do Painter e do
/// vetor sempre que houvesse barro na tela.
pub(super) const fn claim_delete(f: &DeleteFacts) -> DeleteClaim {
    if !f.clay_on_screen {
        return DeleteClaim::NotOurs(
            "nao ha' barro na tela (o pill SCULPT esta' fora, ou a forma nao e' barro)",
        );
    }
    if f.text_focused {
        return DeleteClaim::NotOurs("um campo de texto esta' FOCADO -- clique fora dele");
    }
    if f.over_panel {
        return DeleteClaim::NotOurs(
            "o ponteiro esta' sobre um painel -- quem manda ali e' a linha da Hierarquia",
        );
    }
    if f.vector_has_selection {
        return DeleteClaim::NotOurs(
            "a ferramenta vetorial tem uma seleccao -- o Delete apaga o que esta' escolhido nela",
        );
    }
    DeleteClaim::Ours
}

#[cfg(test)]
mod tests {
    use super::{DeleteClaim as C, DeleteFacts, claim_delete};

    /// O caso do artista a esculpir: barro na tela, ponteiro no canvas, nada focado.
    const fn esculpindo() -> DeleteFacts {
        DeleteFacts {
            clay_on_screen: true,
            text_focused: false,
            over_panel: false,
            vector_has_selection: false,
        }
    }

    /// ⭐⭐⭐ **GATE — com o barro na tela e o ponteiro no canvas, o `Delete` é da escultura.**
    #[test]
    fn no_canvas_com_barro_o_delete_e_da_escultura() {
        assert_eq!(claim_delete(&esculpindo()), C::Ours);
    }

    /// ⭐⭐⭐ **GATE — a FERRAMENTA EM MÃOS já não mata o `Delete`.**
    ///
    /// ⛔ É o report de 2026-09-04, com a linha impressa como prova: *«a ferramenta
    /// Motion/Vector está EM MÃOS e reivindica as teclas nuas»*. O `Delete` **não é** uma tecla
    /// nua — esta lei não tem sequer um campo para «a ferramenta em mãos», e é isso que o
    /// garante.
    #[test]
    fn a_ferramenta_em_maos_nao_mata_o_delete() {
        // Sem selecção no vetor, o veredito é o mesmo com ou sem ferramenta — porque a lei não
        // pergunta pela ferramenta.
        assert_eq!(claim_delete(&esculpindo()), C::Ours);
        let com_seleccao = DeleteFacts {
            vector_has_selection: true,
            ..esculpindo()
        };
        let C::NotOurs(porque) = claim_delete(&com_seleccao) else {
            panic!("⛔ com uma forma selecionada o Delete e' do vetor");
        };
        assert!(porque.contains("vetorial"), "{porque}");
    }

    /// ⭐⭐⭐ **GATE — sobre um painel, quem manda é a linha da Hierarquia.**
    #[test]
    fn sobre_um_painel_a_linha_da_hierarquia_manda() {
        let f = DeleteFacts {
            over_panel: true,
            ..esculpindo()
        };
        let C::NotOurs(porque) = claim_delete(&f) else {
            panic!("⛔ sobre um painel o Delete nao pode ser da escultura");
        };
        assert!(porque.contains("painel"), "{porque}");
    }

    /// ⭐⭐⭐ **GATE — a ORDEM das explicações.**
    ///
    /// ⚠️ Dizer *«o ponteiro está sobre um painel»* a quem tem um campo de texto focado manda o
    /// artista para o sítio errado — e sem barro na tela nenhuma das outras três importa.
    #[test]
    fn a_ordem_das_explicacoes_manda_o_artista_ao_sitio_certo() {
        let tudo_mau = DeleteFacts {
            clay_on_screen: false,
            text_focused: true,
            over_panel: true,
            vector_has_selection: true,
        };
        let C::NotOurs(porque) = claim_delete(&tudo_mau) else {
            panic!("⛔ sem barro nao ha' o que apagar");
        };
        assert!(porque.contains("barro"), "{porque}");

        let focado = DeleteFacts {
            text_focused: true,
            over_panel: true,
            ..esculpindo()
        };
        let C::NotOurs(porque) = claim_delete(&focado) else {
            panic!("⛔ um campo focado e' dono do teclado");
        };
        assert!(porque.contains("texto"), "{porque}");
    }
}
