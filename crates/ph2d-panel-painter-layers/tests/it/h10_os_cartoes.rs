//! ⭐ **O fim da H9: os widgets próprios pintam no corpo do cartão, sem a captura** (`docs/CartoesUI/13_PAINTER_PLANO.md`).
//! Curva, visor, gizmo, rampa, títulos que dobram e linhas por camada: o gesto real, e o mesmo efeito do painel de antes.

use super::h1_os_cartoes::{Mesa, TODAS, pincel};
use super::h5_os_cartoes::com_rampa;
use ph2d_a11y::NodeId;
use ph2d_editor_core::interaction::InteractiveState;
use ph2d_host::PointerKind;
use ph2d_tool_painter::ids as p;
use ph2d_tool_painter::{BrushSettings, Falloff, PaintMedia, TextureKind};
use std::collections::BTreeSet;

fn digital() -> BrushSettings {
    pincel(PaintMedia::Digital, "brush").0
}

fn com_curva() -> BrushSettings {
    let mut b = digital();
    b.falloff = Falloff::Custom.to_u8();
    b
}

fn por_camada() -> BrushSettings {
    let mut b = digital();
    b.shape_kind = TextureKind::Image.to_u8();
    b.shape_has_image = true;
    b.shape_layer_count = 2;
    b.shape_per_layer_color = true;
    b.shape_layer_color_on = [true; 16];
    b
}

fn com_grao() -> BrushSettings {
    let mut b = digital();
    b.texture_kind = TextureKind::COUNT - 2;
    b.texture_ramp_enabled = true;
    b
}

fn estados() -> Vec<(&'static str, BrushSettings)> {
    vec![
        ("digital", digital()),
        ("curva", com_curva()),
        ("rampa", com_rampa()),
        ("por camada", por_camada()),
        ("grao", com_grao()),
        ("molhado", pincel(PaintMedia::WetPaint, "brush").0),
        ("aguada", pincel(PaintMedia::Watercolor, "brush").0),
        ("impasto", pincel(PaintMedia::Impasto, "brush").0),
    ]
}

/// As pegas das telas próprias registam o `CurvePoint` no store, com a tela que o arrasto usa: foi escrito depois de
/// a lista pintar, e o id é o de sempre. *Mutação: tirar o `aplica` de `cards::paint` ⇒ vermelho.*
#[test]
fn a_handle_painted_in_a_card_carries_the_canvas_of_its_drag() {
    for (nome, b, id) in [
        (
            "curva",
            com_curva(),
            p::painter_brush_falloff_point_id(b_ponto()),
        ),
        ("gizmo", digital(), p::painter_brush_dab_handle_id(0)),
        ("visor", digital(), p::painter_taper_handle_id(0)),
    ] {
        let m = Mesa::nova(true, b, false);
        let r = m.rect(id).unwrap_or_else(|| panic!("{nome}: sem alvo"));
        let Some(InteractiveState::CurvePoint { canvas, .. }) = m.host.store().get(id) else {
            panic!("{nome}: a pega não foi registada");
        };
        let (cx, cy) = (r.x + r.w * 0.5, r.y + r.h * 0.5);
        assert!(
            cx >= canvas.x - 1.0
                && cx <= canvas.x + canvas.w + 1.0
                && cy >= canvas.y - 1.0
                && cy <= canvas.y + canvas.h + 1.0,
            "{nome}: a pega {r:?} fora da tela {canvas:?}"
        );
    }
}

fn b_ponto() -> u8 {
    com_curva().falloff_points[0].id
}

/// Arrastar a pega da curva chega à ferramenta, como no painel de antes.
#[test]
fn dragging_the_falloff_handle_reaches_the_tool_as_in_the_old_panel() {
    let id = p::painter_brush_falloff_point_id(b_ponto());
    let gesto = |cartoes: bool| {
        let mut m = Mesa::nova(cartoes, com_curva(), false);
        let r = m.rect(id).expect("a pega");
        let (x, y) = (r.x + r.w * 0.5, r.y + r.h * 0.5);
        let mut ev = m.ponteiro(PointerKind::Down, x, y);
        ev.extend(m.ponteiro(PointerKind::Move, x + 12.0, y + 6.0));
        ev.extend(m.ponteiro(PointerKind::Up, x + 12.0, y + 6.0));
        m.aplica(ev)
    };
    let (velho, novo) = (gesto(false), gesto(true));
    assert!(!novo.is_empty(), "o arrasto não fez nada");
    assert_eq!(velho.len(), novo.len(), "{velho:?} × {novo:?}");
}

