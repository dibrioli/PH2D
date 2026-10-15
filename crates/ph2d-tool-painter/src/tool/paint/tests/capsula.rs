//! **O Continuous que IGUALA o Spacing pequeno** (prompt 34, doc 31 §5): o mesmo traço a Spacing 0,025 (o
//! ALVO) e a 0,25 com a caixa Continuous, nos cinco meios, com a redonda e com as cerdas. A régua é a
//! diferença píxel a píxel contra o alvo NORMALIZADO pela tinta (o CV ao longo passou a 0,055 e o dono viu
//! retângulos) e a COSTURA: a tinta das colunas dobrada pela fase do passo `D`.

use super::super::media::PaintMedia;
use super::*;
use ph2d_painter_brush::TextureKind;
use ph2d_painter_brush::capsula::{Lei, com_lei};

pub(super) const W: u32 = 480;
pub(super) const H: u32 = 128;
const X0: f32 = 60.0;
const X1: f32 = 420.0;
const EVENTOS: usize = 72;

/// Os cinco meios: o Composite é o Digital com a pilha ligada.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Meio {
    Digital,
    Impasto,
    Watercolor,
    WetPaint,
    Composite,
}

pub(super) const MEIOS: [Meio; 5] = [
    Meio::Digital,
    Meio::Impasto,
    Meio::Watercolor,
    Meio::WetPaint,
    Meio::Composite,
];

pub(super) fn pincel(
    meio: Meio,
    kind: TextureKind,
    tam: f32,
    sp: f32,
    continuo: bool,
) -> PainterTool {
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; (W * H * 4) as usize], W, H);
    t.set_paint_media(match meio {
        Meio::Digital | Meio::Composite => PaintMedia::Digital,
        Meio::Impasto => PaintMedia::Impasto,
        Meio::Watercolor => PaintMedia::Watercolor,
        Meio::WetPaint => PaintMedia::WetPaint,
    });
    if meio == Meio::Composite {
        use crate::tool::paint::composite::{CompositeLayer, CompositeOp};
        t.paint.composite_enabled = true;
        t.paint.composite[0] = CompositeLayer {
            op: CompositeOp::Brush,
            strength: 1.0,
            ..CompositeLayer::default()
        };
    }
    t.set_brush_color_srgb8([20, 30, 120]);
    t.set_wet_relogio_fixo(true);
    // A Watercolor em Automatic ignora a ponta; a régua quer a ponta.
    t.paint.brush.watercolor_shape_auto = false;
    // A ponta pedida em todo meio (a de fábrica do Wet Paint já é de cerdas); as cerdas no padrão do tipo.
    t.set_brush_shape_kind(kind.to_u8());
    t.set_brush_size_px(tam);
    t.set_brush_spacing(sp);
    t.paint.brush.continuous_spacing = continuo;
    t
}

/// O traço recto de 360 px a meia altura, um quadro por evento, e a água assenta.
pub(super) fn traco(t: &mut PainterTool) -> Vec<u8> {
    let y = H as f32 / 2.0;
    t.on_canvas_pointer(cp([X0, y], PointerPhase::Down));
    t.on_tick(1000.0 / 60.0);
    for k in 1..=EVENTOS {
        let x = X0 + (X1 - X0) * k as f32 / EVENTOS as f32;
        t.on_canvas_pointer(cp([x, y], PointerPhase::Move));
        t.on_tick(1000.0 / 60.0);
    }
    t.on_canvas_pointer(cp([X1, y], PointerPhase::Up));
    for _ in 0..30 {
        t.on_tick(1000.0 / 60.0);
    }
    t.canvas_rgba.to_vec()
}

fn tinta_em(px: &[u8], x: usize, y: usize) -> f64 {
    let p = &px[(y * W as usize + x) * 4..][..3];
    f64::from(255 - p[0].min(p[1]).min(p[2]))
}

