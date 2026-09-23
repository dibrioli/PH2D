//! Os gates da **composição por QUADRO** ([`super::composite_por_quadro`]) — report do dono de
//! 2026-09-23 (*«FPS cai para 1»*).
//!
//! A pilha do dono (a da foto: Blur em cima, três Brush, Smear e Erase), em escala: o pincel a
//! `12 px` e cada camada com o TAMANHO relativo da foto. ⚠️ A tela tem **ARTE por baixo** (degradê,
//! grelha, um disco): sem ela o esfregão e o borrão são inertes e a régua da imagem não conteria o
//! fenómeno que a composição adiada pode mudar.

use super::*;
use ph2d_editor_core::tool::RasterEditTool;

const SIZE: u32 = 256;
const RAIO: f32 = 12.0;
const Y: f32 = 128.0;
const X0: f32 = 40.0;
const X1: f32 = 216.0;

fn cp(pos: [f32; 2], phase: PointerPhase) -> CanvasPointer {
    CanvasPointer {
        pos,
        pressure: 1.0,
        tilt: [0.0, 0.0],
        phase,
    }
}

/// A arte por baixo: degradê, uma grelha escura a cada `16 px` e um disco colorido.
fn arte() -> Vec<u8> {
    let mut px = vec![0u8; (SIZE * SIZE * 4) as usize];
    for y in 0..SIZE {
        for x in 0..SIZE {
            let i = ((y * SIZE + x) * 4) as usize;
            let grelha = x % 16 == 0 || y % 16 == 0;
            let (dx, dy) = (x as f32 - 150.0, y as f32 - 120.0);
            let disco = dx * dx + dy * dy < 900.0;
            let rgb = if grelha {
                [30, 30, 40]
            } else if disco {
                [40, 160, 220]
            } else {
                [(x * 255 / SIZE) as u8, (y * 255 / SIZE) as u8, 190]
            };
            px[i..i + 3].copy_from_slice(&rgb);
            px[i + 3] = 255;
        }
    }
    px
}

/// A pilha da foto do dono, sobre a arte. `por_quadro` é o que o hospedeiro semeia.
fn pilha(por_quadro: bool) -> PainterTool {
    let mut t = PainterTool::default();
    t.set_source(arte(), SIZE, SIZE);
    t.paint.brush.radius_px = RAIO;
    t.paint.brush.strength = 1.0;
    t.paint.brush.space_attenuation = false;
    t.paint.composite_enabled = true;
    for pos in 0..composite::N_CAMADAS {
        t.paint.composite[pos].strength = 0.0;
    }
    let dono = [
        (CompositeOp::Blur, 1.0f32, 2.048f32, None),
        (CompositeOp::Brush, 0.133, 0.574, Some([1.0, 1.0, 1.0])),
        (CompositeOp::Brush, 0.204, 1.002, Some([1.0, 0.0, 0.0])),
        (CompositeOp::Brush, 0.176, 1.221, Some([0.0, 0.0, 0.0])),
        (CompositeOp::Smear, 0.596, 1.0, None),
        (CompositeOp::Erase, 0.104, 1.0, None),
    ];
    t.paint.composite_len = dono.len();
    for (i, &(op, strength, size, color)) in dono.iter().enumerate() {
        t.paint.composite[i] = CompositeLayer {
            op,
            strength,
            size,
            color,
            ..CompositeLayer::default()
        };
    }
    t.set_compor_por_quadro(por_quadro);
    t
}

/// Um rabisco (vai e volta sobre a própria vizinhança) em passos de `passo`, drenando a
/// pré-visualização a cada `por_quadro` eventos — o que a ponte do app faz uma vez por quadro.
/// `0` = nunca drena. Devolve a tela no fim.
fn rabisco(t: &mut PainterTool, passo: f32, por_quadro: usize) -> Vec<u8> {
    t.on_canvas_pointer(cp([X0, Y], PointerPhase::Down));
    let mut n = 0usize;
    let mut x = X0;
    while x < X1 {
        x += passo;
        let y = Y + 30.0 * (x / 23.0).sin();
        t.on_canvas_pointer(cp([x.min(X1), y], PointerPhase::Move));
        n += 1;
        if por_quadro > 0 && n.is_multiple_of(por_quadro) {
            let _ = t.take_preview_arc();
        }
    }
    t.on_canvas_pointer(cp([X1, Y], PointerPhase::Up));
    (*t.canvas_rgba).clone()
}

fn diferenca(a: &[u8], b: &[u8]) -> (usize, u8) {
    a.iter().zip(b).fold((0usize, 0u8), |(n, p), (&x, &y)| {
        let d = x.abs_diff(y);
        (n + usize::from(d > 0), p.max(d))
    })
}

