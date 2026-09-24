//! Os gates da cena da VIDA — ver o cabeçalho de [`super`].
//!
//! ⚠️ **Eles correm a LEI com os números DA CENA**: cada alvo e cada bala são a CÓPIA que a fábrica
//! faria do molde que o [`montar`] deixou (a porta de cópia do produto), clonada para uma corrida da
//! ponte. *Escrever os
//! números à mão aqui mediria uma cena que o dono não vê.* ⛔ O que nenhum deles mede é o CLIQUE e a
//! fábrica: esses são do dono, e o `PH2D_VIDA_SMOKE=1` é onde vivem.

use super::*;
use ph2d_ecs::SimWorld;
use ph2d_physics_ecs::PhysicsBridge;

fn mundo() -> SimWorld {
    let mut sim = SimWorld::new();
    let _ = montar(sim.world_mut(), 1);
    sim
}

/// ⛔⛔ **A CÓPIA que a fábrica faria de um molde, pelo nome** — pela porta de cópia do PRODUTO
/// ([`crate::instantiate::instantiate_master_many`]) e com o registo que o produto monta.
///
/// ⚠️ **Nasceu do smoke do dono** (*«ninguém sumiu ao levar muitos tiros»*): a 1.ª redacção destes
/// gates lia a RECEITA do mundo e montava o alvo à mão — e a cópia profunda leva só os componentes
/// REGISTADOS, que a `Health`/`Damage` não eram. *Uma fixtura que monta à mão o que o produto COPIA
/// mede outro programa.* ⇒ os números saem da CÓPIA, e uma cópia sem vida faz o gate reprovar alto.
fn copia(sim: &mut SimWorld, nome: &str) -> ph2d_ecs::Entity {
    let molde = {
        let w = sim.world_mut();
        let mut q =
            w.query_filtered::<(ph2d_ecs::Entity, &Name), bevy_ecs::query::With<MasterRoot>>();
        q.iter(w)
            .find(|(_, n)| n.as_str() == nome)
            .map(|(e, _)| e)
            .unwrap_or_else(|| panic!("o molde «{nome}» nao esta' na cena"))
    };
    let registo = crate::component_registry_for_tests::registo();
    let (mut sc, mut mp) = crate::instance_docs::empty_docs();
    let mut docs = crate::instance_docs::OwnedDocs {
        vec_scene: &mut sc,
        vec_entities: &mut mp,
    };
    crate::instantiate::instantiate_master_many(
        sim,
        &registo,
        molde,
        None,
        &mut docs,
        crate::instantiate::ArtLink::Own,
        1,
    )
    .unwrap_or_else(|e| panic!("«{nome}» recusou a cópia: {e:?}"))[0]
}

/// O corpo, a forma e a vida da CÓPIA de um alvo, pelo nome.
fn receita(sim: &mut SimWorld, nome: &str) -> (RigidBody, Collider, Health) {
    let c = copia(sim, nome);
    let w = sim.world();
    (
        *w.get::<RigidBody>(c).expect("a cópia sem corpo"),
        *w.get::<Collider>(c).expect("a cópia sem forma"),
        w.get::<Health>(c)
            .unwrap_or_else(|| {
                panic!("a CÓPIA de «{nome}» nasceu SEM VIDA — a fábrica não a copia")
            })
            .clone(),
    )
}

fn bala(sim: &mut SimWorld) -> (RigidBody, Collider, ProjectileMotion, Damage) {
    let c = copia(sim, "Bala");
    let w = sim.world();
    (
        *w.get::<RigidBody>(c).expect("a bala sem corpo"),
        *w.get::<Collider>(c).expect("a bala sem forma"),
        *w.get::<ProjectileMotion>(c).expect("a bala sem voo"),
        w.get::<Damage>(c)
            .expect("a CÓPIA da bala nasceu SEM DANO — a fábrica não a copia")
            .clone(),
    )
}

/// Atira até `max` balas (uma de cada vez, a bala acabada é tirada da cena como o dreno faria) e
/// devolve **quantos tiros** a vida levou a morrer — `None` se não morreu.
fn tiros_ate_morrer(
    alvo: (RigidBody, Collider, Health),
    bala: &(RigidBody, Collider, ProjectileMotion, Damage),
    max: u32,
) -> Option<u32> {
    let mut sim = SimWorld::new();
    let a = sim
        .world_mut()
        .spawn((
            alvo.0,
            alvo.1,
            alvo.2,
            Transform::from_translation(Vec2::new(X_ALVOS, 0.0)),
        ))
        .id();
    let mut ponte = PhysicsBridge::new();
    let mut t = 0_u64;
    ponte.dispatch(&mut sim, true, t);
    for tiro in 1..=max {
        let b = sim
            .world_mut()
            .spawn((
                bala.0,
                bala.1,
                bala.2,
                bala.3.clone(),
                Transform::from_translation(Vec2::new(-2.0, 0.0)),
            ))
            .id();
        let mut acabou = false;
        for _ in 0..240 {
            t += 1;
            ponte.dispatch(&mut sim, true, t);
            if ponte.projectile_done().iter().any(|(e, _)| *e == b) {
                acabou = true;
                break;
            }
        }
        assert!(acabou, "o tiro {tiro} nunca chegou ao alvo");
        sim.world_mut().despawn(b);
        if ponte.health_of(a).is_some_and(|s| s.vida().morta()) {
            return Some(tiro);
        }
    }
    None
}

