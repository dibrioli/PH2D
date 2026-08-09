//! **O PREVIEW faz o que ele promete** — e o que ele promete é uma coisa só: que
//! a imagem na tela é o padrão que o pincel vai depositar, **na densidade do
//! modelo**.
//!
//! ⚠️ A pergunta que estes gates NÃO respondem é *"ele está bonito?"* — essa é do
//! smoke, e o oráculo é o olho.

use super::*;
use ph2d_sculpt3d::Brush;

fn ui_with(alpha: Alpha, scale: f32, az: u16, elev: u16) -> Sculpt3dUi {
    Sculpt3dUi {
        brush: Brush {
            alpha: Some(alpha),
            alpha_scale: scale,
            alpha_az_deg: az,
            alpha_elev_deg: elev,
            ..Brush::default()
        },
        ..Sculpt3dUi::default()
    }
}

/// Os pixels do swatch para um estado, como bytes.
fn pixels(ui: &Sculpt3dUi, span: f32) -> Vec<u8> {
    with_swatch(ui, span, Theme::Forge, |px| px.as_slice().to_vec()).expect("há padrão armado")
}

/// **SEM padrão armado não há preview** — e é um `None`, não uma moldura vazia.
///
/// Uma moldura de coisa nenhuma ocuparia a mesma altura e diria ao artista que
/// existe algo a ver; a ausência diz a verdade.
#[test]
fn without_a_pattern_there_is_no_preview() {
    let plain = Sculpt3dUi::default();
    assert!(plain.brush.alpha.is_none(), "o pincel de fábrica é liso");
    assert!(
        with_swatch(&plain, 2.0, Theme::Forge, |_| ()).is_none(),
        "um pincel sem padrão desenhou um preview"
    );
}

/// **A ESCALA MUDA A DENSIDADE, e é a razão de o swatch medir unidades de
/// OBJETO.**
///
/// ⚠️ **É o gate central da wave.** Um swatch que abrangesse `N × escala` teria
/// sempre `N` features e sairia IDÊNTICO em toda escala — ele responderia *"que
/// padrão é este?"* (que os nove nomes já respondem) e ficaria mudo sobre a única
/// pergunta que o artista não consegue responder sozinho, que é *este tamanho
/// está certo para o MEU modelo?*.
///
/// O oráculo é o número de TRANSIÇÕES ao longo de uma linha: um padrão mais fino
/// cruza a linha mais vezes. Contar transições, e não comparar bytes, é o que faz
/// o gate falar sobre **densidade** em vez de sobre *"mudou alguma coisa"*.
#[test]
fn a_finer_scale_draws_a_denser_preview() {
    let span = 2.0;
    let coarse = transitions(&pixels(&ui_with(Alpha::Strata, 0.20, 90, 0), span));
    let fine = transitions(&pixels(&ui_with(Alpha::Strata, 0.05, 90, 0), span));
    assert!(
        fine > coarse * 2,
        "escala 4× mais fina desenhou {fine} transições contra {coarse} — \
         o swatch não está medindo unidades de objeto"
    );
}

/// **UM MODELO MAIOR MOSTRA MAIS FEATURES no mesmo swatch.**
///
/// A outra metade da mesma frase: com a escala PARADA, dobrar o modelo dobra o
/// pedaço que o swatch abrange, e o padrão fica mais denso na tela. É isso que
/// faz o preview responder *"para o MEU modelo"* em vez de *"para um modelo"*.
#[test]
fn a_bigger_model_shows_more_features_in_the_same_swatch() {
    let ui = ui_with(Alpha::Strata, 0.1, 90, 0);
    let small = transitions(&pixels(&ui, 1.0));
    let big = transitions(&pixels(&ui, 4.0));
    assert!(
        big > small * 2,
        "um modelo 4× maior desenhou {big} transições contra {small} — \
         o swatch não conhece o tamanho do modelo"
    );
}

/// **GIRAR O EIXO GIRA O PREVIEW** — o controle da W11, visto antes de esculpir.
///
/// O oráculo é a ANISOTROPIA: um estrato deitado tem muitas transições ao descer
/// uma coluna e quase nenhuma ao atravessar uma linha; girado de 90°, os dois
/// números trocam de lugar. ⚠️ Comparar bytes só diria *"mudou"* — e mudar é o
/// que uma imagem faz quando qualquer coisa muda.
#[test]
fn turning_the_axis_turns_the_preview() {
    let span = 2.0;
    let flat = pixels(&ui_with(Alpha::Strata, 0.1, 90, 0), span);
    let upright = pixels(&ui_with(Alpha::Strata, 0.1, 0, 0), span);
    let (fx, fy) = (rows_crossed(&flat), cols_crossed(&flat));
    let (ux, uy) = (rows_crossed(&upright), cols_crossed(&upright));
    assert!(
        fy > fx * 2,
        "no eixo de fábrica as camadas não saíram deitadas ({fx} × {fy})"
    );
    assert!(
        ux > uy * 2,
        "com o eixo a 0° as camadas não ficaram de pé ({ux} × {uy})"
    );
}

