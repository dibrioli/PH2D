//! Os gates da CAMADA numérica.
//!
//! ⚠️ Cada teste corre na própria thread, e a camada é `thread_local` — é isso que deixa um gate
//! armar um override sem envenenar o vizinho numa suíte paralela (o motivo escrito na
//! [`crate::overrides`], e ele vale igual aqui).

use super::{
    AuthoredNum, NumOverride, NumRefusal, NumValue, clear_num_overrides, num_override,
    num_overridden_count, num_overrides, resolved_num_override, set_num_override,
    set_num_overrides,
};
use crate::num::NumToken;
use crate::theme::Theme;
use crate::{Radius, Spacing, StrokeToken};

const MD: NumToken = NumToken::Spacing(Spacing::Md);
const LG: NumToken = NumToken::Spacing(Spacing::Lg);
const XL: NumToken = NumToken::Spacing(Spacing::Xl);
const R_MD: NumToken = NumToken::Radius(Radius::Md);
const S_THIN: NumToken = NumToken::Stroke(StrokeToken::Thin);

fn lit(theme: Theme, token: NumToken, v: f32) {
    set_num_override(theme, token, Some(NumValue::Literal(v))).expect("um literal valido entra");
}

#[test]
fn a_literal_wins_over_the_factory() {
    clear_num_overrides();
    lit(Theme::Forge, MD, 13.0);
    assert_eq!(MD.px(Theme::Forge), 13.0);
    assert_eq!(
        resolved_num_override(Theme::Forge, MD),
        Some(AuthoredNum::Px(13.0))
    );
}

/// **A chave é o PAR `(modo, token)`** — autorar num modo não move os outros três.
#[test]
fn authoring_one_mode_leaves_the_others_at_the_factory() {
    clear_num_overrides();
    lit(Theme::Forge, MD, 13.0);
    assert_eq!(MD.px(Theme::Forge), 13.0);
    for other in [Theme::Workshop, Theme::Sunstone, Theme::Blueprint] {
        assert_eq!(
            MD.px(other).to_bits(),
            MD.factory_px().to_bits(),
            "{other:?} nao devia ter mudado"
        );
    }
}

#[test]
fn an_alias_follows_its_target_in_the_same_mode() {
    clear_num_overrides();
    lit(Theme::Forge, LG, 20.0);
    set_num_override(Theme::Forge, MD, Some(NumValue::Alias(LG))).expect("elo legal");
    assert_eq!(MD.px(Theme::Forge), 20.0);
    // Mover o ALVO move quem o segue — o vinculo e' seguido na LEITURA, nunca achatado na escrita.
    lit(Theme::Forge, LG, 30.0);
    assert_eq!(MD.px(Theme::Forge), 30.0);
}

/// Uma cadeia que termina num token **não autorado** vale a fábrica **dele**, não a de quem começou.
#[test]
fn a_chain_that_ends_unauthored_takes_that_tokens_factory() {
    clear_num_overrides();
    set_num_override(Theme::Forge, MD, Some(NumValue::Alias(XL))).expect("elo legal");
    assert_eq!(
        resolved_num_override(Theme::Forge, MD),
        Some(AuthoredNum::Factory(XL))
    );
    assert_eq!(MD.px(Theme::Forge).to_bits(), XL.factory_px().to_bits());
    assert_ne!(MD.px(Theme::Forge).to_bits(), MD.factory_px().to_bits());
}

/// **Um alias atravessa FAMÍLIAS** — a unidade é a mesma (px), então o grafo é um só.
#[test]
fn an_alias_crosses_families_because_the_unit_is_the_same() {
    clear_num_overrides();
    lit(Theme::Forge, MD, 9.0);
    set_num_override(Theme::Forge, R_MD, Some(NumValue::Alias(MD))).expect("elo legal");
    assert_eq!(R_MD.px(Theme::Forge), 9.0);
    assert_eq!(Radius::Md.px_live(Theme::Forge), 9.0);
}

#[test]
fn a_self_alias_is_refused_at_the_door() {
    clear_num_overrides();
    let err = set_num_override(Theme::Forge, MD, Some(NumValue::Alias(MD))).unwrap_err();
    assert_eq!(
        err,
        NumRefusal::Cycle {
            token: MD,
            target: MD,
            at: MD
        }
    );
    assert_eq!(num_override(Theme::Forge, MD), None, "nada foi escrito");
}

#[test]
fn a_longer_cycle_is_refused_and_says_where_it_closes() {
    clear_num_overrides();
    set_num_override(Theme::Forge, LG, Some(NumValue::Alias(XL))).expect("elo legal");
    // MD -> LG -> XL: fechar XL -> MD nao e' laco; fechar XL -> LG e'.
    let err = set_num_override(Theme::Forge, XL, Some(NumValue::Alias(LG))).unwrap_err();
    assert_eq!(
        err,
        NumRefusal::Cycle {
            token: XL,
            target: LG,
            at: XL
        }
    );
}

/// O ciclo é por MODO: o mesmo elo pode ser legal noutro modo, porque lá a cadeia é outra.
#[test]
fn the_cycle_check_is_per_mode() {
    clear_num_overrides();
    set_num_override(Theme::Forge, LG, Some(NumValue::Alias(MD))).expect("elo legal");
    // No Forge, MD -> LG fecha. No Workshop nao ha' cadeia nenhuma.
    assert!(set_num_override(Theme::Forge, MD, Some(NumValue::Alias(LG))).is_err());
    assert!(set_num_override(Theme::Workshop, MD, Some(NumValue::Alias(LG))).is_ok());
}