/// A régua de um traço contra o alvo, sobre o miolo (`x` a `2r + 10` das pontas, `|y − eixo| ≤ r + 2`).
#[derive(Clone, Copy, Debug)]
pub(super) struct Regua {
    /// `Σ|â − t̂| / Σ t̂`, cada tela dividida pela sua tinta média no miolo — a FORMA, não a quantidade.
    pub dif: f64,
    /// O desvio da tinta das colunas dobrada em 8 fases do passo `D`, sobre a média.
    pub costura: f64,
    /// A mesma costura no ALVO (o piso).
    pub costura_alvo: f64,
    /// A tinta do traço sobre a do alvo, no miolo.
    pub tinta: f64,
    /// A `dif` na abertura (`x ∈ [X0 − r, X0 + 2r]`).
    pub dif_abertura: f64,
}

fn janela(px: &[u8], xs: std::ops::Range<usize>, ys: std::ops::Range<usize>) -> Vec<f64> {
    ys.flat_map(|y| xs.clone().map(move |x| (x, y)))
        .map(|(x, y)| tinta_em(px, x, y))
        .collect()
}

fn dif_normalizada(a: &[f64], t: &[f64]) -> f64 {
    let (ma, mt) = (
        a.iter().sum::<f64>() / a.len() as f64,
        t.iter().sum::<f64>() / t.len() as f64,
    );
    if ma <= 0.0 || mt <= 0.0 {
        return f64::INFINITY;
    }
    let num: f64 = a.iter().zip(t).map(|(a, t)| (a / ma - t / mt).abs()).sum();
    num / (t.len() as f64)
}

fn costura(px: &[u8], xs: std::ops::Range<usize>, ys: std::ops::Range<usize>, d: f64) -> f64 {
    let mut bins = [(0.0f64, 0u32); 8];
    for x in xs {
        let col: f64 = ys.clone().map(|y| tinta_em(px, x, y)).sum();
        let fase = ((x as f64 + 0.5 - f64::from(X0)) / d).rem_euclid(1.0);
        let b = ((fase * 8.0) as usize).min(7);
        bins[b].0 += col;
        bins[b].1 += 1;
    }
    let medias: Vec<f64> = bins
        .iter()
        .filter(|b| b.1 > 0)
        .map(|b| b.0 / f64::from(b.1))
        .collect();
    let m = medias.iter().sum::<f64>() / medias.len() as f64;
    if m <= 0.0 {
        return f64::INFINITY;
    }
    let v = medias.iter().map(|a| (a - m) * (a - m)).sum::<f64>() / medias.len() as f64;
    v.sqrt() / m
}

/// `r` o raio e `d` o passo do traço sob teste, em px.
pub(super) fn regua(alvo: &[u8], teste: &[u8], r: f32, d: f32) -> Regua {
    let eixo = (H / 2) as usize;
    let ys = eixo.saturating_sub(r as usize + 2)..(eixo + r as usize + 3).min(H as usize);
    let xs = (X0 + 2.0 * r + 10.0) as usize..(X1 - 2.0 * r - 10.0) as usize;
    let (a, t) = (
        janela(teste, xs.clone(), ys.clone()),
        janela(alvo, xs.clone(), ys.clone()),
    );
    let xa = (X0 - r).max(0.0) as usize..(X0 + 2.0 * r) as usize;
    let (aa, ta) = (
        janela(teste, xa.clone(), ys.clone()),
        janela(alvo, xa, ys.clone()),
    );
    Regua {
        dif: dif_normalizada(&a, &t),
        costura: costura(teste, xs.clone(), ys.clone(), f64::from(d)),
        costura_alvo: costura(alvo, xs, ys, f64::from(d)),
        tinta: a.iter().sum::<f64>() / t.iter().sum::<f64>().max(1.0),
        dif_abertura: dif_normalizada(&aa, &ta),
    }
}

fn grava(dir: &str, nome: &str, px: &[u8]) {
    let mut ppm = format!("P6 {W} {H} 255\n").into_bytes();
    for k in 0..(W * H) as usize {
        ppm.extend_from_slice(&px[k * 4..k * 4 + 3]);
    }
    std::fs::write(format!("{dir}/{nome}.ppm"), ppm).expect("grava");
}

