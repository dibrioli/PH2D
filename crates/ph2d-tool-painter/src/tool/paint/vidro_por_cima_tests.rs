//! **Outro pincel por cima da aguada** (smoke do dono 2026-10-07, com fotos: *«wet paint sobre
//! watercolor previamente pintada em fundo escuro forma essa outline branca. No fundo branco não
//! aparece»*; BUGS #46, 2.ª volta): o vidro CONTINUA o que o Wet Paint, o Digital, o Impasto e a
//! borracha fizeram por cima — o oráculo é o mesmo traço numa camada PRÓPRIA, sem vidro (entre camadas
//! a lei é exacta e o alfa único de uma camada transparente também).

#![allow(clippy::cast_precision_loss)]

use super::measure_shape_system::cp;
use super::papel_nasce_tests::{GESTOS, LADO, escolhe_o_papel, imagem, novo, pinta_com_a_cor};
use super::vidro_tests::PAPEL;
use crate::tool::paint::media::PaintMedia;
use ph2d_editor_core::tool::{CanvasPaintTool, PointerPhase};

/// O contorno claro (smoke do dono 2026-10-07, com foto: *«wet paint sobre watercolor previamente
/// pintada em fundo escuro forma essa outline branca. No fundo branco não aparece»*). Uma aguada seca,
/// um traço de `meio` por cima, o papel castanho: nos texels da aguada que o traço só ROÇOU (a camada
/// mudou até `roçou` níveis), quantos se mostram mais claros que a aguada sozinha por mais de 20.
fn contorno_claro(meio: PaintMedia, rocou: u8) -> (usize, usize) {
    let aguada = || {
        let mut t = novo(PaintMedia::Watercolor, 14.0);
        t.set_brush_color_srgb8([255, 0, 0]);
        pinta_com_a_cor(&mut t, 1.0, &(GESTOS[2].1)());
        t.dry_session_now();
        t
    };
    let mut so = aguada();
    let camada_so = so.canvas_rgba.to_vec();
    escolhe_o_papel(&mut so, PAPEL);
    let g = imagem(&mut so);
    let mut t = aguada();
    t.set_paint_media(meio);
    t.set_brush_size_px(16.0);
    t.set_brush_color_srgb8([200, 20, 20]);
    pinta_com_a_cor(&mut t, 1.0, &(GESTOS[1].1)());
    escolhe_o_papel(&mut t, PAPEL);
    let a = imagem(&mut t);
    let (mut rocados, mut claros) = (0, 0);
    for k in 0..LADO * LADO {
        let (antes, depois) = (
            &camada_so[k * 4..k * 4 + 4],
            &t.canvas_rgba[k * 4..k * 4 + 4],
        );
        let d = antes
            .iter()
            .zip(depois)
            .map(|(x, y)| x.abs_diff(*y))
            .max()
            .unwrap_or(0);
        if antes[3] > 0 && d > 0 && d <= rocou {
            rocados += 1;
            let lum = |p: &[u8]| {
                0.2126 * f32::from(p[0]) + 0.7152 * f32::from(p[1]) + 0.0722 * f32::from(p[2])
            };
            claros += usize::from(lum(&a[k * 4..k * 4 + 3]) > lum(&g[k * 4..k * 4 + 3]) + 20.0);
        }
    }
    (rocados, claros)
}

/// SONDA — o contorno claro de um traço por cima de uma aguada seca, no papel castanho, por meio.
/// `cargo test -p ph2d-tool-painter --profile smoke --lib diag_o_contorno_claro -- --ignored --nocapture`
#[test]
#[ignore = "diagnóstico"]
fn diag_o_contorno_claro() {
    for meio in [
        PaintMedia::WetPaint,
        PaintMedia::Digital,
        PaintMedia::Impasto,
        PaintMedia::Watercolor,
    ] {
        for rocou in [2u8, 8, 32] {
            let (r, c) = contorno_claro(meio, rocou);
            eprintln!(
                "[contorno] {meio:<11?} roçou ≤ {rocou:>2}: {r:>5} texels roçados · {c:>5} mais claros que a aguada (+20)"
            );
        }
    }
}

/// Um traço de `meio` por cima de uma aguada seca, na MESMA camada (o produto) e numa camada PRÓPRIA
/// por cima (o oráculo: entre camadas a lei do vidro é exacta), sobre o papel castanho: o pior e
/// quantos bytes passam de 2.
fn por_cima_da_aguada(meio: PaintMedia, strength: f32, papel: [u8; 3]) -> (u8, usize) {
    let aguada = |camada_propria: bool| {
        let mut t = novo(PaintMedia::Watercolor, 14.0);
        t.set_brush_color_srgb8([255, 0, 0]);
        pinta_com_a_cor(&mut t, 1.0, &(GESTOS[2].1)());
        t.dry_session_now();
        if camada_propria {
            t.add_raster_layer("por cima").expect("a camada de cima");
        }
        t.set_paint_media(meio);
        t.set_brush_size_px(16.0);
        t.set_brush_color_srgb8([200, 20, 20]);
        pinta_com_a_cor(&mut t, strength, &(GESTOS[1].1)());
        if camada_propria {
            // O oráculo não lê a lei do vidro: numa camada transparente o alfa único é exacto.
            let cima = t.layers.active().expect("a camada de cima");
            t.vidros.remove(&cima);
        }
        escolhe_o_papel(&mut t, papel);
        imagem(&mut t)
    };
    let (a, b) = (aguada(false), aguada(true));
    a.iter().zip(&b).fold((0, 0), |(p, n), (x, y)| {
        let d = x.abs_diff(*y);
        (p.max(d), n + usize::from(d > 2))
    })
}

