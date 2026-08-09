//! **O EMPURRÃO do modo cinemático** (W-KinPush) — de uma lista de contatos
//! para uma lista de *"quanto este corpo leva, e onde"*.
//!
//! # ⚠️ Por que isto é um módulo e não três linhas no laço
//!
//! A **lei** (quanto de um bloqueio atravessa uma superfície) é pura e mora na
//! [`ph2d_platformer::push_transfer`]; o que sobra aqui é a **dedup**, que é uma
//! decisão de orquestração com um modo de falha próprio — e um modo de falha que
//! nenhum gate de mundo distingue de *"o caixote é leve"*: dois relatórios do
//! mesmo corpo entregam o dobro da quantidade de movimento, e o caixote apenas
//! anda mais.
//!
//! Separada, ela é testável sem um mundo, e é isso que torna a contagem dupla
//! uma asserção em vez de uma esperança.

use ph2d_physics::{CharacterHit, RigidBodyHandle};
use ph2d_platformer::{ReactionConfig, push_transfer};

/// Um empurrão devido: **em quem**, **quanto** (metros deste tique) e **onde**.
pub(super) type Push = (RigidBodyHandle, [f32; 2], [f32; 2]);

/// **Os empurrões deste tique, no máximo um por CORPO.**
///
/// `wanted` é o que a lei pediu, `effective` é o que o controlador conseguiu, e
/// a diferença — o que o mundo **não** deixou acontecer — é o que volta,
/// projetado na linha da normal de cada superfície.
///
/// # ⚠️ Um empurrão por corpo, ficando com o MAIOR
///
/// O controlador desliza em iterações e pode relatar o mesmo corpo mais de uma
/// vez. Somar os relatórios entregaria a mesma quantidade de movimento duas
/// vezes; ficar com o maior entrega **o bloqueio que aquele corpo de facto
/// causou**, que é a leitura honesta de um `blocked` que já é o total do tique.
///
/// ⚠️ **Corpos DIFERENTES recebem cada um o seu**, e não é contagem dupla: um
/// personagem encunhado entre duas superfícies empurra as duas, e é isso que um
/// corpo real faz.
pub(super) fn push_from_hits(
    react: &ReactionConfig,
    wanted: [f32; 2],
    effective: [f32; 2],
    hits: &[CharacterHit],
    out: &mut Vec<Push>,
) {
    out.clear();
    let blocked = [wanted[0] - effective[0], wanted[1] - effective[1]];
    if !blocked[0].is_finite() || !blocked[1].is_finite() {
        return;
    }
    for h in hits {
        // Um collider sem corpo não tem o que ser empurrado.
        let Some(body) = h.body else { continue };
        let transfer = push_transfer(react, blocked, h.normal);
        if transfer == [0.0, 0.0] {
            continue;
        }
        let mag = transfer[0] * transfer[0] + transfer[1] * transfer[1];
        if let Some(slot) = out.iter_mut().find(|(b, _, _)| *b == body) {
            if mag > slot.1[0] * slot.1[0] + slot.1[1] * slot.1[1] {
                slot.1 = transfer;
                slot.2 = h.point;
            }
        } else {
            out.push((body, transfer, h.point));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Um handle de corpo qualquer — o valor não importa, só a identidade.
    fn body(i: u32) -> RigidBodyHandle {
        RigidBodyHandle::from_raw_parts(i, 0)
    }

    fn hit(b: u32, normal: [f32; 2], point: [f32; 2]) -> CharacterHit {
        CharacterHit {
            body: Some(body(b)),
            point,
            normal,
        }
    }

    /// **O bloqueio é o pedido menos o efetivo**, e é ele que viaja.
    #[test]
    fn what_the_world_refused_is_what_travels() {
        let mut out = Vec::new();
        push_from_hits(
            &ReactionConfig::STARTING_POINT,
            [0.10, 0.0],
            [0.02, 0.0],
            &[hit(1, [1.0, 0.0], [0.5, 0.3])],
            &mut out,
        );
        assert_eq!(out.len(), 1);
        assert!(
            (out[0].1[0] - 0.08).abs() < 1.0e-6,
            "pediu 0,10 e andou 0,02 => 0,08 atravessa: {:?}",
            out[0].1
        );
        assert_eq!(out[0].2, [0.5, 0.3], "no ponto de contato");
    }

    /// ⚠️ **O MESMO corpo relatado duas vezes leva UM empurrão** — este é o
    /// gate que separa *"a dedup funciona"* de *"o caixote é leve"*, e nenhuma
    /// medição de mundo os distingue.
    #[test]
    fn one_body_reported_twice_is_pushed_once() {
        let mut out = Vec::new();
        push_from_hits(
            &ReactionConfig::STARTING_POINT,
            [0.10, 0.0],
            [0.0, 0.0],
            &[
                hit(1, [1.0, 0.0], [0.5, 0.3]),
                hit(1, [1.0, 0.0], [0.5, 0.1]),
            ],
            &mut out,
        );
        assert_eq!(out.len(), 1, "um empurrao por corpo: {out:?}");
        assert!((out[0].1[0] - 0.10).abs() < 1.0e-6, "{:?}", out[0].1);
    }

    /// **Dois corpos recebem cada um o seu** — encunhado entre duas superfícies,
    /// um corpo real empurra as duas.
    #[test]
    fn two_bodies_each_get_their_own() {
        let mut out = Vec::new();
        push_from_hits(
            &ReactionConfig::STARTING_POINT,
            [0.10, -0.10],
            [0.0, 0.0],
            &[
                hit(1, [1.0, 0.0], [0.5, 0.3]),
                hit(2, [0.0, 1.0], [0.2, 0.0]),
            ],
            &mut out,
        );
        assert_eq!(out.len(), 2, "{out:?}");
        assert!((out[0].1[0] - 0.10).abs() < 1.0e-6 && out[0].1[1].abs() < 1.0e-6);
        assert!(out[1].1[0].abs() < 1.0e-6 && (out[1].1[1] + 0.10).abs() < 1.0e-6);
    }

    /// **Nada bloqueado, nada empurrado** — e um collider sem corpo tampouco.
    #[test]
    fn nothing_blocked_pushes_nothing() {
        let mut out = Vec::new();
        push_from_hits(
            &ReactionConfig::STARTING_POINT,
            [0.10, 0.0],
            [0.10, 0.0],
            &[hit(1, [1.0, 0.0], [0.5, 0.3])],
            &mut out,
        );
        assert!(out.is_empty(), "{out:?}");

        push_from_hits(
            &ReactionConfig::STARTING_POINT,
            [0.10, 0.0],
            [0.0, 0.0],
            &[CharacterHit {
                body: None,
                point: [0.0, 0.0],
                normal: [1.0, 0.0],
            }],
            &mut out,
        );
        assert!(out.is_empty(), "collider sem corpo: {out:?}");
    }

    /// ⚠️ **A lista é LIMPA por quem a preenche** — sem isto o tique seguinte
    /// re-empurraria os corpos do anterior, e o sintoma seria um caixote a
    /// deslizar sozinho depois de o personagem sair de perto.
    #[test]
    fn the_list_belongs_to_this_tick_only() {
        let mut out = vec![(body(9), [1.0, 0.0], [0.0, 0.0])];
        push_from_hits(
            &ReactionConfig::STARTING_POINT,
            [0.0, 0.0],
            [0.0, 0.0],
            &[],
            &mut out,
        );
        assert!(out.is_empty(), "o residuo do tique anterior fica: {out:?}");
    }
}
