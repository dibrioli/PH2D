//! O preview vivo: anima, mostra a obra (o papel de cor e o dente), muda com cada parâmetro, desiste.

use super::*;
use crate::WetTool;
use ph2d_painter_brush::{TextureKind, TextureMapping};
use ph2d_wet_paint::tuning::Knob;

/// Um parâmetro mexido: o nome, como se mexe, e se o traço final tem de mudar com ele.
type Caso<'a> = (&'a str, &'a dyn Fn(&mut PainterTool), bool);

fn molhado() -> PainterTool {
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; 64 * 64 * 4], 64, 64);
    t.set_paint_media(PaintMedia::WetPaint);
    t.set_brush_size_px(20.0);
    t.set_brush_color_srgb8([23, 101, 179]);
    t
}

fn quadros(t: &PainterTool) -> Vec<Vec<u8>> {
    let mut v = Vec::new();
    assert!(pintar_preview(
        &t.wet_preview_cena(),
        &|| false,
        &mut |px, w, h| {
            assert_eq!((w, h), (LADO_MINIATURA, LADO_MINIATURA));
            v.push(px.to_vec());
        }
    ));
    v
}

/// Quantos píxeis diferem do canto (o canto é o papel, longe do S).
fn tinta(px: &[u8]) -> usize {
    let canto = &px[..4];
    px.as_chunks::<4>()
        .0
        .iter()
        .filter(|p| p[..] != canto[..])
        .count()
}

#[test]
fn o_preview_anima_o_traco_e_a_agua() {
    let q = quadros(&molhado());
    assert!(
        q.len() >= (QUADROS_DO_TRACO + TICKS_DEPOIS / TICKS_POR_QUADRO) as usize - 2,
        "{} quadros",
        q.len()
    );
    let cobertura: Vec<usize> = q.iter().map(|p| tinta(p)).collect();
    assert!(cobertura[0] > 0, "o primeiro quadro já tem traço");
    assert!(
        cobertura[0] * 4 < cobertura[QUADROS_DO_TRACO as usize - 1],
        "o traço cresce ao longo dos quadros: {cobertura:?}"
    );
    assert_ne!(
        q[QUADROS_DO_TRACO as usize - 1],
        q[q.len() - 1],
        "a água ainda corre depois do pen-up"
    );
}

#[test]
fn o_preview_mostra_o_papel_de_cor() {
    let creme = [236, 226, 200];
    let mut t = molhado();
    t.papel = Some(creme);
    let ultimo = quadros(&t).pop().unwrap();
    assert_eq!(
        ultimo[..3],
        creme,
        "longe do traço vê-se o papel do documento"
    );
    let branco = quadros(&molhado()).pop().unwrap();
    assert_eq!(branco[..3], [255, 255, 255]);
}

#[test]
fn o_preview_mostra_o_dente_do_papel() {
    let com_papel = |t: &mut PainterTool| {
        t.paint.brush.paper.kind = TextureKind::PaperCold;
        t.paint.brush.paper.mapping = TextureMapping::Tiled;
        t.sync_paper_across_slots();
        t.paint.wetpaint.paper_visual = true;
    };
    let mut t = molhado();
    com_papel(&mut t);
    let liso = quadros(&molhado()).pop().unwrap();
    let dente = quadros(&t).pop().unwrap();
    assert_ne!(liso, dente, "o dente do papel muda a tinta do preview");
}