/// ⭐ **O Continuous IGUALA o Spacing pequeno, nos cinco meios** (dono 13/10: *«fica diferente do traço com
/// spacing menor … RETÂNGULOS … MENOS NÍTIDO»* e *«continuous deve estar disponível para os outros modos»*):
/// a 0,25 com a caixa, a redonda e as cerdas desenham o traço a 0,025 — a régua normalizada pela tinta
/// abaixo de 0,03 (medido ≤ 0,017), a mesma tinta (± 5 %) e a costura do alvo. CONTROLOS: sem a caixa a
/// régua passa de 0,04 (salvo o Impasto redondo, um corpo sólido que já é igual), e nos meios do carimbo a
/// lei do prompt 32 (a queda do CENTRO) passa de 0,1.
#[test]
fn o_continuous_iguala_o_spacing_pequeno_nos_cinco_meios() {
    for meio in MEIOS {
        for kind in [TextureKind::None, TextureKind::Bristles] {
            let traco_a =
                |sp: f32, continuo: bool| traco(&mut pincel(meio, kind, 40.0, sp, continuo));
            let alvo = traco_a(0.025, false);
            let g = regua(&alvo, &traco_a(0.25, true), 20.0, 10.0);
            println!("{meio:?} {kind:?}: {g:?}");
            assert!(g.dif < 0.03, "{meio:?} {kind:?}: dif {:.3}", g.dif);
            assert!(
                (g.tinta - 1.0).abs() < 0.05,
                "{meio:?} {kind:?}: tinta {:.2}",
                g.tinta
            );
            assert!(
                g.costura <= g.costura_alvo + 0.003,
                "{meio:?} {kind:?}: costura {:.3} (alvo {:.3})",
                g.costura,
                g.costura_alvo
            );
            if !(meio == Meio::Impasto && kind == TextureKind::None) {
                let sem = regua(&alvo, &traco_a(0.25, false), 20.0, 10.0);
                assert!(
                    sem.dif > 0.04,
                    "{meio:?} {kind:?}: o controlo não mede ({:.3})",
                    sem.dif
                );
            }
            if kind == TextureKind::Bristles
                && matches!(meio, Meio::Digital | Meio::Impasto | Meio::Composite)
            {
                let hoje = com_lei(Lei::Centro, None, || traco_a(0.25, true));
                let h = regua(&alvo, &hoje, 20.0, 10.0);
                assert!(
                    h.dif > 0.1,
                    "{meio:?}: a lei do prompt 32 não reprova ({:.3})",
                    h.dif
                );
            }
        }
    }
}

/// **A sonda do passo 1** — cada meio × ponta × raio: o alvo (0,025 sem a caixa), 0,25 sem a caixa e
/// 0,25 com ela. `PH2D_SONDA_OUT=<pasta>` grava as telas em PPM.
/// `cargo test -p ph2d-tool-painter --profile smoke --lib sonda_capsula -- --ignored --nocapture`
#[test]
#[ignore = "sonda: o Continuous contra o Spacing pequeno"]
fn sonda_capsula() {
    let out = std::env::var("PH2D_SONDA_OUT").ok();
    println!("| meio | ponta | raio | caso | dif | costura (alvo) | tinta ÷ alvo | dif abertura |");
    println!("|---|---|---:|---|---:|---:|---:|---:|");
    for meio in MEIOS {
        for kind in [TextureKind::None, TextureKind::Bristles] {
            for tam in [40.0f32, 90.0] {
                let r = tam / 2.0;
                let alvo = traco(&mut pincel(meio, kind, tam, 0.025, false));
                let hoje = if meio == Meio::WetPaint {
                    Lei::CentroMax
                } else {
                    Lei::Centro
                };
                for (caso, sp, continuo, lei, sub) in [
                    ("0,25", 0.25f32, false, Lei::Uniao, None),
                    ("0,25 hoje", 0.25, true, hoje, None),
                    ("0,25 média", 0.25, true, Lei::Media, None),
                    ("0,25 união · sub 1 px", 0.25, true, Lei::Uniao, Some(1e-6)),
                    (
                        "0,25 união · sub 0,0125",
                        0.25,
                        true,
                        Lei::Uniao,
                        Some(0.0125),
                    ),
                    ("0,25 união · sub 0,025", 0.25, true, Lei::Uniao, None),
                    ("0,25 união · sub 0,05", 0.25, true, Lei::Uniao, Some(0.05)),
                ] {
                    let px = com_lei(lei, sub, || {
                        traco(&mut pincel(meio, kind, tam, sp, continuo))
                    });
                    let g = regua(&alvo, &px, r, sp * tam);
                    println!(
                        "| {meio:?} | {kind:?} | {r} | {caso} | {:.3} | {:.3} ({:.3}) | {:.2} | {:.3} |",
                        g.dif, g.costura, g.costura_alvo, g.tinta, g.dif_abertura
                    );
                    if let Some(dir) = &out {
                        let sub = sub.unwrap_or(0.025);
                        grava(
                            dir,
                            &format!("{meio:?}_{kind:?}_{r}_{sp}_{continuo}_{lei:?}_{sub}"),
                            &px,
                        );
                    }
                }
                if let Some(dir) = &out {
                    grava(dir, &format!("{meio:?}_{kind:?}_{r}_alvo"), &alvo);
                }
            }
        }
    }
}

