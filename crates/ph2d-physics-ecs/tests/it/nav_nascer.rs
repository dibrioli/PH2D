//! ⭐ (o aberto da W9) **Quem nasce junto procura pela fila** — a procura de um agente SEM caminho (que
//! nasceu, foi religado, ou ainda não achou nenhum) é do orçamento do tique, como a do replaneio: cabe
//! enquanto houver folga (e há sempre pelo menos uma), senão espera parado pelo tique seguinte.
//! Os ajudantes são os de [`super::nav_desvio`].

use ph2d_core::Vec2;
use ph2d_ecs::{Entity, SimWorld, Transform};
use ph2d_physics_ecs::{
    BodyKind, Collider, ColliderShape, NavCostArea, NavTarget, PhysicsBridge, RigidBody,
};

use super::nav_desvio::{agente, pos, regiao};

const N: usize = 12;

/// `N` agentes numa coluna à esquerda, cada um para o seu ponto à direita.
fn nascem() -> (SimWorld, Vec<Entity>) {
    let mut sim = SimWorld::new();
    regiao(&mut sim);
    let quem = (0..N)
        .map(|i| {
            let y = -4.4 + 0.8 * i as f32;
            agente(
                &mut sim,
                &format!("G{i}"),
                (-6.0, y),
                NavTarget::Point([6.0, y]),
                true,
            )
        })
        .collect();
    (sim, quem)
}

/// Os agentes pela ordem das ENTIDADES (a da vez; não é a de nascimento).
fn pela_ordem(quem: &[Entity]) -> Vec<Entity> {
    let mut v = quem.to_vec();
    v.sort();
    v
}

/// Quais já TÊM um caminho. ⚠️ (W15) Não «quais procuraram»: a procura abre-se no pedido (as que não
/// custam nada acabam logo), e quem foi servido é quem tem o caminho.
fn procuraram(b: &PhysicsBridge, quem: &[Entity]) -> Vec<bool> {
    quem.iter()
        .map(|&e| b.nav_agent(e).is_some_and(|r| !r.path.is_empty()))
        .collect()
}

/// A lei da FILA, numa faixa: sem o passo em paralelo e sem o alvo à vista (cada um tem os gates dele,
/// `nav_fatias`; aqui todo alvo está à vista, e a recta não espera a fila — plano 30 §25, C2).
fn uma_faixa(orc: u64) -> PhysicsBridge {
    let mut b = PhysicsBridge::new();
    b.set_nav_replan_budget(orc);
    b.set_nav_parallel(0);
    b.set_nav_sight(false);
    b
}

/// Com o orçamento de UM nó, os que nascem juntos ganham o caminho UM por tique, pela ordem das
/// entidades — e todos acabam a andar. CONTROLO: com o orçamento de fábrica têm-no todos no 1.º tique.
#[test]
fn os_que_nascem_juntos_procuram_pela_fila() {
    let (mut sim, quem) = nascem();
    let quem = pela_ordem(&quem);
    let mut b = uma_faixa(1);
    for t in 1..=N as u64 + 5 {
        b.dispatch(&mut sim, true, t);
        let ja = procuraram(&b, &quem);
        let k = (t as usize).min(N);
        assert_eq!(
            ja,
            (0..N).map(|i| i < k).collect::<Vec<_>>(),
            "no tique {t} procuraram {ja:?}"
        );
    }
    // A fixtura contém o fenómeno: cada procura gasta pelo menos o orçamento (um nó) — ele morde.
    for &e in &quem {
        let nos = b.nav_agent(e).map_or(0, |r| r.last_work);
        assert!(nos >= 1, "a procura de {e:?} gastou {nos} nós");
    }
    let mut c = PhysicsBridge::new();
    let (mut sim2, quem2) = nascem();
    c.dispatch(&mut sim2, true, 1);
    assert!(
        procuraram(&c, &quem2).iter().all(|&p| p),
        "o CONTROLO: com o orçamento de fábrica todos procuram no 1.º tique"
    );
}

