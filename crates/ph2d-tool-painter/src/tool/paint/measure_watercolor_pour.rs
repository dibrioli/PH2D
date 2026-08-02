//! **O `pour_canvas_wet`: quanto ele cobra, e o que a rota do quadro MUDA na tinta.**
//!
//! O pour caminha `wet_stroke_dirty` — a união **cumulativa** desde o pen-down — uma vez por quadro,
//! então o custo por quadro cresce com o comprimento do traço (doc 28 §5.60: 0,35 → 0,94 M texels num
//! traço de 1500 px, contra uma janela de composite PLANA em 0,36 M).
//!
//! ⚠️ **As duas perguntas são separadas e as duas precisam de resposta**, porque a troca **não** é
//! byte-idêntica: `dry_canvas_wet` roda no MESMO `paint_tick`, então `canvas_wet` decai a cada quadro e
//! o pour cumulativo o levanta de volta sobre a pegada inteira. Caminhar só o quadro deixa a CAUDA
//! secar enquanto o artista pinta. Logo:
//!
//! 1. **quanto custa** — ms por quadro, as duas rotas **costas-com-costas na mesma corrida** (§5.46: um
//!    A/B cross-run atribui a deriva deste box à mudança);
//! 2. **o que muda** — quantos bytes de `canvas_wet` diferem, e qual o pior delta. É este número que
//!    vira decisão de PRODUTO, não de engenharia.

use super::*;
use crate::tool::paint::media::PaintMedia;
use ph2d_editor_core::tool::RasterEditTool;
use ph2d_painter_brush::Falloff;
use std::time::Instant;

fn cp(pos: [f32; 2], phase: PointerPhase) -> CanvasPointer {
    CanvasPointer {
        pos,
        pressure: 1.0,
        tilt: [0.0, 0.0],
        phase,
    }
}

/// O pincel e os knobs do report do Enio (screenshot de 2026-08-02): raio 250, `Rewet 0.400`,
/// `Smudge 0.197`, `Dilution 0.168`, `Charge 0.755`, `Pull 0.477`.
fn artist_wash(size: u32, radius: f32) -> PainterTool {
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; (size * size * 4) as usize], size, size);
    let b = BrushSpec {
        radius_px: radius,
        hardness: 0.5,
        falloff: Falloff::Smooth,
        strength: 1.0,
        color: [0.8, 0.2, 0.1],
        space_attenuation: false,
        wet_charge: 0.755,
        wet_dilution: 0.168,
        wet_pull: 0.477,
        wet_rewet: 0.400,
        wet_smudge: 0.197,
        ..Default::default()
    };
    t.paint.brush = b;
    for slot in &mut t.paint.brush_by_mode {
        *slot = b;
    }
    t.set_paint_media(PaintMedia::Watercolor);
    t
}

const EV: u32 = 4;
const FRAMES: u32 = 48;
const DT: f32 = 1.0 / 60.0;
const PATH: f32 = 1500.0;

/// **De que é feito um QUADRO de aquarela, e por que ele responde ao tamanho da TELA?**
///
/// A janela do composite é IDÊNTICA a 2048² e 4096² (0,36 M texels — `measure_the_area_a_watercolor_frame_walks`),
/// e mesmo assim o quadro custa 28 contra 84 ms. Logo o crescimento **não é a janela** — é algo que
/// caminha o PLANO. O suspeito com precedente nesta linha é um **segundo dono do `Arc`** do canvas
/// (§5.12: uma pergunta de identidade paga com POSSE custa uma cópia do documento por escrita).
#[test]
#[ignore = "measurement, not a gate"]
fn measure_why_a_watercolor_frame_grows_with_the_canvas() {
    const RADIUS: f32 = 250.0;
    println!("\nraio {RADIUS:.0}, knobs do Enio — donos do Arc do canvas por quadro\n");
    println!(
        "{:<8} {:>10} {:>12} {:>12} {:>12}",
        "canvas", "quadro", "donos strong", "ms/quadro", "ms sem preview"
    );
    for size in [2048u32, 4096] {
        for skip_preview in [false, true] {
            let mut t = artist_wash(size, RADIUS);
            let mid = f64::from(size / 2) as f32;
            let x0 = RADIUS + 20.0;
            let step = PATH / f64::from(FRAMES * EV) as f32;
            t.on_canvas_pointer(cp([x0, mid], PointerPhase::Down));
            if !skip_preview {
                let _ = t.take_preview_arc();
            }
            let mut owners = 0usize;
            let mut ms = Vec::new();
            let mut k = 0u32;
            for f in 0..FRAMES {
                if f == FRAMES / 2 {
                    owners = std::sync::Arc::strong_count(&t.canvas_rgba);
                }
                let f0 = Instant::now();
                for _ in 0..EV {
                    k += 1;
                    t.on_canvas_pointer(cp(
                        [x0 + step * f64::from(k) as f32, mid],
                        PointerPhase::Move,
                    ));
                }
                t.paint_tick(DT);
                ms.push(f0.elapsed().as_secs_f64() * 1e3);
                if !skip_preview {
                    let _ = t.take_preview_arc();
                }
            }
            t.on_canvas_pointer(cp([x0 + PATH, mid], PointerPhase::Up));
            ms.sort_by(|a, b| a.partial_cmp(b).expect("finite"));
            let med = ms[ms.len() / 2];
            if skip_preview {
                println!("{:>58.3}", med);
            } else {
                print!("{size:<8} {:>10} {owners:>12} {med:>12.3}", "meio do traço");
            }
        }
    }
    println!();
}

