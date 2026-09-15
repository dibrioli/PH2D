//! Os gates das duas cenas da FÁBRICA (`docs/Components/09_plano_spawner.md` §5, W4).
//!
//! ⚠️ **O oráculo é o que a CENA MONTA, não o que o doc diz** — cada asserção aqui é uma frase do
//! doc-comment do irmão, medida sobre o mundo que a `montar` devolve.

use super::{CENAS, montar};
use ph2d_ecs::{DestroyOutside, Factory, Lifetime, MasterRoot, Name, Pick, SimWorld, SpawnAt, Timers};
use ph2d_tags::TagTree;

fn monta(nivel: u32) -> (SimWorld, TagTree, u32) {
    let mut tree = TagTree::new();
    let mut sim = SimWorld::new();
    let cena = montar(sim.world_mut(), &mut tree, nivel);
    (sim, tree, cena)
}

/// ⭐⭐⭐ **A receita da `=1` é um MESTRE com vida, e a fábrica APONTA-LHE** — a metade que o
/// `Pendente` existe para resolver.
///
/// ⚠️ **O oráculo é o `master != 0` E o id CASAR com o mestre.** Um gate que só verificasse
/// `!= 0` passaria sobre uma fábrica a apontar para qualquer coisa — que é exactamente o que um
/// `Pendente` mal resolvido produz.
///
/// (Mutação: apagar o `resolver_receitas` ⇒ `master == 0` e nada nasce, com a cena a montar.)
#[test]
fn the_first_scene_points_the_factory_at_a_master_that_exists() {
    let (mut sim, _, cena) = monta(1);
    assert_eq!(cena, 1);
    let mut q = sim.world_mut().query::<(&Factory, &Timers)>();
    let (fab, timers) = q.iter(sim.world()).next().expect("a nuvem existe");
    assert_ne!(fab.master, 0, "a receita ficou por resolver");
    let alvo = fab.master;
    // ⚠️ O timer é o que dá ritmo — e o sinal dele TEM de ser o que a fábrica escuta.
    assert_eq!(timers.0.len(), 1);
    assert_eq!(timers.0[0].signal, fab.on_signal, "o relogio fala para outro");
    assert!(timers.0[0].repeat && timers.0[0].autostart);

    let mut q = sim
        .world_mut()
        .query::<(&ph2d_ecs::StableId, &MasterRoot, &Lifetime)>();
    let achou = q.iter(sim.world()).any(|(s, _, _)| s.0 == alvo);
    assert!(
        achou,
        "o `master` nao aponta a um mestre COM vida — a copia nasceria imortal"
    );
}

/// ⭐⭐ **A `=2` nasce num ponto MARCADO, com limite e com colhedor** — as três coisas que a cena
/// promete, cada uma medida no mundo.
#[test]
fn the_second_scene_has_tagged_points_a_cap_and_a_reaper() {
    let (mut sim, tree, cena) = monta(2);
    assert_eq!(cena, 2);
    let ponto = tree.find("SpawnPoint").expect("a tag nasceu");
    // Três marcas, e elas são alcançáveis pela PORTA que a fábrica usa.
    let marcas = ph2d_ecs::tags::tagged(sim.world(), &tree, ponto);
    assert_eq!(marcas.len(), 3, "as tres marcas do doc");

    let mut q = sim.world_mut().query::<&Factory>();
    let fab = q.iter(sim.world()).next().expect("a fabrica existe").clone();
    match fab.at {
        SpawnAt::Tagged { tag, pick } => {
            assert_eq!(tag, ponto.0, "a fabrica aponta a outra tag");
            assert_eq!(pick, Pick::Cycle, "a roda-viva e' o que o doc promete");
        }
        outro => panic!("a `=2` nao nasce num ponto marcado: {outro:?}"),
    }
    assert_eq!(fab.alive_max, 6, "o tecto de vivas do doc");

    // ⭐ **A câmera do jogo tem de existir** — sem ela o fora-do-ecrã não mede nada, e a cena
    // ensinaria que o colhedor não funciona.
    assert_eq!(
        ph2d_ecs::camera_2d::camera_count(sim.world_mut()),
        1,
        "sem GameCamera o fora-do-ecra nao mede nada"
    );
    // E a receita leva o colhedor.
    let mut q = sim.world_mut().query::<(&MasterRoot, &DestroyOutside)>();
    assert_eq!(q.iter(sim.world()).count(), 1, "a receita nao leva o colhedor");
}

