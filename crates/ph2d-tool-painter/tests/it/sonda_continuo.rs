//! **Sonda: o Continuous deixa o traço em contas?** (dono 25/10, foto: Spacing 0,401 + Continuous ligado, o traço
//! macio com bandas a cada passo). Mede a ondulação do alfa ao longo de um recto, no eixo e perto da borda.

use ph2d_brush_presets::Biblioteca;
use ph2d_editor_core::tool::{
    CanvasPaintTool, CanvasPointer, PanelEvent, PointerPhase, RasterEditTool, Tool,
};
use ph2d_tool_painter::{PaintMedia, PainterTool, ids};
use std::path::PathBuf;

const W: usize = 400;
const H: usize = 120;

fn tool(nome: &str) -> PainterTool {
    let raiz = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../assets/brushes");
    let p = Biblioteca::carregar(&raiz, None)
        .entradas
        .into_iter()
        .find(|e| e.preset.parametros.name == nome)
        .unwrap()
        .preset;
    let mut t = PainterTool::default();
    t.set_source(vec![0u8; W * H * 4], W as u32, H as u32);
    t.set_paint_media(PaintMedia::Digital);
    t.set_brush_color_srgb8([0, 0, 0]);
    t.preset_aplicar(&p).expect("aplica");
    t.set_brush_size_px(40.0);
    t
}

fn alterna(
    t: &mut PainterTool,
    id: ph2d_a11y::NodeId,
    on: bool,
    le: impl Fn(&PainterTool) -> bool,
) {
    if le(t) != on {
        t.handle_panel_event(PanelEvent::Click(id));
    }
    assert_eq!(le(t), on);
}

fn ondulacao(t: &mut PainterTool) -> (f32, f32) {
    let cp = |x: f32, phase| CanvasPointer {
        pos: [x, 60.0],
        pressure: 1.0,
        tilt: [0.0; 2],
        phase,
    };
    t.on_canvas_pointer(cp(40.0, PointerPhase::Down));
    let mut x = 40.0;
    while x < 360.0 {
        x += 6.0;
        t.on_canvas_pointer(cp(x, PointerPhase::Move));
        t.on_tick(1000.0 / 60.0);
    }
    t.on_canvas_pointer(cp(360.0, PointerPhase::Up));
    let px = t.run_full().0;
    if std::env::var("PH2D_PERFIL").is_ok() {
        let v: Vec<u8> = (120..200)
            .step_by(2)
            .map(|x| px[(60 * W + x) * 4 + 3])
            .collect();
        println!("[perfil] {v:?}");
    }
    let r = |y: usize| {
        let v: Vec<f32> = (120..280)
            .map(|x| f32::from(px[(y * W + x) * 4 + 3]))
            .collect();
        let m = v.iter().sum::<f32>() / v.len() as f32;
        let (lo, hi) = v
            .iter()
            .fold((f32::MAX, 0f32), |(a, b), &c| (a.min(c), b.max(c)));
        if m < 1.0 { 0.0 } else { (hi - lo) / m }
    };
    (r(60), r(60 + 13))
}

#[test]
#[ignore = "sonda: o Continuous em contas (dono 25/10)"]
fn sonda_continuo() {
    for nome in ["Soft Round", "Glaze Round", "Hard Round 1", "Airbrush"] {
        for (cont, acc, s, f) in [
            (false, true, 1.0, 1.0),
            (true, true, 1.0, 1.0),
            (true, false, 1.0, 1.0),
            (true, false, 0.5, 1.0),
            (true, false, 0.5, 0.3),
            (true, true, 0.5, 0.3),
        ] {
            let Some(mut t) = std::panic::catch_unwind(|| tool(nome)).ok() else {
                continue;
            };
            t.set_brush_spacing(0.401);
            alterna(&mut t, ids::PAINTER_BRUSH_CONTINUOUS_SPACING, cont, |t| {
                t.brush_settings().continuous_spacing
            });
            alterna(&mut t, ids::PAINTER_BRUSH_ACCUMULATE, acc, |t| {
                t.brush_settings().accumulate
            });
            t.set_brush_strength(s);
            t.set_brush_flow(f);
            let (eixo, borda) = ondulacao(&mut t);
            println!(
                "[continuo] {nome:12} cont {cont:5} acc {acc:5} S {s} F {f}: eixo {eixo:.3} borda {borda:.3}"
            );
        }
    }
}

/// O alfa no eixo de um recto de `x0` a `x1` (um evento a cada 6 px).
fn perfil(t: &mut PainterTool, x0: f32, x1: f32) -> Vec<u8> {
    let cp = |x: f32, phase| CanvasPointer {
        pos: [x, 60.0],
        pressure: 1.0,
        tilt: [0.0; 2],
        phase,
    };
    t.on_canvas_pointer(cp(x0, PointerPhase::Down));
    let mut x = x0;
    while x < x1 {
        x = (x + 6.0).min(x1);
        t.on_canvas_pointer(cp(x, PointerPhase::Move));
        t.on_tick(1000.0 / 60.0);
    }
    t.on_canvas_pointer(cp(x1, PointerPhase::Up));
    let px = t.run_full().0;
    (0..W).map(|x| px[(60 * W + x) * 4 + 3]).collect()
}

