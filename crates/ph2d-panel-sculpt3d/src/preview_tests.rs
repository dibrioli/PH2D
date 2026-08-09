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

/// Um carimbo de bandas diagonais 32².
///
/// ⚠️ **Diagonal e não em barras**, porque um padrão que varia num eixo só
/// concordaria consigo mesmo sob um deslocamento no outro — e o gate ficaria
/// verde sobre metade da feature.
///
/// ⚠️ **Ele é construído UMA vez e COMPARTILHADO pelos estados que um gate
/// compara** (ver [`ui_stamped`]), e isso é sobre o que está sendo testado: o
/// `PartialEq` do [`Alpha`] compara imagens por **IDENTIDADE**, então dois `Arc`
/// distintos dariam duas chaves diferentes e o cache **nunca acertaria** — o
/// gate passaria a exercitar só o `render`, e um cache com a chave incompleta
/// (que é o defeito que estes gates existem para pegar) ficaria verde.
fn stamp() -> Alpha {
    let n = 32u32;
    let mut rgba = vec![0u8; (n * n * 4) as usize];
    for y in 0..n {
        for x in 0..n {
            let v = u8::from((x + y) % 8 < 4) * 255;
            let i = ((y * n + x) * 4) as usize;
            rgba[i..i + 3].fill(v);
            rgba[i + 3] = 255;
        }
    }
    Alpha::Image(std::sync::Arc::new(
        ph2d_sculpt3d::AlphaImage::from_rgba(n, n, &rgba).expect("imagem válida"),
    ))
}

/// Um pincel com o carimbo DADO armado e o eixo ENCARANDO a vista, que é como o
/// produto o semeia (`Sculpt3dScene::seed_alpha_placement`).
fn ui_stamped(stamp: &Alpha, offset: [f32; 2]) -> Sculpt3dUi {
    let mut ui = ui_with(stamp.clone(), 0.25, 90, ph2d_sculpt3d::MAX_AXIS_ELEV_DEG);
    ui.brush.alpha_offset = offset;
    ui
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
    // ⚠️ **O DESLOCAMENTO precisa de fixture PRÓPRIA, e essa assimetria é a razão
    // de ele ter sido esquecido.** Todos os campos acima são varridos sobre um
    // padrão procedural, e para um procedural o deslocamento é NEUTRO por
    // construção — acrescentar uma linha à lista de cima teria REPROVADO produto
    // correto. Ele só está vivo com uma imagem armada, então é ali que se
    // pergunta.
    let s = stamp();
    let stamped = ui_stamped(&s, [0.0, 0.0]);
    let placed = ui_stamped(&s, [0.31, -0.17]);
    assert_ne!(
        pixels(&placed, 2.0),
        pixels(&stamped, 2.0),
        "mudar o deslocamento não mudou o preview — a chave do cache não o carrega"
    );

    // ⚠️ E o TEMA: os dois extremos da rampa saem dele, e o `draw_image_rgba` só
    // BLITA — não há estágio de tint depois. Um tema ESCURO contra um CLARO, que
    // é o par em que os dois tokens de fato trocam de lugar.
    let dark = with_swatch(&base, 2.0, Theme::Forge, |p| p.as_slice().to_vec());
    let light = with_swatch(&base, 2.0, Theme::Sunstone, |p| p.as_slice().to_vec());
    assert_ne!(dark, light, "o tema não chega ao preview");
}