/// ⚠️ **Um número que não é um comprimento é RECUSADO, e a recusa diz qual era.**
#[test]
fn a_value_that_is_not_a_length_is_refused() {
    clear_num_overrides();
    for bad in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY, -1.0, -0.001] {
        let err = set_num_override(Theme::Forge, MD, Some(NumValue::Literal(bad))).unwrap_err();
        assert!(
            matches!(err, NumRefusal::NotALength(_)),
            "{bad} devia ser recusado"
        );
    }
    assert_eq!(num_override(Theme::Forge, MD), None, "nada foi escrito");
    // Zero E' um comprimento: um espacamento de 0 e' uma escolha (colar duas coisas), nao um erro.
    assert!(set_num_override(Theme::Forge, MD, Some(NumValue::Literal(0.0))).is_ok());
}

/// Resetar é `None`, **nunca escrever o valor de fábrica**: os dois lêem igual na tela e só um
/// deixa o token seguir uma futura edição do `tokens.json`.
#[test]
fn resetting_releases_the_slot_instead_of_freezing_the_factory_value() {
    clear_num_overrides();
    lit(Theme::Forge, MD, 13.0);
    set_num_override(Theme::Forge, MD, None).expect("soltar nunca falha");
    assert_eq!(num_override(Theme::Forge, MD), None);
    assert_eq!(num_overridden_count(Theme::Forge), 0);
}

#[test]
fn the_count_is_per_mode() {
    clear_num_overrides();
    lit(Theme::Forge, MD, 13.0);
    lit(Theme::Forge, R_MD, 5.0);
    lit(Theme::Workshop, S_THIN, 2.0);
    assert_eq!(num_overridden_count(Theme::Forge), 2);
    assert_eq!(num_overridden_count(Theme::Workshop), 1);
    assert_eq!(num_overridden_count(Theme::Sunstone), 0);
}

/// A lista sai em ordem CANÔNICA — dois documentos logicamente iguais dão os mesmos bytes.
#[test]
fn the_list_is_sorted_by_mode_then_key() {
    clear_num_overrides();
    lit(Theme::Workshop, S_THIN, 2.0);
    lit(Theme::Forge, R_MD, 5.0);
    lit(Theme::Forge, MD, 13.0);
    let got: Vec<(u8, &str)> = num_overrides()
        .iter()
        .map(|e| (e.theme as u8, e.token.key()))
        .collect();
    let mut want = got.clone();
    want.sort_unstable();
    assert_eq!(got, want, "a ordem depende da ordem dos cliques");
}

/// O load **descarta** o que não pode entrar e **conta** — uma tabela que encolhe em silêncio
/// lê-se como *"eu nunca autorei isto"*.
#[test]
fn installing_a_table_drops_what_it_cannot_accept_and_counts_it() {
    clear_num_overrides();
    let dropped = set_num_overrides(vec![
        NumOverride {
            theme: Theme::Forge,
            token: MD,
            value: NumValue::Literal(13.0),
        },
        // Um laco vindo de um arquivo editado a' mao.
        NumOverride {
            theme: Theme::Forge,
            token: LG,
            value: NumValue::Alias(LG),
        },
        // Um numero que nao e' comprimento.
        NumOverride {
            theme: Theme::Forge,
            token: XL,
            value: NumValue::Literal(-3.0),
        },
    ]);
    assert_eq!(dropped, 2);
    assert_eq!(MD.px(Theme::Forge), 13.0);
    assert_eq!(num_override(Theme::Forge, LG), None);
    assert_eq!(num_override(Theme::Forge, XL), None);
}

/// ⚠️ **A segunda camada.** A porta não deixa um laço nascer, então isto só é alcançável por uma
/// tabela corrompida por fora — e a resposta é cair na fábrica, nunca girar.
#[test]
fn a_corrupt_cyclic_table_falls_back_to_the_factory_instead_of_spinning() {
    clear_num_overrides();
    // Instalado DIRECTAMENTE no armazem, sem passar pela porta: e' o unico jeito de ter o estado
    // que a porta promete impedir.
    super::OVERRIDES.with(|o| {
        *o.borrow_mut() = vec![
            NumOverride {
                theme: Theme::Forge,
                token: MD,
                value: NumValue::Alias(LG),
            },
            NumOverride {
                theme: Theme::Forge,
                token: LG,
                value: NumValue::Alias(MD),
            },
        ];
    });
    super::ANY.with(|a| a.set(true));
    assert_eq!(resolved_num_override(Theme::Forge, MD), None);
    assert_eq!(MD.px(Theme::Forge).to_bits(), MD.factory_px().to_bits());
}

/// ⚠️ O flag rápido é PRÓPRIO: mexer na camada de COR não pode pôr a escala no caminho lento, nem
/// o contrário. O oráculo é o valor, que é o que o caminho lento e o rápido têm de partilhar.
#[test]
fn the_two_layers_do_not_share_their_fast_path_flag() {
    clear_num_overrides();
    crate::overrides::clear_color_overrides();
    crate::overrides::set_color_override(
        Theme::Forge,
        crate::ColorToken::Accent,
        Some(crate::overrides::TokenValue::Literal(crate::color::Color {
            r: 1,
            g: 2,
            b: 3,
            a: 255,
        })),
    )
    .expect("literal");
    // A camada numerica continua vazia — e o valor numerico continua o de fabrica.
    assert_eq!(num_overridden_count(Theme::Forge), 0);
    assert_eq!(MD.px(Theme::Forge).to_bits(), MD.factory_px().to_bits());
    // E o inverso: autorar um numero nao mexe na cor.
    lit(Theme::Forge, MD, 13.0);
    assert_eq!(
        crate::ColorToken::Accent.resolve(Theme::Forge),
        crate::color::Color {
            r: 1,
            g: 2,
            b: 3,
            a: 255
        }
    );
}
