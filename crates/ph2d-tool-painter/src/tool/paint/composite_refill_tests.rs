//! **UMA EDIÇÃO DA PILHA APAGA O TRAÇO, ACTUALIZA E VOLTA A MOSTRAR** — os gates da ordem do dono de
//! 2026-09-21 (*«nos strokes vivos, o ajuste de propriedades no painel deve apagar o traço,
//! atualizar e voltar a mostrar. Isso já acontece com todos os ajustes, menos o composite»*).
//!
//! ⭐ **Ele nomeou a lei E nomeou o único infractor**, logo o trabalho é ALINHAR e não desenhar: a
//! rota já existe e chama-se [`super::stencil::PainterTool::refill_if_appearance_changed`], que o
//! `handle_panel_event` corre depois de cada edição. A pergunta era *porque é que uma edição na
//! pilha não a acorda*, e a resposta é **uma assinatura que não a vê**: a `AppearanceSig` leva o
//! `BrushSpec` INTEIRO mais treze campos do `paint`, e **nenhum dos seis campos do composite**
//! (`composite_enabled`, `composite`, `composite_len`, `composite_add_op`, `composite_mask`,
//! `composite_arco`) está lá — medido em 2026-09-22.
//!
//! ⚠️ ***Dois sítios que têm de concordar sobre um facto — «a aparência mudou?» — e discordam:*** a
//! pilha muda o que se vê e a assinatura não muda um bit, logo o `!=` lê `false` e o re-carimbo
//! nunca corre. O artista vê a figura com a pilha ANTIGA até que outra coisa qualquer a re-carimbe.
//!
//! ⚠️ **Os CONTROLOS não são decoração:** sem o de «uma edição de PINCEL move a tela», um `0` na
//! metade de cima lê-se igual a *«a fixtura não tem figura viva nenhuma»* — que foi exactamente como
//! a sonda irmã mentiu duas vezes nesta linha.

use super::diag_auditoria_da_pilha::{cp, pilha_do_dono, tela_com_arte};
use super::*;
use ph2d_editor_core::tool::{PanelEvent, Tool};

const S: u32 = 1024;
const C: [f32; 2] = [512.0, 512.0];

/// Tela com arte, a pilha do dono armada, e uma ELIPSE VIVA (largada, logo ainda editável).
fn cena_com_figura_viva() -> PainterTool {
    let mut t = tela_com_arte(60.0, 255);
    pilha_do_dono(&mut t);
    t.paint.composite_enabled = true;
    t.paint.brush.radius_px = 60.0;
    t.paint.brush.stroke_method = ph2d_painter_brush::StrokeMethod::Ellipse;
    t.on_canvas_pointer(cp(C, PointerPhase::Down));
    t.on_canvas_pointer(cp([C[0] + 160.0, C[1]], PointerPhase::Move));
    t.on_canvas_pointer(cp([C[0] + 160.0, C[1]], PointerPhase::Up));
    t
}

fn texels_diferentes(a: &[u8], b: &[u8]) -> usize {
    (0..(S * S) as usize)
        .filter(|i| a[i * 4..i * 4 + 4] != b[i * 4..i * 4 + 4])
        .count()
}

/// **A LEI:** mexer na força de uma camada da pilha tem de mudar o que está na tela, no mesmo
/// instante — porque é isso que *«apagar, actualizar e voltar a mostrar»* quer dizer.
#[test]
fn uma_edicao_da_pilha_re_carimba_a_figura_viva() {
    let mut t = cena_com_figura_viva();
    let antes = t.canvas_rgba.clone();
    t.handle_panel_event(PanelEvent::SetValue(
        crate::ids::PAINTER_BRUSH_COMPOSITE_STRENGTH[0],
        0.95,
    ));
    let d = texels_diferentes(&antes, &t.canvas_rgba);
    assert!(
        d > 0,
        "uma edição da PILHA tem de re-carimbar a figura viva (a tela não se mexeu: {d} texels)"
    );
}

/// **O CONTROLO POSITIVO:** uma edição de PINCEL já obedecia à lei. Sem ele, um `0` no gate acima
/// não distingue *«a pilha não acorda o re-carimbo»* de *«esta fixtura não tem figura viva»*.
#[test]
fn uma_edicao_de_pincel_ja_re_carimbava_a_figura_viva() {
    let mut t = cena_com_figura_viva();
    let antes = t.canvas_rgba.clone();
    t.handle_panel_event(PanelEvent::SetValue(
        crate::ids::PAINTER_BRUSH_SIZE_SLIDER,
        120.0,
    ));
    let d = texels_diferentes(&antes, &t.canvas_rgba);
    assert!(
        d > 0,
        "a fixtura não contém o fenómeno: nem uma edição de PINCEL move a tela ({d} texels)"
    );
}

/// **A FIAÇÃO:** a assinatura que decide *«a aparência mudou?»* tem de VER a pilha. Este gate é o
/// que nomeia o sítio da cura — sem ele, alguém podia satisfazer o irmão de cima re-carimbando à
/// mão no braço do composite, que é a segunda resposta à mesma pergunta.
#[test]
fn a_assinatura_da_aparencia_ve_a_pilha() {
    let mut t = cena_com_figura_viva();
    let antes = t.appearance_sig();
    t.paint.composite[0].strength = 0.95;
    assert!(
        t.appearance_sig() != antes,
        "a AppearanceSig tem de mudar quando a pilha muda — senão o refill_if_appearance_changed \
         lê `false` e o re-carimbo nunca corre"
    );
}

/// **A OUTRA METADE da fiação:** ligar/desligar a pilha inteira também é uma mudança de aparência.
/// ⚠️ Ela é um campo SEPARADO (`composite_enabled`), e uma cura que só levasse o array deixava este
/// caso de fora — *um interruptor que não entra na assinatura é o mesmo defeito com outro nome*.
#[test]
fn a_assinatura_da_aparencia_ve_o_interruptor_da_pilha() {
    let mut t = cena_com_figura_viva();
    let antes = t.appearance_sig();
    t.paint.composite_enabled = !t.paint.composite_enabled;
    assert!(
        t.appearance_sig() != antes,
        "desligar a pilha muda o que se vê ⇒ tem de mudar a assinatura"
    );
}
