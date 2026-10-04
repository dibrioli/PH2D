//! **Onde o meio não oferece o Accumulate, nem ele nem o Space Attenuation entram no traço**
//! (doc 46 §1 — a escolha delegada pelo dono, 2026-10-04).
//!
//! O censo mediu (doc 45 §2.1): no Wet Paint, com os dois ligados, o traço pintava `26` texels em vez
//! de `2 428` — a lei de alfa do Blender (`space_overlap_factor`, para tinta que ACUMULA) entrava
//! como intensidade da água, que não acumula alfa. O painel do Wet Paint deixa de os mostrar, mas o
//! *Sync with other tools* leva os interruptores de um slot para outro: o traço tem de os ignorar.

use super::super::media::PaintMedia;
use super::*;

/// Um traço de fábrica no meio `media`, com os dois interruptores como vieram de outro slot.
fn traco(media: PaintMedia, vindos_ligados: bool) -> Vec<u8> {
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; 96 * 96 * 4], 96, 96);
    t.set_paint_media(media);
    t.set_wet_relogio_fixo(true);
    t.paint.brush.accumulate = vindos_ligados;
    t.paint.brush.space_attenuation = vindos_ligados;
    t.on_canvas_pointer(cp([12.0, 48.0], PointerPhase::Down));
    let mut x = 12.0;
    while x < 84.0 {
        x += 2.0;
        t.on_canvas_pointer(cp([x, 48.0], PointerPhase::Move));
        frame(&mut t);
    }
    t.on_canvas_pointer(cp([x, 48.0], PointerPhase::Up));
    for _ in 0..10 {
        frame(&mut t);
    }
    t.canvas_rgba.as_ref().clone()
}

/// ⭐ **Nos meios que não oferecem o Accumulate, os dois interruptores não mudam um byte.**
///
/// **Mutação que sangra:** tirar a guarda do `authored_spec` (o Wet Paint volta a pintar `~1 %`).
#[test]
fn os_meios_sem_accumulate_ignoram_os_dois_interruptores() {
    for media in [
        PaintMedia::Watercolor,
        PaintMedia::Impasto,
        PaintMedia::WetPaint,
    ] {
        let limpo = traco(media, false);
        let vindo = traco(media, true);
        let diferentes = limpo
            .chunks_exact(4)
            .zip(vindo.chunks_exact(4))
            .filter(|(a, b)| a != b)
            .count();
        assert_eq!(
            diferentes, 0,
            "{media:?}: com Accumulate + Space Attenuation vindos de outro slot o traço mudou \
             ({diferentes} texels)"
        );
    }
}

/// O controlo positivo: no Digital os dois AGEM (o Accumulate deixa passar do Strength).
#[test]
fn no_digital_os_dois_interruptores_agem() {
    let mut a = PainterTool::default();
    a.set_source(vec![255u8; 96 * 96 * 4], 96, 96);
    a.paint.brush.strength = 0.4;
    let mut b = PainterTool::default();
    b.set_source(vec![255u8; 96 * 96 * 4], 96, 96);
    b.paint.brush.strength = 0.4;
    b.paint.brush.accumulate = true;
    for t in [&mut a, &mut b] {
        t.on_canvas_pointer(cp([12.0, 48.0], PointerPhase::Down));
        for k in 0..36 {
            t.on_canvas_pointer(cp([14.0 + k as f32 * 2.0, 48.0], PointerPhase::Move));
            frame(t);
        }
        t.on_canvas_pointer(cp([84.0, 48.0], PointerPhase::Up));
    }
    assert_ne!(
        a.canvas_rgba, b.canvas_rgba,
        "o Accumulate não age no Digital"
    );
}
