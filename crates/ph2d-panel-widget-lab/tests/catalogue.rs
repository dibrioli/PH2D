//! O catálogo de desenhos é percorrível e completo.
//!
//! ⚠️ **A pergunta que estes gates fazem não é "o desenho é bonito"** — é *"a bancada consegue
//! MOSTRAR os seis?"*. Um `next()` que salte um desenho torna-o inalcançável pela UI, e um desenho
//! inalcançável é exactamente equivalente a não existir: o Enio decidiria entre cinco a pensar que
//! viu seis.

use ph2d_tokens::SliderDesign as BoxDesign;
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

/// ⛔ **A `Split` SAIU do catálogo, e este teste é a lápide dela.**
///
/// Ela era o **controlo negativo** do estudo: o desenho de duas colunas (o do Blender), presente
/// para tornar a decisão verificável em vez de lembrada. O Enio escolheu os quatro da caixa única
/// em 2026-09-02, e ela — com a `Notch` — deixou de shipar.
///
/// ⚠️ **O que ela guardava passou a ser guardado noutro sítio**, e é isso que faz esta remoção ser
/// segura: `the_customisation_offers_exactly_the_four_chosen_designs`
/// (`ph2d-editor-core/tests/the_app_default_slider_style_is_the_one_the_owner_chose.rs`) afirma a
/// lista pelo nome, e o `slider_style.rs` regista as duas recusas com o mecanismo de cada uma.
/// *Apagar um gate sem dizer quem herdou a pergunta é como a propriedade se perde.*
#[test]
fn the_negative_control_is_gone_and_its_question_has_an_heir() {
    let names: Vec<&str> = BoxDesign::ALL.iter().map(|d| d.label()).collect();
    assert!(
        !names.contains(&"Split"),
        "a `Split` voltou ao catalogo — ela reserva coluna de rotulo FORA da caixa, que e' a \
         grandeza que a caixa unica po^e a ZERO. Se foi decisao do Enio, actualize o \
         `slider_style.rs`, onde a recusa dela esta' registada."
    );
    assert!(
        !names.contains(&"Notch"),
        "a `Notch` voltou ao catalogo — mesma cura: registe a decisao no `slider_style.rs`."
    );
}
