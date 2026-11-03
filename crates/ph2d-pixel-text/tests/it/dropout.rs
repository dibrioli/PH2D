//! O controlo de «dropout» do Mono: a tabela do `SCANTYPE` e o limiar do `SCANCTRL` do `prep` das
//! DejaVu (ligado até 100 ppem). Os oráculos só tocam os modos que «Ag1» precisa; aqui cada ramo.

use ph2d_pixel_text::outline::{Dropout, face, font_dropout};

#[test]
fn scantype_values_map_to_the_specs_modes() {
    for (n, quer) in [
        (0, Dropout::Simple { stubs: true }),
        (1, Dropout::Simple { stubs: false }),
        (2, Dropout::Off),
        (3, Dropout::Off),
        (4, Dropout::Smart { stubs: true }),
        (5, Dropout::Smart { stubs: false }),
        (6, Dropout::Off),
        (-1, Dropout::Off),
    ] {
        assert_eq!(Dropout::from_scantype(n), quer, "SCANTYPE {n}");
    }
    assert_eq!(Dropout::FREETYPE_DEFAULT, Dropout::Simple { stubs: false });
}

#[test]
fn the_dejavu_prep_turns_dropout_control_on_up_to_100_ppem_and_off_above() {
    let path = "/usr/share/fonts/TTF/DejaVuSans.ttf";
    let data = std::fs::read(path).unwrap_or_else(|e| panic!("{path}: {e}"));
    let f = face(&data, 0).unwrap();
    let on = Dropout::Simple { stubs: false };
    assert_eq!(font_dropout(&f, 8), on);
    assert_eq!(font_dropout(&f, 100), on, "100 ppem ainda liga (≤ limiar)");
    assert_eq!(font_dropout(&f, 101), Dropout::Off, "101 ppem desliga");
}

/// A curva de tom do texto (MEDIDA no Trial, 238 entradas) não muda uma entrada sem aviso: os
/// oráculos só olham o maior |Δ| e as poucas células de «Ag1», uma entrada fora delas passava.
#[test]
fn the_trial_text_tone_curve_is_frozen_entry_by_entry() {
    use ph2d_pixel_text::outline::TRIAL_TEXT_TONE as T;
    let soma: u32 = T.iter().map(|&v| u32::from(v)).sum();
    let ponderada: u64 = T
        .iter()
        .enumerate()
        .map(|(k, &v)| k as u64 * u64::from(v))
        .sum();
    assert_eq!((T[0], T[255]), (0, 255));
    assert!(T.windows(2).all(|w| w[0] <= w[1]), "monótona");
    assert_eq!((soma, ponderada), (24942, 4_374_314));
}
