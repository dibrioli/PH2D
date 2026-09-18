//! Os gates da topologia da árvore de ossos.

use super::*;
use ph2d_ecs::Transform;

use crate::esqueletos_tests_support::cadeias;

/// ⭐ **Contar RAÍZES é o que distingue «uma cadeia de três» de «três esqueletos».**
///
/// ⚠️ **As duas metades são dois defeitos:** contar ossos leria `3` numa cadeia só (e o botão
/// recusaria sempre), e contar só a primeira raiz leria `1` com três esqueletos (e ele nunca
/// recusaria).
#[test]
fn as_raizes_contam_esqueletos_e_nao_ossos() {
    for n in 1..=3_usize {
        let mut sim = SimWorld::default();
        let raizes = cadeias(&mut sim, n);
        let lidas = bone_roots(&sim);
        assert_eq!(
            lidas.len(),
            n,
            "com {n} cadeia(s) de TRES ossos a porta leu {} esqueleto(s) — ela esta' a contar \
             ossos, e o botao passaria a recusar sobre uma cena com um esqueleto so'",
            lidas.len()
        );
        for r in raizes {
            assert!(
                lidas.contains(&r),
                "a raiz {r:?} nao esta' na lista: a porta subiu para o sitio errado"
            );
        }
    }
}

/// ⚠️ **Um esqueleto pendurado dentro de um GRUPO continua a ser UM esqueleto** — a subida pára no
/// primeiro pai que não é osso, e é isso que permite arrumar um personagem numa pasta.
#[test]
fn um_esqueleto_dentro_de_um_grupo_continua_a_ser_um() {
    let mut sim = SimWorld::default();
    let raizes = cadeias(&mut sim, 1);
    let grupo = sim.world_mut().spawn(Transform::IDENTITY).id();
    sim.world_mut()
        .entity_mut(raizes[0])
        .insert(ph2d_ecs::ChildOf(grupo));
    let lidas = bone_roots(&sim);
    assert_eq!(
        lidas, raizes,
        "pendurar o esqueleto num grupo mudou a raiz dele: a subida nao parou no primeiro pai \
         que nao e' osso, e o botao passaria a ver esqueletos onde ha' um so'"
    );
}
