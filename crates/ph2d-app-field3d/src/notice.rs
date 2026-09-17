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
pub fn say(msg: String) {
    let repeat = LAST.with(|l| l.borrow().as_deref() == Some(msg.as_str()));
    if repeat {
        return;
    }
    LAST.with(|l| *l.borrow_mut() = Some(msg.clone()));
    QUEUE.with(|q| q.borrow_mut().push(msg));
}

/// O mesmo para uma lista — o que a reconciliação da W23 devolve.
pub fn say_all(msgs: Vec<String>) {
    for m in msgs {
        say(m);
    }
}

/// O que há para dizer, tirado uma vez. Chamado pelo app, que é quem tem a fila de avisos.
pub fn drain() -> Vec<String> {
    QUEUE.with(|q| std::mem::take(&mut *q.borrow_mut()))
}

/// ⚠️ **A peça voltou a estar bem** — esquece a última frase, para que o mesmo problema, se voltar,
/// volte a ser dito. Sem isto um erro corrigido e recriado ficaria **mudo** na segunda vez.
pub fn clear() {
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
pub fn explain(err: &FieldError) -> String {
    match err {
        FieldError::BadRoot => {
            ph2d_i18n::tr("app.field3d.notice.this_piece_has_nothing_the_model_can_start_from")
                .into()
        }
        FieldError::ForwardReference { .. } => {
            ph2d_i18n::tr("app.field3d.notice.two_parts_of_this_piece_point_at_each_other_in_a")
                .into()
        }
        FieldError::EmptyCombine { .. } => {
            ph2d_i18n::tr("app.field3d.notice.an_operation_here_has_nothing_left_to_combine").into()
        }
        FieldError::NonPositive { what, .. } => ph2d_i18n::tr_with(
            "app.field3d.notice.a_shape_here_has_a_of_zero_or_less",
            &[("what", &what)],
        ),
        FieldError::RoundTooLarge { round, limit, .. } => ph2d_i18n::tr_with(
            "app.field3d.notice.the_rounding_here_is_bigger_than_the_shape_can_t",
            &[
                ("round_3", &format!("{:.3}", round)),
                ("limit_3", &format!("{:.3}", limit)),
            ],
        ),
        FieldError::BadScale { .. } => {
            ph2d_i18n::tr("app.field3d.notice.a_shape_here_has_an_impossible_size").into()
        }
        FieldError::ProfileCrossesAxis { .. } => {
            ph2d_i18n::tr("app.field3d.notice.the_drawn_profile_crosses_the_axis_it_turns_arou")
                .into()
        }
        FieldError::EmptySampledKey { .. } => {
            ph2d_i18n::tr("app.field3d.notice.a_sculpture_here_has_no_file_behind_it").into()
        }
        FieldError::ModsOnSampled { .. } => {
            ph2d_i18n::tr("app.field3d.notice.a_sculpture_cannot_take_shell_offset_mirror_or_t")
                .into()
        }
    }
}

#[cfg(test)]
pub fn forget_last() {
    clear();
}

#[cfg(test)]
#[path = "notice_tests.rs"]
mod tests;
