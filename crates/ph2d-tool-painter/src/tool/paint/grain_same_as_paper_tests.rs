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
const MEIOS: [PaintMedia; 3] = [
    PaintMedia::Digital,
    PaintMedia::Impasto,
    PaintMedia::WetPaint,
];

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
    // A Roughness de ganho 1 (o contraste do papel visível, `ganho_do_dente`): o xadrez da régua vive em
    // `0,05..0,25`, e o ganho de omissão (1,75) achata-o — no papel visível e, desde 28/10, no grão.
    t.set_substrate_roughness(0.2);
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
    let (ma, mb) = pares
        .iter()
        .fold((0.0, 0.0), |(a, b), (x, y)| (a + x / n, b + y / n));
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
            falhas.push(format!(
                "{m:?}: ligado, o grão não segue o papel (r = {com:.3}, {nc} texels)"
            ));
        }
        if ns < 400 || sem.abs() > 0.5 {
            falhas.push(format!(
                "{m:?}: desligado, o grão já seguia o papel (r = {sem:.3}, {ns} texels)"
            ));
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
            falhas.push(format!(
                "{m:?}: o grão não segue o papel de fábrica (r = {r:.3}, {n} texels)"
            ));
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
    assert!(
        !l.grao_e_o_papel(),
        "a Aquarela leu o Same as Paper dos outros meios"
    );
    assert_eq!(t.canvas_rgba.to_vec(), l.canvas_rgba.to_vec());
}

/// O papel VISÍVEL no texel: o dente do substrato que a luz do relevo sombreia (Relief ligado), em cargas.
fn visivel(t: &PainterTool, x: usize, y: usize) -> f32 {
    t.substrate()
        .expect("o Relief está ligado")
        .tooth_loads(x as i64, y as i64)
}

/// Os pares (grão que a tinta mostra, papel visível) no miolo do traço, e o desvio-padrão de cada lado.
fn grao_e_visivel(t: &PainterTool) -> (f64, f64, f64, usize) {
    let px = &t.canvas_rgba;
    let mut pares = Vec::new();
    for y in 52..76 {
        for x in 24..104 {
            let p = &px[(y * LADO + x) * 4..][..4];
            let (r, g) = (f64::from(p[0]), f64::from(p[1]));
            if p[3] < 64 || r + g < 60.0 || f64::from(p[2]) > 160.0 {
                continue;
            }
            pares.push((r / (r + g), f64::from(visivel(t, x, y))));
        }
    }
    let n = pares.len().max(1) as f64;
    let (ma, mb) = pares
        .iter()
        .fold((0.0, 0.0), |(a, b), (x, y)| (a + x / n, b + y / n));
    let (mut sab, mut saa, mut sbb) = (0.0, 0.0, 0.0);
    for (a, b) in &pares {
        sab += (a - ma) * (b - mb);
        saa += (a - ma) * (a - ma);
        sbb += (b - mb) * (b - mb);
    }
    let r = sab / (saa * sbb).sqrt().max(1e-12);
    (r, (saa / n).sqrt(), (sbb / n).sqrt(), pares.len())
}

/// O traço com o papel do cartão Paper montado como o dono o monta: um papel de fábrica (Cold), Relief e Roughness.
fn pinta_com_o_cartao(
    media: PaintMedia,
    papel: TextureSettings,
    relief: f32,
    rough: f32,
) -> PainterTool {
    let mut t = tool(LADO as u32, media, 22.0);
    t.set_paper_field(|b| b.paper = papel);
    t.set_substrate_depth(relief);
    t.set_substrate_roughness(rough);
    let mut t = {
        // O resto do traço é o da régua de cima (o grão pela rampa verde → vermelho).
        let _ = &mut t;
        t
    };
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
    t.toggle_brush_grain_same_as_paper();
    t.on_canvas_pointer(cp([16.0, 64.0], PointerPhase::Down));
    t.on_tick(1000.0 / 60.0);
    for k in 1..=24u8 {
        t.on_canvas_pointer(cp([16.0 + 4.0 * f32::from(k), 64.0], PointerPhase::Move));
        t.on_tick(1000.0 / 60.0);
    }
    t.on_canvas_pointer(cp([112.0, 64.0], PointerPhase::Up));
    for _ in 0..4 {
        t.on_tick(1000.0 / 60.0);
    }
    t
}

