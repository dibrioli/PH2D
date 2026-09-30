//! Os gates da cena dos TIPOS de dano (plano 28, W6) — pelas portas do PRODUTO: as fábricas
//! (`tick_factories` + `apply_births`), a porta de cópia e a ponte da física.
//!
//! ⚠️ **Cada caso monta a cena de raiz com o herói à altura do alvo dele**: o herói é um mover
//! cinemático cuja pose a ponte conduz, e teletransportá-lo a meio mediria o arnês.

use super::*;
use bevy_ecs::query::With;
use ph2d_ecs::{SimWorld, Spawned};
use ph2d_physics_ecs::health::HealthEventKind;
use ph2d_physics_ecs::{HealthNow, PhysicsBridge};

/// O que um caso devolve: a vida final do alvo e os factos de DANO que ele levou, pela ordem.
struct Resultado {
    pontos: f64,
    danos: Vec<f64>,
    curas: Vec<f64>,
}

/// Monta a cena, põe o herói à altura do alvo `i`, atira UMA bala pelo `sinal` e corre `ticks`.
fn caso(i: usize, sinal: &str, ticks: u64) -> Resultado {
    let mut sim = SimWorld::new();
    let heroi = cena_tres(sim.world_mut());
    crate::vida_smoke::resolver_receitas(sim.world_mut());
    sim.world_mut()
        .get_mut::<Transform>(heroi)
        .expect("o herói tem pose")
        .translation
        .y = ALVOS[i].y;
    ph2d_ecs::assign_missing_stable_ids(sim.world_mut());
    let tags = ph2d_tags::TagTree::default();
    let registo = crate::component_registry_for_tests::registo();
    let (mut sc, mut mp) = crate::instance_docs::empty_docs();
    let mut ponte = PhysicsBridge::new();
    let mut nascer = |sim: &mut SimWorld, s: &str, t: u64| {
        let tick = ph2d_ecs::tick_factories(sim.world_mut(), &tags, &[s]);
        let mut docs = crate::instance_docs::OwnedDocs {
            vec_scene: &mut sc,
            vec_entities: &mut mp,
        };
        crate::factory_bridge::apply_births(sim, &registo, &mut docs, &tick.births, t).nasceram
    };
    let mut t = 0u64;
    assert_eq!(
        nascer(&mut sim, COMECAR, t),
        3,
        "os três alvos não nasceram"
    );
    for _ in 0..3 {
        ph2d_ecs::assign_missing_stable_ids(sim.world_mut());
        ponte.dispatch(&mut sim, true, t);
        t += 1;
    }
    assert_eq!(
        nascer(&mut sim, sinal, t),
        1,
        "a arma de «{sinal}» não atirou"
    );
    let alvo = {
        let w = sim.world_mut();
        let mut q = w.query_filtered::<(Entity, &Name), With<Spawned>>();
        q.iter(w)
            .find(|(_, n)| n.as_str().starts_with(ALVOS[i].nome))
            .map(|(e, _)| e)
            .unwrap_or_else(|| panic!("a cópia de «{}» não nasceu", ALVOS[i].nome))
    };
    let mut r = Resultado {
        pontos: f64::NAN,
        danos: Vec::new(),
        curas: Vec::new(),
    };
    for _ in 0..ticks {
        ph2d_ecs::assign_master_pieces(sim.world_mut());
        ponte.dispatch(&mut sim, true, t);
        t += 1;
        for ev in ponte.health_events().iter().filter(|e| e.target == alvo) {
            match ev.kind {
                HealthEventKind::Damaged { amount } => r.danos.push(amount),
                HealthEventKind::Healed { amount } => r.curas.push(amount),
                _ => {}
            }
        }
    }
    r.pontos = sim
        .world()
        .get::<HealthNow>(alvo)
        .map_or(f64::NAN, |h| h.pontos);
    r
}

/// Ticks que chegam para a bala chegar E para a queimadura acabar (`3 s` = `180` tiques).
const LONGO: u64 = 320;

