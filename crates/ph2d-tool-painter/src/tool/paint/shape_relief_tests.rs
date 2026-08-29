//! **A escultura entra na Shape COZIDA** — o slot guarda o que o documento MOSTRA, e o carimbo não
//! deposita relevo nenhum.
//!
//! Ordem do Enio (2026-08-09, depois de smokar o relevo vivo): *"não ficou bom, embora funcione.
//! Vamos fazer o seguinte: a imagem com impasto/relevo, quando colocada em Shape, produz em shape uma
//! versão Cozida (Bake Perfeito) mas sem relevo real"*.
//!
//! ⚠️ **A capacidade de o relevo VIAJAR foi construída, smokada e RETIRADA:** a forma capturada era
//! depositada como altura de verdade e a luz do documento de destino a sombreava. O veredito foi de
//! PRODUTO, não de mecanismo — ela funcionava. Sobrou o que o Enio pediu: um bake, e um bake EXATO.
//!
//! ⚠️ **E "exato" é um número, não um adjetivo:** a versão anterior guardava a cor CRUA e um ganho de
//! LUMINÂNCIA, multiplicados na derivação. MEDIDO contra o que o artista vê, ela errava **96 níveis
//! de 255** com material neutro e **98** com cera âmbar — um escalar não carrega a COR do especular
//! nem a da cera, e satura no teto. Hoje a captura roda o passe CANÔNICO da luz sobre os pixels.

use crate::PainterTool;
use ph2d_editor_core::tool::{CanvasPaintTool, CanvasPointer, PointerPhase, RasterEditTool};

/// Um documento com uma CRISTA de tinta esculpida — a fonte que se captura.
fn sculpted_source(size: u32) -> PainterTool {
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; (size * size * 4) as usize], size, size);
    t.toggle_brush_impasto();
    t.set_brush_size_px(size as f32 * 0.35);
    t.paint.brush.color = [0.8, 0.2, 0.2];
    t.paint.brush.strength = 1.0;
    // Um material com COR: é ele que separa um bake exato de um ganho escalar — a cera tinge o
    // realce, e uma luminância não tem onde guardar isso.
    t.paint.brush.impasto_wax = 0.9;
    t.paint.brush.impasto_wax_color = [1.0, 0.6, 0.2];
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
    t
}

/// Os pixels que o artista VÊ neste documento: o canvas sob o passe canônico da luz.
fn as_seen(t: &PainterTool) -> Vec<u8> {
    let mut px = t.canvas_rgba.to_vec();
    let (w, h) = t.source_size;
    t.apply_impasto_light(&mut px, super::Region { x: 0, y: 0, w, h });
    px
}

/// O campo de altura que a LUZ lê, texel a texel.
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

/// **O BAKE É EXATO: o que o slot guarda é o que o documento mostra, AO BYTE.**
///
/// ⚠️ **O oráculo é o produto contra o produto** — a cor capturada contra o canvas de origem sob o
/// MESMO `apply_impasto_light` que a tela usa. Ele não conhece ganho, nem albedo de sonda, nem
/// luminância: é isso que o torna oráculo em vez de espelho de uma fórmula.
///
/// **Mutação que tem de sangrar:** a captura guardar o RGB cru (o desenho anterior) — MEDIDO, ela
/// erra **136 níveis** nesta fixture.
#[test]
fn the_bake_is_exactly_what_the_document_shows() {
    let size = 96u32;
    let mut src = sculpted_source(size);
    let seen = as_seen(&src);
    src.capture_layers_as_brush_shape();
    let baked = src
        .paint
        .shape_layers
        .rgb_image(0)
        .map(|r| r.rgb.to_vec())
        .expect("a captura de uma camada com cor produz cor");
    let n = (size as usize) * (size as usize);
    assert_eq!(baked.len(), n * 3, "premissa: a captura mede o documento");
    let mut worst = 0i32;
    let mut painted = 0usize;
    for i in 0..n {
        if seen[i * 4 + 3] == 0 {
            continue;
        }
        painted += 1;
        for c in 0..3 {
            worst = worst.max((i32::from(seen[i * 4 + c]) - i32::from(baked[i * 3 + c])).abs());
        }
    }
    assert!(
        painted > 500,
        "premissa: a fonte tem tinta ({painted} texels) — sem ela o gate compara papel"
    );
    assert_eq!(
        worst, 0,
        "a cor capturada difere em {worst} níveis do que o documento MOSTRA — o bake não é o bake"
    );
}