/// ⭐⭐⭐ **COMPOR POR QUADRO DÁ A IMAGEM DE COMPOR POR EVENTO** — no rabisco, com a drenagem a cada
/// `1`, `4`, `16` e `64` eventos, e sem drenagem nenhuma (só o pen-up compõe).
///
/// **Medido** (2026-09-23): pior diferença **`1`** em todas as células, em `44` a `144` bytes de
/// `46 150` que o traço pinta. ⚠️ **O `1` é do BLUR e só dele** — ablação camada a camada: sem a
/// camada Blur a diferença é **`0` bytes** mesmo a drenar a cada evento. A composição adiada corre
/// o borrão de caixa sobre uma caixa diferente, e as somas correntes arredondam no último bit —
/// a mesma barra (`pior ≤ 1`, o último byte da quantização) que o
/// `a_ordem_e_da_pilha_e_nao_da_taxa_do_rato` já tem para a entrega em lotes. E a drenagem SOZINHA
/// não mexe num byte (medido: `0` com a composição por evento a drenar a cada evento).
///
/// ⚠️ **O CONTROLO é que torna a barra honesta:** o traço tem de pintar a tela (`> 40 000` bytes
/// contra a arte crua), senão *«as duas rotas concordam»* seria verdade sobre um traço que não fez
/// nada.
#[test]
fn compor_por_quadro_da_a_imagem_de_compor_por_evento() {
    let por_evento = rabisco(&mut pilha(false), 2.0, 0);
    let (pintados, _) = diferenca(&por_evento, &arte());
    assert!(
        pintados > 40_000,
        "CONTROLO: o rabisco pintou só {pintados} bytes — a fixtura deixou de conter o traço"
    );
    for k in [1usize, 4, 16, 64, 0] {
        let por_quadro = rabisco(&mut pilha(true), 2.0, k);
        let (bytes, pior) = diferenca(&por_evento, &por_quadro);
        assert!(
            pior <= 1 && bytes <= pintados / 100,
            "drenando a cada {k} eventos (0 = só no pen-up) a composição por QUADRO mudou a \
             imagem: {bytes} bytes diferentes de {pintados} pintados, pior {pior} — a barra é o \
             último byte da quantização do borrão"
        );
    }
}

/// ⭐⭐ **A tela compõe-se UMA vez por quadro, e só as drenagens a compõem** — a régua do relógio
/// que a imagem não vê (as duas rotas desenham o mesmo, logo mede-se a CONTA).
///
/// Metades: (1) depois de um evento com dabs a composição ESPERA (`pendente` está lá); (2) as
/// duas drenagens a fazem — a da pista CPU (`take_preview_arc`) e a da GPU (`take_preview_dirty`);
/// (3) o número de composições é o de drenagens, não o de eventos; (4) desligar a composição por
/// quadro compõe o que ficou; (5) o CONTROLO: com ela desligada nada espera.
#[test]
fn a_tela_compoe_uma_vez_por_quadro() {
    let mut t = pilha(true);
    let _ = super::composite_por_quadro::composicoes_e_zera();
    t.on_canvas_pointer(cp([X0, Y], PointerPhase::Down));
    for i in 1..=8 {
        t.on_canvas_pointer(cp([X0 + 4.0 * i as f32, Y], PointerPhase::Move));
    }
    assert!(
        t.paint.pilha.pendente.is_some(),
        "oito eventos com dabs e nada à espera — a composição não foi adiada"
    );
    assert_eq!(super::composite_por_quadro::composicoes_e_zera(), 0);
    let _ = t.take_preview_arc();
    assert!(
        t.paint.pilha.pendente.is_none(),
        "a drenagem da pista CPU não compôs"
    );
    assert_eq!(
        super::composite_por_quadro::composicoes_e_zera(),
        1,
        "oito eventos, UMA drenagem ⇒ UMA composição"
    );
    for i in 9..=16 {
        t.on_canvas_pointer(cp([X0 + 4.0 * i as f32, Y], PointerPhase::Move));
    }
    let _ = t.take_preview_dirty();
    assert!(
        t.paint.pilha.pendente.is_none(),
        "a drenagem da pista GPU não compôs"
    );
    t.on_canvas_pointer(cp([X0 + 70.0, Y], PointerPhase::Move));
    t.set_compor_por_quadro(false);
    assert!(
        t.paint.pilha.pendente.is_none(),
        "desligar a composição por quadro deixou trabalho por compor"
    );
    assert_eq!(super::composite_por_quadro::composicoes_e_zera(), 2);
    // CONTROLO: desligada, cada evento compõe no sítio e nada espera.
    t.on_canvas_pointer(cp([X0 + 80.0, Y], PointerPhase::Move));
    assert!(t.paint.pilha.pendente.is_none());
    assert_eq!(super::composite_por_quadro::composicoes_e_zera(), 0);
    t.on_canvas_pointer(cp([X0 + 80.0, Y], PointerPhase::Up));
}

/// ⛔ **Quem lê a tela antes da drenagem faz a composição correr no evento** — os leitores do
/// cabeçalho do [`super::composite_por_quadro`]. Medem-se os dois que se armam sem cena: um método
/// de RE-CARIMBO (a shell já o entrega uma vez por quadro, e o peel do quadro seguinte restaura a
/// região) e a SELECÇÃO (o portão faz `lerp` sobre a tela carimbada). ⚠️ O Solid, os fios e a
/// máscara de protecção estão na mesma porta ([`PainterTool::pilha_pode_adiar`]) e não têm fixtura
/// aqui — declarado.
#[test]
fn quem_le_a_tela_faz_a_composicao_correr_no_evento() {
    let t = pilha(true);
    assert!(
        t.pilha_pode_adiar(),
        "CONTROLO: a pilha do dono à mão livre adia"
    );
    let mut linha = pilha(true);
    linha.paint.brush.stroke_method = ph2d_painter_brush::StrokeMethod::Line;
    assert!(
        !linha.pilha_pode_adiar(),
        "um método de re-carimbo adiou — o peel do quadro seguinte restauraria a composição"
    );
    let mut sel = pilha(true);
    sel.paint.selection_active = true;
    sel.paint.selection_mask = std::sync::Arc::new(vec![255u8; (SIZE * SIZE) as usize]);
    assert!(
        !sel.pilha_pode_adiar(),
        "com uma selecção viva a pilha adiou — o portão faria `lerp` sobre a tela por compor"
    );
    sel.on_canvas_pointer(cp([X0, Y], PointerPhase::Down));
    sel.on_canvas_pointer(cp([X0 + 30.0, Y], PointerPhase::Move));
    assert!(sel.paint.pilha.pendente.is_none());
    sel.on_canvas_pointer(cp([X0 + 30.0, Y], PointerPhase::Up));
}
