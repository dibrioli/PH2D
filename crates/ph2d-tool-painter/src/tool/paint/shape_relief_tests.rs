//! **O relevo VIAJA com a Shape** — o carimbo de uma imagem esculpida reage à luz do documento de
//! destino, mesmo pintando em Digital.
//!
//! Report do Enio (2026-08-09, depois de a colocação fechar): *"não temos o relevo e o brilho e a
//! reação à luz de uma imagem criada com Impasto. Tente implementar essa capacidade mesmo pintando
//! com o Digital, desde que usando uma imagem em Shape com relevo pintada com impasto"*.
//!
//! ⚠️ **O que shipava era a APARÊNCIA, e a diferença é observável:** a captura levava o *ganho* — como
//! a luz do documento de ORIGEM sombreou aquele relevo —, um número já assado. O carimbo saía uma
//! fotografia de tinta esculpida: não tinha altura, não tinha especular, e mover a lâmpada não mudava
//! nada. Agora a captura leva a **forma**, e quem sombreia é a luz do destino.

use crate::PainterTool;
use ph2d_editor_core::tool::{CanvasPaintTool, CanvasPointer, PointerPhase, RasterEditTool};

/// Um documento com uma CRISTA de tinta esculpida, pronto para ser capturado como Shape.
fn sculpted_source(size: u32) -> PainterTool {
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; (size * size * 4) as usize], size, size);
    t.toggle_brush_impasto();
    t.set_brush_size_px(size as f32 * 0.35);
    t.paint.brush.color = [0.8, 0.2, 0.2];
    t.paint.brush.strength = 1.0;
    let mid = size as f32 * 0.5;
    let at = |x: f32, phase| CanvasPointer {
        pos: [x, mid],
        pressure: 1.0,
        tilt: [0.0, 0.0],
        phase,
    };
    t.on_canvas_pointer(at(size as f32 * 0.25, PointerPhase::Down));
    t.on_canvas_pointer(at(size as f32 * 0.5, PointerPhase::Move));
    t.on_canvas_pointer(at(size as f32 * 0.75, PointerPhase::Up));
    // ⚠️ E CAPTURA — pela porta do produto. Sem isto a fonte tem escultura e nenhuma Shape, e os
    // gates abaixo mediriam uma fixture que não contém o gesto que o artista faz.
    t.capture_layers_as_brush_shape();
    t
}

/// Um tool em **DIGITAL** (impasto nunca armado) com a Shape já capturada instalada.
fn digital_stamper(src: &PainterTool, size: u32) -> PainterTool {
    let (relief, _) = src.captured_relief().expect("a fonte tem escultura");
    let (lum, w, h) = {
        let img = src.brush_shape_image().expect("a fonte virou silhueta");
        (img.0.to_vec(), img.1, img.2)
    };
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; (size * size * 4) as usize], size, size);
    t.set_brush_shape_image(lum, w, h);
    t.paint.shape_layers.set_relief(relief);
    t.set_brush_size_px(size as f32 * 0.4);
    t.paint.brush.color = [0.2, 0.4, 0.9];
    t.paint.brush.strength = 1.0;
    assert!(
        !t.paint.brush.impasto,
        "premissa: o carimbo é DIGITAL — o pedido é justamente que o relevo não dependa do modo"
    );
    t
}

fn tap(t: &mut PainterTool, p: [f32; 2]) {
    let at = |phase| CanvasPointer {
        pos: p,
        pressure: 1.0,
        tilt: [0.0, 0.0],
        phase,
    };
    t.on_canvas_pointer(at(PointerPhase::Down));
    t.on_canvas_pointer(at(PointerPhase::Up));
}

/// Quanto relevo existe na camada ativa, e quão alto ele chega.
fn relief_of(t: &PainterTool) -> (usize, f32) {
    let mut n = 0usize;
    let mut peak = 0.0f32;
    for y in 0..i64::from(t.source_size.1) {
        for x in 0..i64::from(t.source_size.0) {
            let Some(f) = t.impasto_fields() else {
                return (0, 0.0);
            };
            let v = f.height_at(x, y);
            if v > 1e-4 {
                n += 1;
                peak = peak.max(v);
            }
        }
    }
    (n, peak)
}

