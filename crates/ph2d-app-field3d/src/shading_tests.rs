//! Os gates do modo de pintar e do olhar — ver [`super`].

use super::*;

/// ⭐ **A omissão é o que o modelador sempre pintou** — matcap, e o olhar identidade.
#[test]
fn the_default_is_what_the_modeler_always_painted() {
    assert_eq!(Shading::default(), Shading::Matcap);
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
