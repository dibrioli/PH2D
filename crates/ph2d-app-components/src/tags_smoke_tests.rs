//! **Os gates da cena de smoke das TAGS** (plano §6.2) — irmão de [`super`] por CAP de LOC.
//!
//! ⚠️ **Uma cena de smoke que ensina o CONTRÁRIO do que acontece é pior que uma cena ausente** — a
//! ausente não é acreditada (`CLAUDE.md` §5.0). Estes gates medem as frases do doc-comment: os
//! `Enemy (5)` › `Flying (3)` › `Boss (1)` que o painel vai mostrar, e os **cinco** que o sinal
//! alcança — com a `Statue` e o `Hero` de fora.

use super::*;
use ph2d_ecs::SimWorld;

fn monta(nivel: u32) -> (SimWorld, TagTree, Arvore) {
    let mut tree = TagTree::new();
    let a = arvore(&mut tree);
    let mut sim = SimWorld::new();
    match nivel {
        2 => cena_dois(sim.world_mut(), &a),
        // ⚠️ A `=1` devolve o SUJEITO que a cena escolhe (o Herói) — este arnês não o lê, e a
        //    metade que o lê é o `a_cena_um_escolhe_o_heroi` abaixo.
        _ => {
            cena_um(sim.world_mut(), &a);
        }
    }
    (sim, tree, a)
}

fn nome(world: &World, e: ph2d_ecs::Entity) -> String {
    world
        .get::<Name>(e)
        .map(|n| n.0.clone())
        .unwrap_or_default()
}

/// ⭐⭐⭐ **A `=1` mostra o que o doc promete** — as cinco linhas com as contagens que o painel
/// desenha, e que o passo 3 do smoke manda ler na tela.
///
/// **Mutações que devem sangrar:** um objecto a menos · uma tag ligada ao pai errado.
#[test]
fn the_first_scene_shows_the_counts_the_doc_promises() {
    let (sim, tree, a) = monta(1);
    for (tag, esperado) in [
        (a.enemy, 5),
        (a.flying, 3),
        (a.boss, 1),
        (a.statue, 1),
        (a.player, 1),
    ] {
        assert_eq!(
            ph2d_ecs::tags::counts(sim.world(), &tree)
                .get(&tag)
                .copied()
                .unwrap_or(0),
            esperado,
            "{tag:?}"
        );
    }
}

/// ⭐⭐⭐ **O alarme alcança CINCO, e a raiz irmã fica** (o passo 3 e o «como saber que deu errado»).
///
/// ⚠️ **A `Statue` é o CONTROLO da cena** e por isso é medida aqui: se ela entrar, o alcance está a
/// varrer a árvore inteira em vez da subárvore, e o smoke leria como *«funciona»*.
///
/// **Mutação que deve sangrar:** o `SignalTarget::Tagged` a virar `Named`.
#[test]
fn the_alarm_reaches_five_and_the_sibling_root_stays() {
    let (sim, tree, a) = monta(1);
    let atingidos: std::collections::BTreeSet<String> =
        ph2d_ecs::tags::tagged(sim.world(), &tree, a.enemy)
            .into_iter()
            .map(|e| nome(sim.world(), e))
            .collect();
    assert_eq!(atingidos.len(), 5, "{atingidos:?}");
    for fora in ["Statue", "Hero", "Scene Brain"] {
        assert!(
            !atingidos.contains(fora),
            "{fora} foi atingido: {atingidos:?}"
        );
    }
    for dentro in ["Goblin A", "Goblin B", "Bat A", "Bat B", "Dragon"] {
        assert!(
            atingidos.contains(dentro),
            "{dentro} escapou: {atingidos:?}"
        );
    }
}

/// ⚠️ **A tabela do cérebro aponta à TAG, e o campo de nome fica vazio** — com um nome escrito ali
/// o smoke não distinguiria *«o alvo por tag funciona»* de *«o alvo por nome funciona»*.
///
/// **Mutação que deve sangrar:** escrever `"Goblin A"` no `target`.
#[test]
fn the_brain_aims_at_the_tag_and_names_nobody() {
    let (sim, _, a) = monta(1);
    let mut q = sim.world().try_query::<&SignalActions>().expect("existe");
    let linhas: Vec<&SignalAction> = q.iter(sim.world()).flat_map(|t| t.0.iter()).collect();
    assert_eq!(linhas.len(), 1, "a cena tem UMA linha de tabela");
    assert_eq!(linhas[0].target_by, SignalTarget::Tagged(a.enemy.0));
    assert!(
        linhas[0].target.is_empty(),
        "o campo de NOME ficou escrito: {:?}",
        linhas[0].target
    );
}