/// **Carimbar em Digital uma Shape esculpida deposita RELEVO** — a capacidade pedida.
///
/// O oráculo é a altura que o documento de destino passa a ter, lida pela porta que a LUZ lê
/// (`ReliefFields::height_at`): é ela, e não um buffer, que decide se o carimbo vai brilhar.
///
/// **Mutação que tem de sangrar:** `stamps_captured_relief` devolver `false` (a rota volta a ser
/// gateada pelo mestre do Impasto, que em Digital está desligado).
#[test]
fn a_digital_stamp_of_a_sculpted_shape_lays_relief() {
    let size = 96u32;
    let src = sculpted_source(size);
    let (src_n, src_peak) = relief_of(&src);
    assert!(
        src_n > 200 && src_peak > 0.0,
        "premissa: a fonte tem escultura ({src_n} texels, pico {src_peak}) — sem ela a fixture não \
         contém o fenômeno e tudo abaixo é verde por vácuo"
    );

    let mut t = digital_stamper(&src, size);
    tap(&mut t, [size as f32 * 0.5, size as f32 * 0.5]);
    let (n, peak) = relief_of(&t);
    assert!(
        n > 200 && peak > 0.0,
        "o carimbo Digital não deixou relevo ({n} texels, pico {peak}) — a imagem tem escultura e o \
         documento de destino continua plano"
    );
}

/// **E o relevo carimbado REAGE À LUZ** — mover a lâmpada muda os pixels.
///
/// ⚠️ Esta é a metade que separa a capacidade nova da que já shipava: uma sombra ASSADA na cor
/// também escurece, e um gate que só medisse "ficou mais escuro" ficaria verde sobre ela. O que uma
/// aparência assada **não** faz é mudar quando a luz muda.
///
/// **Mutação que tem de sangrar:** a mesma — sem depósito de relevo, as duas luzes pintam igual.
#[test]
fn the_stamped_relief_answers_the_light() {
    let size = 96u32;
    let src = sculpted_source(size);
    let shot = |azimuth: f32| {
        let mut t = digital_stamper(&src, size);
        t.set_impasto_light_angle(azimuth);
        tap(&mut t, [size as f32 * 0.5, size as f32 * 0.5]);
        let mut px = t.canvas_rgba.to_vec();
        let (w, h) = t.source_size;
        t.apply_impasto_light(&mut px, super::Region { x: 0, y: 0, w, h });
        px
    };
    let a = shot(30.0);
    let b = shot(210.0);
    let differing = a.iter().zip(b.iter()).filter(|(x, y)| x != y).count();
    assert!(
        differing > 400,
        "mover a lâmpada de 30° para 210° mudou {differing} bytes — um carimbo com relevo de verdade \
         tem de responder à luz, e uma sombra assada na cor não responde a nada"
    );
}

/// ⚠️ **A sombra não pode viajar DUAS vezes** — a espinha da wave.
///
/// Com o relevo viajando, a cor do carimbo é a PRISTINA e quem sombreia é a luz do destino. Se a cor
/// também trouxesse o ganho assado do documento de origem, a mesma sombra pousaria duas vezes — e o
/// resultado seria plausível o bastante para passar por qualquer gate que só pedisse "está escuro".
///
/// **Mutação que tem de sangrar:** `rebuild_derived` usar o ganho mesmo quando o relevo viaja.
#[test]
fn the_shading_travels_once_the_relief_or_the_gain_never_both() {
    let size = 96u32;
    let src = sculpted_source(size);
    let (relief, _) = src.captured_relief().expect("a fonte tem escultura");

    // A MESMA captura, com e sem a viagem do relevo — só o interruptor muda. Os três planos são
    // instalados direto porque a LEI mora no `rebuild_derived`, e é ela que está sob teste.
    let n = (size as usize) * (size as usize);
    let mut rgba = vec![200u8; n * 4];
    for p in rgba.chunks_exact_mut(4) {
        p[3] = 255;
    }
    let gain = src
        .relief_shade_gain()
        .expect("a fonte tem relevo, logo tem ganho");
    let colour = |travels: bool| {
        let mut t = PainterTool::default();
        t.set_source(vec![255u8; n * 4], size, size);
        t.set_brush_shape_image_rgba(&rgba, size, size, Some(7));
        t.paint.shape_layers.set_gain(gain.clone());
        t.paint.shape_layers.set_relief(relief.clone());
        t.paint.shape_layers.set_relief_from_image(travels);
        t.paint
            .shape_layers
            .rgb_image(0)
            .map(|r| r.rgb.to_vec())
            .unwrap_or_default()
    };
    let pristine = colour(true);
    let baked = colour(false);
    assert!(
        !pristine.is_empty() && pristine.len() == baked.len(),
        "premissa: as duas capturas têm cor ({} e {} bytes)",
        pristine.len(),
        baked.len()
    );
    let differing = pristine
        .iter()
        .zip(baked.iter())
        .filter(|(x, y)| x != y)
        .count();
    assert!(
        differing > 100,
        "a cor é a MESMA com e sem a viagem do relevo ({differing} bytes diferem) — ou a sombra está \
         assada nas duas (e vai pousar duas vezes), ou não está em nenhuma"
    );
}