/// O traço do custo: recto ou em curva, e com a pressão a variar (o raio muda por dab).
fn traco_de(t: &mut PainterTool, curva: bool, pressao: bool) {
    let ponto = |k: usize| {
        let x = X0 + (X1 - X0) * k as f32 / EVENTOS as f32;
        let y = H as f32 / 2.0 + if curva { 30.0 * (x / 40.0).sin() } else { 0.0 };
        let p = if pressao {
            0.4 + 0.6 * (k as f32 / 9.0).sin().abs()
        } else {
            1.0
        };
        CanvasPointer {
            pos: [x, y],
            pressure: p,
            tilt: [0.0, 0.0],
            phase: PointerPhase::Move,
        }
    };
    t.on_canvas_pointer(CanvasPointer {
        phase: PointerPhase::Down,
        ..ponto(0)
    });
    for k in 1..=EVENTOS {
        t.on_canvas_pointer(ponto(k));
        t.on_tick(1000.0 / 60.0);
    }
    t.on_canvas_pointer(CanvasPointer {
        phase: PointerPhase::Up,
        ..ponto(EVENTOS)
    });
}

/// **O custo** (MEDIR_VELOCIDADE: as variantes no MESMO processo, intercaladas em blocos rodados, o
/// MÍNIMO): o traço a 0,25 com o Continuous contra o alvo a 0,025 sem ele (o motivo de existir), e contra
/// 0,25 sem ele; recto, em curva e com pressão; as cerdas e o Checker com a Texture Color, raio 45.
/// `cargo test -p ph2d-tool-painter --profile smoke --lib measure_custo_da_capsula -- --ignored --nocapture`
#[test]
#[ignore = "medição: o custo do dab-cápsula"]
fn measure_custo_da_capsula() {
    use std::time::Instant;
    println!(
        "| meio | ponta | traço | 0,025 (ms) | 0,25 (ms) | cáps. sub 1 px | sub 0,0125 | sub 0,025 | sub 0,05 | 0,025 ÷ alvo |"
    );
    println!("|---|---|---|---:|---:|---:|---:|---:|---:|---:|");
    let variantes = [
        (0.025f32, false, None),
        (0.25, false, None),
        (0.25, true, Some(1e-6f32)),
        (0.25, true, Some(0.0125)),
        (0.25, true, None),
        (0.25, true, Some(0.05)),
    ];
    for meio in MEIOS {
        // A Texture Color sobre o Checker: a cor composta por sub-dab (lista 26 A9).
        for (kind, cor) in [
            (TextureKind::None, CamadaCor::Off),
            (TextureKind::Bristles, CamadaCor::Off),
            (TextureKind::Checker, CamadaCor::Textura),
        ] {
            for (nome, curva, pressao) in [
                ("recto", false, false),
                ("curva", true, false),
                ("pressão", false, true),
            ] {
                let mut min = [f64::MAX; 6];
                for bloco in 0..5 {
                    for i in 0..6 {
                        let v = (i + bloco) % 6;
                        let (sp, cont, sub) = variantes[v];
                        let mut t = pincel(meio, kind, 90.0, sp, cont);
                        if cor != CamadaCor::Off {
                            arma_cor(&mut t, cor);
                            let seed = t.paint.brush;
                            t.paint.brush_by_mode.fill(seed);
                        }
                        t.paint.dynamics.size_pressure = pressao;
                        let t0 = Instant::now();
                        com_lei(Lei::Uniao, sub, || traco_de(&mut t, curva, pressao));
                        min[v] = min[v].min(t0.elapsed().as_secs_f64() * 1e3);
                    }
                }
                println!(
                    "| {meio:?} | {kind:?} {cor:?} | {nome} | {:.1} | {:.1} | {:.1} | {:.1} | {:.1} | {:.1} | {:.2} |",
                    min[0],
                    min[1],
                    min[2],
                    min[3],
                    min[4],
                    min[5],
                    min[4] / min[0]
                );
            }
        }
    }
}

