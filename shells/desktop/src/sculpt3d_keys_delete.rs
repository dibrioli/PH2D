//! ⭐⭐⭐ **A ESCULTURA REIVINDICA ESTE `Delete`?** — a lei, pura, e a razão quando não.
//!
//! # O report
//!
//! Enio, 2026-09-04: *«corrija o deletar com a tecla del»*, sobre o canvas com a escultura na
//! tela. A tecla morria num de três guardas (`App::sculpt3d_keys_dead_reason`) e **nenhum deles
//! dizia nada** — nem ao artista, nem a quem foi diagnosticar.
//!
//! # ⚠️ Por que ela é uma função e não um `if` no teclado
//!
//! O teclado da escultura é um `impl App` — pede janela, superfície e device, e por isso só se
//! podia gatear por TEXTO. *Um gate textual não distingue a chamada viva da chamada atrás de um
//! `if false`.* Aqui a decisão é um par de factos → um veredito, e tem gate a sério.
//!
//! ⚠️ **E ela saiu do irmão pelo tecto de LOC** (`sculpt3d_keys.rs` a `611 / 600`) — o corte é
//! por RESPONSABILIDADE: *quem responde «esta tecla é nossa?»* não é *quem executa o verbo*.

/// O veredito sobre um `Delete` que chegou ao teclado da escultura.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum DeleteClaim {
    /// A escultura apaga a peça activa.
    Ours,
    /// Não é dela — e a razão vai para o log, porque *uma tecla que não faz nada e não diz nada
    /// é indistinguível de uma tecla que não chegou*.
    NotOurs(&'static str),
}

/// ⭐⭐⭐ **A LEI.** `dead_reason` é o que [`crate::App::sculpt3d_keys_dead_reason`] devolve
/// (vazio = as teclas estão vivas) e `over_panel` diz se o ponteiro está sobre um painel do
/// chrome.
///
/// ⚠️ **A ordem das duas perguntas é a feature:** um guarda morto explica-se com o guarda (é o
/// que o artista precisa de saber); só depois é que a **área** decide entre a escultura e a
/// linha da Hierarquia.
///
/// ⛔ **A área importa porque este teclado corre ANTES de toda a cadeia** (é o 2.º ramo do
/// `key_input`): sem ela, com barro na tela ele comia o `Delete` da Hierarquia — onde o mesh
/// agora tem linha —, do Flip, da timeline e do Painter.
pub(super) const fn claim_delete(dead_reason: &'static str, over_panel: bool) -> DeleteClaim {
    if !dead_reason.is_empty() {
        return DeleteClaim::NotOurs(dead_reason);
    }
    if over_panel {
        return DeleteClaim::NotOurs(
            "o ponteiro esta' sobre um painel -- quem manda ali e' a linha da Hierarquia",
        );
    }
    DeleteClaim::Ours
}

#[cfg(test)]
mod tests {
    use super::{DeleteClaim as C, claim_delete};

    /// ⭐⭐⭐ **GATE — com o barro na tela e o ponteiro no canvas, o `Delete` é da escultura.**
    #[test]
    fn no_canvas_com_barro_o_delete_e_da_escultura() {
        assert_eq!(claim_delete("", false), C::Ours);
    }

    /// ⭐⭐⭐ **GATE — sobre um painel, NÃO é** (é a linha da Hierarquia que manda), e a razão
    /// diz-lo.
    #[test]
    fn sobre_um_painel_a_linha_da_hierarquia_manda() {
        let C::NotOurs(porque) = claim_delete("", true) else {
            panic!("⛔ sobre um painel o Delete nao pode ser da escultura");
        };
        assert!(porque.contains("painel"), "{porque}");
    }

    /// ⭐⭐⭐ **GATE — um guarda morto explica-se COM O GUARDA, e ganha da área.**
    ///
    /// ⚠️ *A ordem é a feature:* dizer *«o ponteiro está sobre um painel»* a quem tem um campo
    /// de texto focado manda o artista para o sítio errado.
    #[test]
    fn o_guarda_morto_ganha_da_area_na_explicacao() {
        let razao = "um campo de texto esta' FOCADO -- clique fora dele e tente outra vez";
        assert_eq!(claim_delete(razao, false), C::NotOurs(razao));
        assert_eq!(
            claim_delete(razao, true),
            C::NotOurs(razao),
            "⛔ o guarda explica-se primeiro"
        );
    }
}