/// **OS SEIS ISOTRÓPICOS IGNORAM O EIXO, AO BYTE** — o controle do gate acima.
///
/// Sem ele, um preview que simplesmente não lesse o eixo passaria pela metade
/// *"gira"* de nenhum jeito e por esta de graça.
#[test]
fn an_isotropic_pattern_ignores_the_axis_byte_for_byte() {
    for a in Alpha::ALL.iter().filter(|a| !a.is_directional()) {
        let one = pixels(&ui_with(a.clone(), 0.1, 90, 0), 2.0);
        let other = pixels(&ui_with(a.clone(), 0.1, 17, 63), 2.0);
        assert_eq!(one, other, "{} mudou com o eixo", a.label());
    }
}

/// **O CACHE devolve a MESMA imagem para a MESMA entrada, e uma NOVA quando
/// qualquer entrada muda.**
///
/// ⚠️ O modo de falha de uma chave incompleta não é um erro: é um preview VELHO
/// que ninguém vê que é velho. Este gate varre **cada campo da chave, um por
/// um** — uma varredura que mudasse dois de cada vez não distinguiria *"a chave
/// tem este campo"* de *"a chave tem o outro"*.
#[test]
fn the_cache_key_carries_every_input() {
    let base = ui_with(Alpha::Strata, 0.1, 90, 0);
    let first = pixels(&base, 2.0);
    assert_eq!(
        pixels(&base, 2.0),
        first,
        "a mesma entrada deu duas imagens"
    );

    let mut alpha = base.clone();
    alpha.brush.alpha = Some(Alpha::Weave);
    let mut scale = base.clone();
    scale.brush.alpha_scale = 0.05;
    let mut az = base.clone();
    az.brush.alpha_az_deg = 30;
    let mut elev = base.clone();
    elev.brush.alpha_elev_deg = 45;

    for (name, ui, span) in [
        ("o padrão", alpha, 2.0),
        ("a escala", scale, 2.0),
        ("o azimute", az, 2.0),
        ("a elevação", elev, 2.0),
        ("o tamanho do modelo", base.clone(), 5.0),
    ] {
        assert_ne!(
            pixels(&ui, span),
            first,
            "mudar {name} não mudou o preview — a chave do cache não o carrega"
        );
    }
    // ⚠️ E o TEMA: os dois extremos da rampa saem dele, e o `draw_image_rgba` só
    // BLITA — não há estágio de tint depois. Um tema ESCURO contra um CLARO, que
    // é o par em que os dois tokens de fato trocam de lugar.
    let dark = with_swatch(&base, 2.0, Theme::Forge, |p| p.as_slice().to_vec());
    let light = with_swatch(&base, 2.0, Theme::Sunstone, |p| p.as_slice().to_vec());
    assert_ne!(dark, light, "o tema não chega ao preview");
}

/// **UM MODELO DEGENERADO NÃO COLAPSA O SWATCH.**
///
/// Uma peça vazia tem lado zero, e um span zero faria toda amostra cair no mesmo
/// ponto: o swatch sairia de UMA COR, indistinguível de *"o padrão não
/// funciona"*.
#[test]
fn a_degenerate_model_still_draws_a_pattern() {
    for span in [0.0_f32, -1.0, f32::NAN] {
        assert!(
            span_of(span) > 0.0,
            "um modelo de lado {span} deu um swatch de largura zero"
        );
    }
    let px = pixels(&ui_with(Alpha::Strata, 0.05, 90, 0), 0.0);
    assert!(
        transitions(&px) > 0,
        "o swatch de um modelo degenerado saiu de uma cor só"
    );
}

// ── Os oráculos ────────────────────────────────────────────────────────────
// Eles leem o CANAL VERMELHO, que é o suficiente: a rampa é uma interpolação
// monotônica entre dois tokens, então qualquer canal em que eles difiram carrega
// o mesmo padrão. E o limiar é o MEIO da faixa, que é onde uma transição está.

fn lum(px: &[u8], row: usize, col: usize) -> u8 {
    px[(row * SWATCH + col) * 4]
}

/// Quantas vezes o padrão cruza o meio da faixa, somado sobre linhas e colunas.
fn transitions(px: &[u8]) -> usize {
    rows_crossed(px) + cols_crossed(px)
}

/// As transições ao andar na HORIZONTAL (dentro de cada linha).
fn rows_crossed(px: &[u8]) -> usize {
    let mut n = 0;
    for row in 0..SWATCH {
        for col in 1..SWATCH {
            if (lum(px, row, col - 1) < 128) != (lum(px, row, col) < 128) {
                n += 1;
            }
        }
    }
    n
}

/// As transições ao andar na VERTICAL (descendo cada coluna).
fn cols_crossed(px: &[u8]) -> usize {
    let mut n = 0;
    for col in 0..SWATCH {
        for row in 1..SWATCH {
            if (lum(px, row - 1, col) < 128) != (lum(px, row, col) < 128) {
                n += 1;
            }
        }
    }
    n
}
