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

/// Uma pele sem conteúdo — o que interessa a esta lei é a PRESENÇA do componente.
fn pele_vazia() -> ph2d_skeleton_ecs::SkinBind {
    ph2d_skeleton_ecs::SkinBind {
        source: Vec::new(),
        tendons: Vec::new(),
    }
}

/// ⭐⭐⭐ **O ENVELOPE MANDA ONDE HÁ FORMA VECTORIAL, E SÓ AÍ** (report do dono, 2026-09-18).
///
/// ⚠️ **As DUAS metades, porque as curas são opostas:** uma lei que respondesse sempre `true`
/// deixaria o campo à vista num rig só de imagens (o controlo morto que o dono apanhou), e uma que
/// respondesse sempre `false` esconderia o envelope das formas vectoriais, que é um controlo VIVO.
#[test]
fn o_envelope_manda_onde_ha_forma_vectorial_e_so_ai() {
    use ph2d_ecs::Transform;

    // (a) cena VAZIA — nada preso, e o envelope pode vir a mandar ⇒ a resposta conservadora.
    let mut sim = SimWorld::default();
    cadeias(&mut sim, 1);
    assert!(
        !ha_forma_vectorial_presa(&sim),
        "uma cena sem NADA preso leu «ha' forma vectorial»: a lei esta' a contar entidades que \
         nao tem pele"
    );

    // (b) uma IMAGEM presa — o padrão-ouro manda, o envelope é inerte.
    let imagem = sim
        .world_mut()
        .spawn((
            Transform::IDENTITY,
            ph2d_render::Sprite::atlas(0, [1.0, 1.0], [1.0; 4]),
            pele_vazia(),
        ))
        .id();
    assert!(
        !ha_forma_vectorial_presa(&sim),
        "uma IMAGEM presa leu «ha' forma vectorial»: o envelope voltaria a ser pintado no rig \
         onde ele e' provadamente inerte, que e' o report do dono a' letra"
    );

    // (c) e uma FORMA VECTORIAL presa — sem `Sprite`, a lei euclidiana manda e o campo volta.
    sim.world_mut().spawn((Transform::IDENTITY, pele_vazia()));
    assert!(
        ha_forma_vectorial_presa(&sim),
        "uma FORMA VECTORIAL presa leu «nao ha'»: o envelope dela ficaria escondido, e ali ele \
         manda como sempre — esconder um controlo VIVO e' pior do que mostrar um inerte"
    );

    // ⚠️ E o CONTROLO de que é a PELE que decide, não a existência da entidade: tirar a pele à
    // forma vectorial devolve a cena ao estado (b).
    sim.world_mut()
        .entity_mut(imagem)
        .remove::<ph2d_skeleton_ecs::SkinBind>();
    assert!(
        ha_forma_vectorial_presa(&sim),
        "tirar a pele a' IMAGEM mudou a resposta: a lei esta' a olhar para a entidade errada"
    );
}
