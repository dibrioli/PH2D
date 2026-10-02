//! **O papel na aguada: a tinta assenta nos VALES** (BUGS_painter #30, 2026-10-02).
//!
//! O papel entra na aquarela por dois termos que leem a mesma altura `h` quando "Same as Paper" está
//! ligado: a granulação (`1 − k·h·γ`, vales) e o Tooth. O Tooth puxava para os PICOS (`h − 0,5`) e os
//! dois anulavam-se — a Granulação 1 / Tooth 1 o produto corria `0,535..0,584` sobre um `h` de
//! `0,19..0,54`, e o dono via o mesmo traço com qualquer papel. Estes gates medem o LADO, que é o que
//! nenhum gate anterior media (o `paper_depth_and_granulation_re_render_the_wet_wash` só pede "mudou").

use super::super::media::PaintMedia;
use super::*;
use ph2d_painter_brush::{TextureKind, TextureMapping};

const W: u32 = 128;

/// Um traço de aquarela horizontal no estado de fábrica do app, sobre o papel Cold.
fn aguada(granulation: f32, tooth: f32) -> PainterTool {
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; (W * W * 4) as usize], W, W);
    t.set_paint_media(PaintMedia::Watercolor);
    t.paint.brush.color = [0.2, 0.3, 0.8];
    t.paint.brush.radius_px = 14.0;
    t.set_brush_paper_kind(TextureKind::PaperCold.to_u8());
    t.paint.brush.granulation = granulation;
    t.set_brush_paper_depth(tooth);
    t.on_canvas_pointer(cp([24.0, 64.0], PointerPhase::Down));
    let mut x = 24.0;
    while x < 104.0 {
        x += 2.0;
        t.on_canvas_pointer(cp([x, 64.0], PointerPhase::Move));
        frame(&mut t);
    }
    t.on_canvas_pointer(cp([x, 64.0], PointerPhase::Up));
    frame(&mut t);
    t
}

/// Correlação de Pearson entre o ESCURECIMENTO do miolo do traço e a altura do papel no mesmo texel.
/// Negativa = mais tinta onde o papel é baixo (os vales).
fn correlacao_com_o_papel(t: &PainterTool) -> f64 {
    let p = t.paint.brush.paper;
    let rot = ph2d_painter_brush::texture::angle_basis(p.angle_deg);
    let (mut hs, mut ds) = (Vec::new(), Vec::new());
    for y in 60..69_i64 {
        for x in 40..88_i64 {
            let i = ((y * i64::from(W) + x) * 4) as usize;
            let px = &t.canvas_rgba[i..i + 3];
            let escuro = 255.0 - px.iter().map(|&c| f64::from(c)).sum::<f64>() / 3.0;
            hs.push(f64::from(
                ph2d_painter_brush::texture::sample_tiled_rot_wrapped(
                    &p,
                    x,
                    y,
                    None,
                    rot,
                    [0.0, 0.0],
                ),
            ));
            ds.push(escuro);
        }
    }
    let n = hs.len() as f64;
    let (mh, md) = (hs.iter().sum::<f64>() / n, ds.iter().sum::<f64>() / n);
    let cov: f64 = hs.iter().zip(&ds).map(|(h, d)| (h - mh) * (d - md)).sum();
    let vh: f64 = hs.iter().map(|h| (h - mh).powi(2)).sum();
    let vd: f64 = ds.iter().map(|d| (d - md).powi(2)).sum();
    cov / (vh * vd).sqrt().max(1e-12)
}

/// ⭐ O LADO da lei, nas duas pontas: o padrão de fábrica e tudo no máximo.
///
/// Medido (2026-10-02): fábrica `−0,524` · máximo `−0,684` · Tooth 2 `−0,759`; com a lei antiga a
/// fábrica lia `+0,029`, e sem papel nenhum (granulação e Tooth a `0`) o mesmo miolo lê `−0,178` — o
/// limiar `−0,35` fica longe dos dois lados.
///
/// **Mutação que tem de sangrar:** voltar o `paper_component` a `(paper_h − 0,5)` em
/// `watercolor_rewet_px.rs` (medido: vermelho, `+0,029`).
#[test]
fn o_tooth_e_a_granulacao_assentam_a_tinta_nos_vales() {
    for (nome, gran, tooth) in [
        ("fábrica", 0.3, 1.0),
        ("máximo", 1.0, 1.0),
        ("tooth 2", 1.0, 2.0),
    ] {
        let r = correlacao_com_o_papel(&aguada(gran, tooth));
        assert!(
            r < -0.35,
            "{nome}: a tinta tem de assentar nos VALES do papel (correlação escuro×altura < −0,35), leu {r:.3}"
        );
    }
}