/// **E NENHUM RELEVO VIAJA:** carimbar essa captura deixa o documento de destino PLANO.
///
/// É a metade que a ordem do Enio nomeia (*"mas sem relevo real"*), e ela é sobre o que o carimbo
/// NÃO faz — o tipo de coisa que só um gate impede de voltar por acidente.
///
/// ⚠️ **A ausência é guardada TRÊS vezes, e a mutação teve de derrubar as três** — o que este gate
/// mediu ao ser escrito: tirar o mestre do `impasto_batch_active` **não sangra** (o veto do
/// `touches_height` segura), tirar os dois também **não** (o `effective_impasto_depth` devolve zero
/// com o mestre desligado, e a altura derivada é zero). Só com as três caídas — isto é, com a
/// capacidade retirada de fato **re-introduzida** — o gate fica vermelho.
///
/// Isso é o que ele vale e o que ele não vale: ele não pega um deslize de uma linha em nenhuma das
/// camadas (cada uma tem o seu próprio gate), e pega a coisa que o Enio recusou voltando inteira.
///
/// **Mutação que tem de sangrar:** `impasto_batch_active` sem o mestre **+** o veto do
/// `touches_height` removido **+** `effective_impasto_depth` ignorando o mestre.
#[test]
fn stamping_a_sculpted_shape_deposits_no_relief_at_all() {
    let size = 96u32;
    let mut src = sculpted_source(size);
    src.capture_layers_as_brush_shape();
    let (lum, w, h) = {
        let img = src.brush_shape_image().expect("a fonte virou silhueta");
        (img.0.to_vec(), img.1, img.2)
    };

    let mut t = PainterTool::default();
    t.set_source(vec![255u8; (size * size * 4) as usize], size, size);
    t.set_brush_shape_image(lum, w, h);
    t.set_brush_size_px(size as f32 * 0.4);
    t.paint.brush.color = [0.2, 0.4, 0.9];
    t.paint.brush.strength = 1.0;
    assert!(
        !t.paint.brush.impasto,
        "premissa: o carimbo é DIGITAL — é onde o relevo vivo entrava"
    );
    let at = |phase| CanvasPointer {
        pos: [size as f32 * 0.5, size as f32 * 0.5],
        pressure: 1.0,
        tilt: [0.0, 0.0],
        phase,
    };
    t.on_canvas_pointer(at(PointerPhase::Down));
    t.on_canvas_pointer(at(PointerPhase::Up));

    assert!(
        t.canvas_rgba.iter().any(|&b| b != 255),
        "premissa: o carimbo pintou — um gate de relevo zero sobre tela nua é verde por vácuo"
    );
    let relief = height_field(&t, size);
    let raised = relief.iter().filter(|&&h| h.abs() > 1e-4).count();
    assert_eq!(
        raised, 0,
        "o carimbo deixou {raised} texels de relevo — a Shape entrega um BAKE, não corpo"
    );
}

/// **O CONTROLE: um documento sem escultura é capturado byte-idêntico ao que já shipava.**
///
/// O passe de luz é RELATIVO (divide pela resposta de uma superfície plana), então sem relevo ele
/// multiplica por exatamente 1 — e é isso que faz do bake uma correção em vez de uma mudança de
/// aparência para toda arte já feita. Sem este gate, o de cima ficaria verde sobre um passe que mexe
/// em tudo.
#[test]
fn a_document_without_sculpture_is_captured_untouched() {
    let size = 64u32;
    let n = (size as usize) * (size as usize);
    let mut t = PainterTool::default();
    let mut src = vec![255u8; n * 4];
    for (i, px) in src.as_chunks_mut::<4>().0.iter_mut().enumerate() {
        px[0] = (i % 251) as u8;
        px[1] = (i % 199) as u8;
        px[2] = (i % 173) as u8;
    }
    t.set_source(src.clone(), size, size);
    t.capture_layers_as_brush_shape();
    let baked = t
        .paint
        .shape_layers
        .rgb_image(0)
        .map(|r| r.rgb.to_vec())
        .expect("a captura produz cor");
    let mut worst = 0i32;
    for i in 0..n {
        for c in 0..3 {
            worst = worst.max((i32::from(src[i * 4 + c]) - i32::from(baked[i * 3 + c])).abs());
        }
    }
    assert_eq!(
        worst, 0,
        "um documento sem relevo foi capturado {worst} níveis diferente de si mesmo"
    );
}
