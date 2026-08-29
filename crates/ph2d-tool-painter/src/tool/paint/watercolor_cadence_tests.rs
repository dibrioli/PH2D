//! **A cadência da lavagem: uma reconstrução por QUADRO, não uma por evento de ponteiro.**
//!
//! O caminho óptico da aquarela não deposita dabs: ele ACUMULA cobertura e cor e deixa
//! [`PainterTool::apply_watercolor`] reconstruir a lavagem inteira sobre a base congelada. A
//! reconstrução é uma **função pura** dessa base mais os acumuladores — e o doc dela sempre disse que
//! a cadência é o quadro (*"each frame recomposites"*, *"renderFrame"*, *"the frame dirty rect"*).
//!
//! Ela rodava **dentro de cada `PointerPhase::Move`**. Como a janela da lavagem é padeada pelo raio de
//! influência — do tamanho da pegada —, cada evento pagava uma passada quase cheia, e a mesma pintura
//! custava **2,56×** mais num dispositivo de 960 Hz que num de 120 Hz (medido em
//! `measure_whether_the_watercolor_charges_per_dab_or_per_event`; o Digital, no mesmo teste, é plano).
//! O artista comprava trabalho com a taxa do mouse dele.
//!
//! ⚠️ **A duplicação já tinha sido VISTA e curada do lado errado.** O comentário do *soak* em
//! `paint_tick` registra um profile de 2026-07-07 em que `stamps` e `tool-tick` carregavam cada um um
//! composite cheio; a correção de então suprimiu o do TICK — o único-por-quadro — sob a premissa de que
//! *"o flush de Move já recompôs a janela deste quadro"*. A premissa vale para os métodos COALESCIDOS
//! (uma entrega por quadro) e é falsa para o freehand incremental, que é o pincel de aquarela padrão.
//!
//! ## O que cada gate aqui prova
//!
//! 1. **A conta** — `WashCadence::composites` é 1 por quadro, não 1 por evento. Um CONTADOR e não um relógio:
//!    *uma vez por quadro* contra *uma vez por evento* é uma diferença numa CONTAGEM, e uma barra de
//!    tempo sobre passadas de ~1 ms mede o escalonador desta máquina, não o código.
//! 2. **O quadro：a lavagem está VIVA nele** — deferir não pode virar "aparece um quadro depois".
//! 3. **A figura é a mesma** — byte a byte, em qualquer taxa de polling. É esta que autoriza a
//!    deferição: se a contagem de reconstruções mudasse o resultado, não haveria o que deferir.
//!
//! A alavanca de mutação de todos eles é [`super::WashCadence::per_event`], a rota congelada.

// ⚠️ `super` aqui é `watercolor_field` (esta família de gates é filha do módulo que ela exercita,
// onde `WashCadence` mora), então o escopo de `paint` vem por caminho explícito.
use crate::tool::PainterTool;
use crate::tool::paint::media::PaintMedia;
use ph2d_editor_core::tool::{CanvasPaintTool, CanvasPointer, PointerPhase, RasterEditTool};
use ph2d_painter_brush::{BrushSpec, Falloff};

fn cp(pos: [f32; 2], phase: PointerPhase) -> CanvasPointer {
    CanvasPointer {
        pos,
        pressure: 1.0,
        tilt: [0.0, 0.0],
        phase,
    }
}

/// Um canvas branco armado em aquarela, com o pincel que o Reset entrega.
fn wash(size: u32, radius: f32) -> PainterTool {
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; (size * size * 4) as usize], size, size);
    let b = BrushSpec {
        radius_px: radius,
        hardness: 0.5,
        falloff: Falloff::Smooth,
        strength: 1.0,
        color: [0.8, 0.2, 0.1],
        space_attenuation: false,
        ..Default::default()
    };
    t.paint.brush = b;
    t.paint.brush_by_mode.fill(b);
    t.set_paint_media(PaintMedia::Watercolor);
    t
}

/// Entrega `ev_per_frame` Moves e fecha o quadro, `frames` vezes — o laço do produto
/// (`render_loop` ~698 e depois ~1198).
fn drive(
    t: &mut PainterTool,
    size: u32,
    radius: f32,
    path_px: f32,
    frames: u32,
    ev_per_frame: u32,
) {
    let mid = f64::from(size / 2) as f32;
    let x0 = radius + 10.0;
    let n = frames * ev_per_frame;
    let step = path_px / f64::from(n) as f32;
    t.on_canvas_pointer(cp([x0, mid], PointerPhase::Down));
    let mut k = 0u32;
    for _ in 0..frames {
        for _ in 0..ev_per_frame {
            k += 1;
            t.on_canvas_pointer(cp(
                [x0 + step * f64::from(k) as f32, mid],
                PointerPhase::Move,
            ));
        }
        t.paint_tick(1.0 / 60.0);
    }
}

