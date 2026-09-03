//! ⭐⭐ **O padrão do app é o que o DONO escolheu, e há UM sítio a dizê-lo.**
//!
//! Enio, 2026-09-02, depois de ver os seis desenhos lado a lado:
//! *"O padrão do APP deverá ser Sliders tipo **Underline**, **raio 4**, **linha 22**."*
//!
//! ⛔⛔ **Por que este gate existe:** o valor vive em DOIS sítios que a linguagem não deixa unificar
//! — o [`SliderStyle::default()`](ph2d_tokens::SliderStyle) e o `thread_local!` `const`-inicializado
//! do [`paint::slider_style`](ph2d_editor_core::paint::slider_style), que **não pode** chamar
//! `Default::default()` num contexto `const`. *Duas escritas do mesmo default é exactamente a forma
//! que diverge em silêncio* — e o modo de falha é caro de um jeito específico: o app arrancaria com
//! uma aparência e o painel de customização mostraria outra como «actual».

use ph2d_tokens::{Density, Radius, SliderDesign, SliderStyle};

/// **As duas escritas do default concordam.**
#[test]
fn the_paint_default_matches_the_token_default() {
    // ⚠️ Sem `set_slider_style` antes: o que se mede é o estado com que a thread NASCE.
    let live = ph2d_editor_core::paint::slider_style();
    assert_eq!(
        live,
        SliderStyle::default(),
        "o `thread_local!` do `paint.rs` e o `SliderStyle::default()` divergiram — o app arranca \
         com uma aparencia e o painel de customizacao mostra outra como «actual»"
    );
}

/// **E o default É o que o Enio escolheu**, escrito por nome.
///
/// ⚠️ Ele afirma os TRÊS eixos separadamente de propósito: um `assert_eq!` contra
/// `SliderStyle::default()` seria uma tautologia — mediria a struct contra ela mesma e passaria
/// depois de alguém trocar o desenho padrão.
#[test]
fn the_default_is_underline_radius_four_row_twentytwo() {
    let d = SliderStyle::default();
    assert_eq!(d.design, SliderDesign::Underline, "o desenho padrao mudou");
    assert_eq!(d.radius, Radius::Xs, "o raio padrao mudou");
    assert_eq!(
        d.density,
        Density::Compact,
        "a altura de linha padrao mudou"
    );
    // ⭐ E os NÚMEROS que ele nomeou, contra os tokens — se um token mudar de valor, isto acusa.
    assert!(
        (d.radius_px() - 4.0).abs() < f32::EPSILON,
        "o Enio escolheu raio 4 e o token `Radius::Xs` vale agora {}",
        d.radius_px()
    );
    assert!(
        (d.row_h_px() - 22.0).abs() < f32::EPSILON,
        "o Enio escolheu linha 22 e o token `Density::Compact` vale agora {}",
        d.row_h_px()
    );
}

/// ⛔ **A customização oferece QUATRO desenhos — nem cinco, nem três.**
///
/// A `Notch` e a `Split` foram construídas, vistas e **não escolhidas** (o porquê de cada uma está
/// no `slider_style.rs`). ⚠️ A `Split` em especial não é «uma opção a menos»: ela é o desenho de
/// duas colunas, e tê-la na lista deixaria o artista escolher de volta os `154 px` de cromo que o
/// redesenho inteiro existe para apagar.
#[test]
fn the_customisation_offers_exactly_the_four_chosen_designs() {
    let names: Vec<&str> = SliderDesign::ALL.iter().map(|d| d.label()).collect();
    assert_eq!(
        names,
        vec!["Underline", "Bar", "Inset", "Ghost"],
        "a lista de desenhos mudou — se foi decisao do Enio, actualize tambem o `slider_style.rs`, \
         que e' onde as duas recusas medidas (Notch, Split) estao registadas"
    );
    assert_eq!(
        SliderDesign::ALL[0],
        SliderStyle::default().design,
        "a primeira opcao do selector nao e' o padrao — o olho pousa na primeira, e po^-la a \
         discordar do default faz o artista pensar que mudou algo"
    );
}

/// **Toda a UI desta feature está em INGLÊS** (regra do app).
///
/// ⚠️ O laboratório nasceu com os rótulos em português e o Enio apanhou-o na primeira foto
/// (*"obviamente tudo em inglês"*). Isto varre as strings que o artista LÊ.
#[test]
fn every_user_facing_string_is_english() {
    let mut bad = Vec::new();
    for d in SliderDesign::ALL {
        for s in [d.label(), d.blurb()] {
            if s.chars().any(|c| "ãõçáéíóúâêôàÃÕÇÁÉÍÓÚÂÊÔÀ".contains(c)) {
                bad.push(s);
            }
        }
    }
    for st in ph2d_editor_core::widget::PropertyBoxState::ALL {
        if st.label().chars().any(|c| "ãõçáéíóúâêôà".contains(c)) {
            bad.push(st.label());
        }
    }
    assert!(
        bad.is_empty(),
        "estas strings sao lidas pelo artista e nao estao em ingles: {bad:?}"
    );
}
