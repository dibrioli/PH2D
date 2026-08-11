//! **O POOL DOS CINCO PLANOS escreve o mesmo byte que a alocação** — o gate da cura do pen-down.
//!
//! O pen-down do filme custava **5,6 ms a 2048² e 5,7 a 4096²**: plano na tela e plano no raio, o que
//! exclui a cópia do canvas e o primeiro dab. Quem custa é o ALOCADOR — cada traço pedia os cinco
//! planos canvas-shaped de novo (**56 dos 83 MB por traço**, contados pelo `dhat` em
//! `tests/measure_pendown_alloc.rs`), porque o `reset_stroke_height` faz `clear()` e o primeiro dab do
//! traço seguinte joga a capacidade fora.
//!
//! Agora eles CIRCULAM, zerados só na janela em que ficaram sujos. O argumento de identidade é a
//! janela — um traço só escreve dentro do próprio `stroke_relief_bbox` — e um argumento não é uma
//! prova: estes gates comparam as duas rotas byte a byte.

use crate::tool::PainterTool;
use ph2d_editor_core::tool::{CanvasPaintTool, PointerPhase, RasterEditTool};
use ph2d_painter_brush::TextureKind;

const SIDE: u32 = 512;

fn tool(pool_off: bool) -> PainterTool {
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; (SIDE * SIDE * 4) as usize], SIDE, SIDE);
    t.set_brush_shape_kind(TextureKind::Stripes as u8);
    t.set_shape_relief(1.0);
    t.set_brush_size_px(40.0);
    t.paint.relief.planes_pool_off = pool_off;
    t
}

/// Seis traços que **se sobrepõem parcialmente e pousam em sítios diferentes** — as duas metades são
/// necessárias: sem sobreposição as janelas não se cruzam e um plano sujo nunca é lido; sem deslocação
/// todo traço reescreve exactamente o que o anterior escreveu e a sujidade fica invisível.
fn six_strokes(t: &mut PainterTool) {
    for k in 0..6u8 {
        one_stroke(t, k);
    }
}

fn one_stroke(t: &mut PainterTool, k: u8) {
    let y = 120.0 + f32::from(k) * 14.0;
    let x0 = 100.0 + f32::from(k % 3) * 30.0;
    t.on_canvas_pointer(cp([x0, y], PointerPhase::Down));
    for i in 1..=6u8 {
        t.on_canvas_pointer(cp([x0 + f32::from(i) * 30.0, y], PointerPhase::Move));
    }
    t.on_canvas_pointer(cp([x0 + 180.0, y], PointerPhase::Up));
}

fn cp(pos: [f32; 2], phase: PointerPhase) -> ph2d_editor_core::tool::CanvasPointer {
    ph2d_editor_core::tool::CanvasPointer {
        pos,
        pressure: 1.0,
        tilt: [0.0, 0.0],
        phase,
    }
}

/// **A rota do pool é byte-idêntica à da alocação, nos três planos que a camada guarda.**
///
/// **Mutação que tem de sangrar:** não zerar a janela no reuso (`take_planes` devolver os planos como
/// os recebeu) — o traço seguinte herda a carga do anterior e o relevo commitado diverge.
#[test]
fn the_pooled_planes_write_what_the_freshly_allocated_ones_write() {
    let (mut a, mut b) = (tool(true), tool(false));
    six_strokes(&mut a);
    six_strokes(&mut b);

    // CONTROLE 1: o caminho rápido DISPAROU. Um pool que nunca acerta faz das duas rotas a mesma, e
    // este gate passaria por vácuo enquanto o produto continuava a alocar (ADR-0120).
    assert_eq!(
        a.paint.relief.planes_pooled, 0,
        "a rota de controle usou o pool"
    );
    assert!(
        b.paint.relief.planes_pooled >= 4,
        "o pool serviu {} vezes em 6 tracos — o gate compararia alocacao com alocacao",
        b.paint.relief.planes_pooled
    );
    // CONTROLE 2: houve relevo. Dois mapas vazios sao trivialmente iguais.
    let laid = a
        .heights
        .values()
        .flat_map(|v| v.iter())
        .filter(|h| **h > 0.0)
        .count();
    assert!(
        laid > 5_000,
        "a fixture nao depositou relevo: {laid} texels"
    );

    assert_eq!(
        a.heights.len(),
        b.heights.len(),
        "as duas rotas commitaram numeros de camadas diferentes"
    );
    for (id, ha) in &a.heights {
        let hb = b.heights.get(id).expect("a camada existe nas duas rotas");
        assert_eq!(ha.as_ref(), hb.as_ref(), "o plano de ALTURA divergiu");
    }
    for (id, ca) in &a.covers {
        let cb = b.covers.get(id).expect("a camada existe nas duas rotas");
        assert_eq!(ca.as_ref(), cb.as_ref(), "o plano de COBERTURA divergiu");
    }
    for (id, ma) in &a.mats {
        let mb = b.mats.get(id).expect("a camada existe nas duas rotas");
        assert_eq!(ma.as_ref(), mb.as_ref(), "o plano de MATERIAL divergiu");
    }
    // E os PIXELS: o filme é uma aparência, então o canvas é o oráculo que o artista de facto vê.
    assert_eq!(
        a.canvas_rgba.as_ref(),
        b.canvas_rgba.as_ref(),
        "o CANVAS divergiu entre as duas rotas"
    );
}

/// **A janela guardada cobre as DUAS eras** — o `height`/`film` deste traço e os três ingredientes do
/// anterior, que o commit acabou de aposentar dos `live_*`.
///
/// ⚠️ **A primeira versão deste gate afirmava a coisa errada, e a mutação provou-o:** ela pedia que a
/// janela guardada contivesse a do ÚLTIMO traço — o que a mutação *"guarda só o `rect` deste"* satisfaz
/// **exactamente**, porque nesse caso as duas são a mesma. Um gate que não pode falhar pelo motivo que
/// alega é verde por construção; quem apanhou a mutação foi o gate de identidade ao lado. A pergunta
/// certa é sobre a era ANTERIOR, e é preciso guardá-la para a poder fazer.
///
/// **Mutação que tem de sangrar:** `dirty = Some(rect)` (largar a união com o `old_rect`).
#[test]
fn the_retired_window_covers_both_eras() {
    let mut t = tool(false);
    // Cinco traços, e a janela do quinto guardada ANTES de o sexto a substituir.
    for k in 0..5u8 {
        one_stroke(&mut t, k);
    }
    let fifth = t
        .paint
        .relief
        .live_relief_rect
        .expect("o quinto traco publicou a sua janela");
    one_stroke(&mut t, 5);
    let dirty = t
        .paint
        .relief
        .spare
        .dirty
        .expect("o pool guardou uma janela");
    let sixth = t
        .paint
        .relief
        .live_relief_rect
        .expect("o sexto traco publicou a sua janela");
    // ⚠️ CONTROLE: as duas janelas TÊM de ser diferentes, senão a união é trivial e o gate não afirma
    // nada — é a mesma razão por que os traços da fixture andam de sítio.
    assert_ne!(
        fifth, sixth,
        "a fixture nao mudou de janela entre os dois tracos"
    );
    for (name, r) in [("quinto", fifth), ("sexto", sixth)] {
        assert!(
            dirty.x <= r.x
                && dirty.y <= r.y
                && dirty.x + dirty.w >= r.x + r.w
                && dirty.y + dirty.h >= r.y + r.h,
            "a janela do pool ({dirty:?}) nao contem a do {name} traco ({r:?})"
        );
    }
}
