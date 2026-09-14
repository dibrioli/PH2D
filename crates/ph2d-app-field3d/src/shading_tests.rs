//! Os gates do modo de pintar e do olhar — ver [`super`].

use super::*;

/// ⭐⭐⭐ **COM QUE OLHAR O MODELADOR ABRE** — a decisão do dono de 14/09, com um gate por cima.
///
/// # ⛔⛔ Porque ela precisa de um gate, e de um que nomeie a ORDEM
///
/// Trocar a omissão para `Neutral` custa: o olhar é da **cena**, logo ele governa também o matcap, e
/// `99,9 %` dos texels dele mudam (`13` bytes típicos). ⇒ **é exactamente o tipo de linha que alguém
/// «simplifica» de volta para `Look::default()` numa limpeza** — e a decisão evaporaria em silêncio,
/// com o quadro a voltar ao branco chapado de `8,4 %` sem nenhum teste a acusar.
///
/// ⚠️ **As DUAS metades, e são leis diferentes:**
///
/// | afirmação | de quem |
/// |---|---|
/// | `Look::default()` continua a ser a **identidade** | do **tipo** ([`ph2d_view_transform`]), e vale para todo consumidor dele |
/// | o **módulo** abre em [`OPENING_LOOK`] | decisão de produto, e vive na `crate::shading` |
///
/// ⛔ Quem trocasse a primeira mudaria, à distância, o quadro de quem nunca pediu olhar nenhum.
///
/// **Mutação que deve sangrar:** `view.rs` a voltar a `ph2d_view_transform::Look::default()`.
#[test]
fn the_modeler_opens_with_the_look_the_owner_chose() {
    // ⭐ A metade do PRODUTO: o módulo abre em `Neutral`, e é a `View::default` que o diz — que é a
    // mesma porta que o `boot` usa (o doc dela: *«é esta a definição de vista nova»*).
    assert_eq!(
        crate::smoke::view::View::default().look,
        OPENING_LOOK,
        "a vista nova deixou de abrir com o olhar que o dono escolheu"
    );
    assert_eq!(
        OPENING_LOOK.view,
        ViewTransform::Neutral,
        "o olhar de abertura deixou de ser o `Neutral` — ordem do dono de 2026-09-14, com a tabela \
         do preço no doc da constante"
    );
    assert!(
        (OPENING_LOOK.exposure_stops).abs() < 1.0e-6,
        "a exposição de abertura não é zero — a decisão foi sobre a VISTA, e baixar um stop foi \
         medido como 3,5× pior para o matcap"
    );
    // ⭐ **E o MODO de abertura não se mexeu** — o matcap continua a ser o que um modelador quer
    // ver primeiro: ele lê **forma**. A decisão do dono foi sobre o olhar, não sobre o modo.
    assert_eq!(Shading::default(), Shading::Matcap);
    assert_eq!(
        crate::smoke::view::View::default().shading,
        Shading::Matcap,
        "a vista nova deixou de abrir em matcap"
    );
    // ⛔ **E a metade do TIPO fica intacta:** a identidade continua a ser a identidade.
    assert_eq!(
        Look::default(),
        Look {
            exposure_stops: 0.0,
            view: ViewTransform::Standard,
        }
    );
    assert_eq!(
        exposure_chips(Look::default())
            .iter()
            .filter(|c| c.active)
            .map(|c| c.key)
            .collect::<Vec<_>>(),
        vec!["panel.model3d.exposure.zero"],
        "a exposição de omissão tem de acender o chip do zero, e só ele"
    );
}

/// ⭐ **Toda fileira acende EXACTAMENTE o chip do estado**, para todo estado que um chip escreve.
#[test]
fn every_row_lights_exactly_the_chip_of_the_state() {
    for (i, s) in Shading::ALL.into_iter().enumerate() {
        let chips = shading_chips(s);
        assert_eq!(chips.len(), Shading::ALL.len());
        assert_eq!(chips.iter().filter(|c| c.active).count(), 1);
        assert!(chips[i].active, "o modo {s:?} não acende o chip dele");
    }
    for slot in 0..ViewTransform::ALL.len() {
        let look = with_view(Look::default(), slot);
        let chips = look_chips(look);
        assert_eq!(chips.iter().filter(|c| c.active).count(), 1);
        assert!(
            chips[slot].active,
            "a vista do slot {slot} não acende o chip dela"
        );
    }
    for slot in 0..EXPOSURES.len() {
        let look = with_exposure(Look::default(), slot);
        let chips = exposure_chips(look);
        assert_eq!(chips.iter().filter(|c| c.active).count(), 1);
        assert!(
            chips[slot].active,
            "a exposição do slot {slot} não acende o chip dela"
        );
    }
}

/// ⭐ **Mexer na exposição não mexe na vista, e vice-versa** — os dois vivem no mesmo olhar.
#[test]
fn the_exposure_and_the_view_are_independent_halves_of_the_look() {
    let look = with_view(with_exposure(Look::default(), 4), 1);
    assert_eq!(look.exposure_stops, 2.0);
    assert_eq!(look.view, ViewTransform::Neutral);
    assert_eq!(with_exposure(look, 0).view, ViewTransform::Neutral);
    assert_eq!(with_view(look, 0).exposure_stops, 2.0);
}

/// Um slot que não existe não muda nada — um clique velho não pode zerar o olhar.
#[test]
fn a_slot_that_does_not_exist_changes_nothing() {
    let look = with_view(with_exposure(Look::default(), 3), 1);
    assert_eq!(with_exposure(look, 99), look);
    assert_eq!(with_view(look, 99), look);
}
