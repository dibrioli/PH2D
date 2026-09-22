//! **O RESERVATÓRIO DO MIXER RENASCE COM A COBERTURA** — os gates da cura de 2026-09-22
//! ([auditoria](../../../../../docs/Painter/42_auditoria_do_watercolor_2026-09-22.md)).
//!
//! A lei: **quem reconstrói o LOTE INTEIRO a cada quadro reconstrói também o RESERVATÓRIO.** Os dois
//! chamadores de [`super::watercolor_accum::PainterTool::clear_wet_coverage`] refazem a cobertura do
//! zero por quadro — o re-carimbo de FIGURA
//! ([`super::stamp_preview::PainterTool::stamp_drag_preview_watercolor`]) e o
//! `Anchored`/`Drag Dot`/`Line` do traço ([`super::stamp_preview::PainterTool::stamp_stroke_dabs`])
//! —, e o `wet_mix` (a carga do pincel, que se ESGOTA com o depósito) sobrevivia aos dois: a mesma
//! figura com os mesmos números saía diferente conforme quantas vezes lhe tinham tocado.
//!
//! ⚠️ **Cada gate leva o CONTROLO do mixer DESARMADO ao lado**, e ele não é decoração: com
//! `wet_charge = 1` o caminho do mixer é saltado inteiro e a igualdade vale **por construção** —
//! *um gate só com a metade positiva passaria com a cura apagada se a fixtura não armasse o mixer*.
//! E leva o **piso de população** (`MEXEU`), porque uma fixtura que não pinta lê `0` diferenças, que
//! é o mesmo byte de um produto limpo (foi assim que a 1.ª redacção da sonda irmã mentiu).

use super::diag_auditoria_da_pilha::{cp, tela_com_arte};
use super::*;

const S: u32 = 1024;
const C: [f32; 2] = [512.0, 512.0];

fn cena(charge: f32) -> PainterTool {
    let mut t = tela_com_arte(200.0, 255);
    t.paint.composite_enabled = false;
    t.paint.brush.watercolor = true;
    t.paint.brush.radius_px = 200.0;
    t.paint.brush.edge_spread = 7.0;
    t.paint.brush.warp = 0.0;
    t.paint.brush.wet_rewet = 0.0;
    t.paint.brush.wet_charge = charge;
    t
}

fn texels_diferentes(a: &[u8], b: &[u8]) -> usize {
    (0..(S * S) as usize)
        .filter(|i| a[i * 4..i * 4 + 4] != b[i * 4..i * 4 + 4])
        .count()
}

/// `n` re-carimbos de uma figura PARQUEADA. ⚠️ Parqueada e não em voo: o `restamp_shapes_preview`
/// abre com `if self.draft_stamp() { descasca; return }`, e `draft_stamp()` é verdadeiro durante
/// TODO o gesto até ao `Up` — uma fixtura com a figura na mão não carimba em meio nenhum.
fn figura_recarimbada(n: usize, charge: f32) -> PainterTool {
    let mut t = cena(charge);
    t.paint.parked_shapes.push(stroke_multi::StrokeShape {
        state: crate::undo::ShapeEditState::Ellipse(crate::undo::EllipseState {
            center: C,
            u: [1.0, 0.0],
            rx: 160.0,
            ry: 160.0,
            editing: false,
            seed: 1,
        }),
        op: stroke_multi::StrokeOp::Overlay,
    });
    for _ in 0..n {
        t.restamp_shapes_preview(&[]);
    }
    t
}

/// `n` movimentos de um `Anchored` no MESMO raio — o outro chamador, que não é uma figura.
fn anchored_com_movimentos(n: usize, charge: f32) -> PainterTool {
    let mut t = cena(charge);
    t.paint.brush.stroke_method = ph2d_painter_brush::StrokeMethod::Anchored;
    t.on_canvas_pointer(cp(C, PointerPhase::Down));
    for _ in 0..n {
        t.on_canvas_pointer(cp([C[0] + 160.0, C[1]], PointerPhase::Move));
    }
    t.on_canvas_pointer(cp([C[0] + 160.0, C[1]], PointerPhase::Up));
    t
}

