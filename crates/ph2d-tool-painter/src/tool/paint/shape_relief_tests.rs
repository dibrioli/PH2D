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

/// **O carimbo fica como o artista o largou: o pen-up NÃO assenta um relevo que veio de uma IMAGEM.**
///
/// Report do Enio (2026-08-09, na cena do Grid Stamp): *"no mouse down fica exatamente como deveria
/// ficar, mas no mouse UP sofre uma mudança (provavelmente o smooth ou outro algoritmo de ajuste de
/// mouse up) … a imagem deve ficar como no mouse down"*.
///
/// MEDIDO antes da correção, num carimbo de célula: o pen-down deixava **4032 texels, pico 3,162** e o
/// pen-up devolvia **4624, pico 2,962** — **1264 texels mexidos, pior |Δh| = 2,217** sobre um pico de
/// 3,16, ou seja **70% da altura**. Com o `impasto_smoothing` em zero o commit já saía byte-idêntico
/// ao vivo, então o assentamento era a diferença INTEIRA.
///
/// ⚠️ **O oráculo é o campo que a LUZ lê**, texel a texel — não uma contagem nem um pico, que um blur
/// pode preservar por acaso (a soma aqui muda 0,05%: um gate de massa ficaria verde sobre o defeito).
///
/// **Mutação que tem de sangrar:** tirar o `!self.stamps_captured_relief()` do settle em
/// [`super::impasto_live`].
#[test]
fn a_travelling_relief_lands_as_authored_and_the_pen_up_does_not_settle_it() {
    let size = 96u32;
    let src = sculpted_source(size);
    let mut t = digital_stamper(&src, size);
    t.set_brush_stroke_method(ph2d_painter_brush::StrokeMethod::GridStamp.to_u8());
    t.set_grid_cell_px(super::grid_stamp_settings::GridAxis::X, 48.0);
    t.set_grid_cell_px(super::grid_stamp_settings::GridAxis::Y, 48.0);
    let at = |phase| CanvasPointer {
        pos: [24.0, 24.0],
        pressure: 1.0,
        tilt: [0.0, 0.0],
        phase,
    };
    t.on_canvas_pointer(at(PointerPhase::Down));
    let live = height_field(&t, size);
    assert!(
        live.iter().any(|&h| h > 1e-4),
        "premissa: o pen-down já deixou relevo — sem ele o gate compara dois campos vazios"
    );
    t.on_canvas_pointer(at(PointerPhase::Up));
    let committed = height_field(&t, size);
    let worst = live
        .iter()
        .zip(committed.iter())
        .map(|(a, b)| (a - b).abs())
        .fold(0.0f32, f32::max);
    let moved = live
        .iter()
        .zip(committed.iter())
        .filter(|(a, b)| (*a - *b).abs() > 1e-6)
        .count();
    assert_eq!(
        (moved, worst),
        (0, 0.0),
        "o pen-up mexeu em {moved} texels (pior {worst}) — o carimbo tem de ficar como no mouse down"
    );
}

/// **E o relevo DEPOSITADO continua assentando** — a metade sem a qual o gate acima passaria com o
/// settle apagado do produto.
///
/// O assentamento é tinta molhada relaxando sob o próprio peso, e um depósito É tinta molhada. O que
/// a wave remove é aplicá-lo a uma escultura CAPTURADA, que já secou noutro documento.
///
/// ⚠️ **O oráculo é o KNOB, e a 1ª versão deste gate estava errada:** ela media *"o pen-up mudou
/// alguma coisa"*, que é verdade por outros motivos (o commit re-deriva o campo inteiro e soma o
/// chão de volta) — e por isso **sobreviveu** à mutação que apaga o `settle`. O que prova que o
/// assentamento rodou é o mesmo traço commitado com Smoothing **0** e com Smoothing **1** dar campos
/// DIFERENTES.
///
/// **Mutação que tem de sangrar:** apagar a chamada ao `settle` (a lei nova ficaria verde sozinha).
#[test]
fn a_deposited_relief_still_settles_at_pen_up() {
    let size = 96u32;
    let committed = |smoothing: f32| {
        let mut t = PainterTool::default();
        t.set_source(vec![255u8; (size * size * 4) as usize], size, size);
        t.toggle_brush_impasto();
        t.set_brush_size_px(18.0);
        t.paint.brush.color = [0.7, 0.3, 0.2];
        t.paint.brush.strength = 1.0;
        t.paint.brush.impasto_smoothing = smoothing;
        let mid = size as f32 * 0.5;
        let at = |x: f32, phase| CanvasPointer {
            pos: [x, mid],
            pressure: 1.0,
            tilt: [0.0, 0.0],
            phase,
        };
        t.on_canvas_pointer(at(size as f32 * 0.3, PointerPhase::Down));
        t.on_canvas_pointer(at(size as f32 * 0.7, PointerPhase::Move));
        t.on_canvas_pointer(at(size as f32 * 0.7, PointerPhase::Up));
        height_field(&t, size)
    };
    let smoothed = committed(1.0);
    let raw = committed(0.0);
    assert!(
        raw.iter().any(|&h| h > 1e-4),
        "premissa: o traço deixou relevo — sem ele o gate compara dois campos vazios"
    );
    let moved = smoothed
        .iter()
        .zip(raw.iter())
        .filter(|(a, b)| (*a - *b).abs() > 1e-6)
        .count();
    assert!(
        moved > 100,
        "o Smoothing não mexeu no relevo commitado ({moved} texels) — o assentamento do DEPÓSITO          deixou de rodar"
    );
}

