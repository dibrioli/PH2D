//! **O pen-up paga o TRAÇO, não a sessão** (B16.2, WC-20): dez traços numa sessão molhada; o oráculo é o
//! pen-up pela UNIÃO da sessão (`COMMIT_PELA_UNIAO`, o caminho antigo) no mesmo processo.

use super::COMMIT_PELA_UNIAO;
use crate::tool::PainterTool;
use crate::tool::paint::media::PaintMedia;
use ph2d_editor_core::tool::{CanvasPaintTool, CanvasPointer, PointerPhase, RasterEditTool};

const LADO: u32 = 256;

fn cp(pos: [f32; 2], phase: PointerPhase) -> CanvasPointer {
    CanvasPointer {
        pos,
        pressure: 1.0,
        tilt: [0.0, 0.0],
        phase,
    }
}

fn tool(side: u32, media: PaintMedia, radius: f32) -> PainterTool {
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; (side * side * 4) as usize], side, side);
    t.set_paint_media(media);
    crate::tool::paint::tests::aquarela_neutra(&mut t);
    t.set_brush_size_px(radius);
    t
}
const TRACOS: usize = 10;

#[derive(Clone, Copy, Debug)]
struct Cenario {
    nome: &'static str,
    basic: bool,
    rewet: f32,
    dilution: f32,
    charge: f32,
    raio_varia: bool,
    /// Espera entre traços (s) com Dry Time 3 s: as poças do início secam e ASSAM no pen-down.
    espera_s: f32,
}

const fn cen(nome: &'static str) -> Cenario {
    Cenario {
        nome,
        basic: false,
        rewet: 0.0,
        dilution: 0.0,
        charge: 1.0,
        raio_varia: false,
        espera_s: 0.0,
    }
}

const CENARIOS: [Cenario; 6] = [
    cen("neutro"),
    Cenario {
        basic: true,
        ..cen("Watercolor Basic")
    },
    Cenario {
        rewet: 0.6,
        ..cen("Rewet 0,6")
    },
    Cenario {
        dilution: 0.6,
        charge: 0.5,
        ..cen("Dilution 0,6 + Charge 0,5")
    },
    Cenario {
        raio_varia: true,
        basic: true,
        ..cen("Basic, raio muda por traço")
    },
    Cenario {
        espera_s: 0.75,
        basic: true,
        ..cen("Basic com secagem")
    },
];

fn pincel(c: Cenario) -> PainterTool {
    let mut t = tool(LADO, PaintMedia::Watercolor, 4.0);
    if c.basic {
        crate::tool::paint::tests::watercolor_basic_de_antes(&mut t);
    }
    t.set_brush_color_srgb8([30, 60, 220]);
    t.paint.brush.wet_rewet = c.rewet;
    t.paint.brush.wet_dilution = c.dilution;
    t.paint.brush.wet_charge = c.charge;
    t.set_dry_time_s(if c.espera_s > 0.0 { 3.0 } else { 60.0 });
    t
}

/// A sessão: as telas depois de cada pen-up e os texels que cada pen-up caminhou.
fn sessao(c: Cenario, uniao: bool) -> (Vec<Vec<u8>>, Vec<u64>) {
    COMMIT_PELA_UNIAO.set(uniao);
    let mut t = pincel(c);
    let (mut telas, mut pen_up) = (Vec::new(), Vec::new());
    for k in 0..TRACOS {
        #[allow(clippy::cast_precision_loss)]
        let x = 14.0 + 25.0 * k as f32;
        if c.raio_varia {
            #[allow(clippy::cast_precision_loss)]
            t.set_brush_size_px(3.0 + (k % 3) as f32);
        }
        if k > 0 {
            assert!(
                t.wet_session_continues(),
                "{}: a régua — o traço {k} tem de continuar a sessão",
                c.nome
            );
        }
        t.on_canvas_pointer(cp([x, 40.0], PointerPhase::Down));
        for j in 1..=10 {
            #[allow(clippy::cast_precision_loss)]
            t.on_canvas_pointer(cp([x, 40.0 + 6.0 * j as f32], PointerPhase::Move));
            t.paint_tick(0.05);
        }
        let px0 = t.wash.window_px;
        t.on_canvas_pointer(cp([x, 100.0], PointerPhase::Up));
        pen_up.push(t.wash.window_px - px0);
        telas.push(t.canvas_rgba.to_vec());
        for _ in 0..((c.espera_s / 0.25).round() as usize).max(1) {
            t.paint_tick(0.25_f32.min(c.espera_s.max(0.05)));
        }
    }
    COMMIT_PELA_UNIAO.set(false);
    (telas, pen_up)
}