/// ⭐⭐⭐ **A `=2` filtra pela tag, e o controlo NÃO passa** — as duas metades do passo 5.
///
/// ⚠️ Medido pela porta que a física de facto corre (`belongs`), e não por uma conta à parte: uma
/// segunda resposta aqui deixaria o gate verde sobre uma cena que o produto lê ao contrário.
///
/// **Mutações que devem sangrar:** o filtro a apontar `Enemy` · o filtro apagado (passaria todos).
#[test]
fn the_trap_lets_the_hero_through_and_ignores_the_goblin() {
    let (sim, tree, a) = monta(2);
    let mut q = sim
        .world()
        .try_query::<(&Name, &Tags)>()
        .expect("há corpos marcados");
    let corpos: std::collections::BTreeMap<String, Tags> = q
        .iter(sim.world())
        .map(|(n, t)| (n.0.clone(), t.clone()))
        .collect();
    // ⛔⛔ **O filtro LÊ-SE DO MUNDO, e não da fixtura** — a 1.ª redacção escrevia
    // `TagId(a.player.0)` e uma mutação que apontasse a armadilha a `Enemy` **SOBREVIVEU**: o gate
    // media a lei da árvore, não a FIAÇÃO da cena. *Um gate que re-deriva o valor que devia ler não
    // mede o produto.*
    let filtro = {
        let mut q = sim
            .world()
            .try_query::<&SignalTagFilter>()
            .expect("a armadilha declara um filtro");
        let achados: Vec<u64> = q.iter(sim.world()).map(|f| f.0).collect();
        assert_eq!(achados.len(), 1, "a cena tem UMA armadilha");
        TagId(achados[0])
    };
    assert_eq!(filtro.0, a.player.0, "a armadilha aponta à tag errada");
    assert!(
        ph2d_ecs::tags::belongs(&corpos["Hero"], &tree, filtro),
        "o Hero não passa o filtro — a armadilha não dispara para ninguém"
    );
    assert!(
        !ph2d_ecs::tags::belongs(&corpos["Goblin"], &tree, filtro),
        "o Goblin passa o filtro — a cena ensina que o filtro não decide"
    );
}

/// ⚠️ **A armadilha da `=2` é um SENSOR e declara o filtro** — sem o sensor ela empurra o corpo em
/// vez de o deixar atravessar, e sem o filtro a cena ensina o contrário do que diz.
#[test]
fn the_trap_is_a_sensor_and_carries_its_filter() {
    let (sim, _, a) = monta(2);
    let mut q = sim
        .world()
        .try_query::<(&Name, &Collider, &SignalOnHit, &SignalTagFilter)>()
        .expect("a armadilha existe");
    let achadas: Vec<(String, bool, String, u64)> = q
        .iter(sim.world())
        .map(|(n, c, s, f)| (n.0.clone(), c.is_sensor, s.0.clone(), f.0))
        .collect();
    assert_eq!(
        achadas,
        vec![("Trap".to_string(), true, "trap".to_string(), a.player.0)]
    );
}

/// ⚠️ **Os dois corpos caem de alturas DIFERENTES** — se caíssem juntos o artista não saberia qual
/// deles disparou, e o controlo deixaria de ser um controlo.
#[test]
fn the_two_bodies_fall_one_at_a_time() {
    let (sim, _, _) = monta(2);
    let mut q = sim
        .world()
        .try_query::<(&Name, &Transform, &RigidBody)>()
        .expect("há corpos");
    let alturas: std::collections::BTreeMap<String, f32> = q
        .iter(sim.world())
        .filter(|(_, _, b)| b.kind == BodyKind::Dynamic)
        .map(|(n, t, _)| (n.0.clone(), t.translation.y))
        .collect();
    assert_eq!(alturas.len(), 2);
    assert!(
        (alturas["Hero"] - alturas["Goblin"]).abs() > 2.0,
        "as alturas são quase a mesma: {alturas:?}"
    );
    assert!(
        alturas["Goblin"] < alturas["Hero"],
        "o CONTROLO tem de cair primeiro: {alturas:?}"
    );
}

