//! ⭐ **O Grain «Same as Paper» nos outros meios** (dono 28/10: *«no modo Watercolor, em Grain, temos a opção
//! Same as Paper. Habilite o mesmo para todos os modos de pintura»*). Ligado, o grão do pincel É o papel do
//! documento, preso à tela ([`PainterTool::grao_e_o_papel`] → `substrate_relief::papel_do_documento`); a régua
//! é a correlação, texel a texel, entre o valor do grão que a tinta mostra (as Grain Colors verde → vermelho)
//! e a altura do papel nesse texel. Desligado, a tinta é a de hoje (o grão próprio, que não segue o papel).
//!
//! **Mutações que sangram:** `grao_e_o_papel` a devolver `false` (o Digital, o Impasto e o Wet perdem a
//! correlação); `papel_do_documento` a devolver o `tex` cru (sem papel escolhido o grão some).

use super::measure_shape_system::{cp, tool};
use crate::tool::PainterTool;
use crate::tool::paint::media::PaintMedia;
use ph2d_color::{ColorRamp, RampColorMode, RampInterp, RampStop};
use ph2d_editor_core::Tool;
use ph2d_editor_core::tool::{CanvasPaintTool, PointerPhase};
use ph2d_painter_brush::texture::{self, TextureSettings};
use ph2d_painter_brush::{TextureKind, TextureMapping};

const LADO: usize = 128;

/// Os meios que ganharam a opção (a Aquarela tem a sua, `granulation_use_paper`).
const MEIOS: [PaintMedia; 3] = [PaintMedia::Digital, PaintMedia::Impasto, PaintMedia::WetPaint];

/// O papel do documento da régua: um xadrez preso à tela, de casas grandes o bastante para o carimbo as ver.
fn papel() -> TextureSettings {
    TextureSettings {
        kind: TextureKind::Checker,
        mapping: TextureMapping::Tiled,
        size: [0.15, 0.15],
        ..TextureSettings::default()
    }
}

/// A altura do papel `0..1` no texel, pela mesma amostragem do substrato (`Tiled`: centro e raio inertes).
fn altura(tex: &TextureSettings, x: usize, y: usize) -> f32 {
    let mut rng = 0u64;
    let b = texture::dab_basis(
        tex,
        &mut rng,
        [0.0, 0.0],
        ph2d_painter_brush::footprint::FootprintDeform::identity(),
    );
    texture::sample(tex, &b, x as i64, y as i64, [0.0, 0.0], 1.0, None)
}

/// Um traço largo e azul com as Grain Colors verde → vermelho; o grão próprio do pincel é um Noise (o de
/// hoje com a opção desligada), o papel do documento é `papel`.
fn pinta(media: PaintMedia, papel: Option<TextureSettings>, ligado: bool) -> PainterTool {
    let mut t = tool(LADO as u32, media, 22.0);
    t.set_wet_relogio_fixo(true);
    t.set_brush_color_srgb8([30, 60, 220]);
    t.set_brush_texture_kind(TextureKind::Noise.to_u8());
    t.set_texture_ramp(ColorRamp::new(
        vec![
            RampStop::new(0.0, [0.0, 1.0, 0.0, 1.0]),
            RampStop::new(1.0, [1.0, 0.0, 0.0, 1.0]),
        ],
        RampColorMode::Rgb,
        RampInterp::Linear,
    ));
    t.set_texture_ramp_enabled(true);
    if let Some(p) = papel {
        t.set_paper_field(|b| b.paper = p);
    }
    if ligado {
        t.toggle_brush_grain_same_as_paper();
    }
    t.on_canvas_pointer(cp([16.0, 64.0], PointerPhase::Down));
    t.on_tick(1000.0 / 60.0);
    for k in 1..=24u8 {
        let x = 16.0 + 4.0 * f32::from(k);
        t.on_canvas_pointer(cp([x, 64.0], PointerPhase::Move));
        t.on_tick(1000.0 / 60.0);
    }
    t.on_canvas_pointer(cp([112.0, 64.0], PointerPhase::Up));
    for _ in 0..4 {
        t.on_tick(1000.0 / 60.0);
    }
    t
}

