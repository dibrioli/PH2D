//! ⭐⭐⭐ **A hierarquia das tags é a que o Blender MEDE** — gate 10 do plano de Tags
//! (`docs/Components/08_plano_tags.md` §5.2), e a decisão do dono D1 (*«Faça como o Blender»*)
//! transformada em comparação conjunto a conjunto.
//!
//! # O oráculo
//!
//! [`FIXTURE`] é a saída do Blender 5.2.1 corrido sem interface sobre a cena NOSSA
//! (`docs/Components/ferramentas/blender_tags_hierarchy_probe.py`, cabeçalho no próprio ficheiro).
//! Para cada estágio (`BASE` · `RENAME` · `MOVE`) e cada coleção, o Blender diz quem está no
//! `all_objects` dela; este gate reconstrói a MESMA cena com a [`TagTree`] e o componente [`Tags`] e
//! exige que [`tagged`] devolva exactamente o mesmo conjunto.
//!
//! ⚠️ **As três linhas que o Blender dá e que um modelo por TEXTO erraria:**
//! - `Enemy` alcança os `Bat` e o `Dragon`, que só estão nas filhas (a contenção);
//! - depois do `RENAME` a pertença é a mesma (a referência);
//! - depois do `MOVE` o `Dragon` sai de `Monster` (a hierarquia é da árvore, não do objecto).
//!
//! # ⚠️ A divergência DECLARADA fica aqui, com gate
//!
//! A linha `CASE` mostra o Blender a guardar `Inimigo`, `inimigo` e `inímigo` como três coleções. A
//! decisão do dono D2 é a oposta, e este gate afirma as DUAS metades: o oráculo distingue, nós não.

use ph2d_ecs::tags::{Tags, tagged};
use ph2d_ecs::{Name, StableId, Transform, World};
use ph2d_tags::{TagError, TagId, TagTree};

/// A saída do Blender, com cabeçalho.
const FIXTURE: &str = include_str!("fixtures/blender_tags_hierarchy.txt");

/// Uma linha `ALL <estágio> <caminho> = <nomes>` do oráculo.
struct Linha<'a> {
    estagio: &'a str,
    caminho: &'a str,
    nomes: Vec<&'a str>,
}

fn linhas_all() -> Vec<Linha<'static>> {
    FIXTURE
        .lines()
        .filter_map(|l| l.strip_prefix("ALL "))
        .map(|resto| {
            let (cabeca, nomes) = resto
                .split_once(" = ")
                .expect("`ALL <estagio> <caminho> = …`");
            let (estagio, caminho) = cabeca.split_once(' ').expect("`<estagio> <caminho>`");
            let mut nomes: Vec<&str> = nomes.split(',').filter(|n| !n.is_empty()).collect();
            nomes.sort_unstable();
            Linha {
                estagio,
                caminho,
                nomes,
            }
        })
        .collect()
}

fn veredito(nome: &str) -> &'static str {
    let linha = FIXTURE
        .lines()
        .find_map(|l| l.strip_prefix("CYCLE ")?.strip_prefix(nome))
        .unwrap_or_else(|| panic!("o oraculo nao tem a linha CYCLE {nome}"));
    if linha.trim_start().starts_with("ok") {
        "ok"
    } else {
        assert!(linha.trim_start().starts_with("refused"), "{linha:?}");
        "refused"
    }
}

/// A cena do plano §5.1, do nosso lado.
struct Cena {
    world: World,
    tree: TagTree,
    enemy: TagId,
    flying: TagId,
    boss: TagId,
}

fn cena() -> Cena {
    let mut tree = TagTree::new();
    let boss = tree.create("Enemy/Flying/Boss").expect("cria");
    let enemy = tree.find("Enemy").expect("ancestral criado");
    let flying = tree.find("Enemy/Flying").expect("ancestral criado");
    let statue = tree.create("Statue").expect("cria");
    let player = tree.create("Player").expect("cria");

    let mut world = World::new();
    let objectos = [
        ("Goblin A", enemy),
        ("Goblin B", enemy),
        ("Bat A", flying),
        ("Bat B", flying),
        ("Dragon", boss),
        ("Statue", statue),
        ("Hero", player),
    ];
    for (i, (nome, tag)) in objectos.into_iter().enumerate() {
        world.spawn((
            Transform::IDENTITY,
            Name::new(nome),
            StableId(i as u64 + 1),
            Tags::from_ids([tag]),
        ));
    }
    Cena {
        world,
        tree,
        enemy,
        flying,
        boss,
    }
}

