//! Testes das alças de ponta.
//!
//! O que eles provam é a REGRA de largar — a tabela do `resolve_drop`. É a parte que decide o
//! comportamento; a pintura do círculo é consequência.

use super::*;
use ph2d_vec_scene::{VecScene, rectangle};

/// Uma forma-alvo: retângulo local `[-1,-1]..[1,1]`, com um afim qualquer.
fn target(id: VecPathId, xform: Xform) -> DropTarget {
    DropTarget {
        id,
        lo: [-1.0, -1.0],
        hi: [1.0, 1.0],
        xform,
    }
}

/// **O pedido do Enio, numa asserção:** puxar a alça para LONGE da forma não solta o vínculo.
/// A ponta continua presa, com um `u`/`v` fora de `[0, 1]` — e é por isso que ela continua
/// andando quando a forma anda.
#[test]
fn dragging_the_handle_away_from_the_shape_keeps_it_bound() {
    let t = target(7, Xform::IDENTITY);
    let bound = ConnectorEnd::Bound {
        target: 7,
        anchor: Anchor::Floating,
    };
    // Largou no vazio, bem à direita da forma (que vai de x = −1 a x = 1).
    let out = resolve_drop(&bound, [4.0, 0.0], None, Some(t));

    let ConnectorEnd::Bound {
        target: got,
        anchor: Anchor::Port { u, v },
    } = out
    else {
        panic!("a ponta SOLTOU da forma ao ser afastada: {out:?}");
    };
    assert_eq!(got, 7, "continua presa na MESMA forma");
    // x = 4 na caixa [−1, 1] ⇒ u = (4 − (−1))/2 = 2.5: fora de [0,1], que é o ponto.
    assert!(
        (f64::from(u) - 2.5).abs() < 1e-5,
        "u tinha de sair de [0,1] (afastado): {u}"
    );
    assert!(
        (f64::from(v) - 0.5).abs() < 1e-5,
        "v no meio da altura: {v}"
    );
}

/// **E o afastamento anda com a forma.** É a razão de o `Port` ser medido na caixa LOCAL e não
/// em unidades de mundo: gira e escala junto (ADR-0111), sem nada a manter.
///
/// A forma gira 90°; o mesmo `(u, v)` tem de descrever um ponto de mundo GIRADO junto.
#[test]
fn the_detached_end_rides_the_shapes_transform() {
    let t = target(7, Xform::IDENTITY);
    let bound = ConnectorEnd::Bound {
        target: 7,
        anchor: Anchor::Floating,
    };
    let ConnectorEnd::Bound {
        anchor: Anchor::Port { u, v },
        ..
    } = resolve_drop(&bound, [4.0, 0.0], None, Some(t))
    else {
        panic!("nao virou port");
    };

    // A MESMA forma, girada 90° (x -> y, y -> -x).
    let spun = Xform([0.0, 1.0, -1.0, 0.0, 0.0, 0.0]);
    let t2 = target(7, spun);
    // O ponto de mundo que aquele (u, v) descreve agora:
    let local = [
        t2.lo[0] + f64::from(u) * (t2.hi[0] - t2.lo[0]),
        t2.lo[1] + f64::from(v) * (t2.hi[1] - t2.lo[1]),
    ];
    let world = spun.apply(local);
    // Estava em (4, 0); girado 90°, tem de estar em (0, 4).
    assert!(
        world[0].abs() < 1e-9 && (world[1] - 4.0).abs() < 1e-9,
        "o ponto afastado nao girou com a forma: {world:?}"
    );
}

/// Largar no CENTRO devolve a ponta ao automático — a única forma de desfazer um port fixo, e
/// onde o olho a procura ("sem preferência" = o meio).
#[test]
fn dropping_on_the_centre_returns_the_end_to_automatic() {
    let t = target(7, Xform::IDENTITY);
    let pinned = ConnectorEnd::Bound {
        target: 7,
        anchor: Anchor::Port { u: 0.0, v: 0.5 },
    };
    let out = resolve_drop(&pinned, [0.05, -0.05], Some(t), Some(t));
    assert_eq!(
        out,
        ConnectorEnd::Bound {
            target: 7,
            anchor: Anchor::Floating
        },
        "o centro tem de voltar ao automatico: {out:?}"
    );
}