/// Cada botão da rampa dá a mesma acção nas duas caras. *Mutação: tirar o `prepara` da rampa ⇒ vermelho.*
#[test]
fn every_ramp_button_sends_the_same_action_as_the_old_panel() {
    for (b, ids) in [
        (
            com_rampa(),
            [
                p::PAINTER_SHAPE_RAMP_ADD,
                p::PAINTER_SHAPE_RAMP_REMOVE,
                p::PAINTER_SHAPE_RAMP_INVERT,
            ],
        ),
        (
            com_grao(),
            [
                p::PAINTER_BRUSH_TEXTURE_RAMP_ADD,
                p::PAINTER_BRUSH_TEXTURE_RAMP_REMOVE,
                p::PAINTER_BRUSH_TEXTURE_RAMP_INVERT,
            ],
        ),
    ] {
        for id in ids {
            let velho = Mesa::nova(false, b, false).clica(id);
            let novo = Mesa::nova(true, b, false).clica(id);
            assert!(!velho.is_empty(), "{id:?}: o de antes não fez nada");
            assert_eq!(velho, novo, "{id:?}");
        }
    }
}

/// O campo Position da rampa, arrastado, avisa o editor da rampa. *Mutação: tirar o registo do campo ⇒ vermelho.*
#[test]
fn scrubbing_the_stop_position_edits_the_stop() {
    let mut m = Mesa::nova(true, com_grao(), false);
    let ev = m.arrasta_a_pega(p::PAINTER_BRUSH_TEXTURE_RAMP_STOP_POS, 40.0);
    let accoes = m.aplica(ev);
    assert!(
        accoes.iter().any(|a| a.contains("SelectOption")),
        "{accoes:?}"
    );
}

/// As listas da rampa abrem por baixo da caixa. *Mutação: tirar o `listas_abertas` da rampa ⇒ vermelho.*
#[test]
fn the_ramp_lists_open_their_options() {
    let id = p::PAINTER_BRUSH_TEXTURE_RAMP_MODE;
    let mut m = Mesa::nova(true, com_grao(), false);
    let antes = m.ids().len();
    let _ = m.clica(id);
    m.assenta();
    assert!(m.ids().len() > antes, "a lista do Color mode não abriu");
}

/// O título que dobra dobra, e o Reset do grupo é do grupo. *Mutação: tirar o alvo do título ⇒ vermelho.*
#[test]
fn the_folding_titles_fold_what_is_under_them() {
    let secao = p::PAINTER_BRUSH_COLOR_RAMP_SECTION;
    let so = p::PAINTER_BRUSH_TEXTURE_RAMP_MODE;
    let mut m = Mesa::nova(true, com_grao(), false);
    assert!(m.rect(so).is_some(), "a rampa não está à vista");
    assert!(m.rect(p::PAINTER_BRUSH_COLOR_RAMP_RESET).is_some());
    let _ = m.clica(secao);
    m.assenta();
    assert!(m.rect(so).is_none(), "o título não dobrou");
}

/// A cor e o Blend de cada camada, pelo gesto. *Mutação: tirar `camadas_abertas` ⇒ vermelho.*
#[test]
fn each_shape_layer_row_answers_as_in_the_old_panel() {
    let check = p::painter_shape_layer_color_check_id(1);
    let velho = Mesa::nova(false, por_camada(), false).clica(check);
    let novo = Mesa::nova(true, por_camada(), false).clica(check);
    assert!(!velho.is_empty());
    assert_eq!(velho, novo);
    let blend = p::painter_shape_layer_blend_id(1);
    let mut m = Mesa::nova(true, por_camada(), false);
    let antes = m.ids().len();
    let _ = m.clica(blend);
    m.assenta();
    assert!(m.ids().len() > antes, "a lista do Blend não abriu");
}

/// ⭐ **O inventário dos ids, estado a estado:** tudo o que o painel de antes tinha continua a ter alvo. Fora só o que
/// saiu de propósito: o cromo dos cabeçalhos e as pegas das secções, que agora são do cartão.
#[test]
fn no_id_of_the_old_panel_died() {
    let cromo: BTreeSet<NodeId> = TODAS.iter().copied().collect();
    let mut total = (0usize, 0usize);
    for (nome, b) in estados() {
        let (velho, novo) = (Mesa::nova(false, b, false), Mesa::nova(true, b, false));
        let (v, n) = (velho.ids(), novo.ids());
        let mortos: Vec<_> = v
            .iter()
            .filter(|i| !n.contains(i) && !cromo.contains(i))
            .filter(|i| !super::h1_os_cartoes::saiu_de_proposito(**i))
            .collect();
        println!("inventario {nome}: antes {} depois {}", v.len(), n.len());
        assert!(mortos.is_empty(), "{nome}: morreram {mortos:?}");
        total = (total.0 + v.len(), total.1 + n.len());
    }
    println!("inventario total: antes {} depois {}", total.0, total.1);
    assert!(total.0 > 300, "a régua viu pouco: {}", total.0);
}
