//! ⭐ **As Grain Colors nos QUATRO meios** (dono 22/10): o valor do grão indexa a rampa e a cor da
//! rampa é a da tinta — a lei do digital (`ph2d_painter_brush::RampPaint::cor`). No Digital e no
//! Impasto o grão do carimbo; na Aquarela o mapa de granulação (o Grain com «Same as Paper» desligado);
//! no Wet Paint o grão do dab. Desligadas, cada meio pinta ao byte o de antes (os fingerprints de cada
//! meio e os portões por foto da fábrica).

use super::measure_shape_system::{cp, tool};
use crate::tool::PainterTool;
use crate::tool::paint::media::PaintMedia;
use ph2d_color::{ColorRamp, RampColorMode, RampInterp, RampStop};
use ph2d_editor_core::Tool;
use ph2d_editor_core::tool::{CanvasPaintTool, PointerPhase};
use ph2d_painter_brush::TextureKind;

const MEIOS: [PaintMedia; 4] = [
    PaintMedia::Digital,
    PaintMedia::Watercolor,
    PaintMedia::Impasto,
    PaintMedia::WetPaint,
];

/// A rampa verde (grão `0`) → vermelho (grão `1`): as duas pontas na tinta provam que é o VALOR do grão
/// que a indexa, e não uma cor só.
fn verde_vermelho() -> ColorRamp {
    ColorRamp::new(
        vec![
            RampStop::new(0.0, [0.0, 1.0, 0.0, 1.0]),
            RampStop::new(1.0, [1.0, 0.0, 0.0, 1.0]),
        ],
        RampColorMode::Rgb,
        RampInterp::Linear,
    )
}

/// Um pincel azul com um Grain xadrez (na Aquarela: o mapa de granulação próprio), armado por `arma`,
/// num traço recto com o relógio a correr.
fn pinta_t(media: PaintMedia, arma: impl Fn(&mut PainterTool)) -> PainterTool {
    let mut t = tool(128, media, 12.0);
    t.set_wet_relogio_fixo(true);
    t.set_brush_color_srgb8([30, 60, 220]);
    t.set_brush_texture_kind(TextureKind::Checker.to_u8());
    t.paint.brush.granulation_use_paper = false;
    arma(&mut t);
    t.on_canvas_pointer(cp([20.0, 64.0], PointerPhase::Down));
    t.on_tick(1000.0 / 60.0);
    for k in 1..=22u8 {
        let x = 20.0 + 4.0 * f32::from(k);
        t.on_canvas_pointer(cp([x, 64.0], PointerPhase::Move));
        t.on_tick(1000.0 / 60.0);
    }
    t.on_canvas_pointer(cp([108.0, 64.0], PointerPhase::Up));
    for _ in 0..4 {
        t.on_tick(1000.0 / 60.0);
    }
    t
}

fn pinta(media: PaintMedia, arma: impl Fn(&mut PainterTool)) -> Vec<u8> {
    pinta_t(media, arma).canvas_rgba.to_vec()
}

fn liga(t: &mut PainterTool) {
    t.set_texture_ramp(verde_vermelho());
    t.set_texture_ramp_enabled(true);
}

/// Texels onde o canal `c` domina os outros dois por uma margem visível.
fn dominados_por(px: &[u8], c: usize) -> usize {
    px.chunks(4)
        .filter(|p| {
            let v = i32::from(p[c]);
            (0..3).filter(|&k| k != c).all(|k| v > i32::from(p[k]) + 40)
        })
        .count()
}

/// Quantas vezes, ao longo da linha `y` do traço, a cor dominante troca entre verde e vermelho — a
/// assinatura do xadrez do GRÃO (uma rampa indexada pela queda do carimbo é constante ao longo do
/// traço horizontal: zero trocas).
fn trocas_ao_longo(px: &[u8], y: usize) -> usize {
    let mut ultima = None;
    let mut n = 0;
    for x in 16..112 {
        let p = &px[(y * 128 + x) * 4..][..4];
        let (r, g) = (i32::from(p[0]), i32::from(p[1]));
        let c = if g > r + 40 {
            Some('g')
        } else if r > g + 40 {
            Some('r')
        } else {
            None
        };
        if let Some(c) = c {
            if ultima.is_some_and(|u| u != c) {
                n += 1;
            }
            ultima = Some(c);
        }
    }
    n
}

