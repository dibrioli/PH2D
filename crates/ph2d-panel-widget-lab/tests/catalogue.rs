//! O catálogo de desenhos é percorrível e completo.
//!
//! ⚠️ **A pergunta que estes gates fazem não é "o desenho é bonito"** — é *"a bancada consegue
//! MOSTRAR os seis?"*. Um `next()` que salte um desenho torna-o inalcançável pela UI, e um desenho
//! inalcançável é exactamente equivalente a não existir: o Enio decidiria entre cinco a pensar que
//! viu seis.

use ph2d_panel_widget_lab::BoxDesign;
use std::collections::BTreeSet;

/// `next` percorre TODOS os desenhos e volta ao princípio.
#[test]
fn next_reaches_every_design_and_closes_the_ring() {
    let start = BoxDesign::default();
    let mut seen = BTreeSet::new();
    let mut d = start;
    for _ in 0..BoxDesign::ALL.len() {
        seen.insert(d.label());
        d = d.next();
    }
    assert_eq!(
        d,
        start,
        "o anel do `next` nao fecha em {} passos — algum desenho e' inalcancavel",
        BoxDesign::ALL.len()
    );
    let all: BTreeSet<&str> = BoxDesign::ALL.iter().map(|d| d.label()).collect();
    assert_eq!(seen, all, "o `next` nao passa por todos os desenhos");
}

/// `prev` é a inversa de `next`. ⚠️ Sem isto, um `prev` copiado com o sinal trocado anda para a
/// frente e a bancada perde metade da navegação **sem nenhum sintoma visível**.
#[test]
fn prev_undoes_next() {
    for d in BoxDesign::ALL {
        assert_eq!(d.next().prev(), d, "prev nao desfaz next em {}", d.label());
        assert_eq!(d.prev().next(), d, "next nao desfaz prev em {}", d.label());
    }
}

/// Cada desenho diz **o que troca**, e diz alguma coisa. ⛔ Um `blurb` vazio ou de três palavras é
/// um encolher de ombros — o mesmo detector que o censo do trilho usa.
#[test]
fn every_design_states_what_it_trades() {
    for d in BoxDesign::ALL {
        let words = d.blurb().split_whitespace().count();
        assert!(
            words >= 8,
            "o desenho {} descreve-se em {words} palavras — isso e' um encolher de ombros, \
             nao um trade-off. Diga o que ele GANHA e o que PERDE.",
            d.label()
        );
    }
}

/// ⭐⭐ **Só a `Split` gasta coluna de rótulo.** É a grandeza inteira que a decisão do Enio põe a
/// zero: *"a caixa única é o alvo"*. Se um segundo desenho passar a reservar coluna, a família
/// deixou de ser a caixa única e alguém tem de dizer isso em voz alta.
#[test]
fn only_the_negative_control_spends_an_outer_label_column() {
    let spenders: Vec<&str> = BoxDesign::ALL
        .iter()
        .filter(|d| d.outer_label_w() > 0.0)
        .map(|d| d.label())
        .collect();
    assert_eq!(
        spenders,
        vec!["Split"],
        "estes desenhos reservam coluna de rotulo FORA da caixa: {spenders:?}\n\
         \u{26a0} A caixa unica po^e essa largura a ZERO. A `Split` esta' na lista de proposito, \
         como controlo negativo; um segundo nome aqui significa que a familia mudou."
    );
}
