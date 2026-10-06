//! **O quadro da aguada é o da recomposição TOTAL** (BUGS #36, o pixel do Airbrush) — o oráculo é
//! `WashCadence::recompoe_tudo` no mesmo processo: a tela inteira recomposta em todo quadro. Toda
//! diferença do produto para ele é uma escrita numa entrada do composite que nenhum sujo marcou.

use super::measure_shape_system::{cp, tool};
use crate::tool::PainterTool;
use crate::tool::paint::media::PaintMedia;
use ph2d_editor_core::tool::{CanvasPaintTool, PointerPhase};
use ph2d_painter_brush::StrokeMethod;

/// O caminho que dá a volta ao quadrado e regressa pelo meio (o de `watercolor_solido_tests`).
const VOLTA: [[f32; 2]; 6] = [
    [20.0, 20.0],
    [108.0, 20.0],
    [108.0, 108.0],
    [20.0, 108.0],
    [20.0, 60.0],
    [90.0, 60.0],
];

/// Um cenário: o método, o Solid, o Rewet, a Dilution, o Ragged (`< 0` = o de fábrica), a Charge
/// (`< 1` liga a reserva de pigmento) e quantos Moves chegam por quadro.
#[derive(Clone, Copy, Debug)]
struct Cenario {
    nome: &'static str,
    metodo: StrokeMethod,
    solid: bool,
    rewet: f32,
    dilution: f32,
    ragged: f32,
    charge: f32,
    por_quadro: usize,
}

/// O cenário de fábrica com o método e o Solid escolhidos.
const fn cen(nome: &'static str, metodo: StrokeMethod, solid: bool) -> Cenario {
    Cenario {
        nome,
        metodo,
        solid,
        rewet: 0.0,
        dilution: 0.0,
        ragged: -1.0,
        charge: 1.0,
        por_quadro: 1,
    }
}

const CENARIOS: [Cenario; 9] = [
    cen("Space + Solid", StrokeMethod::Space, true),
    cen("Airbrush", StrokeMethod::Airbrush, false),
    cen("Airbrush + Solid", StrokeMethod::Airbrush, true),
    cen("Space (settle)", StrokeMethod::Space, false),
    Cenario {
        rewet: 0.6,
        ..cen("Airbrush + Rewet", StrokeMethod::Airbrush, false)
    },
    Cenario {
        dilution: 0.6,
        ..cen("Space + Dilution", StrokeMethod::Space, false)
    },
    // O Ragged forte leva as subamostras do AA longe do texel (a soma na TELA, BUGS #36).
    Cenario {
        ragged: 24.0,
        ..cen("Space + Solid + Ragged 24", StrokeMethod::Space, true)
    },
    // A reserva de pigmento guarda um campo entre quadros, recalculado onde o sujo de TODAS as
    // janelas do quadro mexeu.
    Cenario {
        charge: 0.5,
        ..cen("Space + Solid + Charge 0,5", StrokeMethod::Space, true)
    },
    // Vários eventos num quadro: as linhas da mancha repetem-se e fundem-se.
    Cenario {
        por_quadro: 3,
        ..cen(
            "Space + Solid, 3 eventos por quadro",
            StrokeMethod::Space,
            true,
        )
    },
];

fn pincel(c: Cenario, oraculo: bool) -> PainterTool {
    let mut t = tool(128, PaintMedia::Watercolor, 4.0);
    t.set_brush_color_srgb8([30, 60, 220]);
    t.paint.brush.stroke_method = c.metodo;
    t.paint.brush.style_solid = c.solid;
    t.paint.brush.wet_rewet = c.rewet;
    t.paint.brush.wet_dilution = c.dilution;
    if c.ragged >= 0.0 {
        t.paint.brush.warp = c.ragged;
    }
    t.paint.brush.wet_charge = c.charge;
    t.wash.recompoe_tudo = oraculo;
    t
}

/// Os quadros do gesto: `f(t)` depois de cada tique (12 Moves por aresta, `por_quadro` Moves por
/// quadro, e `parados` quadros de caneta parada), e no fim a tela do pen-up.
fn gesto(t: &mut PainterTool, c: Cenario, parados: usize, f: &mut dyn FnMut(&PainterTool)) {
    t.on_canvas_pointer(cp(VOLTA[0], PointerPhase::Down));
    let mut n = 0;
    for w in VOLTA.windows(2) {
        for k in 1..=12 {
            #[allow(clippy::cast_precision_loss)]
            let s = k as f32 / 12.0;
            t.on_canvas_pointer(cp(
                [
                    w[0][0] + (w[1][0] - w[0][0]) * s,
                    w[0][1] + (w[1][1] - w[0][1]) * s,
                ],
                PointerPhase::Move,
            ));
            n += 1;
            if n % c.por_quadro == 0 {
                t.paint_tick(0.1);
                f(t);
            }
        }
    }
    for _ in 0..parados {
        t.paint_tick(0.25);
        f(t);
    }
    t.on_canvas_pointer(cp(VOLTA[5], PointerPhase::Up));
    f(t);
}