/// **O preview do painel mostra o documento COMO ELE SE VÊ — a viagem do relevo não o achata.**
///
/// Report do Enio (2026-08-09): *"o preview do painel mostra a imagem chapada e não com o aspecto do
/// impasto"*. A causa foi a 1ª versão desta wave: ao mandar a cor pristina para o carimbo ela apagou
/// o ganho do plano que o preview fotografa, e a face do slot perdeu a sombra da escultura.
///
/// ⚠️ **O oráculo é a INVARIÂNCIA:** a aparência do slot não é função de o relevo viajar ou não — é
/// função do que foi capturado. Um gate que só pedisse *"o preview não é chapado"* teria de nomear um
/// número de contraste; este compara o produto contra ele mesmo com o interruptor virado.
///
/// **Mutação que tem de sangrar:** o preview ler a `rgb_image` (a porta do CARIMBO) em vez da
/// `rgb_image_shown`.
#[test]
fn the_panel_preview_shows_the_captured_document_however_the_relief_travels() {
    let size = 96u32;
    let src = sculpted_source(size);
    let (relief, _) = src.captured_relief().expect("a fonte tem escultura");
    let gain = src
        .relief_shade_gain()
        .expect("a fonte tem relevo, logo tem ganho");
    let n = (size as usize) * (size as usize);
    let mut rgba = vec![210u8; n * 4];
    for p in rgba.chunks_exact_mut(4) {
        p[3] = 255;
    }
    let preview = |travels: bool| {
        let mut t = PainterTool::default();
        t.set_source(vec![255u8; n * 4], size, size);
        t.set_brush_shape_image_rgba(&rgba, size, size, Some(11));
        t.paint.shape_layers.set_gain(gain.clone());
        t.paint.shape_layers.set_relief(relief.clone());
        t.paint.shape_layers.set_relief_from_image(travels);
        if !t.paint.shape_layers.per_layer_color() {
            t.toggle_brush_shape_per_layer_color();
        }
        assert_eq!(
            travels,
            t.paint.shape_layers.relief_travels(),
            "premissa: o interruptor da viagem do relevo é o que esta chamada diz"
        );
        t.refresh_shape_color_preview();
        t.shape_color_preview()
            .map(|(px, _, _)| px.to_vec())
            .unwrap_or_default()
    };
    let with_travel = preview(true);
    let without = preview(false);
    assert!(
        !with_travel.is_empty() && with_travel.len() == without.len(),
        "premissa: os dois previews existem ({} e {} bytes)",
        with_travel.len(),
        without.len()
    );
    let differing = with_travel
        .iter()
        .zip(without.iter())
        .filter(|(a, b)| a != b)
        .count();
    assert_eq!(
        differing, 0,
        "o preview mudou em {differing} bytes só porque o relevo passou a viajar — ele fotografa o \
         que foi CAPTURADO, e o que o carimbo pinta é outra pergunta"
    );
}

/// O campo de altura que a LUZ lê, texel a texel — o oráculo dos dois gates de assentamento.
fn height_field(t: &PainterTool, size: u32) -> Vec<f32> {
    let mut v = vec![0.0f32; (size as usize) * (size as usize)];
    if let Some(f) = t.impasto_fields() {
        for y in 0..i64::from(size) {
            for x in 0..i64::from(size) {
                v[(y as usize) * (size as usize) + x as usize] = f.height_at(x, y);
            }
        }
    }
    v
}