/// ⭐ **Smoke do dono 28/10: «corresponde PARCIALMENTE, mas não em todas as propriedades».** A régua mede contra o
/// papel VISÍVEL (o dente que o Relief sombreia, com a Roughness do cartão), não contra a altura crua: com cada
/// controlo do cartão Paper mexido (tipo, Size, Angle, Roughness), o grão do traço continua a desenhar o papel que se
/// vê (`r ≥ 0,8`), e a Roughness muda o CONTRASTE dos dois na mesma direcção. Mais: o Relief e a Roughness chegam ao
/// spec do traço como chegam ao papel visível (a profundidade e o ganho do dente).
#[test]
fn same_as_paper_acompanha_cada_controlo_do_cartao_paper() {
    let cold = super::substrate_relief::default_paper();
    let casos: [(&str, TextureSettings, f32, f32); 5] = [
        ("Cold, Roughness 0,1", cold, 1.0, 0.1),
        ("Cold, Roughness 0,9", cold, 1.0, 0.9),
        (
            "Cold, Size 0,5",
            TextureSettings {
                size: [0.5, 0.5],
                ..cold
            },
            1.0,
            0.5,
        ),
        (
            "Cold, Angle 45",
            TextureSettings {
                angle_deg: 45,
                ..cold
            },
            1.0,
            0.5,
        ),
        (
            "Rough",
            TextureSettings {
                kind: TextureKind::PaperRough,
                ..cold
            },
            0.8,
            0.5,
        ),
    ];
    let mut falhas = Vec::new();
    for m in MEIOS {
        let mut desvios = Vec::new();
        for (nome, papel, relief, rough) in casos {
            let t = pinta_com_o_cartao(m, papel, relief, rough);
            let (r, sg, sv, n) = grao_e_visivel(&t);
            eprintln!("{m:?} {nome}: r = {r:.3}, σ grão {sg:.4}, σ visível {sv:.4}, {n} texels");
            if n < 400 || r < 0.8 {
                falhas.push(format!(
                    "{m:?} {nome}: o grão não desenha o papel visível (r = {r:.3}, {n})"
                ));
            }
            desvios.push((sg, sv));
            // O spec do traço leva o Relief e a Roughness do cartão (a profundidade e o contraste do dente).
            let s = t.stroke_spec();
            let ganho = super::substrate_relief::ganho_do_dente(rough);
            if (s.texture.lei.contraste - ganho).abs() > 1e-6 {
                falhas.push(format!(
                    "{m:?} {nome}: contraste {} ≠ o do papel {ganho}",
                    s.texture.lei.contraste
                ));
            }
        }
        // A Roughness: o papel visível ganha contraste de 0,1 para 0,9 — o grão tem de ganhar na mesma ordem.
        let (g0, v0) = desvios[0];
        let (g1, v1) = desvios[1];
        let (rv, rg) = (v1 / v0.max(1e-9), g1 / g0.max(1e-9));
        if rg < 1.0 + 0.5 * (rv - 1.0) {
            falhas.push(format!(
                "{m:?}: Roughness — o papel visível ×{rv:.2}, o grão ×{rg:.2}"
            ));
        }
        // O Relief: a profundidade do grão segue-o (0,6 → 1 com a Depth do Grain inteira).
        let d = |relief| {
            let mut t = tool(LADO as u32, m, 22.0);
            t.set_paper_field(|b| b.paper = cold);
            t.set_substrate_depth(relief);
            t.toggle_brush_grain_same_as_paper();
            t.stroke_spec().grain_depth
        };
        let (d6, d10) = (d(0.6), d(1.0));
        if d10 <= d6 * 1.3 {
            falhas.push(format!(
                "{m:?}: Relief 0,6 → 1 não aprofunda o grão ({d6} → {d10})"
            ));
        }
    }
    assert!(falhas.is_empty(), "{}", falhas.join("\n"));
}

/// O recto do Wet com as pontas rectas e o «Same as Paper» ligado; `desliga_antes_de_soltar` alterna a opção do
/// pincel VIVO depois do último dab da mão e antes de soltar — o FECHO pousa com o pincel do lote.
fn recto_com_fecho(desliga_antes_de_soltar: bool) -> Vec<u8> {
    let mut t = tool(LADO as u32, PaintMedia::WetPaint, 22.0);
    t.set_wet_relogio_fixo(true);
    t.set_brush_color_srgb8([30, 60, 220]);
    t.set_brush_texture_kind(TextureKind::Noise.to_u8());
    t.set_paper_field(|b| b.paper = papel());
    t.toggle_brush_grain_same_as_paper();
    t.toggle_brush_pontas_rectas();
    t.set_brush_pontas_alem(1.0);
    t.set_substrate_roughness(0.2);
    // O grão do pincel cresce com ele (`escala_da_textura` ≠ 1): o «Same as Paper» é o que NÃO cresce.
    t.paint
        .wetpaint
        .knobs
        .set(ph2d_wet_paint::tuning::Knob::TextureSizeExp, 1.0);
    t.on_canvas_pointer(cp([16.0, 64.0], PointerPhase::Down));
    t.on_tick(1000.0 / 60.0);
    for k in 1..=16u8 {
        t.on_canvas_pointer(cp([16.0 + 4.0 * f32::from(k), 64.0], PointerPhase::Move));
        t.on_tick(1000.0 / 60.0);
    }
    if desliga_antes_de_soltar {
        t.toggle_brush_grain_same_as_paper();
    }
    t.on_canvas_pointer(cp([80.0, 64.0], PointerPhase::Up));
    for _ in 0..4 {
        t.on_tick(1000.0 / 60.0);
    }
    t.canvas_rgba.to_vec()
}

/// ⭐ **O lote pinta com o «Same as Paper» do SEU pincel** (auditoria J6, motor 7): o fecho das pontas rectas (e o
/// lote adiado) leva o pincel do instante dos seus dabs; alternar a opção do pincel vivo antes de soltar não muda a
/// imagem do grão nem a escala dele no que esse lote pousa.
#[test]
fn o_lote_pinta_com_o_same_as_paper_do_seu_pincel() {
    let fica = recto_com_fecho(false);
    let alterna = recto_com_fecho(true);
    let difere = fica
        .chunks(4)
        .zip(alterna.chunks(4))
        .filter(|(a, b)| a != b)
        .count();
    let tinta = fica.chunks(4).filter(|p| p[0] < 250 || p[2] < 250).count();
    println!(
        "[same as paper · fecho] texels com tinta {tinta} · diferentes ao alternar antes de soltar {difere}"
    );
    assert!(tinta > 400, "o recto não pintou: {tinta}");
    assert_eq!(
        difere, 0,
        "o fecho leu o «Same as Paper» do pincel vivo: {difere} texels mudaram"
    );
}