/// ⭐⭐⭐ **Vermelho morre ao 1.º tiro, laranja ao 2.º, roxo ao 3.º — e o aliado nunca.**
///
/// ⚠️ **É o passo (2)–(5) do roteiro**, corrido com os números da cena. Cada linha é a mesma lei com
/// uma vida diferente, e a última é o CONTROLO: a mesma bala, a mesma vida de `10` que o vermelho,
/// e a única diferença é a EQUIPA.
///
/// **Mutações que devem sangrar:** a vida do laranja a `10` · o `team` do aliado a `monstros` · o
/// `fere` do `Damage` a devolver sempre `true`.
#[test]
fn cada_alvo_morre_ao_tiro_que_a_vida_dele_diz() {
    let mut sim = mundo();
    let b = bala(&mut sim);
    let esperado = [Some(1), Some(2), Some(3), None];
    for (alvo, n) in ALVOS.iter().zip(esperado) {
        let r = receita(&mut sim, alvo.0);
        assert_eq!(
            tiros_ate_morrer(r, &b, 5),
            n,
            "«{}» (vida {}, equipa {})",
            alvo.0,
            alvo.1,
            alvo.2
        );
    }
}

/// ⭐⭐ **A morte é da VIDA, e não de uma tabela** — a diferença que a W2 compra sobre a cena do
/// golpe (#24), onde cada alvo tinha duas linhas `Destroy`.
#[test]
fn nenhum_alvo_precisa_de_tabela_para_morrer() {
    let mut sim = mundo();
    let w = sim.world_mut();
    let mut q = w.query::<(&Name, &Health, Option<&ph2d_ecs::SignalActions>)>();
    let alvos: Vec<(String, bool)> = q
        .iter(w)
        .map(|(n, _, t)| (n.as_str().to_owned(), t.is_some()))
        .collect();
    assert_eq!(
        alvos.len(),
        ALVOS.len(),
        "a cena tem de ter as quatro receitas: {alvos:?}"
    );
    assert!(
        alvos.iter().all(|(_, t)| !t),
        "um alvo ganhou tabela: {alvos:?}"
    );
}

/// ⭐ **A coluna cabe na banda visível** (`+4,09` / `−1,19` m, medida na cena da arma) — senão o
/// passo (5) manda atirar num quadrado fora do ecrã.
#[test]
fn a_coluna_cabe_na_banda_visivel() {
    for (nome, .., y) in ALVOS {
        assert!(
            y + LADO / 2.0 <= 4.09 && y - LADO / 2.0 >= -1.19,
            "«{nome}» sai da banda em y = {y}"
        );
    }
}

/// ⛔⛔⛔ **O que o molde tem, a CÓPIA tem** — pela porta de cópia do PRODUTO
/// ([`crate::instantiate::instantiate_master_many`], a que a fábrica chama) e com o registo que o
/// produto monta.
///
/// ⚠️ **Nasceu do smoke do dono** (*«ninguém sumiu ao levar muitos tiros»*): a cópia profunda leva
/// **só os componentes REGISTADOS**, e a `Health`/`Damage` não estavam — os alvos nasciam sem vida
/// e as balas sem dano, em silêncio. ⛔ O gate `cada_alvo_morre_ao_tiro_que_a_vida_dele_diz` era
/// cego a isto **por construção** na 1.ª redacção (montava o alvo À MÃO a partir da receita); hoje
/// ele também passa pela [`copia`], e este fica como a régua CAMPO A CAMPO.
#[test]
fn o_que_o_molde_tem_a_copia_tem() {
    let mut sim = mundo();
    let registo = crate::component_registry_for_tests::registo();
    let moldes: Vec<(ph2d_ecs::Entity, String)> = {
        let w = sim.world_mut();
        let mut q =
            w.query_filtered::<(ph2d_ecs::Entity, &Name), bevy_ecs::query::With<MasterRoot>>();
        q.iter(w).map(|(e, n)| (e, n.as_str().to_owned())).collect()
    };
    assert_eq!(
        moldes.len(),
        ALVOS.len() + 1,
        "quatro alvos e a bala: {moldes:?}"
    );
    let (mut sc, mut mp) = crate::instance_docs::empty_docs();
    let mut docs = crate::instance_docs::OwnedDocs {
        vec_scene: &mut sc,
        vec_entities: &mut mp,
    };
    for (molde, nome) in moldes {
        let copia = crate::instantiate::instantiate_master_many(
            &mut sim,
            &registo,
            molde,
            None,
            &mut docs,
            crate::instantiate::ArtLink::Own,
            1,
        )
        .unwrap_or_else(|e| panic!("«{nome}» recusou a cópia: {e:?}"))[0];
        let w = sim.world();
        assert_eq!(
            w.get::<Health>(copia),
            w.get::<Health>(molde),
            "«{nome}»: a VIDA não chegou à cópia"
        );
        assert_eq!(
            w.get::<Damage>(copia),
            w.get::<Damage>(molde),
            "«{nome}»: o DANO não chegou à cópia"
        );
        assert!(
            w.get::<Health>(molde).is_some() || w.get::<Damage>(molde).is_some(),
            "«{nome}»: o molde não tem nem vida nem dano — o gate não mede nada"
        );
    }
}