/// **A metade que cresce: o CARIMBO ou o TICK?**
///
/// O composite é window-bound e MEDIDO idêntico nas duas telas (0,36 M texels), não há trabalho
/// `n`-sized dentro dele, e o `Arc` do canvas tem UM dono (nada de copy-on-write). Sobram duas
/// metades do quadro, e elas se separam sem instrumentação: os `on_canvas_pointer` (o carimbo, que
/// escreve os planos indexados pela LARGURA do canvas) e o `paint_tick` (composite + pour + secagem).
#[test]
#[ignore = "measurement, not a gate"]
fn measure_which_half_of_the_frame_grows() {
    const RADIUS: f32 = 250.0;
    println!("\nraio {RADIUS:.0}, knobs do Enio — as duas metades do quadro\n");
    println!(
        "{:<8} {:>14} {:>12} {:>12} {:>10}",
        "canvas", "carimbo ms", "tick ms", "quadro ms", "carimbo %"
    );
    let mut prev: Option<(f64, f64)> = None;
    for size in [2048u32, 4096] {
        let mut t = artist_wash(size, RADIUS);
        let mid = f64::from(size / 2) as f32;
        let x0 = RADIUS + 20.0;
        let step = PATH / f64::from(FRAMES * EV) as f32;
        t.on_canvas_pointer(cp([x0, mid], PointerPhase::Down));
        let _ = t.take_preview_arc();
        let (mut stamps, mut ticks) = (Vec::new(), Vec::new());
        let mut k = 0u32;
        for _ in 0..FRAMES {
            let s0 = Instant::now();
            for _ in 0..EV {
                k += 1;
                t.on_canvas_pointer(cp(
                    [x0 + step * f64::from(k) as f32, mid],
                    PointerPhase::Move,
                ));
            }
            stamps.push(s0.elapsed().as_secs_f64() * 1e3);
            let t0 = Instant::now();
            t.paint_tick(DT);
            ticks.push(t0.elapsed().as_secs_f64() * 1e3);
            let _ = t.take_preview_arc();
        }
        t.on_canvas_pointer(cp([x0 + PATH, mid], PointerPhase::Up));
        let med = |v: &mut Vec<f64>| {
            v.sort_by(|a, b| a.partial_cmp(b).expect("finite"));
            v[v.len() / 2]
        };
        let (st, tk) = (med(&mut stamps), med(&mut ticks));
        println!(
            "{size:<8} {st:>14.3} {tk:>12.3} {:>12.3} {:>9.0}%",
            st + tk,
            100.0 * st / (st + tk)
        );
        if let Some((ps, pt)) = prev {
            println!(
                "{:<8} {:>14.2}x {:>11.2}x {:>11.2}x",
                "cresce",
                st / ps,
                tk / pt,
                (st + tk) / (ps + pt)
            );
        }
        prev = Some((st, tk));
    }
    println!();
}

/// **Qual knob paga o carimbo que cresce 7,28× com a tela?**
///
/// Ablação pela ENTRADA sobre a metade que a medição isolou (o `on_canvas_pointer`), nas duas telas —
/// a coluna que importa é a RAZÃO 4096÷2048 de cada linha, não o absoluto: quem não cresce não é o
/// culpado, por mais caro que seja.
#[test]
#[ignore = "measurement, not a gate"]
fn measure_which_knob_pays_the_stamp() {
    const RADIUS: f32 = 250.0;
    println!("\nraio {RADIUS:.0} — CARIMBO por quadro, ablação por entrada\n");
    println!(
        "{:<24} {:>12} {:>12} {:>10}",
        "ablacao", "2048 ms", "4096 ms", "cresce"
    );
    type Tweak = fn(&mut BrushSpec);
    let cases: [(&str, Tweak); 6] = [
        ("como o Enio ajustou", |_b| {}),
        ("sem Charge (mixer)", |b| b.wet_charge = 0.0),
        ("sem Dilution (agua)", |b| b.wet_dilution = 0.0),
        ("sem Pull", |b| b.wet_pull = 0.0),
        ("sem Rewet", |b| b.wet_rewet = 0.0),
        ("sem Smudge", |b| b.wet_smudge = 0.0),
    ];
    for (name, tweak) in cases {
        let mut row = [0.0f64; 2];
        for (i, size) in [2048u32, 4096].into_iter().enumerate() {
            let mut t = artist_wash(size, RADIUS);
            tweak(&mut t.paint.brush);
            for slot in &mut t.paint.brush_by_mode {
                tweak(slot);
            }
            let mid = f64::from(size / 2) as f32;
            let x0 = RADIUS + 20.0;
            let step = PATH / f64::from(FRAMES * EV) as f32;
            t.on_canvas_pointer(cp([x0, mid], PointerPhase::Down));
            let _ = t.take_preview_arc();
            let mut stamps = Vec::new();
            let mut k = 0u32;
            for _ in 0..FRAMES {
                let s0 = Instant::now();
                for _ in 0..EV {
                    k += 1;
                    t.on_canvas_pointer(cp(
                        [x0 + step * f64::from(k) as f32, mid],
                        PointerPhase::Move,
                    ));
                }
                stamps.push(s0.elapsed().as_secs_f64() * 1e3);
                t.paint_tick(DT);
                let _ = t.take_preview_arc();
            }
            t.on_canvas_pointer(cp([x0 + PATH, mid], PointerPhase::Up));
            stamps.sort_by(|a, b| a.partial_cmp(b).expect("finite"));
            row[i] = stamps[stamps.len() / 2];
        }
        println!(
            "{name:<24} {:>12.3} {:>12.3} {:>9.2}x",
            row[0],
            row[1],
            row[1] / row[0]
        );
    }
    println!();
}
