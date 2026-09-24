//! ⭐⭐⭐ **Os pedidos de vida da TABELA DE ACÇÕES** (plano 28, W2b) — os verbos `Damage`/`Heal`
//! chegam à ponte por [`PhysicsBridge::pede_vida`], são aplicados no PRÓXIMO tique e gravados numa
//! fita por tique.
//!
//! # ⚠️ Porque uma fita, e não uma fila
//!
//! A tabela corre por QUADRO e a vida anda por TIQUE, dentro do anel de checkpoints. Um pedido
//! aplicado «agora» seria esquecido por um scrub (o replay não o sabia) ou re-aplicado por outro.
//! ⇒ o que se afirma aqui é, antes de tudo, que **um scrub devolve a vida EXACTA** de uma corrida em
//! que a tabela feriu e curou.

use ph2d_core::Vec2;
use ph2d_ecs::{Entity, Name, SimWorld, Transform};
use ph2d_physics::world::checkpoint::STRIDE;
use ph2d_physics_ecs::health::HealthEventKind;
use ph2d_physics_ecs::{
    BodyKind, Collider, ColliderShape, Health, PedidoDeVida, PhysicsBridge, RigidBody,
};

fn alvo(sim: &mut SimWorld, vida: Health) -> Entity {
    sim.world_mut()
        .spawn((
            Name::new("Alvo"),
            RigidBody {
                kind: BodyKind::Kinematic,
            },
            Collider {
                shape: ColliderShape::Cuboid {
                    half_x: 0.4,
                    half_y: 0.4,
                },
                ..Collider::default()
            },
            Transform::from_translation(Vec2::new(0.0, 0.0)),
            vida,
        ))
        .id()
}

fn cem() -> Health {
    Health {
        max: 100.0,
        start: 100.0,
        ..Health::default()
    }
}

fn pontos(ponte: &PhysicsBridge, e: Entity) -> f64 {
    ponte.health_of(e).map_or(f64::NAN, |s| s.vida().pontos)
}

/// ⭐⭐ **O pedido chega no tique SEGUINTE, e não antes** — e o facto sai com a fonte a ser o
/// próprio alvo (um verbo não tem quem bata).
///
/// **Mutações que devem sangrar:** aplicar o `Dano` como `Cura` · não aplicar os pedidos.
#[test]
fn o_pedido_da_tabela_chega_no_tique_seguinte() {
    let mut sim = SimWorld::new();
    let a = alvo(&mut sim, cem());
    let mut ponte = PhysicsBridge::new();
    for t in 0..=3 {
        ponte.dispatch(&mut sim, true, t);
    }
    ponte.pede_vida(a, PedidoDeVida::Dano(30.0));
    assert_eq!(
        pontos(&ponte, a),
        100.0,
        "o pedido não age fora do tique — o scrub não o saberia refazer"
    );
    ponte.dispatch(&mut sim, true, 4);
    assert_eq!(pontos(&ponte, a), 70.0);
    let factos: Vec<_> = ponte.health_events().to_vec();
    assert_eq!(factos.len(), 1, "{factos:?}");
    assert_eq!(factos[0].kind, HealthEventKind::Damaged { amount: 30.0 });
    assert_eq!(factos[0].target, a);
    assert_eq!(factos[0].source, a, "a fonte de um verbo é o próprio alvo");
    // E foi CONSUMIDO: o tique seguinte não o aplica outra vez.
    ponte.dispatch(&mut sim, true, 5);
    assert_eq!(pontos(&ponte, a), 70.0, "o pedido aplicou-se duas vezes");
}

/// ⭐⭐⭐ **A cura devolve até ao tecto e acende o `On Heal`** — o campo que, antes da W2b, não
/// tinha produtor nenhum. E um morto não é curado (a regra da casa).
///
/// **Mutações que devem sangrar:** tirar o braço `Healed` do `factos` · tirar o `on_heal` dos
/// sinais · curar mortos.
#[test]
fn a_cura_devolve_ate_ao_tecto_e_acende_on_heal() {
    let mut sim = SimWorld::new();
    let a = alvo(
        &mut sim,
        Health {
            on_heal: "curou".into(),
            ..cem()
        },
    );
    let mut ponte = PhysicsBridge::new();
    ponte.dispatch(&mut sim, true, 0);
    ponte.pede_vida(a, PedidoDeVida::Dano(50.0));
    ponte.dispatch(&mut sim, true, 1);
    ponte.pede_vida(a, PedidoDeVida::Cura(80.0));
    ponte.dispatch(&mut sim, true, 2);
    assert_eq!(pontos(&ponte, a), 100.0, "a cura passou do tecto");
    assert_eq!(
        ponte.health_events()[0].kind,
        HealthEventKind::Healed { amount: 50.0 },
        "o facto conta o que a vida GANHOU, não o que se pediu"
    );
    let sinais = ponte.signal_events(&sim, &ph2d_tags::TagTree::new());
    assert!(
        sinais
            .iter()
            .any(|s| s.name == "curou" && s.source == a && s.other == a),
        "o `On Heal` não se acendeu: {:?}",
        sinais.iter().map(|s| &s.name).collect::<Vec<_>>()
    );

    // O CONTROLO: um morto não é curado.
    ponte.pede_vida(a, PedidoDeVida::Dano(500.0));
    ponte.dispatch(&mut sim, true, 3);
    assert_eq!(pontos(&ponte, a), 0.0);
    ponte.pede_vida(a, PedidoDeVida::Cura(40.0));
    ponte.dispatch(&mut sim, true, 4);
    assert_eq!(pontos(&ponte, a), 0.0, "um morto foi curado");
    assert!(
        ponte.health_events().is_empty(),
        "curar um morto produziu factos: {:?}",
        ponte.health_events()
    );
}

