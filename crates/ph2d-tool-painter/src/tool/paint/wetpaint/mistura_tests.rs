//! **A MISTURA do Wet Paint é a do Rebelle** (W1e, F6; dono 09/10: *«não acontece em tempo real mas
//! apenas no mouse up… acontece da mesma maneira sobre tinta seca e tinta úmida»*).
//!
//! A cena da sonda [`super::sonda_mistura`]: vermelho horizontal, azul vertical por cima, com o vermelho
//! MOLHADO (0,5 s) ou SECO (Dry canvas). A régua é a de `ferramentas/velatura.py`, em luz linear: `γ` =
//! onde o cruzamento cai entre «o azul por cima» (0) e «multiplicar as duas» (1); a SANGRIA = quanto o
//! vermelho a 15 px do cruzamento ganhou de azul. O Rebelle 8.3.4 na mesma cena (05 §4b): seco γ
//! `0,42–0,51` durante e depois, sangria `+1…+2`; molhado γ `0,62` durante, sangria `+12…+21`.
//!
//! ⚠️ **O controlo está DENTRO do gate:** a mesma cena com o port (γ 0, água parada no traço) tem de
//! REPROVAR a barra — sem isso um verde não diria se a régua discrimina.
use crate::tool::PainterTool;
use ph2d_brush_presets::Biblioteca;
use ph2d_editor_core::Tool;
use ph2d_editor_core::tool::{
    CanvasPaintTool, CanvasPointer, PanelEvent, PointerPhase, RasterEditTool,
};

const W: usize = 600;
const H: usize = 300;

fn cp(pos: [f32; 2], phase: PointerPhase) -> CanvasPointer {
    CanvasPointer {
        pos,
        pressure: 1.0,
        tilt: [0.0, 0.0],
        phase,
    }
}

fn media(px: &[u8], x0: usize, x1: usize, y0: usize, y1: usize) -> [f64; 3] {
    let mut s = [0.0; 3];
    let mut n = 0.0;
    for y in y0..y1 {
        for x in x0..x1 {
            for c in 0..3 {
                s[c] += f64::from(px[(y * W + x) * 4 + c]);
            }
            n += 1.0;
        }
    }
    s.map(|v| v / n)
}

fn lin(c: f64) -> f64 {
    let c = c / 255.0;
    if c <= 0.04045 {
        c / 12.92
    } else {
        ((c + 0.055) / 1.055).powf(2.4)
    }
}

/// O que a régua lê de uma tela: `γ` (média do G e do B — o R do cruzamento sobe com a MISTURA, e
/// aí não mede a velatura) e a sangria: quanto o vermelho a 15 px do cruzamento ganhou de azul desde
/// ANTES do azul (o MESMO sítio — a textura do papel não entra; comparar com o vermelho longe lia −3
/// só pelo papel).
struct Leitura {
    gama: f64,
    sangria: f64,
}

fn le(px: &[u8], antes: &[u8]) -> Leitura {
    let cruz = media(px, 295, 306, 145, 156);
    let verm = media(px, 195, 206, 145, 156);
    let azul = media(px, 295, 306, 60, 100);
    let perto = media(px, 315, 330, 145, 156);
    let g = |c: usize| {
        let (x, v, b) = (lin(cruz[c]), lin(verm[c]), lin(azul[c]));
        (x - b) / (v * b - b)
    };
    Leitura {
        gama: (g(1) + g(2)) / 2.0,
        sangria: perto[2] - media(antes, 315, 330, 145, 156)[2],
    }
}

