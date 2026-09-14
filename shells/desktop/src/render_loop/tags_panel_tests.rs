//! **Os gates do painel TAGS** (plano §5.2, W4 — 26 a 28) — irmão de
//! [`crate::render_loop::tags_panel`] por CAP de LOC.
//!
//! ⚠️ **Só a shell vê os DOIS lados**: a árvore vive numa folha sem ECS, a pertença vive no
//! `ph2d-ecs`, e o painel não conhece nenhum dos dois. É aqui que se prova que a coluna do painel
//! diz o que a consulta diz, que apagar leva as duas metades, e que uma recusa chega com a frase.

use super::*;
use ph2d_ecs::tags::Tags;
use ph2d_ecs::{SimWorld, Transform};

/// **A fixtura do plano §5.1**, que é também a da cena de smoke: `Enemy` › `Flying` › `Boss`, as
/// raízes irmãs `Statue` e `Player`, e sete objectos.
///
/// ⚠️ **A ordem de CRIAÇÃO não é a da árvore**, de propósito (`Statue` nasce antes de `Enemy`): um
/// gate que leia a ordem dos ids passaria sobre uma fixtura alfabética, e essa é a mutação que
/// sobreviveu na W3a.
struct Fixture {
    sim: SimWorld,
    tree: TagTree,
    enemy: TagId,
    flying: TagId,
    boss: TagId,
    statue: TagId,
    player: TagId,
}

fn fixture() -> Fixture {
    let mut tree = TagTree::new();
    let statue = tree.create("Statue").expect("cria");
    let boss = tree.create("Enemy/Flying/Boss").expect("cria");
    let enemy = tree.find("Enemy").expect("ancestral");
    let flying = tree.find("Enemy/Flying").expect("ancestral");
    let player = tree.create("Player").expect("cria");
    let mut sim = SimWorld::new();
    for (nome, tag) in [
        ("Goblin A", enemy),
        ("Goblin B", enemy),
        ("Bat A", flying),
        ("Bat B", flying),
        ("Dragon", boss),
        ("Statue", statue),
        ("Hero", player),
    ] {
        sim.world_mut().spawn((
            Transform::default(),
            ph2d_ecs::Name::new(nome),
            Tags::from_ids([tag]),
        ));
    }
    Fixture {
        sim,
        tree,
        enemy,
        flying,
        boss,
        statue,
        player,
    }
}

fn linha(info: &TagsPanelInfo, id: TagId) -> &TagsPanelRow {
    info.rows
        .iter()
        .find(|r| r.id == id.0)
        .expect("a tag tem linha")
}

/// ⭐⭐⭐ **A coluna do painel é a resposta da CONSULTA, tag a tag** — `Enemy (5)` › `Flying (3)` ›
/// `Boss (1)`, `Statue (1)`, `Player (1)`, que é o que a cena de smoke manda ler na tela.
///
/// ⚠️ **Com a contagem lenta ao lado, no mesmo gate:** o painel usa a porta rápida e o oráculo é a
/// lenta, e é a igualdade das duas que impede a coluna de mentir.
///
/// **Mutações que devem sangrar:** contar só a pertença DIRECTA (`Enemy` leria `2`) · somar as
/// contagens dos filhos (`6`).
#[test]
fn the_panel_column_is_what_the_query_answers_for_each_tag() {
    let f = fixture();
    let info = build_tags_panel_info(f.sim.world(), &f.tree, None);
    for (tag, esperado) in [
        (f.enemy, 5),
        (f.flying, 3),
        (f.boss, 1),
        (f.statue, 1),
        (f.player, 1),
    ] {
        assert_eq!(linha(&info, tag).members, esperado, "{tag:?}");
        assert_eq!(
            linha(&info, tag).members,
            membros_de(f.sim.world(), &f.tree, tag),
            "a coluna discorda da consulta em {tag:?}"
        );
    }
    assert_eq!(info.rows.len(), 5);
}