/// **A FIGURA:** re-carimbar dez vezes tem de dar a MESMA tela que re-carimbar uma vez.
///
/// Sem a cura, medido: `n = 2` difere em `377 756` texels com o pior byte a `15`, e a deriva CRESCE
/// (`3 → 22`, `5 → 38`, `10 → 64`) até saturar em `64/255` — um quarto da gama, sobre a figura toda.
#[test]
fn o_reservatorio_renasce_entre_re_carimbos_de_figura() {
    let uma = figura_recarimbada(1, 0.5);
    let dez = figura_recarimbada(10, 0.5);

    // PISO DE POPULAÇÃO: a fixtura tem de conter o fenómeno (senão `0` não afirma nada).
    let limpa = cena(0.5);
    let mexeu = texels_diferentes(&limpa.canvas_rgba, &uma.canvas_rgba);
    assert!(
        mexeu > 100_000,
        "a fixtura tem de PINTAR para a igualdade abaixo afirmar algo (mexeu {mexeu})"
    );

    let d = texels_diferentes(&uma.canvas_rgba, &dez.canvas_rgba);
    assert_eq!(
        d, 0,
        "dez re-carimbos da MESMA figura têm de dar a MESMA tela que um (difere {d})"
    );
}

/// **O CONTROLO da figura:** com o mixer DESARMADO (`wet_charge = 1`) o caminho dele é saltado e a
/// igualdade vale por construção — este gate existe para provar que o irmão acima mede o MIXER e
/// não uma propriedade trivial do re-carimbo.
#[test]
fn com_o_mixer_desarmado_a_figura_ja_era_idempotente() {
    let uma = figura_recarimbada(1, 1.0);
    let dez = figura_recarimbada(10, 1.0);
    assert_eq!(
        texels_diferentes(&uma.canvas_rgba, &dez.canvas_rgba),
        0,
        "sem mixer o re-carimbo sempre foi idempotente — se ISTO reprovar, o defeito é outro"
    );
}

/// **O TRAÇO:** o `Anchored` no watercolor reconstrói o lote a cada movimento pelo MESMO
/// `clear_wet_coverage`, e tinha o MESMO defeito — sem a cura, `n = 2` difere em `61 010` texels
/// (`pior 4`). *A cura não é das figuras: é de quem reconstrói o lote.*
#[test]
fn o_reservatorio_renasce_entre_movimentos_de_um_anchored() {
    let um = anchored_com_movimentos(1, 0.5);
    let dez = anchored_com_movimentos(10, 0.5);

    let limpa = cena(0.5);
    let mexeu = texels_diferentes(&limpa.canvas_rgba, &um.canvas_rgba);
    assert!(
        mexeu > 10_000,
        "a fixtura tem de PINTAR para a igualdade abaixo afirmar algo (mexeu {mexeu})"
    );

    let d = texels_diferentes(&um.canvas_rgba, &dez.canvas_rgba);
    assert_eq!(
        d, 0,
        "dez movimentos de um Anchored no mesmo raio têm de dar a MESMA tela que um (difere {d})"
    );
}

/// **O CONTROLO do traço**, pelo mesmo motivo do irmão da figura.
#[test]
fn com_o_mixer_desarmado_o_anchored_ja_era_idempotente() {
    let um = anchored_com_movimentos(1, 1.0);
    let dez = anchored_com_movimentos(10, 1.0);
    assert_eq!(
        texels_diferentes(&um.canvas_rgba, &dez.canvas_rgba),
        0,
        "sem mixer o Anchored sempre foi idempotente — se ISTO reprovar, o defeito é outro"
    );
}

/// **A FIAÇÃO, e ela é o que impede a recaída mais provável:** a reposição vive DENTRO do
/// `clear_wet_coverage`, que é a porta que os DOIS chamadores já usam. Escrita em cada chamador ela
/// seria uma lei em dois sítios, e o terceiro chamador nasceria sem ela — que é exactamente como
/// este defeito existiu (o `open_stroke` chamava o `reset_wet_mix` e mais ninguém).
#[test]
fn a_reposicao_vive_na_porta_e_nao_nos_chamadores() {
    let porta = include_str!("watercolor_accum.rs");
    assert!(
        porta.contains("reset_wet_mix()"),
        "o `clear_wet_coverage` tem de repor o reservatório — ele é o sítio onde a cobertura recomeça"
    );
    let chamador = include_str!("stamp_preview.rs");
    assert!(
        !chamador.contains("reset_wet_mix()"),
        "um chamador a repor à mão é a segunda resposta que diverge no dia do terceiro chamador"
    );
}
