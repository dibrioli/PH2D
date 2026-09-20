//! Os gates da SELECÇÃO — *quem um canvas ancora, e contra que rectângulo*.
//!
//! ⚠️ A aritmética da caixa efectiva está provada no `ph2d-hud`, e o que se DESENHA está provado na
//! shell (onde vive a `LiveGeometry`). Aqui afirma-se só a escolha, que é a metade barata: sem
//! cena, sem device, sem passe de layout.

use super::{Ancorado, ancorados};
use crate::hud_bridge::View;
use ph2d_ecs::{ChildOf, Entity, Fit, SimWorld, Transform, UiCanvas, VecAnchors, VecFrame};
use ph2d_vec_entities::entities::VecEntityMap;
use ph2d_vec_scene::{VecScene, rectangle};

const REF: [f64; 4] = [-16.0, -9.0, 16.0, 9.0];

fn vista(hw: f32, hh: f32) -> View {
    View {
        center: [0.0, 0.0],
        half: [hw, hh],
    }
}

/// Um canvas com `n` filhos, e — se `com_moldura` — uma moldura com um filho dela ao lado.
fn cena(n: usize, com_moldura: bool) -> (SimWorld, VecEntityMap, Vec<Entity>, Option<Entity>) {
    let mut sim = SimWorld::default();
    let mut scene = VecScene::new();
    let mut map = VecEntityMap::new();
    for _ in 0..n {
        scene.push_path(rectangle([0.0, 0.0], [10.0, 10.0]));
    }
    let (f, k) = if com_moldura {
        (
            Some(scene.push_path(rectangle([0.0, 0.0], [32.0, 18.0]))),
            Some(scene.push_path(rectangle([0.0, 0.0], [10.0, 10.0]))),
        )
    } else {
        (None, None)
    };
    ph2d_vec_entities::entities::sync(&mut sim, &mut scene, &mut map);

    let root = sim
        .world_mut()
        .spawn((
            Transform::default(),
            UiCanvas {
                ref_w: 32.0,
                ref_h: 18.0,
                fit: Fit::Keep,
            },
        ))
        .id();
    let ids: Vec<_> = scene.paths().iter().map(|p| p.id).collect();
    let kids: Vec<Entity> = ids
        .iter()
        .take(n)
        .map(|id| {
            let e = Entity::from_bits(map[id]);
            sim.world_mut().entity_mut(e).insert((
                ChildOf(root),
                VecAnchors {
                    min: [1.0, 0.0],
                    max: [1.0, 0.0],
                    base: REF,
                },
            ));
            e
        })
        .collect();
    let de_moldura = f.zip(k).map(|(f, k)| {
        let frame = Entity::from_bits(map[&f]);
        sim.world_mut().entity_mut(frame).insert(VecFrame);
        let kid = Entity::from_bits(map[&k]);
        sim.world_mut().entity_mut(kid).insert((
            ChildOf(frame),
            VecAnchors {
                min: [1.0, 0.0],
                max: [1.0, 0.0],
                base: REF,
            },
        ));
        kid
    });
    (sim, map, kids, de_moldura)
}

/// ⭐⭐ **A moldura de um filho de canvas é a caixa EFECTIVA** — a de referência mais a banda.
///
/// A conta, fechada à mão: referência `32×18` numa vista de `42×18` ⇒ `Keep` dá escala `1,0` e uma
/// banda de `(42 − 32)/2 = 5` ⇒ a efectiva é `[−21, −9, 21, 9]`.
///
/// **Mutação que deve sangrar:** devolver a caixa de referência.
#[test]
fn a_moldura_de_um_filho_de_canvas_e_a_caixa_efectiva() {
    let (sim, map, kids, _) = cena(1, false);
    let v = ancorados(&sim, &map, Some(vista(21.0, 9.0)));
    assert_eq!(
        v,
        vec![Ancorado {
            kid: kids[0],
            now: [-21.0, -9.0, 21.0, 9.0],
            scale: [1.0, 1.0],
        }]
    );
}

/// ⛔ **Sem vista, ninguém ancora** — a mesma lei que a raiz do canvas já obedece.
///
/// ⚠️ Metade NEGATIVA: sem ela, uma implementação que caísse num valor de fábrica ancoraria contra
/// uma vista inventada, e o placar saltaria para um canto que ninguém escolheu.
#[test]
fn sem_vista_ninguem_ancora() {
    let (sim, map, _, _) = cena(2, false);
    assert!(ancorados(&sim, &map, None).is_empty());
}

/// ⛔⛔ **Um filho de MOLDURA não entra na lista** — as duas populações são disjuntas.
///
/// ⚠️ É a lei que permite os dois passes conviverem sem um dono de tabela novo. Sem ela, um filho
/// de moldura seria ancorado DUAS vezes, e a segunda leria a caixa que a primeira acabou de mover.
#[test]
fn um_filho_de_moldura_nao_entra_na_lista() {
    let (sim, map, kids, de_moldura) = cena(1, true);
    let v = ancorados(&sim, &map, Some(vista(21.0, 9.0)));
    let vistos: Vec<Entity> = v.iter().map(|a| a.kid).collect();
    assert!(vistos.contains(&kids[0]), "o filho do canvas ficou de fora");
    assert!(
        !vistos.contains(&de_moldura.expect("a cena tem moldura")),
        "o filho de uma MOLDURA entrou na lista do canvas — as populacoes cruzaram-se"
    );
}

/// ⚠️ **Um filho SEM regra não entra**, mesmo sendo filho do canvas — a regra é do filho.
#[test]
fn um_filho_sem_regra_nao_entra() {
    let (mut sim, map, kids, _) = cena(2, false);
    sim.world_mut().entity_mut(kids[1]).remove::<VecAnchors>();
    let v = ancorados(&sim, &map, Some(vista(21.0, 9.0)));
    assert_eq!(v.len(), 1, "entrou quem nao tem regra");
    assert_eq!(v[0].kid, kids[0]);
}

/// ⭐ **A moldura é medida UMA vez e servida a todos os filhos** — dois filhos do mesmo canvas leem
/// exactamente o mesmo rectângulo.
///
/// ⚠️ Não é só economia: dois rectângulos calculados em momentos diferentes seriam duas respostas à
/// mesma pergunta, e divergiriam no dia em que a vista mudasse a meio da varredura.
#[test]
fn os_filhos_do_mesmo_canvas_leem_o_mesmo_rectangulo() {
    let (sim, map, _, _) = cena(3, false);
    let v = ancorados(&sim, &map, Some(vista(21.0, 9.0)));
    assert_eq!(v.len(), 3);
    assert!(
        v.windows(2)
            .all(|p| p[0].now == p[1].now && p[0].scale == p[1].scale),
        "dois filhos do mesmo canvas leram rectangulos diferentes"
    );
}
