//! Gates da **linha do gradiente** (doc 85) — separados do `gradient_row.rs` pelo
//! tecto de LOC dos painéis (600, `architecture_panel_loc_cap`).

use super::*;
use ph2d_editor_core::interaction::WidgetStore;

/// **O TETO DE PARADAS É O PAINEL MAIS ESTREITO A DIVIDIR POR UM ALVO DE PONTEIRO.**
///
/// Bloco Z (doc 91), célula da folha 09 da conferência. A tabela inteira está no
/// doc-comment de [`MAX_GRADIENT_STOPS`]; aqui ela
/// é **refeita** a partir dos números vivos, para que mexer no recuo do painel, no raio de
/// agarrar ou no piso do arrasto de redimensionar reprove em vez de mentir.
///
/// ⚠️ **A medição CONFIRMOU o `8` que já lá estava, e é esse o achado**: o número nunca
/// esteve errado — o que não existia era a razão. *Um teto certo sem derivação e um teto
/// errado leem exactamente igual no dia em que alguém precisa de o mover.*
#[test]
fn the_gradient_stop_ceiling_is_the_narrowest_panel_divided_by_a_pointer_target() {
    let strip = ph2d_tokens::PANEL_MIN_W_PX - ph2d_tokens::PANEL_HEAD_PAD_PX * 2.0;
    // Por parada: o alvo de ponteiro que este editor declara para um marcador, mais a folga
    // que a célula da amostra come dos dois lados.
    let per_stop = GRAB_R * 2.0 + SWATCH_PAD * 2.0;
    let derived = (strip / per_stop).floor() as usize;
    assert_eq!(
        MAX_GRADIENT_STOPS, derived,
        "faixa util {strip} px / {per_stop} px por parada = {derived}"
    );
    // O controle: a régua não é vacuamente pequena — o modelo admite MUITO mais que isto, e
    // é essa distância que torna o teto uma decisão de PAINEL e não do formato.
    assert!(
        derived < ph2d_color::MAX_RAMP_STOPS,
        "o teto do painel e' mais apertado que o do modelo ({} paradas)",
        ph2d_color::MAX_RAMP_STOPS
    );
    // E o outro controle: uma parada a mais já não caberia.
    assert!(
        (derived + 1) as f32 * per_stop > strip,
        "e a parada seguinte nao cabe"
    );
}

#[test]
fn working_falls_back_to_the_default_ramp() {
    // Empty / garbage open on the draggable black→white default (2 stops).
    assert_eq!(working("").len(), 2);
    assert_eq!(working("nonsense").len(), 2);
    let three = working("g1 2 0:1,0,0 0.5:0,1,0 1:0,0,1");
    assert_eq!(three.len(), 3);
}

#[test]
fn add_stop_lands_in_the_widest_gap_without_a_jump() {
    // Default black→white: stops at 0 and 1. Add → midpoint 0.5, colour = eval(0.5).
    let r = parse_gradient(&add_stop("g1 2 0:0,0,0 1:1,1,1")).unwrap();
    assert_eq!(r.len(), 3);
    assert!((r.stops()[1].pos - 0.5).abs() < 1e-6);
    assert!((r.stops()[1].color[0] - 0.5).abs() < 1e-6, "no colour jump");
}

#[test]
fn add_stop_stops_at_the_cap() {
    let mut v = "g1 2 0:0,0,0 1:1,1,1".to_string();
    for _ in 0..MAX_GRADIENT_STOPS + 4 {
        v = add_stop(&v);
    }
    assert_eq!(parse_gradient(&v).unwrap().len(), MAX_GRADIENT_STOPS);
}

#[test]
fn remove_keeps_at_least_two() {
    assert_eq!(
        parse_gradient(&remove_stop("g1 2 0:0,0,0 1:1,1,1", 0))
            .unwrap()
            .len(),
        2
    );
    SELECTED.with(|s| s.set(Some((0, 1))));
    let r = parse_gradient(&remove_stop("g1 2 0:0,0,0 0.5:0,1,0 1:1,1,1", 0)).unwrap();
    assert_eq!(r.len(), 2);
}

#[test]
fn cycle_interp_advances_and_wraps() {
    SELECTED.with(|s| s.set(None));
    let v = cycle_interp("g1 2 0:0,0,0 1:1,1,1", 0); // Linear -> Ease (u8 2 -> 0)
    assert_eq!(parse_gradient(&v).unwrap().interp, RampInterp::Ease);
}

/// ⭐ **A CÉLULA DA FOLHA 09, medida:** com uma parada SELECIONADA o botão
/// cicla a interpolação DELA, não a da rampa — o *ramp parameter* do Houdini.
#[test]
fn with_a_stop_selected_the_button_cycles_that_stops_interp() {
    let base = "g1 2 0:0,0,0 0.5:1,0,0 1:1,1,1";
    SELECTED.with(|s| s.set(Some((0, 1))));
    let v = cycle_interp(base, 0);
    let r = parse_gradient(&v).expect("volta");
    assert_eq!(
        r.interp,
        RampInterp::Linear,
        "a GLOBAL nao se pode mexer quando ha' parada selecionada"
    );
    assert_eq!(r.stops()[1].interp, Some(RampInterp::Linear));
    assert_eq!(r.stops()[0].interp, None, "as vizinhas ficam onde estavam");
    SELECTED.with(|s| s.set(None));
}

/// ⚠️ **E a roda FECHA de volta ao `Global`** — sem caminho de volta, dar
/// interpolação própria a um stop seria uma porta de sentido único.
#[test]
fn the_stops_wheel_comes_back_to_global() {
    SELECTED.with(|s| s.set(Some((0, 1))));
    let mut v = "g1 2 0:0,0,0 0.5:1,0,0 1:1,1,1".to_string();
    let mut seen = Vec::new();
    for _ in 0..6 {
        v = cycle_interp(&v, 0);
        seen.push(parse_gradient(&v).unwrap().stops()[1].interp);
    }
    assert_eq!(
        seen.iter().filter(|x| x.is_none()).count(),
        1,
        "a roda tem de passar pelo Global exactamente uma vez em seis: {seen:?}"
    );
    assert_eq!(
        seen.last().copied().flatten(),
        None,
        "e fechar nele: {seen:?}"
    );
    SELECTED.with(|s| s.set(None));
}

#[test]
fn drain_drag_folds_the_position_and_never_lets_it_cross() {
    let slot = 0;
    let mut store = WidgetStore::with_capacity(2);
    // Drag the MIDDLE stop (index 1) far right (x=2.0). It must clamp strictly below
    // its right neighbour's position — stops never cross.
    store.set_curve_point_drag(param_grad_editor_id(slot), 0, 1, 2.0, 0.5);
    let r =
        parse_gradient(&drain_drag(&mut store, slot, "g1 2 0:1,0,0 0.5:0,1,0 1:0,0,1").unwrap())
            .unwrap();
    assert!(
        r.stops()[0].pos < r.stops()[1].pos && r.stops()[1].pos < r.stops()[2].pos,
        "position order preserved: {:?}",
        r.stops().iter().map(|s| s.pos).collect::<Vec<_>>()
    );
    assert!(
        store
            .take_curve_point_drag_if(|p| p == param_grad_editor_id(slot))
            .is_none(),
        "slot drained"
    );
}
