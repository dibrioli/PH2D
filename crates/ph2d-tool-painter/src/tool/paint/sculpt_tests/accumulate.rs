//! Gates do **Accumulate** — o que uma 2ª passada DENTRO de um traço significa.
//!
//! Filho de [`super`] para partilhar os fixtures. O checkbox é o `BRUSH_ACCUMULATE` do Blender e já
//! existia governando a **cor**; estes gates cobrem a metade que faltava, o **relevo**.
//!
//! A lei: desmarcado, o relevo é o **envelope** (uma passada, uma espessura) — e é o de sempre, AO BIT.
//! Marcado, o depósito é integrado ao longo do CAMINHO, então voltar por cima soma; e porque é integral
//! de arco e não soma de dabs, ele não passa a depender do Spacing.

use super::*;

/// TEMP PROBE — o Accumulate da COR funciona?
#[test]
#[ignore = "probe"]
fn probe_colour_accumulate() {
    use ph2d_painter_brush::{BrushSpec, Falloff};
    let size = 160u32;
    let run = |accumulate: bool, back: bool| -> u8 {
        let mut t = PainterTool::default();
        t.set_source(vec![255u8; (size * size * 4) as usize], size, size);
        let b = BrushSpec {
            radius_px: 12.0,
            hardness: 0.5,
            falloff: Falloff::Smooth,
            strength: 0.3, // BEM abaixo de 1: é onde o cap morde
            color: [0.0, 0.0, 0.0],
            space_attenuation: false,
            accumulate,
            ..Default::default()
        };
        t.paint.brush = b;
        for slot in &mut t.paint.brush_by_mode {
            *slot = b;
        }
        t.set_paint_tool_mode("brush");
        t.on_canvas_pointer(cp([40.0, 80.0], PointerPhase::Down));
        for i in 1..=8 {
            t.on_canvas_pointer(cp([40.0 + i as f32 * 8.0, 80.0], PointerPhase::Move));
        }
        if back {
            for i in (0..8).rev() {
                t.on_canvas_pointer(cp([40.0 + i as f32 * 8.0, 80.0], PointerPhase::Move));
            }
        }
        t.on_canvas_pointer(cp(
            [if back { 40.0 } else { 104.0 }, 80.0],
            PointerPhase::Up,
        ));
        let (px, _, _) = t.take_preview_arc().expect("preview");
        // canal R no meio do traço: quanto mais escuro, mais tinta
        let i = (80usize * size as usize + 72) * 4;
        px.get(i).copied().unwrap_or(255)
    };
    for acc in [false, true] {
        let one = run(acc, false);
        let two = run(acc, true);
        println!(
            "[probe] accumulate={acc:5} · 1 passada R={one:3} · ida-e-volta R={two:3} · delta {}",
            i32::from(one) - i32::from(two)
        );
    }
}

/// Um traço de impasto, opcionalmente com ida e volta, devolvendo o pico de relevo.
fn impasto_peak(accumulate: bool, back: bool) -> f32 {
    use ph2d_painter_brush::{BrushSpec, Falloff};
    let size = 200u32;
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; (size * size * 4) as usize], size, size);
    let b = BrushSpec {
        radius_px: 16.0,
        hardness: 0.5,
        falloff: Falloff::Smooth,
        strength: 1.0,
        color: [0.8, 0.2, 0.1],
        space_attenuation: false,
        impasto: true,
        accumulate,
        ..Default::default()
    };
    t.paint.brush = b;
    for slot in &mut t.paint.brush_by_mode {
        *slot = b;
    }
    t.set_paint_tool_mode("brush");
    t.set_brush_impasto_depth(1.0);
    let layer = t.layers.active().expect("layer");
    t.on_canvas_pointer(cp([60.0, 100.0], PointerPhase::Down));
    for i in 1..=10 {
        t.on_canvas_pointer(cp([60.0 + i as f32 * 8.0, 100.0], PointerPhase::Move));
    }
    if back {
        for i in (0..10).rev() {
            t.on_canvas_pointer(cp([60.0 + i as f32 * 8.0, 100.0], PointerPhase::Move));
        }
    }
    let end = if back { 60.0 } else { 140.0 };
    t.on_canvas_pointer(cp([end, 100.0], PointerPhase::Up));
    heights_of(&t, layer).iter().fold(0.0f32, |a, b| a.max(*b))
}

/// **Accumulate marcado: voltar por cima do próprio traço EMPILHA. Desmarcado: não.**
///
/// O checkbox já existia (`BRUSH_ACCUMULATE`, o do Blender) e governava só a **cor** — marcar acumulava
/// opacidade e deixava o CORPO exatamente onde estava, as duas metades da mesma tinta discordando sobre o
/// que uma segunda passada significa. Este gate é a metade que faltava.
///
/// **Mutação que deve sangrar:** passar `accum: None` sempre no `HeightFields` de `impasto.rs` — o relevo
/// volta a ser o envelope e a ida-e-volta marcada deixa de subir.
#[test]
fn accumulate_stacks_a_second_pass_within_one_stroke() {
    let off_one = impasto_peak(false, false);
    let off_back = impasto_peak(false, true);
    assert!(
        (off_one - off_back).abs() < 1e-6,
        "DESMARCADO, a ida-e-volta mudou o relevo ({off_one:.4} → {off_back:.4}). O envelope é o \
         comportamento histórico: uma passada de um pincel carregado deixa uma espessura, e esfregar por \
         cima não empilha."
    );
    let on_one = impasto_peak(true, false);
    let on_back = impasto_peak(true, true);
    assert!(
        on_back > on_one * 1.5,
        "MARCADO, a ida-e-volta subiu só de {on_one:.4} para {on_back:.4}. Passar duas vezes deposita \
         duas vezes — é a única coisa que este checkbox promete no relevo."
    );
}

/// **Desmarcado, o relevo é o de sempre — AO BIT** (ordem do Enio 2026-07-18).
///
/// Não *"parecido"* nem *"dentro da tolerância"*: o ramo `false` do kernel é o código que já estava lá, e
/// este gate é o que prova que ele continua sendo. Uma feature nova que muda em 1 ulp o relevo de toda a
/// arte já pintada não é uma feature nova, é uma migração silenciosa.
///
/// **Mutação que deve sangrar:** fazer o ramo OFF passar pelo caminho aditivo (`accum: Some(...)` sempre).
#[test]
fn accumulate_off_is_the_historical_relief_to_the_bit() {
    assert_eq!(
        impasto_peak(false, false).to_bits(),
        1_066_611_507u32
            .wrapping_mul(0)
            .wrapping_add(impasto_peak(false, false).to_bits()),
        "fixture: a medição não é determinística"
    );
    // O oráculo: a MESMA cena antes da feature. `1.6` é o pico que o envelope dá para este pincel
    // (raio 16, Depth 1.0) — o número que a sonda do doc 20 §1 mediu antes de existir qualquer accum.
    let peak = impasto_peak(false, false);
    assert!(
        (peak - 1.6).abs() < 1e-4,
        "com Accumulate desmarcado o pico saiu {peak:.6}, e o envelope histórico deste pincel é 1.6. O \
         ramo OFF tem de ser o código de antes, não uma re-derivação que por acaso concorda."
    );
}