#[test]
fn a_chave_muda_com_cada_parametro_e_o_traco_tambem() {
    let base = molhado();
    let ultimo_base = quadros(&base).pop().unwrap();
    let casos: [Caso<'_>; 14] = [
        (
            "pigmento",
            &|t| t.paint.wetpaint.knobs.set(Knob::PigmentPerDab, 1500.0),
            true,
        ),
        ("água", &|t| t.paint.wetpaint.knobs.water = 0.2, true),
        ("tamanho", &|t| t.set_brush_size_px(8.0), true),
        ("cor", &|t| t.set_brush_color_srgb8([191, 0, 18]), true),
        ("papel de cor", &|t| t.papel = Some([200, 220, 240]), true),
        (
            "tipo de papel",
            &|t| {
                t.paint.brush.paper.kind = TextureKind::PaperRough;
                t.sync_paper_across_slots();
            },
            true,
        ),
        (
            "papel visível",
            &|t| t.paint.wetpaint.paper_visual = true,
            false,
        ),
        ("inclinação", &|t| t.paint.wetpaint.tilt_spoke = 0, false),
        // As gotas do PAPEL (W1b): só pingam com a tela inclinada e depois de a orla encher (mais que os
        // 2 s do preview) — a chave tem de as ver; a tinta, a bancada das gotas a mede.
        ("criar gotas", &|t| t.gotas.create = false, false),
        ("tamanho da gota", &|t| t.gotas.size = 9, false),
        ("gotas fáceis", &|t| t.gotas.easy = true, false),
        ("comprimento da gota", &|t| t.gotas.length = 2, false),
        ("grade", &|t| t.paint.wetpaint.grid_ratio = 3, true),
        (
            "forma",
            &|t| t.set_brush_shape_image(vec![255; 9 * 9], 9, 9),
            false,
        ),
    ];
    for (nome, mexe, pinta_diferente) in casos {
        let mut t = molhado();
        mexe(&mut t);
        assert_ne!(
            t.wet_preview_chave(),
            base.wet_preview_chave(),
            "{nome}: a chave não viu"
        );
        if pinta_diferente {
            assert_ne!(
                quadros(&t).pop().unwrap(),
                ultimo_base,
                "{nome}: o preview não mudou"
            );
        }
    }
}

#[test]
fn quando_a_cena_muda_o_pintor_desiste() {
    let mut n = 0;
    let completo = pintar_preview(&molhado().wet_preview_cena(), &|| true, &mut |_, _, _| {
        n += 1
    });
    assert!(!completo);
    assert_eq!(n, 0);
}

#[test]
fn o_preview_pinta_o_pincel_mesmo_com_o_smear_ativo() {
    let mut t = molhado();
    let ultimo_pincel = quadros(&t).pop().unwrap();
    t.paint.wetpaint.tool = WetTool::Smear;
    assert_eq!(quadros(&t).pop().unwrap(), ultimo_pincel);
}

/// **Sonda (ignorada): grava o preview sobre quatro obras** em `PH2D_SONDA_OUT` (PPM, lado a lado):
/// branca · papel creme · papel Cold com o papel visível · o mesmo com o relevo do substrato.
#[test]
#[ignore = "sonda: grava o preview sobre quatro obras; corre-se à mão"]
fn sonda_do_preview_sobre_a_obra() {
    let out = std::env::var("PH2D_SONDA_OUT").unwrap_or_else(|_| "/tmp".into());
    let obras: [&dyn Fn(&mut PainterTool); 4] = [
        &|_| {},
        &|t| t.papel = Some([236, 226, 200]),
        &|t| {
            t.papel = Some([236, 226, 200]);
            t.paint.brush.paper.kind = TextureKind::PaperCold;
            t.paint.brush.paper.mapping = TextureMapping::Tiled;
            t.sync_paper_across_slots();
            t.paint.wetpaint.paper_visual = true;
        },
        &|t| {
            t.papel = Some([236, 226, 200]);
            t.paint.brush.paper.kind = TextureKind::PaperCold;
            t.paint.brush.paper.mapping = TextureMapping::Tiled;
            t.sync_paper_across_slots();
            t.paint.wetpaint.paper_visual = true;
            t.paint.substrate_depth = 1.0;
        },
    ];
    let l = LADO_MINIATURA as usize;
    let mut tira = vec![0u8; l * 4 * l * 3];
    for (i, obra) in obras.iter().enumerate() {
        let mut t = molhado();
        obra(&mut t);
        let px = quadros(&t).pop().unwrap();
        for y in 0..l {
            for x in 0..l {
                let s = (y * l + x) * 4;
                let d = (y * l * 4 + i * l + x) * 3;
                tira[d..d + 3].copy_from_slice(&px[s..s + 3]);
            }
        }
    }
    let mut f = format!("P6\n{} {l}\n255\n", l * 4).into_bytes();
    f.extend_from_slice(&tira);
    std::fs::write(format!("{out}/preview_obras.ppm"), f).unwrap();
}
