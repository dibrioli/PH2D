//! Os gates da CAMADA numérica.
//!
//! ⚠️ Cada teste corre na própria thread, e a camada é `thread_local` — é isso que deixa um gate
//! armar um override sem envenenar o vizinho numa suíte paralela (o motivo escrito na
//! [`crate::overrides`], e ele vale igual aqui).

use super::{
    AuthoredNum, NumOverride, NumRefusal, NumValue, clear_num_overrides, num_overridden_count,
    num_override, num_overrides, resolved_num_override, set_num_override, set_num_overrides,
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
    // ⚠️ E o acessor da família chega ao mesmo número — depois de publicado, que é onde
    // a W4c.2 pôs a resposta a *"qual e o modo vigente?"*.
    crate::num_runtime::publish(Theme::Forge);
    assert_eq!(Radius::Md.px(), 9.0);
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

// ── A MATH (plano UI/UX W4c.3) ──────────────────────────────────────────────
//
// ⚠️ Estes gates instalam um host de BRINQUEDO, e é deliberado: o que eles medem é a CAMADA (a
// porta admite? a leitura resolve? o laço é recusado?), não a linguagem. Instalar o parser real
// faria esta crate depender do substrato de grafo por causa de um teste — a aresta que o
// `num_expr` inteiro existe para não ter.

/// Um host cuja "fórmula" é o nome de um token: `"md"` lê o `MD`, `"lg"` lê o `LG`, e o valor é o
/// DOBRO do que ele valer. Qualquer outro texto é recusado.
fn install_toy_math() {
    crate::num_expr::install_math(crate::num_expr::MathHost {
        deps: |src| match src {
            "md" => Ok(vec![NumToken::Spacing(Spacing::Md)]),
            "lg" => Ok(vec![NumToken::Spacing(Spacing::Lg)]),
            "md+lg" => Ok(vec![
                NumToken::Spacing(Spacing::Md),
                NumToken::Spacing(Spacing::Lg),
            ]),
            "negative" => Ok(Vec::new()),
            _ => Err("nope".to_string()),
        },
        eval: |src, value_of| match src {
            "md" => Ok(value_of(NumToken::Spacing(Spacing::Md)) * 2.0),
            "lg" => Ok(value_of(NumToken::Spacing(Spacing::Lg)) * 2.0),
            "md+lg" => {
                Ok(value_of(NumToken::Spacing(Spacing::Md))
                    + value_of(NumToken::Spacing(Spacing::Lg)))
            }
            "negative" => Ok(-1.0),
            _ => Err("nope".to_string()),
        },
    });
}

fn expr(theme: Theme, token: NumToken, src: &str) -> Result<(), NumRefusal> {
    set_num_override(theme, token, Some(NumValue::Expr(src.to_string())))
}

#[test]
fn a_formula_resolves_to_the_number_it_computes() {
    clear_num_overrides();
    install_toy_math();
    lit(Theme::Forge, MD, 10.0);
    expr(Theme::Forge, LG, "md").expect("uma formula valida entra");
    assert_eq!(LG.px(Theme::Forge), 20.0);
    crate::num_expr::uninstall_math();
}

/// **A fórmula lê o valor EFETIVO da dependência, não a fábrica dela** — é isso que a torna uma
/// relação viva em vez de uma cópia, a mesma lei do alias.
#[test]
fn a_formula_follows_its_dependency_when_it_changes() {
    clear_num_overrides();
    install_toy_math();
    expr(Theme::Forge, LG, "md").expect("entra");
    let from_factory = LG.px(Theme::Forge);
    assert_eq!(from_factory, MD.factory_px() * 2.0);
    lit(Theme::Forge, MD, 100.0);
    assert_eq!(
        LG.px(Theme::Forge),
        200.0,
        "a formula ficou presa no valor de quando foi escrita — ela virou uma copia"
    );
    crate::num_expr::uninstall_math();
}

/// **Uma fórmula que se lê a si mesma é um laço de comprimento um**, e a porta a recusa.
#[test]
fn a_formula_that_reads_its_own_token_is_refused() {
    clear_num_overrides();
    install_toy_math();
    let err = expr(Theme::Forge, MD, "md").expect_err("auto-referencia tem de ser recusada");
    assert!(matches!(err, NumRefusal::Cycle { .. }), "{err:?}");
    assert_eq!(num_override(Theme::Forge, MD), None, "o slot foi escrito");
    crate::num_expr::uninstall_math();
}

/// **O laço por um RAMO do fan-out também é um laço** — o caso que a caminhada de corrente não
/// sabia fazer, e a razão de a lei do ciclo ter passado a ser uma DFS.
#[test]
fn a_loop_down_one_branch_of_a_formula_is_refused() {
    clear_num_overrides();
    install_toy_math();
    // LG segue XL; a fórmula em XL lê MD **e** LG ⇒ o ramo do LG volta ao XL.
    set_num_override(Theme::Forge, LG, Some(NumValue::Alias(XL))).expect("alias");
    let err = expr(Theme::Forge, XL, "md+lg").expect_err("o ramo do LG fecha o laco");
    assert!(matches!(err, NumRefusal::Cycle { .. }), "{err:?}");
    crate::num_expr::uninstall_math();
}

/// **Uma fórmula que não parseia é recusada COM A FRASE** — e o slot fica como estava.
#[test]
fn an_unparseable_formula_is_refused_with_its_sentence() {
    clear_num_overrides();
    install_toy_math();
    lit(Theme::Forge, MD, 7.0);
    let err = expr(Theme::Forge, MD, "?!").expect_err("texto que o host recusa");
    assert!(
        matches!(err, NumRefusal::BadFormula(ref s) if s == "nope"),
        "{err:?}"
    );
    assert_eq!(
        MD.px(Theme::Forge),
        7.0,
        "a recusa apagou o valor que ja' estava la'"
    );
    crate::num_expr::uninstall_math();
}

/// **Uma fórmula cujo resultado não é um comprimento é recusada** — a mesma lei do literal, e não
/// um caso à parte: o que a porta promete é que a tabela só carrega comprimentos.
#[test]
fn a_formula_that_does_not_compute_a_length_is_refused() {
    clear_num_overrides();
    install_toy_math();
    let err = expr(Theme::Forge, MD, "negative").expect_err("-1 nao e' um comprimento");
    assert!(
        matches!(err, NumRefusal::NotALength(v) if v < 0.0),
        "{err:?}"
    );
    crate::num_expr::uninstall_math();
}

/// **Sem host de math nenhuma fórmula entra** — nem por gesto, nem de um arquivo.
///
/// ⚠️ O modo de falha é RECUSAR, nunca dobrar num número: um valor inventado seria indistinguível
/// de um autorado, que é a rachura que o `Bindings` do IR tem e que esta camada não herda.
#[test]
fn without_math_a_formula_never_enters_the_table() {
    clear_num_overrides();
    crate::num_expr::uninstall_math();
    assert!(expr(Theme::Forge, MD, "md").is_err());
    assert_eq!(num_override(Theme::Forge, MD), None);
    // E pelo caminho do ARQUIVO: ela é descartada, e o descarte é CONTADO.
    let dropped = set_num_overrides(vec![NumOverride {
        theme: Theme::Forge,
        token: MD,
        value: NumValue::Expr("md".to_string()),
    }]);
    assert_eq!(dropped, 1, "a formula entrou numa tabela sem quem a leia");
    assert_eq!(num_override(Theme::Forge, MD), None);
}

/// **O arquivo carrega a fórmula, e ela resolve depois de carregada.**
#[test]
fn a_formula_survives_the_round_trip_through_the_loader() {
    clear_num_overrides();
    install_toy_math();
    lit(Theme::Forge, MD, 10.0);
    expr(Theme::Forge, LG, "md").expect("entra");
    let saved = num_overrides();
    clear_num_overrides();
    assert_eq!(set_num_overrides(saved), 0, "o round-trip descartou algo");
    assert_eq!(LG.px(Theme::Forge), 20.0);
    crate::num_expr::uninstall_math();
}

/// **Uma tabela que chega CÍCLICA de um arquivo cai na fábrica em vez de girar** — a rede de
/// profundidade, agora atravessada por uma fórmula.
#[test]
fn a_formula_cycle_that_slipped_past_the_door_falls_back_to_the_factory() {
    clear_num_overrides();
    install_toy_math();
    // Instalado à FORÇA, sem passar pela porta — é o que um arquivo editado à mão produz.
    let dropped = set_num_overrides(vec![
        NumOverride {
            theme: Theme::Forge,
            token: MD,
            value: NumValue::Expr("lg".to_string()),
        },
        NumOverride {
            theme: Theme::Forge,
            token: LG,
            value: NumValue::Expr("md".to_string()),
        },
    ]);
    // O loader recusa a SEGUNDA (a 1ª ainda não tinha com que fechar laço), e é isso que o torna
    // acíclico por construção.
    assert_eq!(dropped, 1);
    assert!(MD.px(Theme::Forge).is_finite(), "a leitura nao terminou");
    crate::num_expr::uninstall_math();
}
