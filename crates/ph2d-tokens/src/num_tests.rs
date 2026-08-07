//! Os gates da IDENTIDADE numérica — a lista, as chaves, e a fábrica.

use super::NumToken;
use crate::color::ColorToken;
use crate::theme::Theme;
use crate::{Radius, Spacing, StrokeToken};

const MODES: [Theme; 4] = [
    Theme::Forge,
    Theme::Workshop,
    Theme::Sunstone,
    Theme::Blueprint,
];

/// A lista cobre as TRÊS famílias inteiras — se um degrau saísse dela, o `key` não compilava, mas
/// nada impediria a lista de conter *menos* famílias do que o enum tem.
#[test]
fn the_list_holds_every_step_of_every_family() {
    let n = NumToken::ALL.len();
    assert_eq!(
        n,
        9 + 7 + 5,
        "a lista tem de cobrir spacing(9) + radius(7) + stroke(5), e tem {n}"
    );
    for s in [
        Spacing::Xxs,
        Spacing::Xs,
        Spacing::Sm,
        Spacing::Md,
        Spacing::Lg,
        Spacing::Xl,
        Spacing::Xl2,
        Spacing::Xl3,
        Spacing::Xl4,
    ] {
        assert!(NumToken::ALL.contains(&NumToken::Spacing(s)), "falta {s:?}");
    }
    for r in [
        Radius::Xs,
        Radius::Sm,
        Radius::Md,
        Radius::Lg,
        Radius::Xl,
        Radius::Xl2,
        Radius::Full,
    ] {
        assert!(NumToken::ALL.contains(&NumToken::Radius(r)), "falta {r:?}");
    }
    for s in [
        StrokeToken::Hairline,
        StrokeToken::Thin,
        StrokeToken::Default,
        StrokeToken::Thick,
        StrokeToken::Heavy,
    ] {
        assert!(NumToken::ALL.contains(&NumToken::Stroke(s)), "falta {s:?}");
    }
}

#[test]
fn every_key_is_unique() {
    let mut keys: Vec<&str> = NumToken::ALL.iter().map(|t| t.key()).collect();
    keys.sort_unstable();
    let before = keys.len();
    keys.dedup();
    assert_eq!(before, keys.len(), "duas linhas partilham uma chave");
}

#[test]
fn from_key_is_the_inverse_of_key() {
    for &t in NumToken::ALL {
        assert_eq!(NumToken::from_key(t.key()), Some(t), "{}", t.key());
    }
    assert_eq!(NumToken::from_key("spacing.nope"), None);
    assert_eq!(NumToken::from_key(""), None);
}

/// ⚠️ **O gate que torna o slot PARTILHADO do arquivo seguro.** As duas famílias viajam na mesma
/// lista de tokens autorados (`ProjectFile.tokens`), e o load decide de quem é uma entrada pela
/// CHAVE. Se uma chave fosse aceite pelas duas, o load teria de escolher um vencedor que ninguém
/// especificou — e a escolha seria silenciosa.
#[test]
fn no_key_is_claimed_by_both_families() {
    for &t in NumToken::ALL {
        assert!(
            ColorToken::from_key(t.key()).is_none(),
            "a chave {:?} e' reclamada pelas DUAS familias",
            t.key()
        );
    }
    for &c in ColorToken::ALL {
        assert!(
            NumToken::from_key(c.key()).is_none(),
            "a chave {:?} e' reclamada pelas DUAS familias",
            c.key()
        );
    }
}

/// A fábrica do wrapper **é** a do `px()` de cada família — não uma segunda tabela ao lado.
#[test]
fn the_factory_is_the_const_fn_of_each_family() {
    for &t in NumToken::ALL {
        let direct = match t {
            NumToken::Spacing(s) => s.px(),
            NumToken::Radius(r) => r.px(),
            NumToken::Stroke(s) => s.px(),
        };
        assert_eq!(t.factory_px().to_bits(), direct.to_bits(), "{}", t.key());
    }
}

/// **A INÉRCIA** — o gate que prova que a camada é gratuita para quem nunca autorou nada.
///
/// ⚠️ Comparação por BITS: `==` em `f32` daria `NaN != NaN` por verdadeiro e um `-0.0` por igual a
/// `0.0`. O que se afirma é *o mesmo número*, não *um número parecido*.
#[test]
fn an_empty_layer_reads_the_factory_bit_for_bit_in_every_mode() {
    crate::num_overrides::clear_num_overrides();
    for mode in MODES {
        for &t in NumToken::ALL {
            assert_eq!(
                t.px(mode).to_bits(),
                t.factory_px().to_bits(),
                "{} em {mode:?}",
                t.key()
            );
        }
    }
}

/// O acessor de cada família chega ao mesmo número que o `NumToken` — um delegate, não uma
/// segunda rota.
///
/// ⚠️ Reescrito na W4c.2: era `px_live(mode)` contra `NumToken::px(mode)`. Hoje o de cada família
/// **não recebe modo** (lê a tabela publicada), então o par que tem de concordar é
/// `Spacing::px()` contra `NumToken::px(modo_publicado)` — e é justamente a publicação que
/// estabelece a premissa que o teste antigo recebia de graça.
#[test]
fn the_family_accessor_is_the_same_answer_as_the_token() {
    crate::num_overrides::clear_num_overrides();
    for mode in MODES {
        crate::num_runtime::publish(mode);
        assert_eq!(
            Spacing::Md.px().to_bits(),
            NumToken::Spacing(Spacing::Md).px(mode).to_bits()
        );
        assert_eq!(
            Radius::Lg.px().to_bits(),
            NumToken::Radius(Radius::Lg).px(mode).to_bits()
        );
        assert_eq!(
            StrokeToken::Thin.px().to_bits(),
            NumToken::Stroke(StrokeToken::Thin).px(mode).to_bits()
        );
    }
    crate::num_runtime::publish(Theme::Forge);
}

/// A fábrica de um número **não tem modo** — é o único ponto em que esta camada difere da de cor,
/// e ele está pinado para que a wave que o mudasse tivesse de o dizer.
#[test]
fn the_factory_of_a_number_is_the_same_in_every_mode() {
    crate::num_overrides::clear_num_overrides();
    for &t in NumToken::ALL {
        let forge = t.px(Theme::Forge).to_bits();
        for mode in MODES {
            assert_eq!(t.px(mode).to_bits(), forge, "{} em {mode:?}", t.key());
        }
    }
}
