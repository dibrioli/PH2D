//! Gates da FICHA de leitura — a largura que ela toma e o sítio onde ela pousa.

use super::*;

fn ts() -> TextSystem {
    TextSystem::without_system_fonts()
}

/// Sonda: quanto mede cada ficha que o app de facto desenha. É daqui que saem os números do
/// cabeçalho do módulo — e é ela que diz se um texto novo ainda cabe no piso de alguém.
#[test]
#[ignore = "sonda: rode com -- --ignored --nocapture"]
fn census_of_chip_widths() {
    let mut t = ts();
    for s in [
        "-1234.5 px",
        "1.5 m",
        "+120.0, -45.0 px",
        "+1234.5, -1234.5 px",
        "\u{d7}1.20",
        "\u{d7}1.20, \u{d7}0.80",
        "+45.0\u{b0}",
        "+1080.0\u{b0}",
    ] {
        println!("{:>24} -> {:6.2} px", s, chip_width(&mut t, s, 0.0));
    }
}

/// A ficha **cresce com o texto**, e nunca encolhe abaixo do piso que lhe deram.
#[test]
fn a_chip_fits_its_text_and_never_goes_under_its_floor() {
    let mut t = ts();
    let short = chip_width(&mut t, "+1, +1 px", 0.0);
    let long = chip_width(&mut t, "+1234.5, -1234.5 px", 0.0);
    assert!(
        long > short,
        "a ficha não cresceu com o texto: curto {short}, longo {long}"
    );
    assert!(
        short > CHIP_PAD_X_PX * 2.0,
        "a ficha nem o respiro comporta ({short} px)"
    );
    let floored = chip_width(&mut t, "+1, +1 px", 200.0);
    assert!(
        (floored - 200.0).abs() < 0.01,
        "o piso não segurou: {floored} px para um piso de 200"
    );
}

/// ⭐ **A ficha INVERTE de lado na borda, em vez de escorregar para debaixo do cursor.**
///
/// *Mutação que sangra:* trocar as duas inversões por um clamp puro — a ficha passa a pousar a
/// menos de meia largura do cursor exactamente na borda, que é onde o ponteiro tapa o número.
#[test]
fn the_chip_flips_sides_instead_of_sliding_under_the_cursor() {
    let canvas = Rect::new(0.0, 0.0, 400.0, 300.0);
    let w = 80.0;
    // Longe da borda: abaixo-e-à-direita.
    let mid = at_cursor([100.0, 100.0], w, canvas);
    assert!(
        mid[0] > 100.0 && mid[1] > 100.0,
        "a ficha não pousou abaixo-e-à-direita: {mid:?}"
    );
    // Encostado ao canto inferior-direito: ela salta para cima-e-à-esquerda.
    let corner = at_cursor([395.0, 295.0], w, canvas);
    assert!(
        corner[0] < 395.0 && corner[1] < 295.0,
        "a ficha não inverteu no canto: {corner:?}"
    );
    // …e o que isso COMPRA: a ficha inteira fica longe do ponteiro.
    let gap = 395.0 - (corner[0] + w * 0.5);
    assert!(
        gap >= CURSOR_GAP_PX - 0.01,
        "a ficha ficou a {gap:.1} px do cursor (mínimo {CURSOR_GAP_PX}): o ponteiro tapa-a"
    );
}

/// Uma tela mais estreita que a ficha não produz um número fora dos dois limites.
#[test]
fn the_chip_survives_a_canvas_narrower_than_itself() {
    let canvas = Rect::new(0.0, 0.0, 40.0, 300.0);
    let at = at_cursor([20.0, 150.0], 200.0, canvas);
    assert!(
        at[0].is_finite() && at[1].is_finite(),
        "pouso não-finito: {at:?}"
    );
    assert!(
        (at[0] - 100.0).abs() < 0.01,
        "com a ficha mais larga que a tela, ela pousa na borda de partida: {at:?}"
    );
}

/// **A migração do rótulo do smart guide é byte-idêntica, e este gate MEDE-O.**
///
/// O piso herdado (60 px) foi dimensionado para `-1234.5 px`. Se o texto medido mais o respiro
/// coubesse acima dele, a ficha do smart guide teria mudado de largura ao passar por esta porta —
/// e uma superfície aprovada em smoke teria mudado de aparência sem ninguém pedir.
#[test]
fn the_snap_labels_widest_text_still_fits_the_floor_it_inherited() {
    let mut t = ts();
    const FLOOR: f32 = 60.0;
    let widest = chip_width(&mut t, "-1234.5 px", 0.0);
    assert!(
        widest <= FLOOR,
        "o texto mais largo do rótulo mede {widest:.1} px > o piso {FLOOR}: a ficha do smart \
         guide MUDOU de tamanho na migração"
    );
    assert!(
        (chip_width(&mut t, "-1234.5 px", FLOOR) - FLOOR).abs() < 0.01,
        "com o piso, a ficha tem de ter exactamente a largura que tinha"
    );
}

/// O rectângulo é centrado no ponto que lhe deram — a lei que o pintor e os gates partilham.
#[test]
fn the_rect_is_centred_on_the_point_it_was_given() {
    let r = chip_rect([100.0, 50.0], 80.0);
    assert!((r.x + r.w * 0.5 - 100.0).abs() < 0.001);
    assert!((r.y + r.h * 0.5 - 50.0).abs() < 0.001);
    assert!((r.h - CHIP_H_PX).abs() < 0.001);
}

/// A ficha pousa onde o canvas está, não onde a JANELA está — um canvas deslocado (o Inspector à
/// esquerda) desloca a ficha com ele.
#[test]
fn the_chip_lands_inside_the_canvas_not_the_window() {
    let canvas = Rect::new(300.0, 40.0, 600.0, 400.0);
    let at = at_cursor([310.0, 50.0], 80.0, canvas);
    assert!(
        at[0] - 40.0 >= canvas.x && at[1] - CHIP_H_PX * 0.5 >= canvas.y,
        "a ficha saiu do canvas: {at:?} contra {canvas:?}"
    );
}

/// ⭐⭐ **A ficha PINTA o texto inteiro que ela mediu** (doc 125 §8, a legenda da cena `=130`).
///
/// ⛔ A largura sai do `prefix_width` mais a folga da ficha (`CHIP_PAD_X_PX`), e o pintor pedia o
/// orçamento de um RÓTULO (`label_budget`, que desconta `Spacing::Md` de cada lado): a folga
/// contada duas vezes à mão divergia, e toda ficha perdia as últimas letras para uma reticência —
/// `«SEM Collide»` saía `«SEM Col…»`. A régua é o que a elisão ESCOLHE para o orçamento que a
/// ficha dá ao texto, sobre as fichas que o app de facto desenha.
#[test]
fn a_chip_paints_the_whole_text_it_measured() {
    let mut t = ts();
    for s in [
        "SEM Collide",
        "COM Collide",
        "-1234.5 px",
        "+120.0, -45.0 px",
        "\u{d7}1.20",
        "+45.0\u{b0}",
        "Collide DESLIGADO: as contas atravessam-se",
    ] {
        let w = chip_width(&mut t, s, 0.0);
        let orcamento = chip_text_budget(&mut t, s, w);
        let mostrado = crate::text_elide::fit(&mut t, s, CHIP_FONT_PX, orcamento);
        assert_eq!(
            mostrado, s,
            "a ficha de {w:.1} px pinta {mostrado:?} em vez de {s:?}"
        );
    }
}
