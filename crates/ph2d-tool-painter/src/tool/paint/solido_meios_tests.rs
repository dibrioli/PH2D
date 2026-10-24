//! **O `Style: Solid` no Digital e no Impasto** (smoke do dono 2026-10-06, com foto: *«em digital e
//! impasto a cor central do solid não respeita o strength do pincel e fica mais escura. Em impasto o
//! relevo deveria preencher o centro com solid.»*). A aguada e a água ficam como estão (ordem do dono).

use super::measure_shape_system::{cp, tool};
use crate::tool::PainterTool;
use crate::tool::paint::media::PaintMedia;
use ph2d_editor_core::tool::{CanvasPaintTool, PointerPhase};

/// Um laço quadrado à mão livre em Solid, `128²`, raio 6, Strength `s`.
fn laco(meio: PaintMedia, s: f32) -> PainterTool {
    let mut t = tool(128, meio, 6.0);
    t.set_brush_color_srgb8([220, 40, 40]);
    t.paint.brush.strength = s;
    t.paint.brush.style_solid = true;
    let cantos = [
        [24.0f32, 24.0],
        [104.0, 24.0],
        [104.0, 104.0],
        [24.0, 104.0],
        [24.0, 28.0],
    ];
    t.on_canvas_pointer(cp(cantos[0], PointerPhase::Down));
    for w in cantos.windows(2) {
        for k in 1..=10 {
            #[allow(clippy::cast_precision_loss)]
            let f = k as f32 / 10.0;
            t.on_canvas_pointer(cp(
                [
                    w[0][0] + (w[1][0] - w[0][0]) * f,
                    w[0][1] + (w[1][1] - w[0][1]) * f,
                ],
                PointerPhase::Move,
            ));
            t.paint_tick(1.0 / 60.0);
        }
    }
    t.on_canvas_pointer(cp(cantos[4], PointerPhase::Up));
    for _ in 0..10 {
        t.paint_tick(1.0 / 60.0);
    }
    t
}

fn px(c: &[u8], x: usize, y: usize) -> [u8; 4] {
    let i = (y * 128 + x) * 4;
    [c[i], c[i + 1], c[i + 2], c[i + 3]]
}

/// SONDA — a cor no centro do traço contra a do miolo da mancha (camada e imagem mostrada), e no
/// Impasto a altura do relevo nos dois sítios.
/// `cargo test -p ph2d-tool-painter --profile smoke --lib diag_o_solid_no_digital_e_no_impasto -- --ignored --nocapture`
#[test]
#[ignore = "diagnóstico"]
fn diag_o_solid_no_digital_e_no_impasto() {
    for meio in [PaintMedia::Digital, PaintMedia::Impasto] {
        for s in [0.4f32, 1.0] {
            let mut t = laco(meio, s);
            t.invalidate_composite();
            let (v, _, _) = t.take_preview_arc().expect("imagem");
            let alturas = t
                .layers
                .active()
                .and_then(|a| t.heights.get(&a).cloned())
                .map(|h| (h[24 * 128 + 64], h[64 * 128 + 64], h[40 * 128 + 64]));
            eprintln!(
                "{meio:?} Strength {s}: camada traço {:?} miolo {:?} · mostrada traço {:?} miolo {:?} · alturas (traço, miolo, perto) {:?}",
                px(&t.canvas_rgba, 64, 24),
                px(&t.canvas_rgba, 64, 64),
                px(&v, 64, 24),
                px(&v, 64, 64),
                alturas
            );
        }
    }
}

/// Os cantos do laço quadrado: a última aresta pára 4 px antes do começo (a corda do Solid fecha-o).
const CANTOS: [[f32; 2]; 5] = [
    [24.0, 24.0],
    [104.0, 24.0],
    [104.0, 104.0],
    [24.0, 104.0],
    [24.0, 28.0],
];

