//! **A cena `=74` contém o que o roteiro dela pede.**
//!
//! ⚠️ É a disciplina de fixture aplicada a um smoke: um roteiro que pede ao Enio para ver um
//! buraco NASCER sobre uma geometria em que `Union` e `Subtract` desenham a mesma coisa **passa**,
//! e faz o smoke inteiro mentir. A cena não corre aqui (ela precisa de janela e GPU); o que corre
//! é a GEOMETRIA dela, que é o que decide.

use super::*;

/// O que o `build` desenha para o rig `r`, sem app nenhum.
fn shapes(r: usize) -> [VecPath; 2] {
    let x = RIG_X[r];
    [
        rectangle([x - 1.15, 0.7], [x + 1.15, 2.1]),
        rectangle([x - 0.5, 1.05], [x + 0.5, 1.75]),
    ]
}

fn cook(items: &[VecPath], op: ph2d_vec_boolean::PathfinderOp) -> Vec<VecPath> {
    let refs: Vec<&VecPath> = items.iter().collect();
    ph2d_vec_boolean::pathfinder(&refs, op).unwrap_or_default()
}

/// ⭐ **O de dentro está INTEIRAMENTE dentro do de fora** — a única disposição em que o roteiro
/// tem o que mostrar.
///
/// A régua é a TOPOLOGIA, e não a área: duas formas que apenas se cruzam também mudam de área
/// entre `Union` e `Subtract`, e um gate de área ficaria verde sobre uma cena em que nenhum buraco
/// nasce. O que o passo 2 promete é *um buraco a nascer*, e um buraco é um contorno a mais.
#[test]
fn the_ready_rig_grows_a_hole_and_not_merely_a_smaller_shape() {
    for r in 0..RIG_X.len() {
        let art = shapes(r);
        let union = cook(&art, ph2d_vec_boolean::PathfinderOp::Union);
        let sub = cook(&art, ph2d_vec_boolean::PathfinderOp::Subtract);
        let (u, s) = (
            union.first().map_or(0, VecPath::contour_count),
            sub.first().map_or(0, VecPath::contour_count),
        );
        assert_eq!(u, 1, "o rig {r}: a uniao tinha de dar UM contorno, deu {u}");
        assert_eq!(
            s, 2,
            "o rig {r}: a subtracao tinha de dar DOIS contornos (o de fora e o buraco), deu {s} \
             — as duas formas nao estao encaixadas, e o roteiro pede um buraco que nao nasce"
        );
    }
}

/// **O CHIP sobra por fora do de fora** — sem isso o passo 1 é impossível.
///
/// ⚠️ O chip é o hospedeiro dos estados, e um hospedeiro é *a forma ÚNICA selecionada*: clicar num
/// operando acende o grupo booleano inteiro. A borda que sobra é a única superfície em que o
/// artista o pega sozinho — e um smoke que peça um clique que a geometria não permite é pior que
/// nenhum.
#[test]
fn the_chip_sticks_out_far_enough_to_be_clicked_alone() {
    for (r, x) in RIG_X.iter().enumerate() {
        let chip = rectangle([x - 1.7, 0.2], [x + 1.7, 2.6]);
        let outer = &shapes(r)[0];
        let margin = shapes(r)[0]
            .verts
            .iter()
            .map(|v| v.anchor[1])
            .fold(f64::INFINITY, f64::min)
            - chip
                .verts
                .iter()
                .map(|v| v.anchor[1])
                .fold(f64::INFINITY, f64::min);
        assert!(
            margin > 0.25,
            "o rig {r}: o chip so' sobra {margin:.3} por baixo do de fora — o passo 1 pede um \
             clique que nao ha' onde dar"
        );
        assert!(
            ph2d_vec_boolean::area(&chip).abs() > ph2d_vec_boolean::area(outer).abs(),
            "o rig {r}: o chip nao e' maior que o de fora"
        );
    }
}

/// **Os dois rigs são o MESMO material** — é isso que faz o da direita ser um controle.
///
/// ⚠️ Se eles diferissem em geometria, o artista que visse um mexer e o outro não teria duas
/// explicações possíveis, e o smoke deixaria de responder a pergunta que faz.
#[test]
fn the_control_rig_is_the_same_material_as_the_ready_one() {
    let (a, b) = (shapes(0), shapes(1));
    for i in 0..2 {
        let (wa, ha) = span(&a[i]);
        let (wb, hb) = span(&b[i]);
        assert!(
            (wa - wb).abs() < 1e-9 && (ha - hb).abs() < 1e-9,
            "a forma {i} dos dois rigs difere: {wa:.3}x{ha:.3} contra {wb:.3}x{hb:.3}"
        );
    }
}

fn span(p: &VecPath) -> (f64, f64) {
    let xs: Vec<f64> = p.verts.iter().map(|v| v.anchor[0]).collect();
    let ys: Vec<f64> = p.verts.iter().map(|v| v.anchor[1]).collect();
    let f = |v: &[f64]| {
        v.iter().copied().fold(f64::NEG_INFINITY, f64::max)
            - v.iter().copied().fold(f64::INFINITY, f64::min)
    };
    (f(&xs), f(&ys))
}
