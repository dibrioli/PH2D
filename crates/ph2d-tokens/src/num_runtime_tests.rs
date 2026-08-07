//! Gates da tabela achatada (plano UI/UX W4c.2).
//!
//! ⚠️ Estes testes partilham `thread_local`s com os da camada, e o `cargo test` corre a suíte em
//! paralelo — cada um limpa o que sujou (`clear_num_overrides` + `publish`), pelo mesmo motivo e
//! com o mesmo cuidado dos gates de `num_overrides`.

use super::*;
use crate::num_overrides::{NumValue, clear_num_overrides, set_num_override};
use crate::spacing::Spacing;
use crate::stroke::StrokeToken;

fn reset() {
    clear_num_overrides();
    publish(Theme::Forge);
}

#[test]
fn an_unauthored_scale_reads_the_factory_bit_for_bit() {
    reset();
    for tok in NumToken::ALL {
        assert_eq!(
            tok.px(Theme::Forge),
            tok.factory_px(),
            "{} devia valer a fabrica",
            tok.key()
        );
    }
    assert!(!is_filled(), "sem autoria a tabela nao e enchida");
    assert_eq!(Spacing::Md.px(), Spacing::Md.factory_px());
}

/// A wave inteira num teste: **autorar move o que os widgets lêem**, sem que nenhum deles saiba.
#[test]
fn publishing_makes_the_authored_value_the_one_the_widgets_read() {
    reset();
    let factory = Spacing::Md.factory_px();
    set_num_override(
        Theme::Forge,
        NumToken::Spacing(Spacing::Md),
        Some(NumValue::Literal(99.0)),
    )
    .expect("um literal nunca fecha um laco");

    // ⚠️ ANTES da publicação a leitura ainda é a de fábrica — a tabela é a forma de runtime, e
    // ela só existe depois de alguém a projetar. É este passo que torna o modo de falha
    // *lento-e-certo*: esquecer o `publish` deixa o app na fábrica, nunca num número errado.
    assert_eq!(Spacing::Md.px(), factory, "sem publicar, vale a fabrica");

    publish(Theme::Forge);
    assert_eq!(Spacing::Md.px(), 99.0);
    assert_eq!(
        Spacing::Md.factory_px(),
        factory,
        "a FABRICA nunca se move — ela e a tabela gerada"
    );
    reset();
}

/// ⚠️ O modo é perguntado **uma vez**, e é isto que a assinatura sem-tema significa.
#[test]
fn the_published_mode_decides_which_column_the_widgets_read() {
    reset();
    set_num_override(
        Theme::Forge,
        NumToken::Spacing(Spacing::Lg),
        Some(NumValue::Literal(77.0)),
    )
    .unwrap();

    publish(Theme::Forge);
    assert_eq!(Spacing::Lg.px(), 77.0);

    publish(Theme::Workshop);
    assert_eq!(
        Spacing::Lg.px(),
        Spacing::Lg.factory_px(),
        "autorar no Forge nao move o Workshop"
    );
    assert_eq!(published_mode(), Theme::Workshop);
    reset();
}

/// Um alias atravessa famílias porque a unidade é a mesma — e a tabela o resolve na projeção.
#[test]
fn an_alias_reaches_the_table_resolved() {
    reset();
    set_num_override(
        Theme::Forge,
        NumToken::Spacing(Spacing::Xs),
        Some(NumValue::Literal(13.0)),
    )
    .unwrap();
    set_num_override(
        Theme::Forge,
        NumToken::Stroke(StrokeToken::Thin),
        Some(NumValue::Alias(NumToken::Spacing(Spacing::Xs))),
    )
    .expect("px segue px");
    publish(Theme::Forge);
    assert_eq!(StrokeToken::Thin.px(), 13.0);
    reset();
}

/// **Soltar tudo devolve a fábrica** sem ninguém limpar o vector: a bandeira é a resposta.
#[test]
fn dropping_every_override_returns_the_factory_without_clearing_the_table() {
    reset();
    set_num_override(
        Theme::Forge,
        NumToken::Spacing(Spacing::Md),
        Some(NumValue::Literal(99.0)),
    )
    .unwrap();
    publish(Theme::Forge);
    assert!(is_filled());

    clear_num_overrides();
    publish(Theme::Forge);
    assert!(!is_filled(), "sem autoria a tabela deixa de valer");
    assert_eq!(Spacing::Md.px(), Spacing::Md.factory_px());
    reset();
}

/// O índice é gerado pela MESMA lista do `ALL` — um degrau novo não pode entrar num e faltar noutro.
#[test]
fn every_token_indexes_its_own_slot() {
    for (i, tok) in NumToken::ALL.iter().enumerate() {
        assert_eq!(tok.index(), i, "{} indexa o slot errado", tok.key());
    }
    assert_eq!(COUNT, NumToken::ALL.len());
}

/// A tabela de fábrica em `const` **é** a soma dos `factory_px` — não uma segunda cópia.
#[test]
fn the_const_factory_table_is_the_generated_scale() {
    for (i, tok) in NumToken::ALL.iter().enumerate() {
        assert_eq!(FACTORY[i], tok.factory_px(), "{}", tok.key());
    }
}
