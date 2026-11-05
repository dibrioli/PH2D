//! As portas que cortam numa linha: o peso e o espaçamento entre letras chegam à tinta.

use super::*;

/// ⭐⭐ **E O QUE FOI PINTADO TEM O PESO QUE FOI MEDIDO** — a outra metade, que a
/// auditoria de 2026-08-30 também deixou sem gate (a mutação *"pinta sempre em Medium"*
/// sobrevivia a 1 100 testes).
///
/// A régua é a TINTA, lida da cena emitida. ⚠️ **E ela custou duas tentativas:** contar
/// `n_paths` e `n_path_segments` dá **zero** nos dois (um glifo não entra na cena como
/// caminho, entra por `draw_glyphs`), e contar os glifos dá **17 nos dois** (é a mesma
/// string). O que separa os pesos é o **eixo normalizado da fonte VARIÁVEL** —
/// `resources.normalized_coords` —, que é literalmente onde o peso viaja.
/// ⛔ Não é comparar duas construções: é perguntar à saída.
#[test]
fn the_ink_carries_the_weight_that_was_measured() {
    let axes = |bold: bool| {
        let mut text = TextSystem::without_system_fonts();
        let mut scene = VectorScene::new();
        let name = "Tropism Direction";
        let f = if bold {
            paint_text_title_elided
        } else {
            paint_text_elided
        };
        f(
            &mut text,
            &mut scene,
            name,
            0.0,
            0.0,
            13.0,
            f32::INFINITY,
            Color::from_rgba8(255, 255, 255, 255),
        );
        scene.inner().encoding().resources.normalized_coords.clone()
    };
    assert_ne!(
        axes(true),
        axes(false),
        "as duas portas pintaram a MESMA tinta — o peso nao chega ao pintor"
    );
}

/// A tinta de um pintor de uma linha: os glifos (id e posição), as transformações e os dados de
/// desenho, ao bit (o `Debug` de um `f32` é a forma mais curta que volta ao mesmo bit).
fn ink(f: impl FnOnce(&mut TextSystem, &mut VectorScene)) -> String {
    let mut text = TextSystem::without_system_fonts();
    let mut scene = VectorScene::new();
    f(&mut text, &mut scene);
    let e = scene.inner().encoding();
    format!(
        "{:?}|{:?}|{:?}|{:?}",
        e.resources.glyphs, e.transforms, e.draw_data, e.resources.normalized_coords
    )
}

/// ⭐ Espaçamento `0.0` = o pintor de sempre, ao bit — no texto que cabe, no que se corta e no
/// que não cabe nem a reticência.
#[test]
fn zero_tracking_paints_the_same_ink_as_the_title_painter_to_the_bit() {
    let white = Color::from_rgba8(255, 255, 255, 255);
    for (name, max_w) in [
        ("SHORT FILM — DOING", f32::INFINITY),
        ("SHORT FILM — DOING", 60.0),
        ("Tropism Direction", 31.5),
        ("REVIEW", 4.0),
        ("Ação ÇÃO 字", 200.0),
    ] {
        let plain =
            ink(|ts, sc| paint_text_title_elided(ts, sc, name, 3.0, 7.0, 15.0, max_w, white));
        let tracked = ink(|ts, sc| {
            paint_text_title_elided_tracked(ts, sc, name, [3.0, 7.0], 15.0, max_w, white, 0.0);
        });
        assert_eq!(plain, tracked, "{name} a {max_w}");
    }
}