/// Largar sobre a BORDA (fora do centro) fixa o port ali — e NÃO volta ao automático. É o
/// contraponto do teste acima: se a zona do centro engolisse a caixa toda, nunca daria para
/// fixar nada.
#[test]
fn dropping_on_the_edge_pins_the_port_there() {
    let t = target(7, Xform::IDENTITY);
    let floating = ConnectorEnd::Bound {
        target: 7,
        anchor: Anchor::Floating,
    };
    // O topo da caixa local (y = 1 ⇒ v = 1).
    let out = resolve_drop(&floating, [0.0, 1.0], Some(t), Some(t));
    let ConnectorEnd::Bound {
        anchor: Anchor::Port { u, v },
        ..
    } = out
    else {
        panic!("a borda tinha de FIXAR o port, e voltou ao automatico: {out:?}");
    };
    assert!((f64::from(u) - 0.5).abs() < 1e-5);
    assert!((f64::from(v) - 1.0).abs() < 1e-5);
}

/// Largar sobre OUTRA forma **religa** a linha — é como se muda o destino de um conector sem
/// apagá-lo e redesenhá-lo.
#[test]
fn dropping_on_another_shape_rebinds_the_line_to_it() {
    let old = ConnectorEnd::Bound {
        target: 7,
        anchor: Anchor::Floating,
    };
    let other = target(99, Xform::IDENTITY);
    let out = resolve_drop(
        &old,
        [0.0, 1.0],
        Some(other),
        Some(target(7, Xform::IDENTITY)),
    );
    assert!(
        matches!(out, ConnectorEnd::Bound { target: 99, .. }),
        "a ponta tinha de religar na forma nova: {out:?}"
    );
}

/// Uma ponta que já estava SOLTA e é largada no vazio continua solta — arrastar a alça não pode
/// inventar um vínculo que nunca existiu.
#[test]
fn a_free_end_dropped_on_nothing_stays_free() {
    let free = ConnectorEnd::Free { at: [1.0, 1.0] };
    let out = resolve_drop(&free, [5.0, 5.0], None, None);
    assert_eq!(out, ConnectorEnd::Free { at: [5.0, 5.0] });
}

/// A alça do FIM ganha o desempate quando as duas coincidem (um conector colapsado num ponto):
/// é a que tem a seta, e é a que o usuário está mirando.
#[test]
fn the_end_handle_wins_when_both_sit_on_the_same_point() {
    let mut scene = VecScene::new();
    let id = scene.push_path(ph2d_vec_scene::line([3.0, 3.0], [3.0, 3.0]));
    let got = hit(&scene, id, [3.0, 3.0], 0.5);
    assert_eq!(got.map(|(s, _)| s), Some(EndSide::End));
    // E longe das duas, nada é pego (a alça não pode roubar o clique do canvas).
    assert!(hit(&scene, id, [9.0, 9.0], 0.5).is_none());
}

/// As alças ficam nas PONTAS da linha cozida — não na bbox dela, não no meio.
#[test]
fn the_handles_sit_on_the_two_ends_of_the_cooked_line() {
    let mut scene = VecScene::new();
    let id = scene.push_path(ph2d_vec_scene::line([0.0, 0.0], [10.0, 4.0]));
    let (a, b) = handle_points(&scene, id).expect("as duas alcas");
    assert_eq!(a, [0.0, 0.0]);
    assert_eq!(b, [10.0, 4.0]);
    // Uma forma FECHADA não é um conector; o módulo não se aplica, mas não pode explodir.
    let r = scene.push_path(rectangle([0.0, 0.0], [1.0, 1.0]));
    assert!(handle_points(&scene, r).is_some());
}