/// Quem espera pela vez fica PARADO, e anda quando a vez chega: no fim todos chegam.
#[test]
fn quem_espera_pela_vez_fica_parado_e_depois_chega() {
    let (mut sim, quem) = nascem();
    let quem = pela_ordem(&quem);
    let partida: Vec<(f32, f32)> = quem.iter().map(|&e| pos(&sim, e)).collect();
    let mut b = uma_faixa(1);
    b.dispatch(&mut sim, true, 1);
    b.dispatch(&mut sim, true, 2);
    // No 2.º tique o último da vez ainda não procurou: está onde nasceu.
    assert_eq!(pos(&sim, quem[N - 1]), partida[N - 1], "o último já andou");
    for t in 3..=600 {
        b.dispatch(&mut sim, true, t);
    }
    for (i, &e) in quem.iter().enumerate() {
        let (x, y) = pos(&sim, e);
        assert!(
            (x - 6.0).abs() < 0.2 && (y - partida[i].1).abs() < 0.2,
            "o {i}.º da vez não chegou: ({x}, {y})"
        );
    }
}

/// ⭐ **Um scrub para o MEIO dos nascimentos devolve a mesma corrida** — quem já procurou e quem
/// espera é estado do agente, e vai no anel.
#[test]
fn um_scrub_a_meio_dos_nascimentos_devolve_a_mesma_corrida() {
    const FIM: u64 = 40;
    // O anel guarda uma âncora de 10 em 10 tiques: o 7 semeia do 0 e o 13 do 10 — os dois a meio.
    let (mut sim, quem) = nascem();
    let mut b = PhysicsBridge::new();
    b.set_nav_replan_budget(1);
    let posicoes =
        |sim: &SimWorld| -> Vec<(f32, f32)> { quem.iter().map(|&e| pos(sim, e)).collect() };
    let mut primeira = Vec::new();
    for t in 1..=FIM {
        b.dispatch(&mut sim, true, t);
        primeira.push(posicoes(&sim));
    }
    for meio in [7u64, 11] {
        b.dispatch(&mut sim, false, meio);
        assert_eq!(
            posicoes(&sim),
            primeira[(meio - 1) as usize],
            "o scrub para {meio}"
        );
        let resto: Vec<_> = ((meio + 1)..=FIM)
            .map(|t| {
                b.dispatch(&mut sim, true, t);
                posicoes(&sim)
            })
            .collect();
        assert_eq!(
            resto,
            primeira[meio as usize..].to_vec(),
            "o resto depois de {meio}"
        );
    }
}

/// (W14) **Na lama, a vez conta o TRABALHO, não os nós** — com o orçamento igual ao trabalho da 1.ª
/// procura, só ela cabe no 1.º tique (contados em nós, a 1.ª gastava menos e o 2.º entrava).
/// CONTROLO: com o orçamento de fábrica procuram todos.
#[test]
fn na_lama_a_vez_de_quem_nasce_conta_o_trabalho() {
    let nascem_na_lama = || {
        let (mut sim, quem) = nascem();
        // Uma faixa de lama de alto a baixo: todo o caminho a atravessa (a procura é a ponderada).
        sim.world_mut().spawn((
            RigidBody {
                kind: BodyKind::Static,
            },
            Collider {
                shape: ColliderShape::Cuboid {
                    half_x: 1.5,
                    half_y: 7.0,
                },
                is_sensor: true,
                ..Collider::default()
            },
            NavCostArea {
                cost: 4.0,
                forbidden: false,
            },
            Transform::from_translation(Vec2::new(0.5, 0.0)),
        ));
        let quem = pela_ordem(&quem);
        (sim, quem)
    };
    let (mut sim, quem) = nascem_na_lama();
    let mut c = PhysicsBridge::new();
    c.dispatch(&mut sim, true, 1);
    assert!(procuraram(&c, &quem).iter().all(|&p| p), "o CONTROLO");
    let w0 = c.nav_agent(quem[0]).map_or(0, |r| r.last_work);
    let (mut sim, quem) = nascem_na_lama();
    let mut b = uma_faixa(w0);
    b.dispatch(&mut sim, true, 1);
    assert_eq!(
        procuraram(&b, &quem),
        (0..N).map(|i| i == 0).collect::<Vec<_>>(),
        "com o orçamento = o trabalho da 1.ª procura ({w0})"
    );
}