/// ⭐⭐ **A linha do painel diz quantas TAGS o apagar leva** — é isso que faz o botão *Delete* poder
/// escrever *«3 tags, 5 objects»* ANTES de ser carregado.
///
/// **Mutações que devem sangrar:** `subtree` a valer sempre `1` · a contar só os descendentes (sem
/// a própria).
#[test]
fn a_row_says_how_many_tags_deleting_it_would_take() {
    let f = fixture();
    let info = build_tags_panel_info(f.sim.world(), &f.tree, None);
    assert_eq!(linha(&info, f.enemy).subtree, 3, "Enemy leva Flying e Boss");
    assert_eq!(linha(&info, f.flying).subtree, 2);
    assert_eq!(linha(&info, f.boss).subtree, 1, "uma folha leva-se a si só");
    assert_eq!(linha(&info, f.statue).subtree, 1);
}

/// ⭐⭐⭐ **A ORDEM do painel é a da ÁRVORE**, que é a mesma dos chips do Inspector e da caixa de
/// escolha — *uma ordem para tags em todo o app*.
///
/// ⚠️ **Com o controlo de que a fixtura contém o fenómeno:** a ordem de criação (`Statue` primeiro)
/// difere da da árvore, senão um gate sobre uma árvore alfabética não afirmaria nada.
///
/// **Mutação que deve sangrar:** ordenar por id.
#[test]
fn the_panel_lists_tags_in_tree_order_with_parents_before_children() {
    let f = fixture();
    assert!(
        f.statue.0 < f.enemy.0,
        "a fixtura tem de ter a ordem de criação DIFERENTE da da árvore"
    );
    let info = build_tags_panel_info(f.sim.world(), &f.tree, None);
    let caminhos: Vec<(&str, usize)> = info
        .rows
        .iter()
        .map(|r| (r.label.as_str(), r.depth))
        .collect();
    assert_eq!(
        caminhos,
        vec![
            ("Enemy", 0),
            ("Flying", 1),
            ("Boss", 2),
            ("Player", 0),
            ("Statue", 0),
        ]
    );
}

/// ⭐⭐⭐ **Criar dá um nome que ainda não está ocupado ALI**, e devolve a tag nascida — é ela que o
/// painel põe em modo de renomear.
///
/// ⚠️ **«Ali» é o que importa:** um `Tag` debaixo de `Enemy` não colide com o `Tag` de raiz, e
/// procurar o nome na árvore inteira faria o segundo nascer `Tag 2` sem razão.
///
/// **Mutações que devem sangrar:** o nome fixo (o 2.º gesto seria recusado) · `born: None`.
#[test]
fn creating_picks_a_name_that_is_free_where_it_lands() {
    let mut f = fixture();
    let a = apply_tag_tree_edit(
        f.sim.world_mut(),
        &mut f.tree,
        &TagTreeEdit::Create { parent: None },
    );
    let TagEditOutcome::Changed { born: Some(a) } = a else {
        panic!("a raiz nasce: {a:?}")
    };
    assert_eq!(f.tree.get(TagId(a)).expect("existe").path, "Tag");

    let b = apply_tag_tree_edit(
        f.sim.world_mut(),
        &mut f.tree,
        &TagTreeEdit::Create { parent: None },
    );
    let TagEditOutcome::Changed { born: Some(b) } = b else {
        panic!("a 2.ª raiz nasce: {b:?}")
    };
    assert_eq!(f.tree.get(TagId(b)).expect("existe").path, "Tag 2");

    // ⚠️ Debaixo de `Enemy` o nome `Tag` está livre outra vez.
    let c = apply_tag_tree_edit(
        f.sim.world_mut(),
        &mut f.tree,
        &TagTreeEdit::Create {
            parent: Some(f.enemy.0),
        },
    );
    let TagEditOutcome::Changed { born: Some(c) } = c else {
        panic!("o filho nasce: {c:?}")
    };
    assert_eq!(f.tree.get(TagId(c)).expect("existe").path, "Enemy/Tag");
}