/// ⭐⭐⭐ **Um scrub devolve a vida EXACTA de uma corrida em que a tabela feriu e curou** — a razão
/// de a fita existir. E um replay não publica (um scrub não é uma tempestade de golpes).
///
/// ⛔⛔⛔ **Os pedidos caem FORA do passo do anel, e é isso que o gate mede.** O anel guarda um
/// retrato a cada [`STRIDE`] tiques e um scrub semeia do mais novo e REPLAYA o resto. A 1.ª redacção
/// pedia nos tiques `10`/`20`/`40` — EXACTAMENTE sobre o passo —, logo todo scrub semeava de um
/// retrato que já tinha o pedido aplicado e **nenhum replay atravessava um pedido**: a prova de
/// mutação leu o replay sem a fita e a fita sem gravar **os dois VERDES**. *A fixtura não continha o
/// fenómeno.* Hoje o gate afirma, antes de medir, que nenhum pedido cai no passo.
///
/// **Mutações que devem sangrar:** o replay não ler a fita · a fita não gravar.
#[test]
fn um_scrub_refaz_os_pedidos_da_tabela() {
    let mut sim = SimWorld::new();
    let a = alvo(
        &mut sim,
        Health {
            regen: 3.0,
            regen_delay_s: 0.2,
            ..cem()
        },
    );
    let pedidos: [(u64, PedidoDeVida); 3] = [
        (13, PedidoDeVida::Dano(30.0)),
        (27, PedidoDeVida::Cura(10.0)),
        (45, PedidoDeVida::Dano(55.0)),
    ];
    assert!(
        pedidos.iter().all(|(t, _)| t % STRIDE != 0),
        "um pedido no passo do anel é semeado pelo retrato e nunca replayado — o gate mediria nada"
    );
    let mut ponte = PhysicsBridge::new();
    let mut corrida = Vec::new();
    for t in 0..=60_u64 {
        for (quando, p) in pedidos {
            if quando == t {
                ponte.pede_vida(a, p);
            }
        }
        ponte.dispatch(&mut sim, true, t);
        corrida.push(pontos(&ponte, a).to_bits());
    }
    // O PISO: a corrida tem de ter sido mexida pelos TRÊS pedidos.
    let lidas: Vec<f64> = corrida.iter().map(|&b| f64::from_bits(b)).collect();
    assert!(
        lidas[12] > 99.0 && lidas[13] < 71.0,
        "o dano do tique 13: {lidas:?}"
    );
    assert!(
        lidas[46] < lidas[44] - 50.0,
        "o dano do tique 45: {lidas:?}"
    );

    // ⚠️ **Só para TRÁS, e em ordem decrescente:** um salto para a FRENTE anda os tiques como um play
    // e GRAVA POR CIMA (o gate seguinte), que é a regra da fita do dedo. Os alvos ficam dos dois
    // lados de cada pedido, e cada replay a partir do retrato anterior ATRAVESSA-o (`49` semeia do
    // `40` e replaya o `45`; `29` do `20` e replaya o `27`; `15` do `10` e replaya o `13`).
    for alvo_t in [59_u64, 49, 46, 45, 44, 29, 28, 27, 26, 15, 14, 13, 12, 5, 0] {
        let de = ponte.health_of(a).map(|s| s.vida().pontos);
        ponte.dispatch(&mut sim, true, alvo_t);
        assert_eq!(
            pontos(&ponte, a).to_bits(),
            corrida[alvo_t as usize],
            "o scrub para o tique {alvo_t} devolveu {} contra {} da corrida (vinha de {de:?})",
            pontos(&ponte, a),
            f64::from_bits(corrida[alvo_t as usize])
        );
    }
    // E um scrub para TRÁS não publica.
    ponte.dispatch(&mut sim, true, 50);
    ponte.dispatch(&mut sim, true, 12);
    assert!(
        ponte.health_events().is_empty(),
        "o replay publicou {:?}",
        ponte.health_events()
    );
}