/// **A cor do grão MUDA a tinta em cada meio** — com as Grain Colors ligadas o traço azul sai verde nas
/// casas baixas do grão e vermelho nas altas (o xadrez troca a cor ao longo do traço); sem elas, nem um
/// texel verde ou vermelho.
///
/// **Mutações que sangram:** a aguada voltar a só dar a dona Shape (`rampa_da_aguada` com
/// `color_ramp_owner(false)`) — Aquarela e Wet Paint ficam azuis; a aguada ou o Wet Paint indexarem a
/// rampa pela queda (`grao` = `None`) — a cor deixa de seguir o xadrez.
#[test]
fn as_grain_colors_pintam_a_cor_do_grao_nos_quatro_meios() {
    let mut falhas = Vec::new();
    for m in MEIOS {
        let sem = pinta(m, |_| {});
        let com = pinta(m, liga);
        let (sv, sr) = (dominados_por(&sem, 1), dominados_por(&sem, 0));
        let (cv, cr) = (dominados_por(&com, 1), dominados_por(&com, 0));
        if sv + sr > 0 {
            falhas.push(format!(
                "{m:?}: o controlo azul já tinha {sv} verdes e {sr} vermelhos"
            ));
        }
        if cv < 40 || cr < 40 {
            falhas.push(format!(
                "{m:?}: as Grain Colors não chegaram à tinta ({cv} verdes, {cr} vermelhos)"
            ));
        }
        let n = trocas_ao_longo(&com, 64);
        if n < 3 {
            falhas.push(format!(
                "{m:?}: a cor não segue o grão ao longo do traço ({n} trocas verde↔vermelho)"
            ));
        }
    }
    assert!(falhas.is_empty(), "{}", falhas.join("\n"));
}

/// **Onde o meio NÃO lê o grão, a linha esmaece com o motivo** — e a tinta não muda: a Aquarela com
/// «Same as Paper» (a lavagem assenta no papel, o slot Grain não é lido) e o Wet Paint fora da
/// ferramenta Paint (Smear, Blend, Wet, Dry, Blow não pousam a cor da rampa).
#[test]
fn as_grain_colors_esmaecem_onde_o_meio_nao_le_o_grao() {
    use crate::ids::PAINTER_BRUSH_TEXTURE_RAMP_ENABLE as ID;
    let dica = |t: &PainterTool| t.inercias().dica_de(ID);
    for m in MEIOS {
        let t = pinta_t(m, liga);
        assert_eq!(
            dica(&t),
            None,
            "{m:?}: as Grain Colors agem e a linha esmaeceu"
        );
    }
    let papel = |t: &mut PainterTool| {
        liga(t);
        t.paint.brush.granulation_use_paper = true;
    };
    let t = pinta_t(PaintMedia::Watercolor, papel);
    assert_eq!(
        dica(&t),
        Some("panel.painter_layers.inerte.same_as_paper_off"),
        "Aquarela com Same as Paper: a linha não esmaeceu"
    );
    assert_eq!(
        pinta(PaintMedia::Watercolor, papel),
        pinta(PaintMedia::Watercolor, |t| t
            .paint
            .brush
            .granulation_use_paper =
            true),
        "Aquarela com Same as Paper: esmaecida mas a tinta mudou"
    );
    let mut t = pinta_t(PaintMedia::WetPaint, liga);
    t.pick_wet_tool(2);
    assert_eq!(
        dica(&t),
        Some("panel.painter_layers.inerte.paint_tool_only"),
        "Wet Paint fora da ferramenta Paint: a linha não esmaeceu"
    );
}

/// O lado da tela das provas de resolução (um traço largo de raio 70 cabe com folga).
const L: usize = 384;

/// Um traço largo com o Voronoi e as Grain Colors cinza → rosa (os miolos das células saem pontos
/// cinzentos — o caso da captura do dono, 22/10), armado por `arma`.
fn pinta_voronoi(m: PaintMedia, arma: impl Fn(&mut PainterTool)) -> Vec<u8> {
    let mut t = tool(L as u32, m, 70.0);
    t.set_wet_relogio_fixo(true);
    t.set_brush_color_srgb8([230, 150, 150]);
    t.set_brush_texture_kind(TextureKind::Voronoi.to_u8());
    t.paint.brush.granulation_use_paper = false;
    t.set_texture_ramp(ColorRamp::new(
        vec![
            RampStop::new(0.0, [0.3, 0.3, 0.3, 1.0]),
            RampStop::new(0.3, [0.9, 0.6, 0.6, 1.0]),
        ],
        RampColorMode::Rgb,
        RampInterp::Linear,
    ));
    t.set_texture_ramp_enabled(true);
    arma(&mut t);
    let y = L as f32 / 2.0;
    t.on_canvas_pointer(cp([90.0, y], PointerPhase::Down));
    t.on_tick(1000.0 / 60.0);
    for k in 1..=50u16 {
        t.on_canvas_pointer(cp([90.0 + 4.0 * f32::from(k), y], PointerPhase::Move));
        t.on_tick(1000.0 / 60.0);
    }
    t.on_canvas_pointer(cp([290.0, y], PointerPhase::Up));
    for _ in 0..4 {
        t.on_tick(1000.0 / 60.0);
    }
    t.canvas_rgba.to_vec()
}

