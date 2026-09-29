//! **O REGISTO da costura do Painter na peça** (`PH2D_PAINTER3D_LOG=1`) — irmão
//! (`#[path]`) do [`super`] pelo tecto de LOC.
//!
//! Report do dono (29/09): *«entre a primeira e segunda pincelada, a tinta seca;
//! depois da terceira funciona»*. O arnês de teste NÃO o reproduz (três traços,
//! com e sem pausa, a água nunca seca e o `edits` não se mexe) ⇒ a diferença
//! vive em algo que só o app real faz, e a cura para isso é um INSTRUMENTO e
//! não uma hipótese: cada linha diz o que a costura decidiu e PORQUÊ, e o
//! quadro em que a água morre é nomeado com o que se passou nele.
//!
//! ⚠️ Terminal só, lido UMA vez por processo — e cada frase vive DENTRO de um `eprintln!` no
//! chamador: o censo de texto da UI só isenta o terminal quando o literal está lá dentro.

use std::cell::Cell;
use std::sync::OnceLock;

/// O registo está ligado?
pub(crate) fn ligado() -> bool {
    static ON: OnceLock<bool> = OnceLock::new();
    *ON.get_or_init(|| std::env::var_os("PH2D_PAINTER3D_LOG").is_some())
}

thread_local! {
    static ESTAVA_MOLHADA: Cell<bool> = const { Cell::new(false) };
}

/// Fim de quadro: `Some(true)` se a água estava viva no quadro anterior e morreu neste,
/// `Some(false)` se nasceu, `None` sem mudança ou com o registo desligado. ⚠️ Quem escreve a linha
/// é o chamador, DENTRO de um `eprintln!` — o censo de texto só isenta o terminal assim.
pub(crate) fn transicao(molhada: bool) -> Option<bool> {
    if !ligado() {
        return None;
    }
    let antes = ESTAVA_MOLHADA.with(|c| c.replace(molhada));
    (antes != molhada).then_some(antes)
}