/// ⭐⭐ **Andar para a frente depois de um scrub GRAVA POR CIMA** — a regra da fita do dedo
/// (`InputTape::record`): o artista que volta atrás e toca de novo está a autorar por cima.
///
/// ⚠️ Com a regra oposta (a fita a guardar a corrida velha), um tique vivo sem pedidos voltaria a
/// ferir com o dano de uma corrida que já não existe. ⚠️ O pedido cai FORA do passo do anel (o
/// `13`), senão o scrub final semeia de um retrato e nunca replaya o tique que a regra decide.
///
/// **Mutação que deve sangrar:** o tique vivo sem pedidos não apagar a entrada velha.
#[test]
fn um_tique_vivo_depois_do_scrub_grava_por_cima() {
    let mut sim = SimWorld::new();
    let a = alvo(&mut sim, cem());
    let mut ponte = PhysicsBridge::new();
    for t in 0..=20_u64 {
        if t == 13 {
            ponte.pede_vida(a, PedidoDeVida::Dano(40.0));
        }
        ponte.dispatch(&mut sim, true, t);
    }
    assert_eq!(pontos(&ponte, a), 60.0);
    ponte.dispatch(&mut sim, true, 5);
    assert_eq!(pontos(&ponte, a), 100.0);
    // A corrida NOVA: do 6 ao 20 sem pedido nenhum.
    for t in 6..=20_u64 {
        ponte.dispatch(&mut sim, true, t);
    }
    assert_eq!(
        pontos(&ponte, a),
        100.0,
        "a fita re-aplicou o dano de uma corrida que foi autorada por cima"
    );
    // E um scrub DENTRO da corrida nova confirma-a — do retrato do `10`, a replayar o `13`.
    ponte.dispatch(&mut sim, true, 15);
    assert_eq!(
        pontos(&ponte, a),
        100.0,
        "o replay do tique 13 re-aplicou o pedido da corrida velha"
    );
}

/// ⚠️ **Um pedido não finito ou `≤ 0` é recusado na porta**, e um relógio parado GUARDA o pedido
/// para o próximo tique (um golpe da tabela não acontece fora do tempo da corrida).
///
/// ⛔⛔ **A recusa da porta NÃO é redundante com a da lei, e a prova de mutação disse-o ao
/// contrário primeiro:** a lei já recusa o não finito e o negativo, e a 1.ª redacção deste gate só
/// olhava os PONTOS — a porta a aceitar `≤ 0` ficava VERDE. Mas um golpe de **zero** passa a lei:
/// ele **sorteia a esquiva** e, com ela certa, anuncia um `Dodged` — um toque da tabela que não
/// feriu a gritar «esquivou». ⇒ a régua é a ausência de FACTO sobre um alvo que esquiva sempre, com o
/// CONTROLO de que o mesmo alvo esquiva um golpe válido.
///
/// **Mutação que deve sangrar:** a porta aceitar `≤ 0`.
#[test]
fn o_pedido_invalido_e_recusado_e_o_relogio_parado_espera() {
    let mut sim = SimWorld::new();
    let a = alvo(&mut sim, cem());
    let mut ponte = PhysicsBridge::new();
    ponte.dispatch(&mut sim, true, 0);
    for p in [
        PedidoDeVida::Dano(f64::NAN),
        PedidoDeVida::Dano(0.0),
        PedidoDeVida::Dano(-5.0),
        PedidoDeVida::Cura(f64::INFINITY),
    ] {
        ponte.pede_vida(a, p);
    }
    ponte.dispatch(&mut sim, true, 1);
    assert_eq!(pontos(&ponte, a), 100.0, "um pedido inválido passou");

    ponte.pede_vida(a, PedidoDeVida::Dano(10.0));
    ponte.dispatch(&mut sim, false, 1);
    assert_eq!(
        pontos(&ponte, a),
        100.0,
        "o pedido agiu com o relógio parado"
    );
    ponte.dispatch(&mut sim, true, 2);
    assert_eq!(pontos(&ponte, a), 90.0, "o pedido perdeu-se na pausa");

    // ⭐ O golpe de ZERO sobre quem esquiva sempre.
    let mut sim = SimWorld::new();
    let e = alvo(
        &mut sim,
        Health {
            dodge: 1.0,
            ..cem()
        },
    );
    let mut ponte = PhysicsBridge::new();
    ponte.dispatch(&mut sim, true, 0);
    ponte.pede_vida(e, PedidoDeVida::Dano(0.0));
    ponte.dispatch(&mut sim, true, 1);
    assert!(
        ponte.health_events().is_empty(),
        "um pedido de dano ZERO chegou à lei e produziu factos: {:?}",
        ponte.health_events()
    );
    // O CONTROLO: o mesmo alvo esquiva um golpe válido — senão a régua acima mede nada.
    ponte.pede_vida(e, PedidoDeVida::Dano(5.0));
    ponte.dispatch(&mut sim, true, 2);
    assert!(
        ponte
            .health_events()
            .iter()
            .any(|f| f.kind == HealthEventKind::Dodged),
        "controlo: o alvo com esquiva 1 não esquivou — a fixtura não contém o fenómeno: {:?}",
        ponte.health_events()
    );
}