/// **A resolução do padrão de cor** no miolo do traço, contra a referência `refe` (o Digital, que
/// amostra o grão no pixel): `(bloco, correlação)`. O bloco é `1 / (1 − r)`, com `r` = a fracção dos
/// pares vizinhos (linha e coluna) onde a referência MUDA (≥ 12 níveis na soma RGB) e a tinta sai com o
/// MESMO RGB — o texel repetido de um padrão ampliado sem filtro; `1,0` = resolução do pixel. A
/// correlação da luminância com a referência apanha o padrão torcido (deslocado) mesmo sem blocos.
fn resolucao(px: &[u8], refe: &[u8]) -> (f32, f32) {
    let lum = |b: &[u8], x: usize, y: usize| {
        let p = &b[(y * L + x) * 4..][..3];
        f32::from(p[0]) + f32::from(p[1]) + f32::from(p[2])
    };
    let (mut pares, mut iguais) = (0usize, 0usize);
    let (mut sa, mut sb, mut saa, mut sbb, mut sab, mut n) = (0f64, 0f64, 0f64, 0f64, 0f64, 0f64);
    for y in 150..234 {
        for x in 130..250 {
            let (a, b) = (f64::from(lum(px, x, y)), f64::from(lum(refe, x, y)));
            (sa, sb, saa, sbb, sab, n) = (
                sa + a,
                sb + b,
                saa + a * a,
                sbb + b * b,
                sab + a * b,
                n + 1.0,
            );
            for (xn, yn) in [(x + 1, y), (x, y + 1)] {
                if (lum(refe, x, y) - lum(refe, xn, yn)).abs() >= 12.0 {
                    pares += 1;
                    let p = &px[(y * L + x) * 4..][..3];
                    let q = &px[(yn * L + xn) * 4..][..3];
                    iguais += usize::from(p == q);
                }
            }
        }
    }
    let r = iguais as f32 / pares.max(1) as f32;
    let cov = sab / n - (sa / n) * (sb / n);
    let var = (saa / n - (sa / n).powi(2)) * (sbb / n - (sb / n).powi(2));
    (
        1.0 / (1.0 - r).max(1e-3),
        (cov / var.sqrt().max(1e-9)) as f32,
    )
}

/// A tela sobre BRANCO (o que o artista vê), para as fotos.
fn sobre_branco(px: &[u8]) -> Vec<u8> {
    px.as_chunks::<4>()
        .0
        .iter()
        .flat_map(|p| {
            let a = f32::from(p[3]) / 255.0;
            let c = |i: usize| (f32::from(p[i]) + 255.0 * (1.0 - a)).min(255.0) as u8;
            [c(0), c(1), c(2), 255]
        })
        .collect()
}