/// As telas de cada quadro do gesto e os texels que o composite caminhou.
fn telas_e_trabalho(c: Cenario, oraculo: bool) -> (Vec<Vec<u8>>, u64) {
    let mut t = pincel(c, oraculo);
    let mut v = Vec::new();
    gesto(&mut t, c, 6, &mut |t| v.push(t.canvas_rgba.to_vec()));
    (v, t.wash.window_px)
}

/// As telas de cada quadro do gesto.
fn telas(c: Cenario, oraculo: bool) -> Vec<Vec<u8>> {
    telas_e_trabalho(c, oraculo).0
}

/// As entradas do composite que não são os seis planos (que o gate da mancha já compara): cada
/// uma como bytes, para achar a caixa do que mudou entre dois quadros.
fn entradas(t: &PainterTool) -> Vec<(&'static str, usize, Vec<u8>)> {
    let sub: Vec<u8> = t
        .paint
        .wet_substrate
        .iter()
        .flat_map(|v| v.to_bits().to_le_bytes())
        .collect();
    vec![
        ("coverage", 1, t.paint.stroke_coverage.clone()),
        ("color", 4, t.paint.stroke_color.clone()),
        ("density", 1, t.paint.stroke_density.clone()),
        ("deplete", 1, t.paint.stroke_deplete.clone()),
        ("owner", 1, t.paint.wet_styles.owner.clone()),
        ("soak", 1, t.paint.wet_soak.clone()),
        ("water", 1, t.paint.stroke_water.clone()),
        ("canvas_wet", 1, t.paint.canvas_wet.clone()),
        ("substrate", 16, sub),
        (
            "session_base",
            4,
            t.paint
                .wet_session_base
                .as_ref()
                .map(|b| b.to_vec())
                .unwrap_or_default(),
        ),
        (
            "backdrop",
            4,
            t.paint
                .wet_backdrop
                .as_ref()
                .map(|b| b.to_vec())
                .unwrap_or_default(),
        ),
    ]
}

/// A caixa `(x0, y0, x1, y1)` dos texels que diferem entre `a` e `b` (`bpp` bytes por texel).
fn caixa(a: &[u8], b: &[u8], bpp: usize, lado: usize) -> Option<(usize, usize, usize, usize)> {
    if a.len() != b.len() {
        return Some((0, 0, lado - 1, lado - 1));
    }
    let mut c: Option<(usize, usize, usize, usize)> = None;
    for (i, (p, q)) in a.chunks(bpp).zip(b.chunks(bpp)).enumerate() {
        if p != q {
            let (x, y) = (i % lado, i / lado);
            c = Some(c.map_or((x, y, x, y), |(x0, y0, x1, y1)| {
                (x0.min(x), y0.min(y), x1.max(x), y1.max(y))
            }));
        }
    }
    c
}

/// SONDA — o produto e o oráculo da recomposição total, quadro a quadro; no primeiro quadro em que
/// a tela difere, a caixa de cada entrada do composite que mudou em cada quadro até ali.
/// `cargo test -p ph2d-tool-painter --lib diag_o_que_o_quadro_nao_marca -- --ignored --nocapture`
#[test]
#[ignore = "diagnóstico"]
fn diag_o_que_o_quadro_nao_marca() {
    for c in CENARIOS {
        let oraculo = telas(c, true);
        let mut t = pincel(c, false);
        let mut antes = entradas(&t);
        let mut log = Vec::new();
        let mut q = 0usize;
        let mut achou = false;
        gesto(&mut t, c, 6, &mut |t| {
            if achou {
                return;
            }
            let agora = entradas(t);
            let mut linha = format!("  quadro {q:>3}:");
            for ((nome, bpp, a), (_, _, b)) in antes.iter().zip(&agora) {
                if let Some(cx) = caixa(a, b, *bpp, 128) {
                    linha += &format!(" {nome}{cx:?}");
                }
            }
            log.push(linha);
            antes = agora;
            let o = &oraculo[q];
            let difere: Vec<usize> = t
                .canvas_rgba
                .iter()
                .zip(o)
                .enumerate()
                .filter(|(_, (p, s))| p != s)
                .map(|(i, _)| i / 4)
                .collect();
            if !difere.is_empty() {
                let p = difere[0];
                let i = p * 4;
                eprintln!(
                    "{}: o quadro {q} difere em {} texels; o 1.º ({}, {}) produto {:?} oráculo {:?}",
                    c.nome,
                    difere.len(),
                    p % 128,
                    p / 128,
                    &t.canvas_rgba[i..i + 4],
                    &o[i..i + 4]
                );
                for l in &log {
                    eprintln!("{l}");
                }
                achou = true;
            }
            q += 1;
        });
        if !achou {
            eprintln!("{}: {} quadros iguais ao oráculo", c.nome, q);
        }
    }
}