/// O laço com a pressão `p` constante e os ajustes `f` aplicados ao pincel.
pub(super) fn laco_com(meio: PaintMedia, p: f32, f: &dyn Fn(&mut PainterTool)) -> PainterTool {
    laco_pelos(meio, p, f, &CANTOS)
}

/// O traço pela poligonal `cantos`, 10 eventos por aresta.
fn laco_pelos(
    meio: PaintMedia,
    p: f32,
    f: &dyn Fn(&mut PainterTool),
    cantos: &[[f32; 2]],
) -> PainterTool {
    use ph2d_editor_core::tool::CanvasPointer;
    let ptr = |pos: [f32; 2], phase| CanvasPointer {
        pos,
        pressure: p,
        tilt: [0.0, 0.0],
        phase,
    };
    let mut t = tool(128, meio, 6.0);
    t.set_brush_color_srgb8([220, 40, 40]);
    t.paint.brush.style_solid = true;
    f(&mut t);
    t.on_canvas_pointer(ptr(cantos[0], PointerPhase::Down));
    for w in cantos.windows(2) {
        for k in 1..=10 {
            #[allow(clippy::cast_precision_loss)]
            let s = k as f32 / 10.0;
            t.on_canvas_pointer(ptr(
                [
                    w[0][0] + (w[1][0] - w[0][0]) * s,
                    w[0][1] + (w[1][1] - w[0][1]) * s,
                ],
                PointerPhase::Move,
            ));
            t.paint_tick(1.0 / 60.0);
        }
    }
    t.on_canvas_pointer(ptr(cantos[cantos.len() - 1], PointerPhase::Up));
    t
}