/// ⭐⭐⭐ **Apagar leva a subárvore E a pertença, no MESMO gesto** — as duas metades, senão ficam ids
/// órfãos nos objectos: eles ocupam lugar no tecto do objecto e não contam para tag nenhuma.
///
/// **Mutações que devem sangrar:** saltar o `scrub` · apagar só a própria tag.
#[test]
fn deleting_takes_the_subtree_and_the_membership_in_one_gesture() {
    let mut f = fixture();
    let out = apply_tag_tree_edit(
        f.sim.world_mut(),
        &mut f.tree,
        &TagTreeEdit::Delete { id: f.enemy.0 },
    );
    assert_eq!(out, TagEditOutcome::Changed { born: None });
    assert_eq!(f.tree.len(), 2, "sobram Player e Statue");
    let info = build_tags_panel_info(f.sim.world(), &f.tree, None);
    assert_eq!(info.rows.len(), 2);
    // ⚠️ **A metade que o `scrub` faz** — sem ela os cinco objectos ficariam com um id que a
    // árvore já não conhece.
    let com_tag = f
        .sim
        .world()
        .iter_entities()
        .filter(|e| e.get::<Tags>().is_some_and(|t| !t.is_empty()))
        .count();
    assert_eq!(com_tag, 2, "só a Statue e o Hero continuam marcados");
}

/// ⭐⭐⭐ **A recusa chega com a FRASE e com a LINHA** — `Enemy/Flying` para dentro de `Boss` é o
/// ciclo que o Blender também recusa, e renomear sobre um irmão é a colisão.
///
/// ⚠️ **A frase vem da lei** (`TagError::message`), nunca do painel: duas superfícies a traduzir a
/// mesma recusa escrevem duas frases, e elas divergem na primeira vez que alguém mexe numa.
///
/// **Mutações que devem sangrar:** devolver `Nothing` em vez de `Refused` (a recusa ficaria muda) ·
/// devolver `tag: 0` (a frase pintar-se-ia fora da linha).
#[test]
fn a_refused_gesture_arrives_with_its_sentence_and_its_row() {
    let mut f = fixture();
    let ciclo = apply_tag_tree_edit(
        f.sim.world_mut(),
        &mut f.tree,
        &TagTreeEdit::Move {
            id: f.flying.0,
            parent: Some(f.boss.0),
        },
    );
    assert_eq!(
        ciclo,
        TagEditOutcome::Refused {
            tag: f.flying.0,
            why: TagError::IntoOwnSubtree
        }
    );
    let colisao = apply_tag_tree_edit(
        f.sim.world_mut(),
        &mut f.tree,
        &TagTreeEdit::Rename {
            id: f.statue.0,
            label: "player".into(),
        },
    );
    assert!(
        matches!(
            colisao,
            TagEditOutcome::Refused {
                tag,
                why: TagError::Collision { .. }
            } if tag == f.statue.0
        ),
        "a colisão dobrada tem de ser recusada NA LINHA: {colisao:?}"
    );
    // ⚠️ E a árvore fica **intacta** — uma recusa que já mexeu é pior que uma que não avisa.
    assert_eq!(f.tree.get(f.statue).expect("existe").path, "Statue");
    assert_eq!(f.tree.get(f.flying).expect("existe").path, "Enemy/Flying");
}