// ── A9 (lista 26): a cor do Shape no Continuous ─────────────────────────────────────────────────────

/// Quem pinta a cor: a rampa vermelho → azul na Texture Color, na Falloff Color, ou sem rampa.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum CamadaCor {
    Off,
    Textura,
    Queda,
}

pub(super) fn vermelho_azul() -> ph2d_color::ColorRamp {
    use ph2d_color::{ColorRamp, RampColorMode, RampInterp, RampStop};
    ColorRamp::new(
        vec![
            RampStop::new(0.0, [1.0, 0.0, 0.0, 1.0]),
            RampStop::new(1.0, [0.0, 0.0, 1.0, 1.0]),
        ],
        RampColorMode::Rgb,
        RampInterp::Linear,
    )
}

/// Arma a cor do Shape com a rampa vermelho → azul numa só camada (ou desliga-a).
pub(super) fn arma_cor(t: &mut PainterTool, cor: CamadaCor) {
    if cor == CamadaCor::Off {
        t.set_shape_ramp_enabled(false);
        return;
    }
    t.set_shape_color_ramp(vermelho_azul());
    t.paint.shape_color_ramp_bw = false;
    t.paint.shape_camadas.textura = cor == CamadaCor::Textura;
    t.paint.shape_camadas.queda = cor == CamadaCor::Queda;
    t.set_shape_falloff_ramp(vermelho_azul()); // re-assa as LUTs
    t.set_shape_ramp_enabled(true);
}

/// Os texels do miolo do traço de raio `r` (longe das pontas, dentro do raio + 2).
pub(super) fn miolo(r: f32) -> Vec<(usize, usize)> {
    let eixo = (H / 2) as usize;
    let ys = eixo.saturating_sub(r as usize + 2)..(eixo + r as usize + 3).min(H as usize);
    let xs = (X0 + 2.0 * r + 10.0) as usize..(X1 - 2.0 * r - 10.0) as usize;
    ys.flat_map(|y| xs.clone().map(move |x| (x, y))).collect()
}

pub(super) fn rgb(px: &[u8], x: usize, y: usize) -> [f32; 3] {
    let p = &px[(y * W as usize + x) * 4..][..3];
    [p[0].into(), p[1].into(), p[2].into()]
}

pub(super) fn correlacao(a: &[f32], b: &[f32]) -> f32 {
    let n = a.len() as f32;
    let (ma, mb) = (a.iter().sum::<f32>() / n, b.iter().sum::<f32>() / n);
    let (mut sab, mut saa, mut sbb) = (0.0, 0.0, 0.0);
    for (x, y) in a.iter().zip(b) {
        sab += (x - ma) * (y - mb);
        saa += (x - ma) * (x - ma);
        sbb += (y - mb) * (y - mb);
    }
    if saa < 1e-9 || sbb < 1e-9 {
        f32::NAN
    } else {
        sab / (saa * sbb).sqrt()
    }
}