/// ⭐ Com espaçamento, mede e pinta com o MESMO valor: o texto que mede `w` cabe a `w` inteiro e
/// corta-se a `w − 1` — onde sem espaçamento caberia.
#[test]
fn tracking_measures_and_cuts_with_the_value_it_paints() {
    let white = Color::from_rgba8(255, 255, 255, 255);
    let (name, px, em) = ("SHORT FILM — DOING", 15.0, 0.04);
    let mut ts = TextSystem::without_system_fonts();
    let w = ts.prefix_width_tracked(name, px, FontWeight::SEMI_BOLD, em);
    let plain_w = ts.prefix_width_tracked(name, px, FontWeight::SEMI_BOLD, 0.0);
    assert!(
        w - 1.0 > plain_w,
        "o espaçamento abre o texto (de {plain_w} para {w})"
    );
    let glyphs = |max_w: f32| {
        let mut ts = TextSystem::without_system_fonts();
        let mut sc = VectorScene::new();
        paint_text_title_elided_tracked(&mut ts, &mut sc, name, [0.0, 0.0], px, max_w, white, em);
        sc.inner().encoding().resources.glyphs.clone()
    };
    let whole = glyphs(w);
    let cut = glyphs(w - 1.0);
    assert_eq!(whole.len(), name.chars().count(), "a `w` cabe inteiro");
    assert!(
        cut.len() < whole.len(),
        "a `w − 1` corta: {} glifos",
        cut.len()
    );
    // A tinta tem o espaçamento: o último glifo está mais à direita do que sem ele.
    let last_plain = ink(|ts, sc| paint_text_title_elided(ts, sc, name, 0.0, 0.0, px, w, white));
    assert_ne!(
        last_plain,
        ink(|ts, sc| {
            paint_text_title_elided_tracked(ts, sc, name, [0.0, 0.0], px, w, white, em);
        })
    );
    let x_last = whole.last().map(|g| g.x).unwrap_or_default();
    let mut ts2 = TextSystem::without_system_fonts();
    let mut sc2 = VectorScene::new();
    paint_text_title_elided(&mut ts2, &mut sc2, name, 0.0, 0.0, px, f32::INFINITY, white);
    let x_last_plain = sc2
        .inner()
        .encoding()
        .resources
        .glyphs
        .last()
        .map(|g| g.x)
        .unwrap_or_default();
    assert!(x_last > x_last_plain, "{x_last} > {x_last_plain}");
}

/// ⭐ `v2_dica.png`: o véu do modal só aparecia do último texto do HUD para a direita e para baixo.
/// O texto pede o transform de novo ao próximo desenho, e colar uma camada vazia apagava o pedido: o
/// preenchimento herdava o transform do glifo. *Mutação: tirar o `force_next…` de `append_transformed`.*
#[test]
fn o_preenchimento_depois_de_um_texto_e_de_uma_camada_vazia_nao_herda_o_transform_do_glifo() {
    let mut ts = TextSystem::new();
    let mut scene = VectorScene::new();
    let branco = Color::WHITE;
    // O fundo: sem um transform igual no fluxo, o vello re-emitia-o de qualquer modo.
    scene.fill_rect(ph2d_vector::Rect::new(0.0, 0.0, 9.0, 9.0), branco);
    paint_text(
        &mut ts,
        &mut scene,
        "default-scene",
        1380.0,
        971.0,
        12.0,
        200.0,
        branco,
    );
    scene.append_transformed(&VectorScene::new(), ph2d_vector::Affine::IDENTITY);
    let antes = scene.inner().encoding().transforms.len();
    scene.fill_rect(ph2d_vector::Rect::new(0.0, 0.0, 9.0, 9.0), branco);
    let e = scene.inner().encoding();
    assert_eq!(
        e.transforms.len(),
        antes + 1,
        "o preenchimento não emitiu o transform dele"
    );
    assert_eq!(e.transforms.last().map(|t| t.translation), Some([0.0, 0.0]));
}

/// ⭐ 18: o «·» do resumo do Variant abria a 2.ª linha. Em toda largura em que a frase quebra, nenhuma
/// linha começa por ele. *Mutação: `cola_o_separador` devolve o texto como veio.*
#[test]
fn um_texto_que_quebra_nunca_comeca_uma_linha_pelo_separador() {
    let frase = "Follows the prefab \u{b7} 2 unused \u{b7} 1 added \u{b7} 3 removed";
    let colado = cola_o_separador(frase);
    let mut ts = TextSystem::new();
    let (mut quebrou, mut caiu_na_palavra) = (0, 0);
    for largura in 40..320 {
        let layout = ts.layout(&colado, 12.0, largura as f32);
        for (n, linha) in layout.lines().enumerate() {
            let texto = &colado[linha.text_range()];
            assert!(!texto.starts_with('\u{b7}'), "a {largura} px: {texto:?}");
            if n > 0 {
                quebrou += 1;
                caiu_na_palavra += usize::from(texto.starts_with(char::is_alphanumeric));
            }
        }
    }
    assert!(
        quebrou > 100 && caiu_na_palavra == quebrou,
        "{quebrou} × {caiu_na_palavra}"
    );
}