/// ⭐⭐ **Renomear e mover NÃO tocam na pertença** — a identidade é o id, e é isso que faz a cena de
/// smoke continuar a esconder os mesmos 5 objectos depois de `Enemy` virar `Inimigo`.
///
/// **Mutação que deve sangrar:** o `Move` a re-atribuir ids.
#[test]
fn renaming_or_moving_never_touches_a_member() {
    let mut f = fixture();
    let antes = build_tags_panel_info(f.sim.world(), &f.tree, None);
    let r = apply_tag_tree_edit(
        f.sim.world_mut(),
        &mut f.tree,
        &TagTreeEdit::Rename {
            id: f.enemy.0,
            label: "Inimigo".into(),
        },
    );
    assert_eq!(r, TagEditOutcome::Changed { born: None });
    let m = apply_tag_tree_edit(
        f.sim.world_mut(),
        &mut f.tree,
        &TagTreeEdit::Move {
            id: f.flying.0,
            parent: None,
        },
    );
    assert_eq!(m, TagEditOutcome::Changed { born: None });
    let depois = build_tags_panel_info(f.sim.world(), &f.tree, None);
    // ⚠️ O `Flying` saiu de debaixo do `Inimigo`, então a subárvore dele encolheu — o que NÃO pode
    // mudar é quem pertence a cada tag pelo próprio id.
    assert_eq!(linha(&depois, f.flying).members, 3, "Flying mantém os seus");
    assert_eq!(linha(&depois, f.boss).members, 1);
    assert_eq!(
        linha(&antes, f.player).members,
        linha(&depois, f.player).members
    );
    assert_eq!(depois.rows.len(), antes.rows.len(), "nenhuma tag nasceu");
}

/// ⭐⭐⭐ **O *Select* devolve a subárvore INTEIRA** (gate 28) — `Enemy` alcança os dois goblins, os
/// dois morcegos e o dragão, e a raiz irmã fica de fora.
///
/// **Mutações que devem sangrar:** devolver só os ids directos · alcançar a árvore toda.
#[test]
fn select_tagged_returns_the_whole_subtree_and_not_the_sibling_root() {
    let mut f = fixture();
    let out = apply_tag_tree_edit(
        f.sim.world_mut(),
        &mut f.tree,
        &TagTreeEdit::SelectTagged { id: f.enemy.0 },
    );
    let TagEditOutcome::Select(alvos) = out else {
        panic!("devolve alvos: {out:?}")
    };
    assert_eq!(alvos.len(), 5);
    let nomes: std::collections::BTreeSet<String> = alvos
        .iter()
        .filter_map(|e| f.sim.world().get::<ph2d_ecs::Name>(*e).map(|n| n.0.clone()))
        .collect();
    assert!(!nomes.contains("Statue"), "a raiz irmã casou: {nomes:?}");
    assert!(!nomes.contains("Hero"));
    assert!(
        nomes.contains("Dragon"),
        "a hierarquia não chegou: {nomes:?}"
    );
}

/// ⚠️ **Um gesto sobre uma tag que já não existe não mexe em nada, e DI-LO** — o painel pode ter um
/// clique em voo sobre uma linha que outro gesto apagou no mesmo quadro.
///
/// **Mutação que deve sangrar:** o `Delete` de um id morto a devolver `Changed` (a shell apagaria a
/// frase da recusa anterior e regista um passo de undo vazio).
#[test]
fn a_gesture_on_a_tag_that_is_gone_changes_nothing() {
    let mut f = fixture();
    let morto = TagId(9_999);
    assert_eq!(
        apply_tag_tree_edit(
            f.sim.world_mut(),
            &mut f.tree,
            &TagTreeEdit::Delete { id: morto.0 }
        ),
        TagEditOutcome::Nothing
    );
    assert_eq!(
        apply_tag_tree_edit(
            f.sim.world_mut(),
            &mut f.tree,
            &TagTreeEdit::SelectTagged { id: morto.0 }
        ),
        TagEditOutcome::Nothing
    );
    assert!(matches!(
        apply_tag_tree_edit(
            f.sim.world_mut(),
            &mut f.tree,
            &TagTreeEdit::Create {
                parent: Some(morto.0)
            }
        ),
        TagEditOutcome::Refused {
            why: TagError::Missing,
            ..
        }
    ));
    assert_eq!(f.tree.len(), 5, "nenhuma das três mexeu na árvore");
}