/// O Glaze Round a 40 px, Strength 0,5, Accumulate desligado, com o Continuous, o Spacing e o Flow dados.
fn glaze(cont: bool, sp: f32, flow: f32) -> PainterTool {
    let mut t = tool("Glaze Round");
    t.set_brush_spacing(sp);
    alterna(&mut t, ids::PAINTER_BRUSH_CONTINUOUS_SPACING, cont, |t| {
        t.brush_settings().continuous_spacing
    });
    alterna(&mut t, ids::PAINTER_BRUSH_ACCUMULATE, false, |t| {
        t.brush_settings().accumulate
    });
    t.set_brush_strength(0.5);
    t.set_brush_flow(flow);
    t
}

#[test]
#[ignore = "sonda: o primeiro carimbo do Continuous (dono 25/10, R1)"]
fn sonda_continuo_pontas() {
    for (cont, f, sp) in [0.4, 0.8, 1.0]
        .into_iter()
        .flat_map(|sp| [(false, 0.3, sp), (true, 0.3, sp), (true, 0.05, sp)])
    {
        let v = perfil(&mut glaze(cont, sp, f), 80.0, 320.0);
        let miolo = v[160..240].iter().map(|&a| f32::from(a)).sum::<f32>() / 80.0;
        let pico = |r: std::ops::Range<usize>| r.map(|x| v[x]).max().unwrap();
        println!(
            "[pontas] cont {cont:5} F {f} sp {sp:.3}: miolo {miolo:.1} início máx {} fim máx {}",
            pico(50..130),
            pico(270..350)
        );
        println!(
            "  início {:?}",
            (50..130).step_by(3).map(|x| v[x]).collect::<Vec<_>>()
        );
        println!(
            "  fim    {:?}",
            (270..350).step_by(3).map(|x| v[x]).collect::<Vec<_>>()
        );
    }
}

/// ⭐ **O início do Continuous não marca** (dono 25/10, R1: «o primeiro carimbo do traço usando Continuous em Digital
/// fica muito marcado»). O dab de abertura levava o Flow inteiro por baixo da 1.ª cápsula, cujos sub-dabs levam cada
/// um a taxa de UM sub-dab: o início ficava `+23 %` sobre o miolo a Spacing 0,4 e `+74 %` a 1,0 (Flow 0,3). Com a
/// abertura levada pela 1.ª cápsula (`ph2d-painter-brush` `stroke/abertura.rs`) nenhum texel do início passa do
/// miolo, e a entrada é a rampa da saída.
#[test]
fn o_inicio_do_continuo_nao_marca() {
    for sp in [0.4, 0.8, 1.0] {
        for f in [0.3, 0.05] {
            let v = perfil(&mut glaze(true, sp, f), 80.0, 320.0);
            let miolo = v[160..240].iter().map(|&a| f32::from(a)).sum::<f32>() / 80.0;
            let pico = f32::from((40..160).map(|x| v[x]).max().unwrap());
            assert!(
                pico <= miolo * 1.03 + 1.0,
                "Spacing {sp} Flow {f}: o início {pico} sobre o miolo {miolo:.1}"
            );
        }
    }
}

/// ⭐ **Um toque com o Continuous pinta o dab inteiro**: sem cápsula a seguir, o dab de abertura retido carimba-se no
/// pen-up, ao byte do mesmo toque sem o Continuous.
#[test]
fn um_toque_do_continuo_pinta_o_dab_inteiro() {
    let toque = |cont: bool| {
        let mut t = glaze(cont, 0.8, 0.3);
        for phase in [PointerPhase::Down, PointerPhase::Up] {
            t.on_canvas_pointer(CanvasPointer {
                pos: [200.0, 60.0],
                pressure: 1.0,
                tilt: [0.0; 2],
                phase,
            });
        }
        t.run_full().0
    };
    let com = toque(true);
    assert!(com[(60 * W + 200) * 4 + 3] > 0, "o toque não pintou");
    assert!(
        com == toque(false),
        "o toque com o Continuous difere do sem"
    );
}

/// ⭐ **O Continuous com o Accumulate desligado e o Flow baixo é LISO** (dono 25/10, a foto do traço em contas): a
/// cápsula aplicava o Flow uma vez por passo sobre a união dos sub-dabs, e a costura entre dois passos levava-o
/// duas vezes — 15 % de ondulação no Glaze Round a Flow 0,3. Com a taxa repartida pelos sub-dabs
/// (`dab/bands.rs`, `c_cap`) a ondulação é a do arredondamento (≤ 3 %), no eixo e na borda.
#[test]
fn o_continuo_com_flow_baixo_e_liso() {
    let mut t = tool("Glaze Round");
    t.set_brush_spacing(0.401);
    alterna(&mut t, ids::PAINTER_BRUSH_CONTINUOUS_SPACING, true, |t| {
        t.brush_settings().continuous_spacing
    });
    alterna(&mut t, ids::PAINTER_BRUSH_ACCUMULATE, false, |t| {
        t.brush_settings().accumulate
    });
    t.set_brush_strength(0.5);
    t.set_brush_flow(0.3);
    let (eixo, borda) = ondulacao(&mut t);
    assert!(
        eixo <= 0.03 && borda <= 0.03,
        "o traço em contas: eixo {eixo:.3} borda {borda:.3}"
    );
}
