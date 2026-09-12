//! **O gate que ficou na SHELL porque o SUJEITO dele ficou aqui.**
//!
//! ⚠️ A lei da pele mudou-se para [`ph2d_skeleton_live`] (W2/L4 Fase B, 2.ª volta) e os gates dela
//! foram com ela — **menos este**. Ele reproduz a SEQUÊNCIA do smoke (montar a forma, criar a
//! corrente, POSAR, e só então ver a pele responder), e a pose vive no `crate::bone_pose`, que é
//! da família do esqueleto e ainda está na shell.
//!
//! ⭐ É o HOWTO §1.2 à letra: *os testes seguem o SUJEITO, não o ficheiro*. Levá-lo para a crate
//! obrigaria a levar o `bone_pose` — e com ele o `bone_limit`, o `skeleton_goal` e o resto do
//! `bone_gesture`, que é a família do esqueleto inteira a sair da shell. Essa é outra wave.
//!
//! ⛔⛔ **E ele só foi encontrado por `--all-targets`:** o `cargo check -p ph2d-skeleton-live`
//! ficou VERDE sobre a crate com este gate partido lá dentro, porque um `#[cfg(test)]` não entra
//! num `check` normal. É a mesma família da §2.6 — o que falha só quando o teste é COMPILADO.

use ph2d_ecs::{Entity, SimWorld};
use ph2d_skeleton_live::test_support::{pior_desvio, quadro};
use ph2d_vec_entities::entity_map::VecEntityMap;
use ph2d_vec_scene::VecScene;

/// ⚠️ **SONDA da cena de smoke** (report do Enio, 2026-09-06: *"o bind não funciona e nenhuma forma
/// pode ser deformada"*): a MESMA sequência do `vec_bone_smoke`, com as MESMAS portas.
#[test]
fn probe_the_smoke_sequence() {
    let mut sim = SimWorld::default();
    let mut scene = VecScene::new();
    let mut map = VecEntityMap::new();
    let id = scene.push_path(crate::build_smoke::shape(
        ph2d_vec_scene::ShapeKind::RoundRect,
        [-8.5, 2.0],
        [-1.5, 3.0],
        &[0.5],
        [230, 170, 90],
    ));
    ph2d_vec_entities::entities::sync(&mut sim, &mut scene, &mut map);
    // A cadeia, como o smoke a faz: pela porta do GESTO, em coordenadas de MUNDO.
    let mut pai: Option<Entity> = None;
    let mut raiz = None;
    for i in 0..3 {
        let x = -8.2 + f64::from(i) * 2.1333;
        let bits =
            crate::bone_gesture::create(&mut sim, pai, [x, 2.5], [x + 2.1333, 2.5]).expect("osso");
        pai = Some(Entity::from_bits(bits));
        raiz = raiz.or(pai);
    }
    eprintln!(
        "[probe] ossos = {:?}",
        ph2d_skeleton_live::skin_live::bone_segments(&sim)
    );
    let n = ph2d_skeleton_live::skin_live::bind(&mut sim, &scene, &map, &[id], raiz);
    eprintln!("[probe] bind devolveu {n}");
    let antes = quadro(&sim, &mut scene, id);
    // Posa o ÚLTIMO osso pela porta do gesto.
    let ultimo = pai.expect("ultimo");
    let ok = crate::bone_pose::pose(
        &mut sim,
        ultimo,
        [-2.0, 6.0],
        ph2d_skeleton_render::BonePart::Body,
    );
    eprintln!("[probe] pose devolveu {ok}");
    let depois = quadro(&sim, &mut scene, id);
    eprintln!("[probe] desvio = {}", pior_desvio(&antes, &depois));
    assert!(
        pior_desvio(&antes, &depois) > 0.5,
        "a forma NAO deformou - reproduzido o report"
    );
}