/// ⭐ **O décimo traço da sessão não paga os nove anteriores** — e a tela de cada pen-up é a do pen-up
/// pela união, ao byte. Controlo: pela união, o décimo paga mais que o dobro do primeiro.
#[test]
fn o_decimo_traco_da_sessao_nao_paga_os_nove_anteriores() {
    let mut falhas = Vec::new();
    let mut ultima = Vec::new();
    for c in CENARIOS {
        let (novo, px_novo) = sessao(c, false);
        let (velho, px_velho) = sessao(c, true);
        ultima.push(novo[TRACOS - 1].clone());
        eprintln!(
            "{:<28} pen-up (texels)  traço: 1.º {:>7} 10.º {:>7}  união: 1.º {:>7} 10.º {:>7}",
            c.nome,
            px_novo[0],
            px_novo[TRACOS - 1],
            px_velho[0],
            px_velho[TRACOS - 1]
        );
        if let Some(k) = (0..TRACOS).find(|&k| novo[k] != velho[k]) {
            let n = novo[k]
                .chunks(4)
                .zip(velho[k].chunks(4))
                .filter(|(a, b)| a != b)
                .count();
            falhas.push(format!(
                "{}: pen-up {k} difere da união em {n} texels",
                c.nome
            ));
        }
        if c.raio_varia {
            // Fora do Wet Paint o estilo não uniforme fica na união (o B16.2 cont3 só no Wet, BUGS_painter #115): só a
            // paridade conta.
            continue;
        }
        if px_velho[TRACOS - 1] <= 2 * px_velho[0] {
            falhas.push(format!(
                "{}: o controlo — pela união o 10.º ({}) devia pagar > 2× o 1.º ({})",
                c.nome,
                px_velho[TRACOS - 1],
                px_velho[0]
            ));
        }
        if 2 * px_novo[TRACOS - 1] > 3 * px_novo[0] {
            falhas.push(format!(
                "{}: o 10.º pen-up caminhou {} texels, o 1.º {} (paga a sessão)",
                c.nome,
                px_novo[TRACOS - 1],
                px_novo[0]
            ));
        }
    }
    // A régua: o Basic de antes pega (Edge 3, papel cold-press) — a janela é a do neutro (o `pad` não
    // lê o ganho nem o papel), a tinta não.
    if ultima[0] == ultima[1] {
        falhas.push("a régua: «Watercolor Basic» pintou os bytes do neutro".into());
    }
    assert!(falhas.is_empty(), "{falhas:#?}");
}

/// Sonda B16.2 (`--ignored`, `--profile smoke`): o pen-up da aguada a 3840×2160 numa sessão de 16
/// traços (Basic de antes, raio 30), pelo traço (A) contra pela união (B) no MESMO processo, ABBA; o
/// mínimo dos 8 últimos de cada (a união só cresce: o viés ABBA cancela).
#[test]
#[ignore]
fn mede_o_pen_up_da_aguada_4k() {
    for misto in [false, true] {
        mede_uma_sessao_4k(misto);
    }
}

fn mede_uma_sessao_4k(misto: bool) {
    let (fw, fh) = (3840u32, 2160u32);
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; (fw * fh * 4) as usize], fw, fh);
    t.set_paint_media(PaintMedia::Watercolor);
    crate::tool::paint::tests::aquarela_neutra(&mut t);
    t.set_brush_size_px(30.0);
    crate::tool::paint::tests::watercolor_basic_de_antes(&mut t);
    t.set_brush_color_srgb8([30, 60, 220]);
    t.set_dry_time_s(60.0);
    let (mut best, mut px, mut sessao) = ([f64::MAX; 2], [0u64; 2], 0);
    for k in 0..16usize {
        let v = [0usize, 1, 1, 0][k % 4];
        #[allow(clippy::cast_precision_loss)]
        let (x, y) = (200.0 + 220.0 * k as f32, 300.0 + 400.0 * (k % 4) as f32);
        sessao += usize::from(k > 0 && t.wet_session_continues());
        if misto {
            t.set_brush_size_px([30.0, 20.0, 40.0][k % 3]); // estilos mistos (B16.2 cont3)
        }
        t.on_canvas_pointer(cp([x, y], PointerPhase::Down));
        for j in 1..=10 {
            #[allow(clippy::cast_precision_loss)]
            let j = j as f32;
            t.on_canvas_pointer(cp([x + 12.0 * j, y + 8.0 * j], PointerPhase::Move));
            t.paint_tick(1.0 / 60.0);
        }
        COMMIT_PELA_UNIAO.set(v == 1);
        let px0 = t.wash.window_px;
        let t0 = std::time::Instant::now();
        t.on_canvas_pointer(cp([x + 120.0, y + 80.0], PointerPhase::Up));
        let ms = t0.elapsed().as_secs_f64() * 1e3;
        COMMIT_PELA_UNIAO.set(false);
        if k >= 8 {
            best[v] = best[v].min(ms);
            px[v] = t.wash.window_px - px0;
        }
    }
    println!(
        "[B16.2] 3840x2160, 16 traços ({sessao} continuam a sessão, raio misto {misto}): pen-up pelo \
         traço {:.2} ms ({} texels) | pela união {:.2} ms ({} texels)",
        best[0], px[0], best[1], px[1]
    );
}