/// **COLOCAR O CARIMBO DESLIZA O PREVIEW — e desliza EXATAMENTE o que se pediu.**
///
/// ⚠️ **Este é o gate do report** *"Pattern Offset parece sem efeito"* (Enio,
/// 2026-08-09). O deslocamento sempre chegou ao BARRO (medido: um passo do slider
/// muda 10.159 de 13.682 vértices); quem estava cego era o swatch, a única
/// superfície que responde sem esculpir — por DOIS motivos, e é preciso os dois
/// mortos para ele passar: a chave do cache não carregava o campo, **e** o
/// `render` remontava um pincel `..Brush::default()`, cujo `alpha: None` faz o
/// `alpha_frame` zerar o deslocamento pela regra de neutralidade.
///
/// ⚠️ **O oráculo é TRANSLAÇÃO, não *"mudou"*.** Comparar bytes ficaria verde
/// para qualquer coisa que mexesse na imagem — um padrão que se deformasse ou
/// piscasse passaria igual. Aqui a imagem deslocada tem de ser a original
/// **corrida por um número exato de texels**, que é o que *colocar* significa.
///
/// ⚠️ **A DIREÇÃO não é afirmada, de propósito:** *"arrastar para a direita move
/// para a direita"* é pergunta de smoke (o olho é o oráculo), e cravá-la aqui
/// faria este gate espelhar o sinal do `t` do frame em vez de julgá-lo.
#[test]
fn placing_the_stamp_slides_the_preview() {
    let span = 2.0;
    // ⚠️ **O texel do swatch mede `span_of(span)`, não `span`** — ele abrange um
    // OITAVO do modelo ([`SPAN_FRACTION`]). A primeira versão deste gate dividiu
    // pelo modelo inteiro e pediu 3 texels achando que pedia 3 colunas: eram 24,
    // que no período de 16 do carimbo caem em 8 — e o gate acusou o PRODUTO por
    // um erro da própria régua. *A conversão que o oráculo usa é a mesma que o
    // desenho usa, ou o oráculo mede outra coisa.*
    let step = span_of(span) / SWATCH as f32;
    // ⚠️ **UM carimbo para todos os estados** — ver o doc de [`stamp`]: com dois
    // `Arc` o cache nunca acertaria, e o gate deixaria de julgar a chave.
    let s = stamp();
    let base = pixels(&ui_stamped(&s, [0.0, 0.0]), span);

    for texels in [3i32, 7] {
        let along = pixels(&ui_stamped(&s, [step * texels as f32, 0.0]), span);
        assert_ne!(along, base, "deslocar em X não mexeu no preview");
        assert_eq!(
            shift_cols(&base, &along),
            Some(texels),
            "deslocar {texels} texels em X não correu o preview {texels} colunas"
        );

        let across = pixels(&ui_stamped(&s, [0.0, step * texels as f32]), span);
        assert_ne!(across, base, "deslocar em Y não mexeu no preview");
        assert_eq!(
            shift_rows(&base, &across),
            Some(texels),
            "deslocar {texels} texels em Y não correu o preview {texels} linhas"
        );
    }
}

/// **UM PADRÃO PROCEDURAL IGNORA O DESLOCAMENTO, AO BYTE** — o controle do gate
/// acima, e ele NÃO é higiene.
///
/// O deslocamento é neutralizado dentro do [`ph2d_sculpt3d::Brush::alpha_frame`]
/// quando não há imagem armada, porque os nove procedurais são campos infinitos e
/// homogêneos: eles não têm posição, só fase. Se esta neutralidade se perdesse, a
/// row — que o painel esconde para eles — passaria a agir sem como ser desfeita.
#[test]
fn a_procedural_pattern_ignores_the_stamp_offset() {
    let quiet = ui_with(Alpha::Strata, 0.25, 90, ph2d_sculpt3d::MAX_AXIS_ELEV_DEG);
    let mut moved = quiet.clone();
    moved.brush.alpha_offset = [0.37, -0.11];
    assert_eq!(
        pixels(&moved, 2.0),
        pixels(&quiet, 2.0),
        "o deslocamento vazou para um padrão procedural"
    );
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

/// **Por quantas COLUNAS `moved` é `base` corrido?** `None` se não for uma
/// translação pura.
///
/// ⚠️ O sinal é devolvido em MÓDULO: ver o gate que chama — a direção é pergunta
/// de smoke, e o que se afirma aqui é *deslizou, e deslizou tanto*.
fn shift_cols(base: &[u8], moved: &[u8]) -> Option<i32> {
    (1..SWATCH as i32).find(|d| {
        [*d, -*d]
            .iter()
            .any(|s| matches_shift(base, moved, *s, |r, c| (r, c)))
    })
}

/// A irmã do [`shift_cols`] no outro eixo — a mesma varredura, com as
/// coordenadas trocadas na entrada do oráculo.
fn shift_rows(base: &[u8], moved: &[u8]) -> Option<i32> {
    (1..SWATCH as i32).find(|d| {
        [*d, -*d]
            .iter()
            .any(|s| matches_shift(base, moved, *s, |r, c| (c, r)))
    })
}

/// `moved` é `base` corrido de `s` no eixo que o `axes` escolhe?
///
/// ⚠️ **Só o INTERIOR é comparado**, e é honesto: o swatch é uma JANELA sobre um
/// campo infinito, então o que entra por uma borda não tem original com que ser
/// comparado. Exigir as bordas seria exigir que o gate conhecesse o campo fora da
/// janela.
fn matches_shift(
    base: &[u8],
    moved: &[u8],
    s: i32,
    axes: fn(usize, usize) -> (usize, usize),
) -> bool {
    let mut compared = 0;
    for a in 0..SWATCH {
        for b in 0..SWATCH {
            let Ok(src) = usize::try_from(b as i32 + s) else {
                continue;
            };
            if src >= SWATCH {
                continue;
            }
            let (mr, mc) = axes(a, b);
            let (br, bc) = axes(a, src);
            if lum(moved, mr, mc) != lum(base, br, bc) {
                return false;
            }
            compared += 1;
        }
    }
    // Um deslocamento que não deixa NADA em comum não é uma translação medida:
    // seria um `true` por vácuo.
    compared > SWATCH * SWATCH / 2
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