/// ⭐ **EM TODO QUADRO O PRODUTO É A RECOMPOSIÇÃO TOTAL, AO BYTE** — Airbrush (o tique carimba com a
/// caneta a andar), com e sem Solid, Space com a caneta parada (o `settle`), Rewet (o soak) e
/// Dilution (a água carregada). Vermelho antes: os borrões somavam em `f32` a partir da origem da
/// janela, e o pixel `(38, 18)` lia `117` no quadro e `116` na tela inteira.
#[test]
fn o_quadro_da_aguada_e_o_da_recomposicao_total() {
    for c in CENARIOS {
        let ((produto, trabalho), (oraculo, total)) =
            (telas_e_trabalho(c, false), telas_e_trabalho(c, true));
        // Um quadro do gesto (o pen-up não conta: ele recompõe o traço inteiro nos dois).
        let tela_inteira = (oraculo.len() as u64 - 1) * 128 * 128;
        assert!(
            total >= tela_inteira && total > trabalho,
            "controlo: o oráculo tem de recompor a tela inteira em todo quadro ({total} texels, \
             produto {trabalho})"
        );
        assert_eq!(produto.len(), oraculo.len());
        for (q, (p, o)) in produto.iter().zip(&oraculo).enumerate() {
            let difere = p.chunks(4).zip(o.chunks(4)).filter(|(a, b)| a != b).count();
            assert_eq!(
                difere, 0,
                "{}: no quadro {q} o produto difere da recomposição total em {difere} texels",
                c.nome
            );
        }
        let ultima = produto.last().expect("quadros");
        assert!(
            ultima.iter().any(|&v| v < 240),
            "controlo: {} pintou",
            c.nome
        );
    }
}