/// SONDA — a opacidade (o `a` que leva o branco ao vermelho no canal G) no centro do traço e no
/// miolo da mancha, sob as combinações que mudam o traço.
#[test]
#[ignore = "diagnóstico"]
fn diag_a_opacidade_do_traco_e_da_mancha() {
    let a = |c: &[u8], x: usize, y: usize| (255.0 - f32::from(c[(y * 128 + x) * 4 + 1])) / 215.0;
    type Ajuste = (&'static str, f32, fn(&mut PainterTool));
    let casos: [Ajuste; 9] = [
        ("fábrica", 1.0, |_| {}),
        ("pressão 0,5", 0.5, |_| {}),
        ("Strength 0,4", 1.0, |t| t.paint.brush.strength = 0.4),
        ("Strength 0,4 + pressão 0,5", 0.5, |t| {
            t.paint.brush.strength = 0.4
        }),
        ("Flow 0,3", 1.0, |t| t.paint.brush.flow = 0.3),
        ("Strength 0,4 + Accumulate", 1.0, |t| {
            t.paint.brush.strength = 0.4;
            t.paint.brush.accumulate = true;
        }),
        ("Strength 0,4 + Flow 0,3", 1.0, |t| {
            t.paint.brush.strength = 0.4;
            t.paint.brush.flow = 0.3;
        }),
        ("dureza 1", 1.0, |t| t.paint.brush.hardness = 1.0),
        ("Strength 0,4 + dureza 1", 1.0, |t| {
            t.paint.brush.strength = 0.4;
            t.paint.brush.hardness = 1.0;
        }),
    ];
    for meio in [PaintMedia::Digital, PaintMedia::Impasto] {
        for (nome, p, f) in casos {
            let t = laco_com(meio, p, &f);
            let c = &t.canvas_rgba;
            eprintln!(
                "{meio:?} {nome:<28}: traço {:.3} · miolo {:.3}",
                a(c, 64, 24),
                a(c, 64, 64)
            );
        }
    }
}

/// SONDA — a opacidade do miolo de um traço COMUM (sem Solid) contra a Strength.
#[test]
#[ignore = "diagnóstico"]
fn diag_a_strength_do_traco_comum() {
    for s in [0.2f32, 0.4, 0.7, 1.0] {
        let t = laco_com(PaintMedia::Digital, 1.0, &|t: &mut PainterTool| {
            t.paint.brush.style_solid = false;
            t.paint.brush.strength = s;
        });
        let g = f32::from(t.canvas_rgba[(24 * 128 + 64) * 4 + 1]);
        eprintln!(
            "Strength {s}: o miolo do traço tem opacidade {:.3}",
            (255.0 - g) / 215.0
        );
    }
}

/// O canal G mais baixo (o texel mais opaco) da camada.
pub(super) fn mais_opaco(t: &PainterTool) -> u8 {
    t.canvas_rgba.chunks(4).map(|p| p[1]).min().expect("tela")
}

/// ⭐ **A MANCHA NUNCA COMPÕE POR CIMA DO TRAÇO, E O MIOLO TEM A OPACIDADE DE UMA PASSAGEM** (smoke do dono
/// 2026-10-06: *«a cor central do solid não respeita o strength do pincel e fica mais escura»*) — no Digital e no
/// Impasto, com Strength e Flow baixos. Desde que o Flow é a TAXA até o tecto da Strength (Accumulate off, 24/10)
/// o traço sozinho já não é plano: os cantos acumulam acima da passagem única. Por isso: (1) o miolo da mancha tem a
/// opacidade do CENTRO do traço (uma passagem), e (2) fora da corda que fecha o laço cada texel do gesto é
/// `max(traço, mancha)` — nunca mais opaco que o traço sozinho E que o miolo, ao mesmo tempo. Vermelho antes: a
/// mancha a `0,4` por cima de um traço a `0,163` (#40); com a lei do Flow, a mancha ao máximo do traço (Strength
/// 1 · Flow 0,3: miolo `1,0` contra o traço `0,81`) e a corda somada à mancha (Strength 0,4 · Flow 0,3: `0,39`).
#[test]
fn a_mancha_obedece_ao_tecto_do_traco() {
    type Ajuste = (&'static str, fn(&mut PainterTool));
    let casos: [Ajuste; 5] = [
        ("Strength 0,4", |t| t.paint.brush.strength = 0.4),
        ("Strength 0,7", |t| t.paint.brush.strength = 0.7),
        ("Strength 0,4 + Flow 0,3", |t| {
            t.paint.brush.strength = 0.4;
            t.paint.brush.flow = 0.3;
        }),
        ("Strength 1 + Flow 0,3", |t| t.paint.brush.flow = 0.3),
        ("Strength 0,4 + Flow 1", |t| {
            t.paint.brush.strength = 0.4;
            t.paint.brush.flow = 1.0;
        }),
    ];
    let g = |t: &PainterTool, x: usize, y: usize| t.canvas_rgba[(y * 128 + x) * 4 + 1];
    for meio in [PaintMedia::Digital, PaintMedia::Impasto] {
        for (nome, f) in casos {
            let com = laco_com(meio, 1.0, &f);
            let sem = laco_com(meio, 1.0, &|t: &mut PainterTool| {
                f(t);
                t.paint.brush.style_solid = false;
            });
            // (1) O centro do traço, longe dos cantos: uma passagem.
            let (miolo, centro) = (g(&com, 64, 64), g(&sem, 64, 24));
            assert!(
                miolo.abs_diff(centro) <= 2,
                "{meio:?} {nome}: o miolo da mancha (G {miolo}) não tem a opacidade do centro do traço (G {centro})"
            );
            assert!(
                miolo < 250,
                "controlo: {meio:?} {nome}: a mancha pintou o miolo"
            );
            // (2) `max`, nunca a soma — fora da corda (de (24, 28) a (24, 24), raio 6: ela é traço que o gesto
            // sem Solid não tem; o gate abaixo mede-a). Tolerância `3`, MEDIDA: o traço e o complemento da mancha
            // são dois `over` em 8 bits sobre o mesmo texel (Strength 0,4 · Flow 1: `166` contra `169`, já no HEAD
            // de antes da lei do Flow; depois, todo excesso é `3` e nenhum maior).
            for y in 0..128usize {
                for x in 0..128usize {
                    let dy = y.saturating_sub(28).max(24usize.saturating_sub(y));
                    if x.abs_diff(24).pow(2) + dy.pow(2) <= 9 * 9 {
                        continue;
                    }
                    let (c, s) = (g(&com, x, y), g(&sem, x, y));
                    assert!(
                        u16::from(c) + 3 >= u16::from(s.min(miolo)),
                        "{meio:?} {nome}: em ({x}, {y}) o gesto com Solid (G {c}) passou do traço (G {s}) e do miolo (G {miolo}) — a mancha somou-se"
                    );
                }
            }
        }
    }
}

/// ⭐ **A CORDA QUE FECHA A FORMA TEM A OPACIDADE DO TRAÇO** — num «U» a corda (de (104, 24) de volta a (24, 24))
/// é uma aresta inteira sem traço por baixo: no meio dela o gesto tem a opacidade do centro de uma perna, não a da
/// corda SOMADA à mancha. Com o Flow como taxa (Accumulate off) a corda carimbada por cima da mancha subia o texel
/// do nível dela para o tecto da Strength; a corda vai ANTES da mancha (`stamp_solid_loops_with_chord`).
#[test]
fn a_corda_tem_a_opacidade_do_traco() {
    const U: [[f32; 2]; 4] = [[24.0, 24.0], [24.0, 104.0], [104.0, 104.0], [104.0, 24.0]];
    let g = |t: &PainterTool, x: usize, y: usize| t.canvas_rgba[(y * 128 + x) * 4 + 1];
    for meio in [PaintMedia::Digital, PaintMedia::Impasto] {
        for (s, fl) in [(0.4f32, 0.3f32), (1.0, 0.3), (0.4, 1.0)] {
            let f = |t: &mut PainterTool| {
                t.paint.brush.strength = s;
                t.paint.brush.flow = fl;
            };
            let t = laco_pelos(meio, 1.0, &f, &U);
            let (corda, perna) = (g(&t, 64, 24), g(&t, 24, 64));
            assert!(
                perna < 250,
                "controlo: {meio:?} S {s} F {fl}: a perna pintou"
            );
            assert!(
                corda.abs_diff(perna) <= 3,
                "{meio:?} Strength {s} · Flow {fl}: o meio da corda (G {corda}) contra o centro da perna (G {perna})"
            );
        }
    }
}

/// ⭐ **NO IMPASTO O RELEVO ENCHE A MANCHA** (smoke do dono 2026-10-06: *«em impasto o relevo
/// deveria preencher o centro com solid»*) — o miolo ganha corpo à altura do traço (planalto, sem
/// degrau na junção), com a Strength de fábrica e baixa. Vermelho antes: altura `0` no miolo.
#[test]
fn no_impasto_o_relevo_enche_a_mancha() {
    for s in [1.0f32, 0.4] {
        let t = laco(PaintMedia::Impasto, s);
        let ativa = t.layers.active().expect("camada");
        let h = t.heights.get(&ativa).expect("o relevo da camada");
        // O corpo do TRAÇO: o máximo ao longo de uma aresta, longe dos cantos.
        let traco = (40..88).map(|x| h[24 * 128 + x]).fold(0.0f32, f32::max);
        let miolo: Vec<f32> = (48..80)
            .flat_map(|y| (48..80).map(move |x| (y, x)))
            .map(|(y, x)| h[y * 128 + x])
            .collect();
        let (lo, hi) = miolo
            .iter()
            .fold((f32::MAX, 0.0f32), |(a, b), &v| (a.min(v), b.max(v)));
        assert!(traco > 0.0, "controlo: Strength {s}: o traço tem corpo");
        assert!(
            lo > 0.8 * traco,
            "Strength {s}: o miolo da mancha ficou sem corpo ({lo} contra o traço {traco})"
        );
        assert!(
            hi < 1.2 * traco,
            "Strength {s}: o planalto passa do corpo do traço ({hi} contra {traco}) — degrau na junção"
        );
        // A borda fraccionária da mancha nunca BAIXA o corpo do traço (a mancha só fica com o texel
        // onde a carga dela é maior): o laço deslocado MEIO píxel põe a cobertura da borda a ~50 %
        // sob o centro do traço.
        let relevo = |solid: bool| {
            let mut t = tool(128, PaintMedia::Impasto, 6.0);
            t.paint.brush.strength = s;
            t.paint.brush.style_solid = solid;
            let c = [
                [24.5f32, 24.5],
                [104.5, 24.5],
                [104.5, 104.5],
                [24.5, 104.5],
                [24.5, 28.5],
            ];
            t.on_canvas_pointer(cp(c[0], PointerPhase::Down));
            for w in c.windows(2) {
                for k in 1..=10 {
                    #[allow(clippy::cast_precision_loss)]
                    let f = k as f32 / 10.0;
                    t.on_canvas_pointer(cp(
                        [
                            w[0][0] + (w[1][0] - w[0][0]) * f,
                            w[0][1] + (w[1][1] - w[0][1]) * f,
                        ],
                        PointerPhase::Move,
                    ));
                }
            }
            t.on_canvas_pointer(cp(c[4], PointerPhase::Up));
            let a = t.layers.active().expect("camada");
            t.heights.get(&a).expect("relevo").to_vec()
        };
        let (hc, hs) = (relevo(true), relevo(false));
        let baixou = (16..34usize)
            .flat_map(|y| (40..88).map(move |x| y * 128 + x))
            .filter(|&i| hc[i] + 1e-4 < hs[i])
            .count();
        // E a mancha é TINTA na cobertura do relevo (o que a luz pesa) — mais abaixo.
        assert_eq!(
            baixou, 0,
            "Strength {s}: a mancha baixou o corpo do traço em {baixou} texels"
        );
        let cobre = t.covers.get(&ativa).expect("a cobertura do relevo");
        assert!(
            cobre[64 * 128 + 64] > 200,
            "Strength {s}: o miolo da mancha não conta como tinta no relevo ({})",
            cobre[64 * 128 + 64]
        );
    }
}

/// Um gesto da varredura: os vértices, os eventos por aresta, o tique entre eventos e se a caneta
/// levanta no último ponto ou volta ao primeiro.
#[derive(Clone, Debug)]
struct Gesto {
    pontos: Vec<[f32; 2]>,
    por_aresta: usize,
    tique: Option<f32>,
    fecha: bool,
    strength: f32,
}

fn gestos_da_varredura(n: usize) -> Vec<Gesto> {
    let mut s = 0x9E37_79B9u32;
    let mut u = move || {
        s ^= s << 13;
        s ^= s >> 17;
        s ^= s << 5;
        #[allow(clippy::cast_precision_loss)]
        let v = (s >> 8) as f32 / (1u32 << 24) as f32;
        v
    };
    (0..n)
        .map(|_| {
            let lados = 3 + (u() * 6.0) as usize;
            let raio = 25.0 + u() * 30.0;
            let fase = u() * std::f32::consts::TAU;
            let sentido = if u() < 0.5 { 1.0 } else { -1.0 };
            let pontos = (0..lados)
                .map(|k| {
                    #[allow(clippy::cast_precision_loss)]
                    let a = fase + sentido * k as f32 / lados as f32 * std::f32::consts::TAU;
                    let r = raio * (0.8 + 0.4 * u());
                    [64.0 + r * a.cos(), 64.0 + r * a.sin()]
                })
                .collect();
            Gesto {
                pontos,
                por_aresta: 1 + (u() * 12.0) as usize,
                tique: (u() < 0.6).then(|| 0.004 + u() * 0.05),
                fecha: u() < 0.5,
                strength: if u() < 0.5 { 1.0 } else { 0.4 },
            }
        })
        .collect()
}

fn corre_gesto(meio: PaintMedia, g: &Gesto) -> PainterTool {
    let mut t = tool(128, meio, 6.0);
    t.set_brush_color_srgb8([220, 40, 40]);
    t.paint.brush.style_solid = true;
    t.paint.brush.strength = g.strength;
    let mut pts = g.pontos.clone();
    if g.fecha {
        pts.push(pts[0]);
    }
    t.on_canvas_pointer(cp(pts[0], PointerPhase::Down));
    for w in pts.windows(2) {
        for k in 1..=g.por_aresta {
            #[allow(clippy::cast_precision_loss)]
            let f = k as f32 / g.por_aresta as f32;
            t.on_canvas_pointer(cp(
                [
                    w[0][0] + (w[1][0] - w[0][0]) * f,
                    w[0][1] + (w[1][1] - w[0][1]) * f,
                ],
                PointerPhase::Move,
            ));
            if let Some(dt) = g.tique {
                t.paint_tick(dt);
            }
        }
    }
    t.on_canvas_pointer(cp(*pts.last().expect("pontos"), PointerPhase::Up));
    t
}

/// SONDA — «de vez em quando o impasto com solid não preenche» (dono, 2026-10-06): 300 gestos
/// variados; em cada, a cor e o relevo no centro (64, 64), que todo polígono da varredura cerca.
#[test]
#[ignore = "diagnóstico"]
fn diag_o_solid_que_nao_preenche() {
    for meio in [PaintMedia::Impasto, PaintMedia::Digital] {
        let mut falhas = 0;
        for (k, g) in gestos_da_varredura(300).iter().enumerate() {
            let t = corre_gesto(meio, g);
            let cor = t.canvas_rgba[(64 * 128 + 64) * 4 + 1];
            let alt = t
                .layers
                .active()
                .and_then(|a| t.heights.get(&a))
                .map_or(-1.0, |h| h[64 * 128 + 64]);
            let sem_cor = cor > 250;
            let sem_corpo = meio == PaintMedia::Impasto && alt <= 1e-4;
            if sem_cor || sem_corpo {
                falhas += 1;
                if falhas <= 8 {
                    eprintln!("{meio:?} #{k}: G {cor} altura {alt} · {g:?}");
                }
            }
        }
        eprintln!("{meio:?}: {falhas} de 300 sem preencher");
    }
}

/// ⭐ **O SOLID PREENCHE EM TODO GESTO** (dono, 2026-10-06: *«de vez em quando o impasto com solid
/// não preenche»*) — 120 gestos variados (3 a 8 lados, os dois sentidos, 1 a 12 eventos por aresta,
/// com e sem tique, Strength 1 e 0,4, e metade a FECHAR no ponto de partida): o centro, que todo
/// polígono cerca, tem cor nos dois meios e corpo no Impasto. Vermelho antes: o laço que fecha no
/// ponto de partida não tem corda, e o corpo da mancha saía junto com ela (135 de 300 sem corpo).
#[test]
fn o_solid_preenche_em_todo_gesto() {
    let gestos = gestos_da_varredura(120);
    assert!(
        gestos.iter().any(|g| g.fecha) && gestos.iter().any(|g| !g.fecha),
        "controlo"
    );
    for meio in [PaintMedia::Impasto, PaintMedia::Digital] {
        for (k, g) in gestos.iter().enumerate() {
            let t = corre_gesto(meio, g);
            let cor = t.canvas_rgba[(64 * 128 + 64) * 4 + 1];
            assert!(
                cor < 250,
                "{meio:?} gesto {k}: o centro ficou sem cor · {g:?}"
            );
            if meio == PaintMedia::Impasto {
                let alt = t
                    .layers
                    .active()
                    .and_then(|a| t.heights.get(&a))
                    .map_or(0.0, |h| h[64 * 128 + 64]);
                assert!(
                    alt > 1e-4,
                    "Impasto gesto {k}: o centro ficou sem corpo · {g:?}"
                );
            }
        }
    }
}

/// O desvio de `v` sobre os índices `ids`.
fn desvio(v: &[f32], ids: &[usize]) -> f32 {
    #[allow(clippy::cast_precision_loss)]
    let n = ids.len() as f32;
    let m = ids.iter().map(|&i| v[i]).sum::<f32>() / n;
    (ids.iter().map(|&i| (v[i] - m) * (v[i] - m)).sum::<f32>() / n).sqrt()
}

/// Um laço largo (raio 14) em Solid no Impasto com o Relief do papel ligado (papel Cold).
fn laco_no_papel(relief: f32) -> PainterTool {
    laco_com(PaintMedia::Impasto, 1.0, &|t: &mut PainterTool| {
        t.set_brush_size_px(14.0);
        t.set_brush_paper_kind(26);
        t.paint.substrate_depth = relief;
    })
}

/// SONDA — o dente do papel no corpo do TRAÇO e no da MANCHA: o desvio da altura numa faixa do
/// traço (longe dos cantos) e num quadrado do miolo, com e sem o Relief.
#[test]
#[ignore = "diagnóstico"]
fn diag_o_dente_no_corpo_da_mancha() {
    let traco: Vec<usize> = (20..28)
        .flat_map(|y| (44..84).map(move |x| y * 128 + x))
        .collect();
    let miolo: Vec<usize> = (48..80)
        .flat_map(|y| (48..80).map(move |x| y * 128 + x))
        .collect();
    for relief in [0.0f32, 1.0] {
        let t = laco_no_papel(relief);
        let h = t
            .heights
            .get(&t.layers.active().expect("camada"))
            .expect("relevo");
        eprintln!(
            "Relief {relief}: grain {:?} · desvio da altura no traço {:.4} · no miolo {:.4} (média traço {:.3}, miolo {:.3})",
            t.paint.brush.texture.kind,
            desvio(h, &traco),
            desvio(h, &miolo),
            traco.iter().map(|&i| h[i]).sum::<f32>() / traco.len() as f32,
            miolo.iter().map(|&i| h[i]).sum::<f32>() / miolo.len() as f32,
        );
    }
}

/// SONDA — o dente na IMAGEM MOSTRADA: o desvio de `lum(Relief 1) − lum(Relief 0)` no traço e no
/// miolo do Solid do Impasto, e a média da cobertura do relevo (`covers`) nos dois sítios.
#[test]
#[ignore = "diagnóstico"]
fn diag_o_dente_mostrado_na_mancha() {
    let traco: Vec<usize> = (20..28)
        .flat_map(|y| (44..84).map(move |x| y * 128 + x))
        .collect();
    let miolo: Vec<usize> = (48..80)
        .flat_map(|y| (48..80).map(move |x| y * 128 + x))
        .collect();
    let mostra = |relief: f32| {
        let mut t = laco_no_papel(relief);
        t.invalidate_composite();
        let (px, _, _) = t.take_preview_arc().expect("imagem");
        let lum: Vec<f32> = px
            .chunks(4)
            .map(|p| 0.299 * f32::from(p[0]) + 0.587 * f32::from(p[1]) + 0.114 * f32::from(p[2]))
            .collect();
        let a = t.layers.active().expect("camada");
        let cov: Vec<f32> = t
            .covers
            .get(&a)
            .expect("cobertura")
            .iter()
            .map(|&c| f32::from(c))
            .collect();
        (lum, cov)
    };
    let ((l1, c1), (l0, _)) = (mostra(1.0), mostra(0.0));
    let d: Vec<f32> = l1.iter().zip(&l0).map(|(a, b)| a - b).collect();
    let media =
        |v: &[f32], ids: &[usize]| ids.iter().map(|&i| v[i]).sum::<f32>() / ids.len() as f32;
    eprintln!(
        "dente mostrado: traço {:.3} · miolo {:.3} · cobertura do relevo: traço {:.1} miolo {:.1} · papel limpo {:.3}",
        desvio(&d, &traco),
        desvio(&d, &miolo),
        media(&c1, &traco),
        media(&c1, &miolo),
        desvio(
            &d,
            &(2..10)
                .flat_map(|y| (44..84).map(move |x| y * 128 + x))
                .collect::<Vec<_>>()
        ),
    );
}

/// SONDA — o grão no CORPO do traço e da mancha: Grain = papel Cold (Tiled), com a fonte do relevo
/// Uniform e Grain; o desvio da altura da camada no traço e no miolo.
#[test]
#[ignore = "diagnóstico"]
fn diag_o_grao_no_corpo_da_mancha() {
    use ph2d_painter_brush::height::DepthSource;
    let traco: Vec<usize> = (20..28)
        .flat_map(|y| (44..84).map(move |x| y * 128 + x))
        .collect();
    let miolo: Vec<usize> = (48..80)
        .flat_map(|y| (48..80).map(move |x| y * 128 + x))
        .collect();
    for (nome, fonte) in [
        ("Uniform", DepthSource::Uniform),
        ("Grain", DepthSource::Grain),
    ] {
        for grao in [0u8, 26] {
            let t = laco_com(PaintMedia::Impasto, 1.0, &|t: &mut PainterTool| {
                t.set_brush_size_px(14.0);
                t.set_brush_texture_kind(grao);
                t.paint.brush.impasto_source = fonte;
            });
            let h = t
                .heights
                .get(&t.layers.active().expect("camada"))
                .expect("relevo");
            eprintln!(
                "fonte {nome:<7} grain {:?} ({:?}): desvio da altura no traço {:.4} · no miolo {:.4}",
                t.paint.brush.texture.kind,
                t.paint.brush.texture.mapping,
                desvio(h, &traco),
                desvio(h, &miolo)
            );
        }
    }
}

/// ⭐ **O GRÃO DO PINCEL ESCULPE O CORPO DA MANCHA COMO ESCULPE O DO TRAÇO** (smoke do dono
/// 2026-10-06: *«em impasto o relevo do papel está corretamente sendo transmitido para o traço, mas
/// no preenchimento do solid não»*) — com a fonte do relevo em Grain e o Grain = papel Cold, o miolo
/// varia com o grão tanto quanto o traço; com a fonte Uniform os dois ficam sem grão. Vermelho antes:
/// o miolo era um planalto liso (desvio `0`) contra `0,153` no traço.
#[test]
fn o_grao_esculpe_o_corpo_da_mancha() {
    use ph2d_painter_brush::height::DepthSource;
    let traco: Vec<usize> = (14..22)
        .flat_map(|y| (44..84).map(move |x| y * 128 + x))
        .collect();
    let miolo: Vec<usize> = (48..80)
        .flat_map(|y| (48..80).map(move |x| y * 128 + x))
        .collect();
    let corre = |fonte: DepthSource| {
        let t = laco_com(PaintMedia::Impasto, 1.0, &|t: &mut PainterTool| {
            t.set_brush_size_px(14.0);
            t.set_brush_texture_kind(26);
            t.paint.brush.impasto_source = fonte;
        });
        let h = t
            .heights
            .get(&t.layers.active().expect("camada"))
            .expect("relevo")
            .clone();
        (desvio(&h, &traco), desvio(&h, &miolo))
    };
    let (gt, gm) = corre(DepthSource::Grain);
    assert!(gt > 0.05, "controlo: o grão esculpe o traço ({gt})");
    assert!(
        gm > 0.5 * gt,
        "o grão não esculpe o corpo da mancha: desvio {gm} no miolo contra {gt} no traço"
    );
    let (_, um) = corre(DepthSource::Uniform);
    assert!(um < 1e-4, "com a fonte Uniform o miolo ganhou grão ({um})");
}