/// SONDA — o traço por cima da aguada, na mesma camada contra numa camada própria.
/// `cargo test -p ph2d-tool-painter --profile smoke --lib diag_por_cima_da_aguada -- --ignored --nocapture`
#[test]
#[ignore = "diagnóstico"]
fn diag_por_cima_da_aguada() {
    for meio in [
        PaintMedia::Digital,
        PaintMedia::WetPaint,
        PaintMedia::Impasto,
    ] {
        for s in [1.0f32, 0.6] {
            let (pb, nb) = por_cima_da_aguada(meio, s, [255, 255, 255]);
            let (p, n) = por_cima_da_aguada(meio, s, PAPEL);
            eprintln!(
                "[por cima] {meio:<9?} s {s}: branco pior {pb:>3} · {nb:>5} bytes > 2 | castanho pior {p:>3} · {n:>5} bytes > 2"
            );
        }
    }
}

/// ⭐ **UM TRAÇO POR CIMA DA AGUADA, NA MESMA CAMADA, É O MESMO QUE NUMA CAMADA PRÓPRIA** (smoke do dono
/// 2026-10-07: *«wet paint sobre watercolor previamente pintada em fundo escuro forma essa outline
/// branca»*): Digital, Wet Paint e Impasto, Strength 1 (o tecto desarmado: a cobertura sai do `over`) e
/// 0,6 (o tecto), sobre o papel castanho, contra o mesmo traço numa camada por cima — até ao piso das
/// duas representações medido NO BRANCO, onde o vidro não age (`3` no Digital). E o contorno: nenhum
/// texel da aguada que o traço só roçou se mostra mais claro que ela. Vermelho antes: o Wet Paint
/// deixava `33`–`40` texels de contorno claro, o Digital `33`–`95`, o Impasto `12`–`24`, e o traço
/// mostrava-se até `33` níveis longe da camada própria.
#[test]
fn um_traco_por_cima_da_aguada_e_o_mesmo_que_numa_camada_propria() {
    for meio in [
        PaintMedia::Digital,
        PaintMedia::WetPaint,
        PaintMedia::Impasto,
    ] {
        for s in [1.0f32, 0.6] {
            let (p, n) = por_cima_da_aguada(meio, s, PAPEL);
            assert!(
                p <= 3,
                "{meio:?} s {s}: o traço por cima da aguada difere da camada própria — pior {p} níveis, {n} bytes > 2"
            );
        }
        let (rocados, claros) = contorno_claro(meio, 32);
        assert!(rocados > 0, "controlo: {meio:?} roçou a aguada");
        assert_eq!(claros, 0, "{meio:?}: contorno claro em {claros} texels");
    }
}

/// **A BORRACHA SOBRE A AGUADA NÃO A ACENDE** — apagar a meio uma aguada seca, no papel castanho: cada
/// texel fica entre o papel e a aguada inteira (apagar `k` dela é mostrar `k` dela), canal a canal, ±2.
#[test]
fn a_borracha_sobre_a_aguada_nao_a_acende() {
    let aguada = || {
        let mut t = novo(PaintMedia::Watercolor, 14.0);
        t.set_brush_color_srgb8([255, 0, 0]);
        pinta_com_a_cor(&mut t, 1.0, &(GESTOS[2].1)());
        t.dry_session_now();
        t
    };
    let mut so = aguada();
    escolhe_o_papel(&mut so, PAPEL);
    let g = imagem(&mut so);
    let mut t = aguada();
    t.set_paint_media(PaintMedia::Digital);
    t.toggle_brush_eraser();
    t.set_brush_size_px(16.0);
    pinta_com_a_cor(&mut t, 0.5, &(GESTOS[1].1)());
    escolhe_o_papel(&mut t, PAPEL);
    let a = imagem(&mut t);
    let mut apagados = 0;
    for k in 0..LADO * LADO {
        if g[k * 4..k * 4 + 3] == a[k * 4..k * 4 + 3] {
            continue;
        }
        apagados += 1;
        for c in 0..3 {
            let (lo, hi) = (PAPEL[c].min(g[k * 4 + c]), PAPEL[c].max(g[k * 4 + c]));
            assert!(
                a[k * 4 + c].saturating_add(2) >= lo && a[k * 4 + c] <= hi.saturating_add(2),
                "texel {k} canal {c}: {} fora de [papel {} · aguada {}] — a borracha acendeu a aguada",
                a[k * 4 + c],
                PAPEL[c],
                g[k * 4 + c]
            );
        }
    }
    assert!(
        apagados > 100,
        "controlo: a borracha apagou ({apagados} texels)"
    );
}