/// ⭐⭐ **A vida e o colhedor vivem na RECEITA, nunca na fábrica** — a lei do §2.6.
///
/// ⛔ Sem este gate, pôr o `Lifetime` no objecto errado deixaria a cena a montar e a **nunca**
/// limpar nada: a fábrica não nasce numa corrida, logo a vida dela é inerte por lei.
#[test]
fn the_lifecycle_lives_on_the_recipe_and_not_on_the_factory() {
    for nivel in [1u32, 2] {
        let (mut sim, _, _) = monta(nivel);
        let mut q = sim.world_mut().query::<(&Factory, Option<&Lifetime>, Option<&DestroyOutside>)>();
        for (_, vida, fora) in q.iter(sim.world()) {
            assert!(
                vida.is_none() && fora.is_none(),
                "a cena =**{nivel}** poe o ciclo de vida na FABRICA — ali ele e' inerte"
            );
        }
    }
}

/// **O `CENAS` é contado no `match`, e a família publica o mesmo número.**
#[test]
fn the_scene_count_is_counted_from_the_match_and_published() {
    let fonte = include_str!("factory_smoke.rs");
    let corpo = fonte
        .split_once("pub(crate) fn montar(")
        .expect("o roteador existe")
        .1
        .split_once("\n}\n")
        .expect("o corpo fecha")
        .0;
    let nomeados = corpo
        .lines()
        .filter(|l| l.trim_start().starts_with(|c: char| c.is_ascii_digit()) && l.contains("=>"))
        .count();
    assert!(nomeados >= 1, "o censo varreu ZERO braços — ele partiu-se");
    assert_eq!(
        CENAS as usize,
        nomeados + 1,
        "o `CENAS` não descreve os braços do `match` (nomeados: {nomeados})"
    );
    let publicado = crate::FAMILY
        .routers
        .iter()
        .find(|r| r.env == "PH2D_FACTORY_SMOKE")
        .expect("a família declara o roteador");
    assert_eq!(publicado.max_level, CENAS);
}

/// **Um nível desconhecido cai na `=1`, e as duas cenas são DIFERENTES.**
#[test]
fn an_unknown_level_falls_back_to_the_first_scene() {
    let conta = |nivel: u32| {
        let (sim, _, cena) = monta(nivel);
        (cena, sim.world().iter_entities().count())
    };
    let (c1, n1) = conta(1);
    let (c2, n2) = conta(2);
    let (c9, n9) = conta(9);
    assert_eq!((c1, c9), (1, 1), "o nível desconhecido não caiu na `=1`");
    assert_eq!(c2, 2);
    assert_eq!(n9, n1, "a `=9` montou outra coisa que não a `=1`");
    assert_ne!(n1, n2, "as duas cenas montam a MESMA coisa");
}

/// ⚠️ **Nenhuma cópia nasce com o relógio parado, e nenhuma nasce sem sinal** — as duas cercas que
/// a lei pura tem, medidas sobre a cena REAL em vez de uma fixtura.
#[test]
fn the_scene_spawns_nothing_until_a_signal_arrives() {
    let (mut sim, tree, _) = monta(1);
    let vazio = ph2d_ecs::tick_factories(sim.world_mut(), &tree, &[]);
    assert!(vazio.births.is_empty(), "nasceu sem sinal nenhum");
    let errado = ph2d_ecs::tick_factories(sim.world_mut(), &tree, &["outro"]);
    assert!(errado.births.is_empty(), "nasceu com o sinal de outra pessoa");
    let certo = ph2d_ecs::tick_factories(sim.world_mut(), &tree, &["drop"]);
    assert_eq!(certo.births.len(), 1, "o sinal certo nao fez nascer");
}
