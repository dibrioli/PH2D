//! **A rampa do Shape indexa a TEXTURA do Shape, como a do Grain indexa o grão** — prompt 38 (dono,
//! 15/10, smoke do P36): «em shape parece atuar sobre o falloff e não sobre a textura como deveria ser…
//! compare com grain». A régua mede pelo PRODUTO: num carimbo, a matiz `t = b/(r+b)` (rampa vermelho →
//! azul: `t` = o índice) contra o PADRÃO (α do mesmo Shape sem rampa e queda dura) e contra a QUEDA (α
//! sem Shape, a mesma queda). Doc 38 / handoff P38.

use super::*;
use crate::tool::paint::PaintMedia;
use ph2d_color::{ColorRamp, RampColorMode, RampInterp, RampStop};
use ph2d_painter_brush::texture::{TextureKind, TextureMapping};

const W: u32 = 96;
const R: f32 = 30.0;
const C: f32 = 48.5;

#[derive(Clone, Copy, Debug, PartialEq)]
enum Forma {
    Checker,
    Noise,
    /// Uma imagem 32×32 em degradê horizontal (a luminância é o valor do Shape).
    Imagem,
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum Queda {
    Dura,
    Macia,
}

/// Quem pinta a cor: nenhuma rampa, as camadas do Shape em cor, ou a rampa do Grain em cor (a
/// referência). As camadas: a Texture Color e a Falloff Color (as duas com a rampa vermelho → azul, a
/// Falloff ao contrário quando as duas estão ligadas, para a mistura se ler), a ordem, o Blend e o Mix.
#[derive(Clone, Copy, Debug, PartialEq)]
enum Rampa {
    Off,
    Textura,
    Queda,
    Ambas {
        queda_em_cima: bool,
        blend: ph2d_painter_brush::BrushBlend,
        mix: f32,
    },
    Grain,
}

impl Rampa {
    fn do_shape(self) -> bool {
        !matches!(self, Rampa::Off | Rampa::Grain)
    }
}

fn vermelho_azul() -> ColorRamp {
    ColorRamp::new(
        vec![
            RampStop::new(0.0, [1.0, 0.0, 0.0, 1.0]),
            RampStop::new(1.0, [0.0, 0.0, 1.0, 1.0]),
        ],
        RampColorMode::Rgb,
        RampInterp::Linear,
    )
}

/// Um carimbo no centro. `forma` vai para o Shape (View, Size 1,5 — vários ladrilhos no dab), ou para
/// o GRAIN com `Rampa::Grain` (a referência: o mesmo padrão como grão).
fn carimbo(meio: PaintMedia, forma: Option<Forma>, queda: Queda, rampa: Rampa) -> PainterTool {
    carimbo_em(meio, forma, queda, rampa, TextureMapping::ViewPlane)
}

/// [`carimbo`] com o Mapping do Shape escolhido: o View vai ao carimbo assado, o Tiled ao por pixel.
fn carimbo_em(
    meio: PaintMedia,
    forma: Option<Forma>,
    queda: Queda,
    rampa: Rampa,
    mapping: TextureMapping,
) -> PainterTool {
    let mut t = PainterTool::default();
    t.set_source(vec![0u8; (W * W * 4) as usize], W, W);
    t.set_paint_media(meio);
    let b = &mut t.paint.brush;
    b.radius_px = R;
    b.color = [0.0, 0.0, 0.0];
    b.space_attenuation = false;
    b.watercolor_shape_auto = false;
    (b.falloff, b.hardness) = match queda {
        Queda::Dura => (Falloff::Constant, 1.0),
        Queda::Macia => (Falloff::Smooth, 0.0),
    };
    let kind = |f: Forma| match f {
        Forma::Checker => TextureKind::Checker,
        Forma::Noise => TextureKind::Noise,
        Forma::Imagem => TextureKind::Image,
    };
    match (forma, rampa) {
        (Some(f), Rampa::Grain) => {
            t.set_brush_texture_kind(kind(f).to_u8());
            t.set_brush_texture_mapping(TextureMapping::ViewPlane.to_u8());
            t.paint.brush.texture.size = [1.5; 2];
            t.set_texture_ramp(vermelho_azul());
            t.set_texture_ramp_enabled(true);
            t.paint.texture_ramp_bw = false;
        }
        (Some(Forma::Imagem), _) => {
            let lum = (0..32 * 32).map(|i| ((i % 32) * 255 / 31) as u8).collect();
            t.set_brush_shape_image(lum, 32, 32);
        }
        (Some(f), _) => {
            t.set_brush_shape_kind(kind(f).to_u8());
            t.paint.brush.shape.mapping = mapping;
            // O Size MULTIPLICA a coordenada: no View a unidade é o raio, presa à tela é o ladrilho base.
            t.paint.brush.shape.size = [if mapping.is_canvas_fixed() { 6.0 } else { 1.5 }; 2];
        }
        (None, _) => {}
    }
    t.set_shape_color_ramp(vermelho_azul());
    t.set_shape_falloff_ramp(vermelho_azul());
    t.set_shape_ramp_enabled(rampa.do_shape());
    t.paint.shape_color_ramp_bw = false;
    let (textura, queda) = match rampa {
        Rampa::Queda => (false, true),
        Rampa::Ambas { .. } => (true, true),
        _ => (true, false),
    };
    t.paint.shape_camadas.textura = textura;
    t.paint.shape_camadas.queda = queda;
    t.set_shape_falloff_ramp(vermelho_azul()); // marca as camadas mudadas (re-assa as LUTs)
    if let Rampa::Ambas {
        queda_em_cima,
        blend,
        mix,
    } = rampa
    {
        let mut inv = vermelho_azul();
        inv.invert();
        t.set_shape_falloff_ramp(inv);
        if queda_em_cima {
            t.swap_shape_color_layers();
        }
        t.set_shape_color_blend(blend.to_u8());
        // `mix` = a opacidade da camada de CIMA.
        if queda_em_cima {
            t.set_shape_falloff_opacity(mix);
        } else {
            t.set_shape_texture_opacity(mix);
        }
    }
    let seed = t.paint.brush;
    t.paint.brush_by_mode.fill(seed);
    t.on_canvas_pointer(cp([C, C], PointerPhase::Down));
    t.on_canvas_pointer(cp([C, C], PointerPhase::Up));
    t
}

/// Os texels do MIOLO do carimbo (fora da orla AA), `(rgb reto, α)`.
fn miolo(t: &PainterTool) -> Vec<([f32; 3], f32)> {
    let mut v = Vec::new();
    for y in 0..W {
        for x in 0..W {
            let (dx, dy) = (x as f32 + 0.5 - C, y as f32 + 0.5 - C);
            if dx * dx + dy * dy >= (R - 2.0) * (R - 2.0) {
                continue;
            }
            let i = ((y * W + x) * 4) as usize;
            let p = &t.canvas_rgba[i..i + 4];
            v.push((
                [
                    f32::from(p[0]) / 255.0,
                    f32::from(p[1]) / 255.0,
                    f32::from(p[2]) / 255.0,
                ],
                f32::from(p[3]) / 255.0,
            ));
        }
    }
    v
}

/// Correlação de Pearson; `None` quando um dos lados não varia (o número não diria nada).
fn correlacao(a: &[f32], b: &[f32]) -> Option<f32> {
    let n = a.len() as f32;
    let (ma, mb) = (a.iter().sum::<f32>() / n, b.iter().sum::<f32>() / n);
    let (mut sab, mut saa, mut sbb) = (0.0, 0.0, 0.0);
    for (x, y) in a.iter().zip(b) {
        sab += (x - ma) * (y - mb);
        saa += (x - ma) * (x - ma);
        sbb += (y - mb) * (y - mb);
    }
    (saa > 1e-6 && sbb > 1e-6).then(|| sab / (saa * sbb).sqrt())
}

/// O que a régua lê de um caso: correlação matiz × padrão, matiz × queda, e a cobertura média
/// relativa à da queda sozinha (1 = o padrão não recorta; < 1 = recorta).
#[derive(Debug)]
struct Leitura {
    padrao: Option<f32>,
    queda: Option<f32>,
    cobertura: f32,
    n: usize,
}

fn le(meio: PaintMedia, forma: Forma, queda: Queda, rampa: Rampa) -> Leitura {
    le_em(meio, forma, queda, rampa, TextureMapping::ViewPlane)
}

fn le_em(
    meio: PaintMedia,
    forma: Forma,
    queda: Queda,
    rampa: Rampa,
    mapping: TextureMapping,
) -> Leitura {
    // O padrão: o α do mesmo Shape (ou grão) sem rampa e com a queda DURA — `valor × 1` no miolo.
    let referencia = if rampa == Rampa::Grain {
        Rampa::Grain
    } else {
        Rampa::Off
    };
    let padrao_de = carimbo_em(
        PaintMedia::Digital,
        Some(forma),
        Queda::Dura,
        referencia,
        mapping,
    );
    let padrao: Vec<f32> = if rampa == Rampa::Grain {
        // Com a rampa do Grain em cor o grão não morde: o padrão é a própria matiz, a queda dura.
        miolo(&padrao_de)
            .iter()
            .map(|(c, _)| c[2] / (c[0] + c[2]).max(1e-6))
            .collect()
    } else {
        miolo(&padrao_de).iter().map(|p| p.1).collect()
    };
    let so_queda: Vec<f32> = miolo(&carimbo(meio, None, queda, Rampa::Off))
        .iter()
        .map(|p| p.1)
        .collect();
    let caso = miolo(&carimbo_em(meio, Some(forma), queda, rampa, mapping));
    let (mut t, mut p, mut q) = (Vec::new(), Vec::new(), Vec::new());
    for (i, (c, a)) in caso.iter().enumerate() {
        if *a > 0.05 {
            t.push(c[2] / (c[0] + c[2]).max(1e-6));
            p.push(padrao[i]);
            q.push(so_queda[i]);
        }
    }
    let soma = |v: &[f32]| v.iter().sum::<f32>();
    Leitura {
        padrao: correlacao(&t, &p),
        queda: correlacao(&t, &q),
        cobertura: soma(&caso.iter().map(|p| p.1).collect::<Vec<_>>()) / soma(&so_queda).max(1e-6),
        n: t.len(),
    }
}

#[test]
#[ignore = "régua do prompt 38 — corra por nome com --ignored --nocapture"]
fn regua_a_rampa_do_shape_segue_a_textura() {
    let f = |c: Option<f32>| c.map_or("—".to_owned(), |c| format!("{c:.3}"));
    println!(
        "\n| meio | forma | queda | rampa | corr(matiz, padrão) | corr(matiz, queda) | cobertura | n |"
    );
    println!("|---|---|---|---|---|---|---|---|");
    for meio in [
        PaintMedia::Digital,
        PaintMedia::Impasto,
        PaintMedia::Watercolor,
    ] {
        for forma in [Forma::Checker, Forma::Noise, Forma::Imagem] {
            for queda in [Queda::Dura, Queda::Macia] {
                for rampa in [Rampa::Textura, Rampa::Queda, Rampa::Grain] {
                    if rampa == Rampa::Grain
                        && (forma == Forma::Imagem || meio != PaintMedia::Digital)
                    {
                        continue; // o Grain é a referência do Digital; a imagem não é um grão aqui
                    }
                    let l = le(meio, forma, queda, rampa);
                    println!(
                        "| {meio:?} | {forma:?} | {queda:?} | {rampa:?} | {} | {} | {:.3} | {} |",
                        f(l.padrao),
                        f(l.queda),
                        l.cobertura,
                        l.n
                    );
                }
            }
        }
    }
}

/// **A Texture Color segue a TEXTURA do Shape, como a do Grain segue o grão** (prompt 38): a matiz de
/// cada texel acompanha o padrão (correlação ≥ 0,95) e o padrão procedural deixa de recortar (a
/// cobertura é a da queda sozinha), nas duas rotas — o carimbo assado (Shape View) e o por pixel (Tiled)
/// — com queda dura e macia, no Digital e no Impasto; na aguada, que espalha a cor, ≥ 0,85 no Noise.
/// Controlo: a Falloff Color sozinha, na queda macia, segue a queda e não o padrão — a régua distingue.
#[test]
fn a_texture_color_segue_a_textura_do_shape() {
    use TextureMapping::{Tiled, ViewPlane};
    let controlo = le(
        PaintMedia::Digital,
        Forma::Checker,
        Queda::Macia,
        Rampa::Queda,
    );
    assert!(
        controlo.queda.is_some_and(|c| c >= 0.9) && controlo.padrao.is_some_and(|c| c <= 0.5),
        "controlo: a Falloff Color sozinha tem de seguir a queda ({controlo:?})"
    );
    for meio in [PaintMedia::Digital, PaintMedia::Impasto] {
        for forma in [Forma::Checker, Forma::Noise] {
            for queda in [Queda::Dura, Queda::Macia] {
                for mapping in [ViewPlane, Tiled] {
                    let l = le_em(meio, forma, queda, Rampa::Textura, mapping);
                    assert!(
                        l.padrao.is_some_and(|c| c >= 0.95),
                        "a cor não segue a textura do Shape ({meio:?}, {forma:?}, {queda:?}, \
                         {mapping:?}): {l:?}"
                    );
                    assert!(
                        l.cobertura >= 0.99,
                        "o padrão do Shape ainda recorta com a Texture Color ({meio:?}, {forma:?}, \
                         {queda:?}, {mapping:?}): {l:?}"
                    );
                }
            }
        }
    }
    for queda in [Queda::Dura, Queda::Macia] {
        let l = le(PaintMedia::Watercolor, Forma::Noise, queda, Rampa::Textura);
        assert!(
            l.padrao.is_some_and(|c| c >= 0.85),
            "na aguada a cor não segue a textura do Shape ({queda:?}): {l:?}"
        );
    }
}

/// **As duas camadas misturam-se na ordem escolhida** (prompt 38: «organizar qual é o primeiro, com
/// Blend entre eles»): com Mix a 1 a de CIMA manda — a Texture Color segue o padrão, a Falloff Color
/// (trocada para cima) a queda —, e com Mix a 0 aparece a de baixo. E sem Shape as duas camadas leem o
/// mesmo índice: Texture só e Falloff só pintam o mesmo byte (a LUT combinada do carimbo em cache).
#[test]
fn as_camadas_misturam_na_ordem_escolhida() {
    use ph2d_painter_brush::BrushBlend::Mix;
    let caso = |queda_em_cima, mix| {
        le(
            PaintMedia::Digital,
            Forma::Checker,
            Queda::Macia,
            Rampa::Ambas {
                queda_em_cima,
                blend: Mix,
                mix,
            },
        )
    };
    let textura_em_cima = caso(false, 1.0);
    assert!(
        textura_em_cima.padrao.is_some_and(|c| c.abs() >= 0.95),
        "Texture Color em cima com Mix 1 tem de mandar: {textura_em_cima:?}"
    );
    let queda_em_cima = caso(true, 1.0);
    assert!(
        queda_em_cima.queda.is_some_and(|c| c.abs() >= 0.9)
            && queda_em_cima.padrao.is_some_and(|c| c.abs() <= 0.2),
        "trocada a ordem, a Falloff Color em cima tem de mandar: {queda_em_cima:?}"
    );
    let invisivel = caso(true, 0.0);
    assert!(
        invisivel.padrao.is_some_and(|c| c.abs() >= 0.95),
        "com Mix 0 a camada de baixo (a Texture Color) tem de aparecer: {invisivel:?}"
    );
    // Smoke do dono (15/10): «o mesmo color ramp está atuando para textura e para o falloff». Sem
    // textura no Shape a Texture Color não tem o que colorir: pinta a cor do pincel, e a Falloff não.
    let so = |r| carimbo(PaintMedia::Digital, None, Queda::Macia, r).canvas_rgba;
    assert_eq!(
        so(Rampa::Textura),
        so(Rampa::Off),
        "sem textura no Shape a Texture Color não pode pintar (pintou a rampa pela queda)"
    );
    assert_ne!(
        so(Rampa::Queda),
        so(Rampa::Off),
        "controlo: sem Shape a Falloff Color pinta a rampa pela queda"
    );
    // E com as DUAS ligadas: sem textura só a Falloff pinta, logo a ordem não muda um byte.
    let ambas = |queda_em_cima| {
        so(Rampa::Ambas {
            queda_em_cima,
            blend: Mix,
            mix: 1.0,
        })
    };
    assert_eq!(
        ambas(false),
        ambas(true),
        "sem textura no Shape a Texture Color pintou por cima da Falloff Color"
    );
}

/// A tela sobre branco, em PPM.
fn ppm(rgba: &[u8], caminho: &std::path::Path) {
    let mut out = format!("P6\n{W} {W}\n255\n").into_bytes();
    for p in rgba.chunks(4) {
        let a = f32::from(p[3]) / 255.0;
        for &c in &p[..3] {
            out.push((f32::from(c) * a + 255.0 * (1.0 - a)).round() as u8);
        }
    }
    std::fs::write(caminho, out).unwrap();
}

#[test]
#[ignore = "prancha do prompt 38 — PH2D_PRANCHA_DIR=<pasta> e --ignored"]
fn prancha_rampa_do_shape() {
    use ph2d_painter_brush::BrushBlend::{Mix, Multiply};
    let dir =
        std::path::PathBuf::from(std::env::var("PH2D_PRANCHA_DIR").expect("PH2D_PRANCHA_DIR"));
    std::fs::create_dir_all(&dir).unwrap();
    for forma in [Forma::Checker, Forma::Noise, Forma::Imagem] {
        for queda in [Queda::Dura, Queda::Macia] {
            let nome = format!("{forma:?}_{queda:?}");
            let colunas = [
                ("1_grain", Rampa::Grain),
                ("2_textura", Rampa::Textura),
                ("3_queda", Rampa::Queda),
                (
                    "5_ambas_mix",
                    Rampa::Ambas {
                        queda_em_cima: true,
                        blend: Mix,
                        mix: 0.5,
                    },
                ),
                (
                    "6_ambas_multiply",
                    Rampa::Ambas {
                        queda_em_cima: true,
                        blend: Multiply,
                        mix: 1.0,
                    },
                ),
            ];
            for (col, r) in colunas {
                if r == Rampa::Grain && forma == Forma::Imagem {
                    continue;
                }
                let t = carimbo(PaintMedia::Digital, Some(forma), queda, r);
                ppm(&t.canvas_rgba, &dir.join(format!("{nome}_{col}.ppm")));
            }
            // O desenho A, RECUSADO: a cor da textura mas o padrão a recortar (α × o padrão).
            let tex = carimbo(PaintMedia::Digital, Some(forma), queda, Rampa::Textura).canvas_rgba;
            let pad =
                carimbo(PaintMedia::Digital, Some(forma), Queda::Dura, Rampa::Off).canvas_rgba;
            let mut a = tex.to_vec();
            for (p, q) in a.chunks_mut(4).zip(pad.chunks(4)) {
                p[3] = ((u16::from(p[3]) * u16::from(q[3]) + 127) / 255) as u8;
            }
            ppm(&a, &dir.join(format!("{nome}_4_recusado_recorta.ppm")));
        }
    }
}

/// **O alfa de cada cor da rampa é o alfa da tinta, e a Opacity de cada camada multiplica-o** (dono
/// 15/10: *«que o alpha não precise desses modos mas que obedeça o alfa de cada cor do color ramp»*).
/// Falloff Color só, rampa vermelha: alfa 1 e Opacity 1 pintam vermelho cheio; Opacity 0 não pinta;
/// Opacity ½ ou uma cor de alfa ½ pintam meia tinta — vermelha, nunca a cor do pincel (verde). E duas
/// camadas: a de cima com cor transparente deixa ver a de baixo.
#[test]
fn o_alfa_de_cada_cor_e_a_opacity_sao_a_tinta() {
    let vermelha = |a: f32| {
        ColorRamp::new(
            vec![
                RampStop::new(0.0, [1.0, 0.0, 0.0, a]),
                RampStop::new(1.0, [1.0, 0.0, 0.0, a]),
            ],
            RampColorMode::Rgb,
            RampInterp::Linear,
        )
    };
    let centro = |alfa: f32, opacidade: f32, textura_por_baixo: bool| {
        let mut t = PainterTool::default();
        t.set_source(vec![0u8; (W * W * 4) as usize], W, W);
        t.paint.brush.radius_px = R;
        t.paint.brush.color = [0.0, 1.0, 0.0];
        if textura_por_baixo {
            t.set_brush_shape_kind(TextureKind::Checker.to_u8());
            t.paint.brush.shape.size = [0.01; 2]; // um ladrilho só: o valor do padrão é uniforme
        }
        let seed = t.paint.brush;
        t.paint.brush_by_mode.fill(seed);
        t.set_shape_ramp_enabled(true);
        t.paint.shape_color_ramp_bw = false;
        t.paint.shape_camadas.textura = textura_por_baixo;
        t.paint.shape_camadas.queda = true;
        t.paint.shape_camadas.queda_primeiro = true;
        t.set_shape_color_ramp(ColorRamp::new(
            vec![
                RampStop::new(0.0, [0.0, 0.0, 1.0, 1.0]),
                RampStop::new(1.0, [0.0, 0.0, 1.0, 1.0]),
            ],
            RampColorMode::Rgb,
            RampInterp::Linear,
        ));
        t.set_shape_falloff_ramp(vermelha(alfa));
        t.set_shape_falloff_opacity(opacidade);
        t.on_canvas_pointer(cp([C, C], PointerPhase::Down));
        t.on_canvas_pointer(cp([C, C], PointerPhase::Up));
        let i = ((48 * W + 48) * 4) as usize;
        [
            t.canvas_rgba[i],
            t.canvas_rgba[i + 1],
            t.canvas_rgba[i + 2],
            t.canvas_rgba[i + 3],
        ]
    };
    let cheia = centro(1.0, 1.0, false);
    assert!(
        cheia[0] > 240 && cheia[1] < 15 && cheia[3] > 240,
        "alfa 1, Opacity 1: vermelho cheio: {cheia:?}"
    );
    let nada = centro(1.0, 0.0, false);
    assert!(nada[3] < 5, "Opacity 0: não pinta: {nada:?}");
    for (nome, px) in [
        ("Opacity ½", centro(1.0, 0.5, false)),
        ("alfa ½", centro(0.5, 1.0, false)),
    ] {
        assert!(
            (100..=160).contains(&px[3]) && px[0] > 200 && px[1] < 15,
            "{nome}: meia tinta VERMELHA (sem a cor do pincel): {px:?}"
        );
    }
    let vazada = centro(0.0, 1.0, true);
    assert!(
        vazada[2] > 200 && vazada[0] < 15 && vazada[3] > 240,
        "a de cima transparente deixa ver a de baixo (azul): {vazada:?}"
    );
}

/// **Com B&W as duas camadas fazem o tom** (dono 15/10: «B&W está fechando um dos color ramps. Não
/// permita»): a Falloff Color por cima, opaca, de cinza constante — preta apaga a ponta, branca deixa-a
/// inteira. Controlo: só a Texture Color (a de sempre) dá o tom de fábrica dela.
#[test]
fn com_bw_as_duas_camadas_fazem_o_tom() {
    let cinza = |v: f32| {
        ColorRamp::new(
            vec![
                RampStop::new(0.0, [v, v, v, 1.0]),
                RampStop::new(1.0, [v, v, v, 1.0]),
            ],
            RampColorMode::Rgb,
            RampInterp::Linear,
        )
    };
    let tinta = |queda: Option<f32>| {
        let mut t = PainterTool::default();
        t.set_source(vec![0u8; (W * W * 4) as usize], W, W);
        t.paint.brush.radius_px = R;
        t.set_brush_shape_kind(TextureKind::Noise.to_u8());
        let seed = t.paint.brush;
        t.paint.brush_by_mode.fill(seed);
        t.set_shape_ramp_enabled(true);
        t.paint.shape_color_ramp_bw = true;
        t.paint.shape_camadas.textura = true;
        t.paint.shape_camadas.queda = queda.is_some();
        t.paint.shape_camadas.queda_primeiro = true;
        t.set_shape_falloff_ramp(cinza(queda.unwrap_or(0.0)));
        t.on_canvas_pointer(cp([C, C], PointerPhase::Down));
        t.on_canvas_pointer(cp([C, C], PointerPhase::Up));
        t.canvas_rgba
            .chunks(4)
            .map(|p| u64::from(p[3]))
            .sum::<u64>()
    };
    let (preta, branca, so_textura) = (tinta(Some(0.0)), tinta(Some(1.0)), tinta(None));
    assert!(so_textura > 0, "controlo: o tom da Texture Color pinta");
    assert!(
        preta * 10 < branca && branca > so_textura,
        "com B&W a Falloff Color por cima tem de fazer o tom: preta {preta}, branca {branca}, só a \
         textura {so_textura}"
    );
}