/// **O QUADRO AO VIVO JÁ MOSTRA O VIDRO SEGUIDO** — com o papel castanho escolhido ANTES, um traço
/// Digital por cima de uma aguada: o que se vê antes de largar a caneta é o que fica depois dela.
#[test]
fn o_quadro_ao_vivo_ja_mostra_o_vidro_seguido() {
    let mut t = novo(PaintMedia::Watercolor, 14.0);
    t.set_brush_color_srgb8([255, 0, 0]);
    pinta_com_a_cor(&mut t, 1.0, &(GESTOS[2].1)());
    t.dry_session_now();
    escolhe_o_papel(&mut t, PAPEL);
    let _ = imagem(&mut t);
    t.set_paint_media(PaintMedia::Digital);
    t.set_brush_size_px(16.0);
    t.set_brush_color_srgb8([200, 20, 20]);
    let pts = (GESTOS[1].1)();
    t.on_canvas_pointer(cp(pts[0], PointerPhase::Down));
    for p in &pts[1..] {
        t.on_canvas_pointer(cp(*p, PointerPhase::Move));
        t.paint_tick(1.0 / 60.0);
    }
    let ao_vivo = t.take_preview_arc().expect("o quadro").0.to_vec();
    let tela_ao_vivo = t.canvas_rgba.to_vec();
    t.on_canvas_pointer(cp(*pts.last().expect("pontos"), PointerPhase::Up));
    let depois = imagem(&mut t);
    // Só os texels que o pen-up não repintou (a cauda do traço deposita ao largar).
    let (mut pior, mut iguais) = (0u8, 0usize);
    for k in 0..LADO * LADO {
        if tela_ao_vivo[k * 4..k * 4 + 4] != t.canvas_rgba[k * 4..k * 4 + 4] {
            continue;
        }
        iguais += 1;
        for c in 0..3 {
            pior = pior.max(ao_vivo[k * 4 + c].abs_diff(depois[k * 4 + c]));
        }
    }
    assert!(iguais > 1000, "controlo: {iguais} texels");
    assert_eq!(
        pior, 0,
        "o quadro ao vivo mostrou o vidro de antes do traço"
    );
}

/// **O WET PAINT AO VIVO SOBRE A AGUADA** — papel castanho escolhido antes, o fluido a meio do traço
/// (antes de largar a caneta: é o composite do Wet Paint quem escreve), na mesma camada contra numa
/// camada própria, até ao mesmo piso.
#[test]
fn o_wet_paint_ao_vivo_sobre_a_aguada() {
    let corre = |propria: bool| {
        let mut t = novo(PaintMedia::Watercolor, 14.0);
        t.paint.brush.opacity = 0.0;
        t.set_brush_color_srgb8([255, 0, 0]);
        pinta_com_a_cor(&mut t, 1.0, &(GESTOS[2].1)());
        t.dry_session_now();
        escolhe_o_papel(&mut t, PAPEL);
        if propria {
            t.add_raster_layer("por cima").expect("a camada de cima");
        }
        t.set_paint_media(PaintMedia::WetPaint);
        t.set_wet_relogio_fixo(true);
        t.set_brush_size_px(16.0);
        t.set_brush_color_srgb8([200, 20, 20]);
        let pts = (GESTOS[1].1)();
        t.on_canvas_pointer(cp(pts[0], PointerPhase::Down));
        for p in &pts[1..] {
            t.on_canvas_pointer(cp(*p, PointerPhase::Move));
            t.paint_tick(1.0 / 60.0);
        }
        // O fluido assenta com a caneta parada (o pigmento deposita-se com o tempo).
        for _ in 0..90 {
            t.paint_tick(1.0 / 60.0);
        }
        if propria {
            // O oráculo não lê a lei do vidro: numa camada transparente o alfa único é exacto.
            let cima = t.layers.active().expect("a camada de cima");
            t.vidros.remove(&cima);
        }
        imagem(&mut t)
    };
    let (a, b) = (corre(false), corre(true));
    assert_ne!(a, corre_sem_traco(), "controlo: o fluido pintou");
    let pior = a
        .iter()
        .zip(&b)
        .map(|(x, y)| x.abs_diff(*y))
        .max()
        .unwrap_or(0);
    assert!(
        pior <= 3,
        "o Wet Paint ao vivo difere da camada própria em {pior} níveis"
    );
}

fn corre_sem_traco() -> Vec<u8> {
    let mut t = novo(PaintMedia::Watercolor, 14.0);
    t.paint.brush.opacity = 0.0;
    t.set_brush_color_srgb8([255, 0, 0]);
    pinta_com_a_cor(&mut t, 1.0, &(GESTOS[2].1)());
    t.dry_session_now();
    escolhe_o_papel(&mut t, PAPEL);
    imagem(&mut t)
}