/// ⭐ **O padrão das Grain Colors sai à resolução do PIXEL em todos os meios** (captura do dono 22/10:
/// o Voronoi na Aquarela em blocos quadrados, no Digital liso). A Aquarela lia a cor depositada no
/// ponto deslocado pelo Ragged Edge, ao vizinho mais próximo — o campo de deslocamento estica e
/// comprime, e o padrão do grão saía repetido em blocos e torcido, fora do sítio onde a granulação
/// assenta (`watercolor_render::pigment::texel_do_deposito`). Medido (miolo do traço, Digital = 1,00 px):
///
/// | meio | Ragged Edge | antes: bloco · corr | depois: bloco · corr |
/// |---|---|---|---|
/// | Aquarela | 6 (fábrica) | 1,29 px · 0,730 | 1,00 px · 0,979 |
/// | Aquarela | 20 + Paper Edge 1 | 2,20 px · 0,127 | 1,00 px · 0,952 |
/// | Wet Paint | — | 1,00 px · 0,984 | 1,00 px · 0,984 |
/// | Impasto | — | 1,00 px · 0,985 | 1,00 px · 0,985 |
///
/// (O resto da correlação com a borda forte são os buracos que o Ragged Edge abre na COBERTURA, não a
/// cor.) **Mutação que sangra:** `texel_do_deposito` devolver sempre o texel deslocado (a leitura de
/// antes) — 1,29 px · 0,730 e 2,20 px · 0,127.
#[test]
fn o_padrao_das_grain_colors_sai_a_resolucao_do_pixel_em_todos_os_meios() {
    let refe = pinta_voronoi(PaintMedia::Digital, |_| {});
    let fotos = std::env::var("PH2D_VORONOI_FOTOS").ok();
    let mut falhas = Vec::new();
    let casos: [(PaintMedia, &str, f32, f32); 4] = [
        (PaintMedia::Watercolor, "aquarela", 6.0, 0.0),
        (PaintMedia::Watercolor, "aquarela_ragged20", 20.0, 1.0),
        (PaintMedia::WetPaint, "wet", 6.0, 0.0),
        (PaintMedia::Impasto, "impasto", 6.0, 0.0),
    ];
    for (m, nome, warp, borda) in casos {
        let px = pinta_voronoi(m, |t| {
            t.paint.brush.warp = warp;
            t.paint.brush.paper_edge = borda;
        });
        let (bloco, corr) = resolucao(&px, &refe);
        eprintln!("{nome}: bloco {bloco:.2} px · correlação {corr:.3}");
        if bloco > 1.05 || corr < 0.9 {
            falhas.push(format!(
                "{nome}: o padrão do grão saiu em blocos de {bloco:.2} px (correlação {corr:.3} com o Digital)"
            ));
        }
        if let Some(d) = &fotos {
            let png = crate::tool::paint::composite_look::png_rgba(
                &sobre_branco(&px),
                L as u32,
                L as u32,
            );
            std::fs::write(format!("{d}/voronoi_{nome}.png"), png).expect("a foto grava");
        }
    }
    assert!(falhas.is_empty(), "{}", falhas.join("\n"));
}

/// **O custo das Grain Colors na Aquarela** — ms por quadro (move + tick) de um traço de raio 70 numa
/// tela 2048² (perfil `smoke`), com e sem elas, intercalados; resumo = a MEDIANA de cada traço, o
/// MÍNIMO entre rondas. Corre-se à mão: `cargo test -p ph2d-tool-painter --profile smoke --lib --
/// --ignored sonda_custo_das_grain_colors_na_aquarela --nocapture`.
#[test]
#[ignore = "sonda: o custo das Grain Colors na Aquarela; corre-se à mão"]
fn sonda_custo_das_grain_colors_na_aquarela() {
    const T: u32 = 2048;
    let traco = |cor: bool| -> (f64, f64) {
        let mut t = tool(T, PaintMedia::Watercolor, 70.0);
        t.set_wet_relogio_fixo(true);
        t.set_brush_texture_kind(TextureKind::Voronoi.to_u8());
        t.paint.brush.granulation_use_paper = false;
        if cor {
            liga(&mut t);
        }
        let mut ms = Vec::new();
        t.on_canvas_pointer(cp([600.0, 1024.0], PointerPhase::Down));
        t.on_tick(1000.0 / 60.0);
        for k in 1..=60u16 {
            let t0 = std::time::Instant::now();
            t.on_canvas_pointer(cp(
                [600.0 + 12.0 * f32::from(k), 1024.0],
                PointerPhase::Move,
            ));
            t.on_tick(1000.0 / 60.0);
            ms.push(t0.elapsed().as_secs_f64() * 1e3);
        }
        t.on_canvas_pointer(cp([1320.0, 1024.0], PointerPhase::Up));
        ms.sort_by(f64::total_cmp);
        (ms[ms.len() / 2], ms[ms.len() * 95 / 100])
    };
    let (mut sem, mut com) = ((f64::MAX, f64::MAX), (f64::MAX, f64::MAX));
    for ronda in 0..6 {
        for cor in if ronda % 2 == 0 {
            [false, true]
        } else {
            [true, false]
        } {
            let (m, p) = traco(cor);
            let r = if cor { &mut com } else { &mut sem };
            *r = (r.0.min(m), r.1.min(p));
        }
    }
    let load = std::fs::read_to_string("/proc/loadavg").unwrap_or_default();
    eprintln!(
        "CUSTO aquarela 2048² r70: sem Grain Colors {:.2} ms (p95 {:.2}) · com {:.2} ms (p95 {:.2}) · load {}",
        sem.0,
        sem.1,
        com.0,
        com.1,
        load.trim()
    );
}