/// Pearson entre a fração de vermelho da tinta (o valor do grão pela rampa) e a altura do papel, nos texels
/// pintados com cor legível. `(r, n)`.
fn correlacao(px: &[u8], tex: &TextureSettings) -> (f64, usize) {
    let mut pares = Vec::new();
    // O miolo do traço (a borda mistura a cobertura do carimbo com o grão).
    for y in 52..76 {
        for x in 24..104 {
            let p = &px[(y * LADO + x) * 4..][..4];
            let (r, g) = (f64::from(p[0]), f64::from(p[1]));
            // O papel branco (o fundo) não é tinta: só os texels que a rampa coloriu.
            if p[3] < 64 || r + g < 60.0 || f64::from(p[2]) > 160.0 {
                continue;
            }
            pares.push((r / (r + g), f64::from(altura(tex, x, y))));
        }
    }
    let n = pares.len() as f64;
    if n < 2.0 {
        return (0.0, pares.len());
    }
    let (ma, mb) = pares.iter().fold((0.0, 0.0), |(a, b), (x, y)| (a + x / n, b + y / n));
    let (mut sab, mut saa, mut sbb) = (0.0, 0.0, 0.0);
    for (a, b) in &pares {
        sab += (a - ma) * (b - mb);
        saa += (a - ma) * (a - ma);
        sbb += (b - mb) * (b - mb);
    }
    (sab / (saa * sbb).sqrt().max(1e-12), pares.len())
}

/// **Ligado, o grão do traço é o papel do documento; desligado, é o de hoje** — nos três meios. A
/// correlação com a altura do papel sobe acima de `0,8` com a opção e fica abaixo de `0,5` sem ela.
#[test]
fn same_as_paper_o_grao_do_traco_segue_o_papel_em_cada_meio() {
    let mut falhas = Vec::new();
    let tex = papel();
    for m in MEIOS {
        let (sem, ns) = correlacao(&pinta(m, Some(tex), false).canvas_rgba, &tex);
        let (com, nc) = correlacao(&pinta(m, Some(tex), true).canvas_rgba, &tex);
        if nc < 400 || com < 0.8 {
            falhas.push(format!("{m:?}: ligado, o grão não segue o papel (r = {com:.3}, {nc} texels)"));
        }
        if ns < 400 || sem.abs() > 0.5 {
            falhas.push(format!("{m:?}: desligado, o grão já seguia o papel (r = {sem:.3}, {ns} texels)"));
        }
    }
    assert!(falhas.is_empty(), "{}", falhas.join("\n"));
}

/// **Sem papel escolhido, o papel do documento é o de fábrica** (o mesmo que o filme do Wet mostra): ligar a
/// opção nunca deixa o pincel sem grão — o traço segue o papel de fábrica em cada meio.
#[test]
fn same_as_paper_sem_papel_escolhido_le_o_papel_de_fabrica() {
    let mut falhas = Vec::new();
    let fabrica = super::substrate_relief::default_paper();
    for m in MEIOS {
        let (r, n) = correlacao(&pinta(m, None, true).canvas_rgba, &fabrica);
        if n < 400 || r < 0.6 {
            falhas.push(format!("{m:?}: o grão não segue o papel de fábrica (r = {r:.3}, {n} texels)"));
        }
    }
    assert!(falhas.is_empty(), "{}", falhas.join("\n"));
}

/// **Na Aquarela a opção é a dela** (`granulation_use_paper`): o campo dos outros meios não lhe toca.
#[test]
fn same_as_paper_dos_outros_meios_nao_toca_a_aquarela() {
    let t = pinta(PaintMedia::Watercolor, Some(papel()), false);
    assert!(!t.grao_e_o_papel());
    let l = pinta(PaintMedia::Watercolor, Some(papel()), true);
    assert!(!l.grao_e_o_papel(), "a Aquarela leu o Same as Paper dos outros meios");
    assert_eq!(t.canvas_rgba.to_vec(), l.canvas_rgba.to_vec());
}