/// A matiz `b/(r+b)` do miolo — com a rampa vermelho → azul, o índice dela.
pub(super) fn matiz(px: &[u8], r: f32) -> Vec<f32> {
    miolo(r)
        .iter()
        .map(|&(x, y)| {
            let c = rgb(px, x, y);
            c[2] / (c[0] + c[2]).max(1.0)
        })
        .collect()
}

/// Média |ΔRGB| / 255 no miolo.
pub(super) fn dif_cor(a: &[u8], b: &[u8], r: f32) -> f32 {
    let pts = miolo(r);
    let s: f32 = pts
        .iter()
        .map(|&(x, y)| {
            let (p, q) = (rgb(a, x, y), rgb(b, x, y));
            (0..3).map(|k| (p[k] - q[k]).abs()).sum::<f32>() / 3.0
        })
        .sum();
    s / pts.len() as f32 / 255.0
}

/// O traço de 40 px com a cor do Shape `cor` sobre um Shape `kind` (mapping de fábrica, ou Tiled).
fn traco_cor(
    meio: Meio,
    kind: TextureKind,
    cor: CamadaCor,
    tiled: bool,
    sp: f32,
    cont: bool,
) -> Vec<u8> {
    let mut t = pincel(meio, kind, 40.0, sp, cont);
    if tiled {
        t.paint.brush.shape.mapping = ph2d_painter_brush::texture::TextureMapping::Tiled;
        t.paint.brush.shape.size = [6.0; 2];
    }
    arma_cor(&mut t, cor);
    let seed = t.paint.brush;
    t.paint.brush_by_mode.fill(seed);
    traco(&mut t)
}

/// ⭐ **A cor do Shape no Continuous iguala o Spacing pequeno** (lista 26 A9, smoke do dono 15/10): a
/// Texture Color e a Falloff Color sobre um Shape procedural no mapping do dab — a 0,25 com a caixa, o
/// traço a 0,025 sem ela: Δcor ≤ 0,02 e a matiz correlacionada ≥ 0,9. A cor compõe-se sobre os MESMOS
/// sub-dabs da cobertura ([`ph2d_painter_brush::dab::silhouette_capsula`]); lida uma vez no centro do
/// dab da frente, dava Δ 0,39 e correlação −0,18. CONTROLOS: Tiled e sem rampa ficam ≤ 0,005 (o
/// resíduo medido da caixa sem cor, Digital Noise sem rampa: 0,003).
#[test]
fn a_cor_do_shape_no_continuous_iguala_o_spacing_pequeno() {
    let r = 20.0;
    for meio in [Meio::Digital, Meio::Impasto] {
        for kind in [TextureKind::Checker, TextureKind::Noise] {
            for cor in [CamadaCor::Textura, CamadaCor::Queda] {
                let alvo = traco_cor(meio, kind, cor, false, 0.025, false);
                let px = traco_cor(meio, kind, cor, false, 0.25, true);
                let (d, h) = (
                    dif_cor(&px, &alvo, r),
                    correlacao(&matiz(&px, r), &matiz(&alvo, r)),
                );
                println!("{meio:?} {kind:?} {cor:?}: Δcor {d:.3}, matiz {h:.3}");
                assert!(d <= 0.02, "{meio:?} {kind:?} {cor:?}: Δcor {d:.3}");
                assert!(h >= 0.9, "{meio:?} {kind:?} {cor:?}: matiz {h:.3}");
            }
            for (cor, tiled) in [(CamadaCor::Textura, true), (CamadaCor::Off, false)] {
                let alvo = traco_cor(meio, kind, cor, tiled, 0.025, false);
                let d = dif_cor(&traco_cor(meio, kind, cor, tiled, 0.25, true), &alvo, r);
                println!("{meio:?} {kind:?} {cor:?} tiled {tiled}: controlo Δ {d:.4}");
                assert!(
                    d <= 0.005,
                    "{meio:?} {kind:?} {cor:?} tiled {tiled}: controlo Δ {d:.3}"
                );
            }
        }
    }
}
