//! ⭐ **A VOZ do módulo** — o que ele tem a dizer ao artista quando alguma coisa não deu
//! ([ADR-0161], plano W6: *"erro na UI"*).
//!
//! # Porque é UM canal e não um por assunto
//!
//! A W23 abriu o primeiro (a escultura que não voltou do arquivo) e a W25 precisava do segundo (a
//! peça que não cozinha). Dois canais paralelos teriam duas leis de repetição, dois drenos e dois
//! sítios onde alguém se esquece de drenar. Aqui há **um**, com uma lei:
//!
//! ⚠️ **O canal não repete a última coisa que disse.** O cozimento corre a cada quadro e uma peça
//! inválida **continua inválida** — sem isto seriam 60 avisos por segundo sobre a mesma frase, e a
//! tela ficaria ilegível exactamente quando o artista precisa de a ler. Uma frase **diferente**
//! passa sempre, e a primeira volta a passar depois dela: o que se recusa é a **repetição**, não a
//! frase.
//!
//! # ⚠️ As frases são para o ENIO, não para a próxima LLM
//!
//! Elas dizem **o que está errado na peça** — nunca o nome da variante, do campo ou do nó. Um
//! `ModsOnSampled { node: 7 }` no ecrã é a mesma coisa que silêncio para quem está a modelar.
//!
//! [ADR-0161]: ../../../docs/architecture/decisions/0161-3d-modeling-is-an-implicit-field-tree-and-what-the-artist-sees-is-the-traced-field.md

use ph2d_field::FieldError;

/// Diz uma coisa ao artista — **a menos que seja a mesma da última vez**.
pub(crate) fn say(msg: String) {
    let repeat = LAST.with(|l| l.borrow().as_deref() == Some(msg.as_str()));
    if repeat {
        return;
    }
    LAST.with(|l| *l.borrow_mut() = Some(msg.clone()));
    QUEUE.with(|q| q.borrow_mut().push(msg));
}

/// O mesmo para uma lista — o que a reconciliação da W23 devolve.
pub(crate) fn say_all(msgs: Vec<String>) {
    for m in msgs {
        say(m);
    }
}

/// O que há para dizer, tirado uma vez. Chamado pelo app, que é quem tem a fila de avisos.
pub(crate) fn drain() -> Vec<String> {
    QUEUE.with(|q| std::mem::take(&mut *q.borrow_mut()))
}

/// ⚠️ **A peça voltou a estar bem** — esquece a última frase, para que o mesmo problema, se voltar,
/// volte a ser dito. Sem isto um erro corrigido e recriado ficaria **mudo** na segunda vez.
pub(crate) fn clear() {
    LAST.with(|l| *l.borrow_mut() = None);
}

thread_local! {
    static QUEUE: std::cell::RefCell<Vec<String>> = const { std::cell::RefCell::new(Vec::new()) };
    static LAST: std::cell::RefCell<Option<String>> = const { std::cell::RefCell::new(None) };
}

/// ⭐ **O que dizer de um documento que o cozimento recusou.**
///
/// ⚠️ **Sem braço `_`, de propósito.** Um `match` com apanha-tudo compila para sempre e deixa a
/// variante NOVA sem frase — que é a forma mais silenciosa de uma mensagem apodrecer. Quem
/// acrescentar um erro ao documento vê este `match` a não compilar, que é o momento certo para
/// escolher as palavras.
pub(crate) fn explain(err: &FieldError) -> String {
    match err {
        FieldError::BadRoot => "This piece has nothing the model can start from".into(),
        FieldError::ForwardReference { .. } => {
            "Two parts of this piece point at each other in a loop".into()
        }
        FieldError::EmptyCombine { .. } => "An operation here has nothing left to combine".into(),
        FieldError::NonPositive { what, .. } => {
            format!("A shape here has a {what} of zero or less")
        }
        FieldError::RoundTooLarge { round, limit, .. } => {
            format!("The rounding here ({round:.3}) is bigger than the shape can take ({limit:.3})")
        }
        FieldError::BadScale { .. } => "A shape here has an impossible size".into(),
        FieldError::ProfileCrossesAxis { .. } => {
            "The drawn profile crosses the axis it turns around".into()
        }
        FieldError::EmptySampledKey { .. } => "A sculpture here has no file behind it".into(),
        FieldError::ModsOnSampled { .. } => {
            "A sculpture cannot take shell, offset, mirror or the other modifiers".into()
        }
    }
}

#[cfg(test)]
pub(crate) fn forget_last() {
    clear();
}

#[cfg(test)]
#[path = "field3d_notice_tests.rs"]
mod tests;
