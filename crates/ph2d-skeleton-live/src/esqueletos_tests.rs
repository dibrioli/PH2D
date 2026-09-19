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

/// ⭐⭐⭐ **A PERGUNTA É POR OSSO, e é isso que faz uma cena MISTA ser legível** (report do dono,
/// 2026-09-18: *«melhor montar uma cena específica para me mostrar isso»*).
///
/// ⛔⛔⛔ **A 1.ª redacção perguntava à CENA**, e numa cena com as duas mídias ela respondia `true`
/// para **todos** os ossos — a cena que o dono pediu não mostraria diferença nenhuma. A premissa que
/// a justificava era minha e **caiu**: eu escrevi que o `SkinBind` não guarda os ossos, e ele
/// guarda (`Tendon::bone`, um `StableId`).
#[test]
fn o_envelope_manda_no_osso_que_uma_forma_vectorial_usa_e_so_nele() {
    use ph2d_ecs::Transform;

    let mut sim = SimWorld::default();
    let a = cadeias(&mut sim, 1)[0];
    let b = cadeias(&mut sim, 1)[0];
    assert_ne!(a, b, "as duas cadeias tem de ser esqueletos diferentes");
    ph2d_ecs::assign_missing_stable_ids(sim.world_mut());
    let id = |e: Entity| {
        sim.world()
            .get::<ph2d_ecs::StableId>(e)
            .copied()
            .expect("id")
    };

    // Uma FORMA VECTORIAL presa ao esqueleto `a`, e uma IMAGEM presa ao `b`.
    let tendao = |e: Entity| ph2d_skeleton_ecs::SkinBind {
        source: Vec::new(),
        tendons: vec![ph2d_skeleton_ecs::Tendon {
            bone: id(e),
            rest: [1.0, 0.0, 0.0, 1.0, 0.0, 0.0],
        }],
    };
    let forma = tendao(a);
    let imagem = tendao(b);
    sim.world_mut().spawn((Transform::IDENTITY, forma));
    sim.world_mut().spawn((
        Transform::IDENTITY,
        ph2d_render::Sprite::atlas(0, [1.0, 1.0], [1.0; 4]),
        imagem,
    ));

    assert!(
        crate::esqueletos::o_envelope_deste_osso_manda(&sim, a),
        "o osso com a FORMA VECTORIAL perdeu o envelope: ali ele manda pela lei euclidiana, e \
         esconde-lo tira um controlo VIVO"
    );
    assert!(
        !crate::esqueletos::o_envelope_deste_osso_manda(&sim, b),
        "o osso so' com IMAGEM manteve o envelope: numa cena MISTA a pergunta larga acende os dois, \
         e a cena que mostra a diferenca deixa de a mostrar"
    );
}

/// ⭐⭐⭐ **A MANCHA E A ALÇA DO ENVELOPE PASSAM PELA MESMA PORTA** (report do dono, 2026-09-18:
/// *«o gizmo do envelope fica sempre visível mesmo quando não é usado?»* — sim, ficava).
///
/// ⚠️ **A lei entra na `influence_region` e não em quem desenha, porque essa porta tem DOIS
/// consumidores** — o desenho da mancha e o **hit-test da alça**. *Curar só o pintor deixaria o
/// artista a arrastar uma alça invisível, que é pior do que a mancha a mais.*
///
/// ⛔⛔ **A 1.ª redacção deste gate media a lei por CENA e a premissa dele MORREU** quando ela passou
/// a ser por OSSO — ele fica com a morte visível no diff, a medir a PORTA (que é o que o desenho e
/// o pick chamam) em vez da lei, que já tem o gate dela acima.
#[test]
fn a_mancha_e_a_alca_passam_pela_mesma_porta() {
    use ph2d_ecs::Transform;

    let mut sim = SimWorld::default();
    let so_imagem = cadeias(&mut sim, 1)[0];
    let com_forma = cadeias(&mut sim, 1)[0];
    ph2d_ecs::assign_missing_stable_ids(sim.world_mut());
    let id = |e: Entity| {
        sim.world()
            .get::<ph2d_ecs::StableId>(e)
            .copied()
            .expect("id")
    };
    let tendao = |e: Entity| ph2d_skeleton_ecs::SkinBind {
        source: Vec::new(),
        tendons: vec![ph2d_skeleton_ecs::Tendon {
            bone: id(e),
            rest: [1.0, 0.0, 0.0, 1.0, 0.0, 0.0],
        }],
    };
    let forma = tendao(com_forma);
    let imagem = tendao(so_imagem);
    sim.world_mut().spawn((Transform::IDENTITY, forma));
    sim.world_mut().spawn((
        Transform::IDENTITY,
        ph2d_render::Sprite::atlas(0, [1.0, 1.0], [1.0; 4]),
        imagem,
    ));

    assert!(
        crate::skin_live::influence_region(&sim, so_imagem.to_bits()).is_none(),
        "a mancha foi desenhada num osso que so' uma IMAGEM usa: ela diz «ate' onde este osso \
         alcanca» sobre uma lei que nao usa alcance nenhum, e a alca dela fica agarravel por cima"
    );
    assert!(
        crate::skin_live::influence_region(&sim, com_forma.to_bits()).is_some(),
        "a mancha sumiu do osso que uma FORMA VECTORIAL usa: o artista perdeu o controlo do \
         alcance exactamente onde ele decide a deformacao"
    );
}