fn perto(a: f64, b: f64) -> bool {
    (a - b).abs() < 1e-6
}

/// ⭐⭐⭐ **A salamandra: o fogo não a fere nem a queima, o gelo tira-lhe o DOBRO** — o smoke da
/// wave pelas portas do produto.
///
/// **Mutações que devem sangrar:** tirar as resistências da receita da salamandra; a arma de gelo
/// atirar a bala de fogo.
#[test]
fn a_salamandra_e_imune_ao_fogo_e_fraca_ao_gelo() {
    let fogo = caso(0, SINAL_FOGO, LONGO);
    assert_eq!(fogo.pontos, f64::from(VIDA), "o fogo feriu a salamandra");
    assert!(
        fogo.danos.is_empty(),
        "nem golpe nem queimadura: {:?}",
        fogo.danos
    );
    let gelo = caso(0, SINAL_GELO, LONGO);
    assert_eq!(
        gelo.danos,
        vec![f64::from(DANO) * 2.0],
        "o gelo tira o dobro"
    );
    assert!(perto(gelo.pontos, f64::from(VIDA - 2.0 * DANO)));
}

/// ⭐⭐⭐ **O CONTROLO: o fogo tira o golpe e depois QUEIMA três vezes; o gelo, só o golpe** — é o
/// controlo que prova que o nada da salamandra não é uma bala que falha.
///
/// **Mutação que deve sangrar:** tirar a queimadura da bala de fogo.
#[test]
fn o_controlo_leva_o_golpe_e_a_queimadura() {
    let fogo = caso(1, SINAL_FOGO, LONGO);
    assert_eq!(
        fogo.danos.len(),
        4,
        "o golpe e três pulsos: {:?}",
        fogo.danos
    );
    assert!(perto(fogo.danos[0], f64::from(DANO)));
    let queima: f64 = fogo.danos[1..].iter().sum();
    assert!(
        perto(queima, f64::from(QUEIMA_POR_S * QUEIMA_S)),
        "a queimadura tira {queima}"
    );
    assert!(perto(
        fogo.pontos,
        f64::from(VIDA - DANO - QUEIMA_POR_S * QUEIMA_S)
    ));
    let gelo = caso(1, SINAL_GELO, LONGO);
    assert_eq!(gelo.danos, vec![f64::from(DANO)]);
}

/// ⭐⭐⭐ **O elemental ABSORVE o fogo** — o golpe e a queimadura CURAM-no, e o gelo fere-o.
///
/// **Mutação que deve sangrar:** o elemental nascer com a vida cheia (a cura não teria para onde
/// subir, e o smoke mostraria nada).
#[test]
fn o_elemental_absorve_o_fogo() {
    let fogo = caso(2, SINAL_FOGO, LONGO);
    assert!(
        fogo.danos.is_empty(),
        "o fogo feriu o elemental: {:?}",
        fogo.danos
    );
    assert_eq!(fogo.curas.len(), 4, "a cura do golpe e três da queimadura");
    assert!(perto(
        fogo.pontos,
        f64::from(ALVOS[2].comeca + DANO + QUEIMA_POR_S * QUEIMA_S)
    ));
    assert!(fogo.pontos <= f64::from(VIDA), "a cura passou do máximo");
    let gelo = caso(2, SINAL_GELO, LONGO);
    assert_eq!(gelo.danos, vec![f64::from(DANO)]);
}

/// ⭐⭐ **O prólogo cria as acções DESTA cena** — o `Q` é fogo e o `J` gelo, e o roteiro nomeia as
/// mesmas teclas que as acções ligam.
#[test]
fn o_prologo_cria_as_accoes_desta_cena() {
    assert_eq!(crate::vida_smoke::accoes(3), &ACCOES[..]);
    assert_eq!(crate::vida_smoke::accoes(1), &crate::vida_smoke::ACCOES[..]);
    assert_eq!(ACCOES[0].1, crate::trigger_smoke::TECLA);
    assert_eq!(ACCOES[1].1, crate::vida_smoke::TECLA_VENENO);
}
