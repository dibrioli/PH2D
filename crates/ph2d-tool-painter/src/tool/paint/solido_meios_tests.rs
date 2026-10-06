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

/// O laço com a pressão `p` constante e os ajustes `f` aplicados ao pincel.
fn laco_com(meio: PaintMedia, p: f32, f: &dyn Fn(&mut PainterTool)) -> PainterTool {
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
    let cantos = [
        [24.0f32, 24.0],
        [104.0, 24.0],
        [104.0, 104.0],
        [24.0, 104.0],
        [24.0, 28.0],
    ];
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
    t.on_canvas_pointer(ptr(cantos[4], PointerPhase::Up));
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
fn mais_opaco(t: &PainterTool) -> u8 {
    t.canvas_rgba.chunks(4).map(|p| p[1]).min().expect("tela")
}

/// ⭐ **NENHUM TEXEL DO GESTO COM SOLID PASSA DO TECTO DO TRAÇO** (smoke do dono 2026-10-06: *«a
/// cor central do solid não respeita o strength do pincel e fica mais escura»*) — no Digital e no
/// Impasto, com Strength e Flow baixos: o texel mais opaco do gesto com Solid é o do traço SEM Solid
/// (a mancha leva cada texel só até o tecto, nunca compõe por cima), e o miolo da mancha tem a
/// opacidade do traço. Vermelho antes: a mancha a `0,4` por cima de um traço a `0,163`.
#[test]
fn a_mancha_obedece_ao_tecto_do_traco() {
    type Ajuste = (&'static str, fn(&mut PainterTool));
    let casos: [Ajuste; 3] = [
        ("Strength 0,4", |t| t.paint.brush.strength = 0.4),
        ("Strength 0,7", |t| t.paint.brush.strength = 0.7),
        ("Strength 0,4 + Flow 0,3", |t| {
            t.paint.brush.strength = 0.4;
            t.paint.brush.flow = 0.3;
        }),
    ];
    for meio in [PaintMedia::Digital, PaintMedia::Impasto] {
        for (nome, f) in casos {
            let com = laco_com(meio, 1.0, &f);
            let sem = laco_com(meio, 1.0, &|t: &mut PainterTool| {
                f(t);
                t.paint.brush.style_solid = false;
            });
            let (g_com, g_sem) = (mais_opaco(&com), mais_opaco(&sem));
            assert!(
                g_com + 1 >= g_sem,
                "{meio:?} {nome}: o gesto com Solid passou do tecto do traço (G {g_com} contra {g_sem})"
            );
            let miolo = com.canvas_rgba[(64 * 128 + 64) * 4 + 1];
            assert!(
                miolo.abs_diff(g_sem) <= 2,
                "{meio:?} {nome}: o miolo da mancha (G {miolo}) não tem a opacidade do traço (G {g_sem})"
            );
            assert!(
                miolo < 250,
                "controlo: {meio:?} {nome}: a mancha pintou o miolo"
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
