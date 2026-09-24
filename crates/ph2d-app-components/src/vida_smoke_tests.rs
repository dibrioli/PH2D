//! Os gates da cena da VIDA — ver o cabeçalho de [`super`].
//!
//! ⚠️ **Eles correm a LEI com os números DA CENA**: cada receita é lida do mundo que o [`montar`]
//! deixou e clonada para uma corrida da ponte, com as balas a sair da receita da bala. *Escrever os
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

/// O corpo, a forma e a vida de uma receita, pelo nome.
fn receita(sim: &mut SimWorld, nome: &str) -> (RigidBody, Collider, Health) {
    let w = sim.world_mut();
    let mut q = w.query::<(&Name, &RigidBody, &Collider, &Health)>();
    q.iter(w)
        .find(|(n, ..)| n.as_str() == nome)
        .map(|(_, r, c, h)| (*r, *c, h.clone()))
        .unwrap_or_else(|| panic!("a receita «{nome}» nao esta' na cena"))
}

fn bala(sim: &mut SimWorld) -> (RigidBody, Collider, ProjectileMotion, Damage) {
    let w = sim.world_mut();
    let mut q = w.query::<(&Name, &RigidBody, &Collider, &ProjectileMotion, &Damage)>();
    q.iter(w)
        .find(|(n, ..)| n.as_str() == "Bala")
        .map(|(_, r, c, p, d)| (*r, *c, *p, d.clone()))
        .expect("a receita da bala")
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