/// **A lavagem reconstrói uma vez por QUADRO, não uma por evento.**
///
/// Dezesseis Moves num quadro (um mouse de 960 Hz a 60 fps) têm de produzir UM composite. A rota
/// congelada, no mesmo teste, produz um por evento — é ela que faz este gate ir VERMELHO.
#[test]
fn the_wash_composites_once_per_frame_not_once_per_pointer_event() {
    const SIZE: u32 = 512;
    const RADIUS: f32 = 40.0;
    const FRAMES: u32 = 4;
    const EV: u32 = 16;

    let mut t = wash(SIZE, RADIUS);
    let before = t.wash.composites;
    drive(&mut t, SIZE, RADIUS, 300.0, FRAMES, EV);
    let per_frame = t.wash.composites - before;

    // O pen-down congela o chão e compõe uma vez; o que este gate mede são os QUADROS depois dele.
    assert!(
        per_frame <= FRAMES + 1,
        "a lavagem compôs {per_frame} vezes em {FRAMES} quadros de {EV} eventos — a reconstrução \
         voltou a ser por evento (o teto é um por quadro, mais o composite do pen-down)"
    );

    // A mesma cena pela rota congelada: um composite por EVENTO. Sem esta metade o gate passaria
    // com a reconstrução deletada — ela é o controle positivo de que a cena de fato compõe.
    let mut b = wash(SIZE, RADIUS);
    let before = b.wash.composites;
    b.wash.per_event = true;
    drive(&mut b, SIZE, RADIUS, 300.0, FRAMES, EV);
    let per_event = b.wash.composites - before;
    assert!(
        per_event > per_frame * 3,
        "a rota congelada tem de compor MUITO mais ({per_event} contra {per_frame}) — se ela não \
         compõe, este gate não está medindo cadência nenhuma"
    );
}

/// **Deferir não é atrasar: a lavagem está viva NO quadro em que o artista pintou.**
///
/// O tick roda depois do flush de ponteiro (~698) e antes do upload do preview (~3397), então o quadro
/// que recebeu os Moves é o quadro que mostra a tinta. Um `paint_tick` que não pagasse a dívida
/// deixaria o canvas branco aqui — que é exactamente o modo de falha desta wave.
#[test]
fn the_frame_that_took_the_moves_is_the_frame_that_shows_the_wash() {
    const SIZE: u32 = 256;
    const RADIUS: f32 = 20.0;
    let mut t = wash(SIZE, RADIUS);
    let mid = f64::from(SIZE / 2) as f32;

    t.on_canvas_pointer(cp([RADIUS + 10.0, mid], PointerPhase::Down));
    for i in 1..=8 {
        let x = RADIUS + 10.0 + f64::from(i) as f32 * 8.0;
        t.on_canvas_pointer(cp([x, mid], PointerPhase::Move));
    }
    // Ainda com a caneta em BAIXO: um único tick, e a tinta tem de estar na tela.
    t.paint_tick(1.0 / 60.0);

    let i = ((SIZE / 2 * SIZE + (RADIUS as u32 + 40)) * 4) as usize;
    let px = [t.canvas_rgba[i], t.canvas_rgba[i + 1], t.canvas_rgba[i + 2]];
    assert_ne!(
        px,
        [255, 255, 255],
        "o quadro que recebeu os Moves não mostrou a lavagem — a dívida não foi paga pelo tick"
    );
}

/// **A figura é a mesma em qualquer taxa de polling** — a propriedade que AUTORIZA a deferição.
///
/// Se o número de reconstruções mudasse o resultado, não haveria o que deferir; e o defeito seria pior
/// que lentidão, porque a aparência da aquarela dependeria do hardware do artista. Aqui o mesmo
/// caminho é entregue em 1, 2, 4 e 8 eventos por quadro e as telas têm de sair **byte a byte iguais**.
///
/// ⚠️ Vale para as DUAS rotas de propósito: é uma afirmação sobre a pureza da reconstrução, não sobre
/// a cadência dela — por isso o gate acima é que prova a cadência, e este prova que ela é livre.
#[test]
fn the_wash_is_the_same_picture_at_any_polling_rate() {
    const SIZE: u32 = 256;
    const RADIUS: f32 = 24.0;
    const PATH: f32 = 160.0;
    const FRAMES: u32 = 8;

    let picture = |ev: u32| {
        let mut t = wash(SIZE, RADIUS);
        drive(&mut t, SIZE, RADIUS, PATH, FRAMES, ev);
        let mid = f64::from(SIZE / 2) as f32;
        t.on_canvas_pointer(cp([RADIUS + 10.0 + PATH, mid], PointerPhase::Up));
        t.canvas_rgba.to_vec()
    };

    let reference = picture(1);
    assert!(
        reference.iter().any(|&b| b != 255),
        "a cena tem de PINTAR — comparar duas telas brancas é verde por vácuo"
    );
    for ev in [2u32, 4, 8] {
        let got = picture(ev);
        let diff = reference
            .iter()
            .zip(got.iter())
            .filter(|(a, b)| a != b)
            .count();
        assert_eq!(
            diff, 0,
            "{ev} eventos por quadro pintaram um quadro DIFERENTE de 1 por quadro ({diff} bytes) — \
             a aparência da aquarela não pode depender da taxa do dispositivo"
        );
    }
}