/// SONDA — qual termo do composite depende da JANELA: o cenário do Airbrush com um termo desligado
/// de cada vez; o que faz a divergência sumir é o culpado.
/// Uma ablação: o nome e o que ela desliga no pincel.
type Ablacao = (&'static str, fn(&mut PainterTool));

#[test]
#[ignore = "diagnóstico"]
fn diag_qual_termo_depende_da_janela() {
    let c = CENARIOS[1];
    let ablacoes: [Ablacao; 7] = [
        ("nada", |_| {}),
        ("aro (edge_gain 0)", |t| t.paint.brush.edge_gain = 0.0),
        ("ragged (warp 0)", |t| t.paint.brush.warp = 0.0),
        ("granulação 0", |t| t.paint.brush.granulation = 0.0),
        ("dente do papel 0", |t| t.paint.brush.paper_depth = 0.0),
        ("sem AA", |t| t.paint.brush.smooth_edges = false),
        ("paper edge 0", |t| t.paint.brush.paper_edge = 0.0),
    ];
    for (nome, f) in ablacoes {
        let corre = |oraculo: bool| {
            let mut t = pincel(c, oraculo);
            f(&mut t);
            let mut v = Vec::new();
            gesto(&mut t, c, 6, &mut |t| v.push(t.canvas_rgba.to_vec()));
            v
        };
        let (p, o) = (corre(false), corre(true));
        let total: usize = p
            .iter()
            .zip(&o)
            .map(|(a, b)| a.chunks(4).zip(b.chunks(4)).filter(|(x, y)| x != y).count())
            .sum();
        eprintln!("ablação {nome:<20}: {total} texels diferentes somados nos quadros");
    }
}

/// ⭐ **O COMPOSITE É O MESMO EM QUALQUER JANELA** — o mesmo estado da aguada (Solid, aro, AA e o
/// Ragged 24 que leva as subamostras longe do texel) recomposto em 48 janelas de origens e tamanhos
/// diferentes: a saída de cada uma é, ao byte, a da tela inteira. É a lei de que dependem o quadro
/// por janelas da mancha e o oráculo da recomposição total (BUGS #36).
#[test]
fn o_composite_e_o_mesmo_em_qualquer_janela() {
    use crate::tool::paint::Region;
    let c = Cenario {
        ragged: 24.0,
        ..cen("Ragged 24 + Solid", StrokeMethod::Space, true)
    };
    let mut t = pincel(c, false);
    t.on_canvas_pointer(cp(VOLTA[0], PointerPhase::Down));
    for w in VOLTA.windows(2).take(3) {
        for k in 1..=12 {
            #[allow(clippy::cast_precision_loss)]
            let s = k as f32 / 12.0;
            t.on_canvas_pointer(cp(
                [
                    w[0][0] + (w[1][0] - w[0][0]) * s,
                    w[0][1] + (w[1][1] - w[0][1]) * s,
                ],
                PointerPhase::Move,
            ));
            t.paint_tick(0.1);
        }
    }
    t.wash.recompoe_tudo = true;
    t.apply_watercolor(false).expect("a tela inteira compôs");
    let tela = t.canvas_rgba.to_vec();
    t.wash.recompoe_tudo = false;
    let mut caminhados = 0usize;
    for o in 0u32..48 {
        t.paint.wet_frame_dirty = Some(Region {
            x: 14 + o,
            y: 6 + (o * 7) % 50,
            w: 9 + o % 13,
            h: 11 + o % 7,
        });
        let r = t.apply_watercolor(false).expect("a janela compôs");
        for y in r.y..r.y + r.h {
            for x in r.x..r.x + r.w {
                let i = ((y * 128 + x) * 4) as usize;
                assert_eq!(
                    t.canvas_rgba[i..i + 4],
                    tela[i..i + 4],
                    "janela {o}: o texel ({x}, {y}) difere da tela inteira"
                );
                caminhados += 1;
            }
        }
    }
    assert!(
        caminhados > 48 * 900,
        "controlo: as janelas compuseram ({caminhados} texels)"
    );
    assert!(tela.iter().any(|&v| v < 200), "controlo: a aguada pintou");
}

/// ⭐ **O PONTO AMOSTRADO É O MESMO EM QUALQUER JANELA** — a porta
/// [`super::watercolor_render::ponto_na_janela`] devolvida à tela dá o mesmo bit para toda origem da
/// janela que o contém (a origem à esquerda do ponto, que a folga `pad` da janela garante), em texels
/// de toda a faixa da tela, com as subamostras do AA e deslocamentos fraccionários.
/// CONTROLO: a soma na janela (a lei de antes) diverge.
#[test]
fn o_ponto_amostrado_e_o_mesmo_em_qualquer_janela() {
    use super::watercolor_render::ponto_na_janela;
    let mut s = 0x2545_f491_u32;
    let mut sorteio = || {
        s ^= s << 13;
        s ^= s >> 17;
        s ^= s << 5;
        #[allow(clippy::cast_precision_loss)]
        let u = (s >> 8) as f32 / (1u32 << 24) as f32;
        u
    };
    let (mut antes, mut casos) = (0usize, 0usize);
    for g in [3usize, 97, 517, 1500, 2049, 4095] {
        for o in [0.0f32, -0.333_333_34, 0.25, 0.333_333_34] {
            for _ in 0..8 {
                let d = (sorteio() - 0.5) * 48.0;
                let na_tela = ponto_na_janela(g, o, d, 0);
                for origem in (0..=g).step_by(g / 50 + 1) {
                    // O domínio da garantia: o ponto à direita da origem (a folga `pad` da janela).
                    #[allow(clippy::cast_precision_loss)]
                    if (origem as f32) > na_tela {
                        continue;
                    }
                    #[allow(clippy::cast_precision_loss)]
                    let de_volta = ponto_na_janela(g, o, d, origem) + origem as f32;
                    assert_eq!(
                        de_volta.to_bits(),
                        na_tela.to_bits(),
                        "g {g} o {o} d {d} origem {origem}: o ponto mudou com a janela"
                    );
                    #[allow(clippy::cast_precision_loss)]
                    let lei_antiga = ((g - origem) as f32 + o + d) + origem as f32;
                    antes += usize::from(lei_antiga.to_bits() != na_tela.to_bits());
                    casos += 1;
                }
            }
        }
    }
    assert!(
        antes > 0,
        "controlo: a soma na janela tinha de divergir ({casos} casos)"
    );
}