/// ⭐⭐ **O `CENAS` é CONTADO nos braços do `match`, e a família publica-o.**
///
/// ⚠️ **`include_str!` e não `read_to_string`**: um ficheiro que mude de sítio tem de falhar a
/// COMPILAR, não no dia em que alguém corre a suíte com o filtro certo (a armadilha §2.4 do HOWTO
/// — o gémeo em runtime só falha *quando o teste corre*).
///
/// **Mutações que devem sangrar:** um braço novo sem mexer no `CENAS` · o `FAMILY` a declarar outro
/// número.
#[test]
fn the_scene_count_is_counted_from_the_match_and_published() {
    let fonte = include_str!("tags_smoke.rs");
    let corpo = fonte
        .split_once("pub(crate) fn montar(")
        .expect("o roteador existe")
        .1
        .split_once("\n}\n")
        .expect("o corpo fecha")
        .0;
    // Os braços que NOMEIAM um nível, mais o `_` que serve a `=1`.
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
        .find(|r| r.env == "PH2D_TAGS_SMOKE")
        .expect("a família declara o roteador");
    assert_eq!(publicado.max_level, CENAS);
}

/// ⚠️ **Um nível que o roteador não conhece cai na `=1`, e as DUAS cenas são diferentes.**
///
/// ⛔ Sem a segunda metade este gate seria vácuo: uma `montar` que ignorasse o nível e montasse
/// sempre a mesma coisa passaria nele.
///
/// **Mutações que devem sangrar:** o `_ =>` a não montar nada · o `2 =>` a cair na `=1`.
#[test]
fn an_unknown_level_falls_back_to_the_first_scene() {
    let conta = |nivel: u32| {
        let mut tree = TagTree::new();
        let mut sim = SimWorld::new();
        let (cena, _) = montar(sim.world_mut(), &mut tree, nivel);
        (cena, sim.world().iter_entities().count())
    };
    let (c1, n1) = conta(1);
    let (c2, n2) = conta(2);
    let (c7, n7) = conta(7);
    assert_eq!((c1, c7), (1, 1), "o nível desconhecido não caiu na `=1`");
    assert_eq!(c2, 2);
    assert_eq!(n7, n1, "a `=7` montou outra coisa que não a `=1`");
    assert!(n1 >= 8, "a `=1` monta os sete objectos e o cérebro");
    assert_ne!(n1, n2, "as duas cenas montam a MESMA coisa");
}

/// ⭐⭐⭐ **A `=1` NOMEIA quem ela escolhe, e é o HERÓI — o único objecto desta cena que sobrevive
/// ao alarme E carrega o rótulo mais largo.**
///
/// ⛔⛔ **Ela nasceu de um report do dono** (2026-09-19): o smoke desta cena mandava ler a secção
/// *Tags* do Inspector, e a cena abria **sem objecto escolhido** — logo o painel mostrava o estado
/// vazio e não existia um único chip no ecrã. *A cena estava certa como DADOS e era impossível
/// como GESTO*, a mesma forma que o #15 pagou.
///
/// ⚠️ **As duas metades**: o sujeito existe, e ele é um dos que FICA. Escolher um inimigo poria o
/// artista a olhar para um objecto que desaparece aos 2 s — a cena a ensinar o contrário de si.
#[test]
fn a_cena_um_escolhe_o_heroi_e_ele_sobrevive_ao_alarme() {
    let mut tree = TagTree::new();
    let mut sim = SimWorld::new();
    let (cena, sujeito) = montar(sim.world_mut(), &mut tree, 1);
    assert_eq!(cena, 1);
    let bits =
        sujeito.expect("a `=1` tem de NOMEAR quem ela escolhe — sem isso o smoke é uma caça");
    let e = ph2d_ecs::Entity::from_bits(bits);
    assert_eq!(
        nome(sim.world(), e),
        "Hero",
        "a cena escolheu outro objecto — e os cinco inimigos SOMEM aos 2 s"
    );
    // ⛔ CONTROLO: a `=2` NÃO escolhe ninguém. Ali o assunto é a armadilha a decidir no canvas, e
    //    pôr o Inspector à frente seria tapar a cena com um painel.
    let mut tree2 = TagTree::new();
    let mut sim2 = SimWorld::new();
    let (cena2, sujeito2) = montar(sim2.world_mut(), &mut tree2, 2);
    assert_eq!(cena2, 2);
    assert!(
        sujeito2.is_none(),
        "a `=2` escolheu um objecto — ela é uma cena de CANVAS, e o Inspector taparia o que ela \
         existe para mostrar"
    );
}