fn confere(c: &Cena, estagio: &str) -> usize {
    let mut n = 0;
    for linha in linhas_all().iter().filter(|l| l.estagio == estagio) {
        let q = c
            .tree
            .find(linha.caminho)
            .unwrap_or_else(|| panic!("{estagio}: a arvore nao tem `{}`", linha.caminho));
        let mut nossos: Vec<&str> = tagged(&c.world, &c.tree, q)
            .into_iter()
            .map(|e| c.world.get::<Name>(e).expect("tem nome").as_str())
            .collect();
        nossos.sort_unstable();
        assert_eq!(
            nossos, linha.nomes,
            "{estagio} `{}`: o Blender diz {:?} e nos {:?}",
            linha.caminho, linha.nomes, nossos
        );
        n += 1;
    }
    n
}

/// ⭐⭐⭐ **Conjunto a conjunto, os três estágios.**
///
/// **Mutações que devem sangrar:** `belongs` a ler só o conjunto directo (o `BASE Enemy` perde os
/// `Bat` e o `Dragon`) · `tagged` a casar por prefixo de TEXTO · o `move_under` sem levar a subárvore.
#[test]
fn the_hierarchy_is_the_one_blender_measures() {
    let mut c = cena();
    // ⚠️ Piso de população: cinco coleções por estágio. Uma fixture truncada não passa por vácuo.
    assert_eq!(confere(&c, "BASE"), 5);

    c.tree.rename(c.enemy, "Monster").expect("renomeia");
    assert_eq!(confere(&c, "RENAME"), 5);

    c.tree.move_under(c.boss, None).expect("move para a raiz");
    assert_eq!(confere(&c, "MOVE"), 5);
}

/// ⭐⭐ **O ciclo que o Blender recusa, nós recusamos; o que ele aceita, nós aceitamos.**
///
/// **Mutação que deve sangrar:** o `move_under` sem conferir a subárvore do destino.
#[test]
fn the_cycle_verdicts_are_the_ones_blender_measures() {
    let mut c = cena();
    c.tree.rename(c.enemy, "Monster").expect("renomeia");
    c.tree.move_under(c.boss, None).expect("move para a raiz");

    let dentro_do_boss = c.tree.move_under(c.enemy, Some(c.boss));
    assert_eq!(
        (veredito("monster_into_boss"), dentro_do_boss.is_ok()),
        ("ok", true)
    );
    c.tree.move_under(c.enemy, None).expect("volta para a raiz");

    c.tree
        .move_under(c.boss, Some(c.flying))
        .expect("volta para dentro");
    let ciclo = c.tree.move_under(c.flying, Some(c.boss));
    assert_eq!(veredito("flying_into_its_own_child"), "refused");
    assert_eq!(ciclo, Err(TagError::IntoOwnSubtree));
}

/// ⚠️ **A divergência declarada D2, nas duas metades.**
///
/// **Mutação que deve sangrar:** tirar a dobra da árvore (as três grafias viram três tags) — e a
/// metade do oráculo cai se alguém «corrigir» a fixture para dizer que o Blender as funde.
#[test]
fn the_blender_keeps_three_spellings_and_the_owner_asked_for_one() {
    let caso = FIXTURE
        .lines()
        .find_map(|l| l.strip_prefix("CASE "))
        .expect("o oraculo tem a linha CASE");
    let grafias: std::collections::BTreeSet<&str> = caso.split(" | ").collect();
    assert_eq!(grafias.len(), 3, "o Blender distingue as tres: {caso:?}");

    let mut tree = TagTree::new();
    let ids: std::collections::BTreeSet<TagId> = grafias
        .iter()
        .map(|g| tree.create(g).expect("cria"))
        .collect();
    assert_eq!(ids.len(), 1, "D2: maiuscula e acento nao importam");
    assert_eq!(tree.tags().len(), 1);
}