/// A cena: devolve as telas ANTES do azul, DURANTE o azul (ponteiro em baixo) e 2 s DEPOIS do Up.
fn cena(seco: bool, produto: bool) -> (Vec<u8>, Vec<u8>, Vec<u8>) {
    let raiz = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../assets/brushes");
    let b = Biblioteca::carregar(&raiz, None);
    let p = &b
        .entradas
        .iter()
        .find(|e| e.preset.parametros.name == "Wet Round")
        .expect("o pincel de fábrica")
        .preset;
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; W * H * 4], W as u32, H as u32);
    t.set_paint_tool_mode("wetpaint");
    t.set_wet_relogio_fixo(true);
    assert!(t.wet_preset_aplicar(p).is_empty());
    t.wet_preset_aplicar_volumes(&p.parametros.volumes);
    t.paint.wetpaint.tilt_on = false;
    t.set_brush_size_px(30.0);
    assert!(t.ensure_wet_session());
    if !produto {
        let s = t.paint.wetpaint.session.as_mut().expect("sessão");
        s.engine.sim.glaze = 0.0;
        s.engine.sim.sim_while_painting = false;
    }
    t.set_brush_color_srgb8([200, 30, 30]);
    t.on_canvas_pointer(cp([100.0, 150.0], PointerPhase::Down));
    for k in 1..=133 {
        t.on_canvas_pointer(cp([100.0 + 3.0 * k as f32, 150.0], PointerPhase::Move));
        t.on_tick(8.0);
    }
    t.on_canvas_pointer(cp([500.0, 150.0], PointerPhase::Up));
    for _ in 0..30 {
        t.on_tick(1000.0 / 60.0);
    }
    if seco {
        assert!(t.route_brush_wetpaint_event(&PanelEvent::Click(
            crate::ids::PAINTER_WETPAINT_DRYCANVAS
        )));
        for _ in 0..30 {
            t.on_tick(1000.0 / 60.0);
        }
    }
    let antes = t.canvas_rgba.to_vec();
    t.set_brush_color_srgb8([30, 60, 210]);
    t.on_canvas_pointer(cp([300.0, 40.0], PointerPhase::Down));
    let mut k: u16 = 0;
    while 40.0 + 3.0 * f32::from(k) < 260.0 {
        k += 1;
        t.on_canvas_pointer(cp([300.0, 40.0 + 3.0 * f32::from(k)], PointerPhase::Move));
        t.on_tick(8.0);
    }
    for _ in 0..6 {
        t.on_tick(1000.0 / 60.0);
    }
    let durante = t.canvas_rgba.to_vec();
    t.on_canvas_pointer(cp([300.0, 260.0], PointerPhase::Up));
    for _ in 0..120 {
        t.on_tick(1000.0 / 60.0);
    }
    (antes, durante, t.canvas_rgba.to_vec())
}

/// A barra (o Rebelle com folga): o seco VELA (γ entre 0,30 e 0,70, durante e depois) e não sangra
/// (`|sangria| ≤ 3`); o molhado SANGRA (`≥ +8`) e a mistura vê-se a pintar (γ durante `≥ 0,30`).
fn barra(seco: (Leitura, Leitura), molhado: (Leitura, Leitura)) -> Result<(), String> {
    let mut f = Vec::new();
    for (nome, l) in [("seco durante", &seco.0), ("seco depois", &seco.1)] {
        if !(0.30..=0.70).contains(&l.gama) {
            f.push(format!("{nome}: γ {:.2} fora de 0,30–0,70 (Rebelle 0,42–0,51)", l.gama));
        }
    }
    if seco.1.sangria.abs() > 3.0 {
        f.push(format!("seco: sangria {:+.0} (Rebelle +1…+2)", seco.1.sangria));
    }
    if molhado.1.sangria < 8.0 {
        f.push(format!("molhado: sangria {:+.0} < +8 (Rebelle +12…+21)", molhado.1.sangria));
    }
    if molhado.0.gama < 0.30 {
        f.push(format!("molhado durante: γ {:.2} < 0,30 (Rebelle 0,62)", molhado.0.gama));
    }
    if f.is_empty() { Ok(()) } else { Err(f.join(" · ")) }
}

#[test]
fn azul_sobre_vermelho_vela_no_seco_e_sangra_no_molhado_como_no_rebelle() {
    let ler = |(z, a, b): (Vec<u8>, Vec<u8>, Vec<u8>)| (le(&a, &z), le(&b, &z));
    let produto = barra(ler(cena(true, true)), ler(cena(false, true)));
    let port = barra(ler(cena(true, false)), ler(cena(false, false)));
    assert!(
        port.is_err(),
        "CONTROLO: o port (γ 0, água parada no traço) passou a barra — a régua não discrimina"
    );
    if let Err(e) = produto {
        panic!("a mistura do produto falhou a barra do Rebelle: {e}");
    }
}