/// O Tooth vai de `0` a `2`, e o `2` é um degrau que se vê — não um teto que o motor corta a `1`.
///
/// **Mutação que tem de sangrar:** voltar qualquer um dos `clamp(0, 1)` (o setter, o estilo por traço
/// ou o render) — o setter lê `1`, os outros dão a imagem do Tooth 1.
#[test]
fn o_tooth_vai_de_zero_a_dois_e_o_dois_se_ve() {
    let mut t = PainterTool::default();
    t.set_brush_paper_depth(3.0);
    assert_eq!(
        t.paint.brush.paper_depth,
        ph2d_painter_brush::PAPER_TOOTH_MAX
    );
    assert_eq!(ph2d_painter_brush::PAPER_TOOTH_MAX, 2.0);
    t.set_brush_paper_depth(-1.0);
    assert_eq!(t.paint.brush.paper_depth, 0.0);
    let um = correlacao_com_o_papel(&aguada(0.3, 1.0));
    let dois = correlacao_com_o_papel(&aguada(0.3, 2.0));
    assert!(
        dois < um - 0.02,
        "o Tooth 2 tem de prender MAIS a tinta aos vales que o 1 (leu {dois:.3} contra {um:.3})"
    );
}

/// O Grain nasce **Tiled** (dono, 2026-10-02), e o reset da secção volta a Tiled, não ao View.
///
/// **Mutação que tem de sangrar:** voltar o `texture` do `BrushSpec::default` a
/// `TextureSettings::default()`, ou o `reset_brush_texture` a ele.
#[test]
fn o_grain_nasce_tiled_e_o_reset_volta_a_tiled() {
    let mut t = PainterTool::default();
    assert_eq!(t.paint.brush.texture.mapping, TextureMapping::Tiled);
    t.set_brush_texture_mapping(TextureMapping::ViewPlane.to_u8());
    t.reset_brush_texture();
    assert_eq!(t.paint.brush.texture.mapping, TextureMapping::Tiled);
    assert_eq!(
        t.brush_settings().texture_mapping,
        TextureMapping::Tiled.to_u8()
    );
}

/// Em Tiled a unidade do Size é a do papel (`px·size/256`): um procedural a `1` é uma mancha de 256 px e
/// o traço sai liso. Escolher o Grain dá o tamanho da CLASSE, e só na troca de classe.
///
/// **Mutação que tem de sangrar:** apagar o bloco do tamanho em `set_brush_texture_kind`.
#[test]
fn o_grain_tiled_escolhe_o_tamanho_da_classe_do_padrao() {
    use super::super::watercolor_settings::PAPER_PROCEDURAL_DEFAULT_SIZE as FINO;
    let mut t = PainterTool::default();
    t.set_brush_texture_kind(TextureKind::Noise.to_u8());
    assert_eq!(
        t.paint.brush.texture.size,
        [FINO, FINO],
        "procedural em Tiled = dente fino"
    );
    t.paint.brush.texture.size = [7.0, 7.0];
    t.set_brush_texture_kind(TextureKind::Voronoi.to_u8());
    assert_eq!(
        t.paint.brush.texture.size,
        [7.0, 7.0],
        "entre procedurais o Size do artista fica"
    );
    t.set_brush_texture_kind(TextureKind::PaperCold.to_u8());
    assert_eq!(
        t.paint.brush.texture.size,
        [1.0, 1.0],
        "ladrilho assado = um ladrilho por 256 px"
    );
    // Em View o Size é por pegada, e a regra não se aplica.
    let mut v = PainterTool::default();
    v.set_brush_texture_mapping(TextureMapping::ViewPlane.to_u8());
    v.set_brush_texture_kind(TextureKind::Noise.to_u8());
    assert_eq!(v.paint.brush.texture.size, [1.0, 1.0]);
}
