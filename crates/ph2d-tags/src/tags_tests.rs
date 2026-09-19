//! Os gates da árvore de tags — `docs/Components/08_plano_tags.md` §5.2 (W1), escritos contra um
//! *stub* (`todo!()`) e vistos vermelhos.
//!
//! ⚠️ **O oráculo é o que a UI desenharia**: os caminhos por ordem, que ids sobrevivem a um gesto, e o
//! que uma tag alcança — nunca o estado interno.

use super::{Tag, TagError, TagId, TagTree};
use std::collections::BTreeSet;

fn caminhos(t: &TagTree) -> Vec<&str> {
    t.tags().map(|g| g.path.as_str()).collect()
}

/// ⭐⭐ **Criar `Enemy/Flying/Boss` cria os ancestrais**, pela ordem da árvore.
///
/// **Mutação que deve sangrar:** criar só o nível final.
#[test]
fn creating_a_nested_tag_creates_the_ancestors_it_needs() {
    let mut t = TagTree::new();
    let boss = t.create("Enemy/Flying/Boss").expect("cria");
    assert_eq!(
        caminhos(&t),
        vec!["Enemy", "Enemy/Flying", "Enemy/Flying/Boss"]
    );
    let g = t.get(boss).expect("existe");
    assert_eq!((g.label(), g.depth()), ("Boss", 2));
}

/// ⭐⭐⭐ **A decisão do dono D2: criar uma tag que já existe DOBRADA devolve a que existe** — e o nome
/// mostrado continua a ser o de quem escreveu primeiro.
///
/// **Mutações que devem sangrar:** comparar o texto exacto (nasceria `énemy` ao lado de `Enemy`) ·
/// sobrescrever a grafia com a do segundo gesto.
#[test]
fn creating_a_tag_that_exists_folded_returns_the_existing_one() {
    let mut t = TagTree::new();
    let enemy = t.create("Enemy").expect("cria");
    for outra in ["enemy", "ENEMY", "Énemy", "  énemy  "] {
        assert_eq!(t.create(outra), Ok(enemy), "{outra:?} criou uma tag nova");
    }
    let flying = t.create("Enemy/Flying").expect("cria");
    assert_eq!(
        t.create("ÉNEMY/flying"),
        Ok(flying),
        "o caminho inteiro dobra, nível a nível"
    );
    assert_eq!(
        caminhos(&t),
        vec!["Enemy", "Enemy/Flying"],
        "a grafia é a do primeiro"
    );
    assert_eq!(t.find("enemy/FLYING"), Some(flying));
    assert_eq!(t.create(" / "), Err(TagError::Empty));
}

/// ⭐⭐⭐ **Renomear mantém o id e leva os filhos** — o Blender medido: renomear a coleção não tira
/// ninguém dela.
///
/// **Mutação que deve sangrar:** reescrever sem a fronteira de segmento (`Heroine` mudaria).
#[test]
fn renaming_keeps_the_id_and_carries_the_children_and_never_the_neighbour() {
    let mut t = TagTree::new();
    let hero = t.create("Hero").expect("cria");
    let arma = t.create("Hero/Sword").expect("cria");
    let heroine = t.create("Heroine").expect("cria");
    assert_eq!(t.rename(hero, "Villain"), Ok(()));
    assert_eq!(t.get(arma).map(|g| g.path.as_str()), Some("Villain/Sword"));
    assert_eq!(t.get(heroine).map(|g| g.path.as_str()), Some("Heroine"));
    assert_eq!(
        t.get(hero).map(|g| g.path.as_str()),
        Some("Villain"),
        "o id é o mesmo"
    );
}

/// ⛔ **Renomear para cima de um irmão (dobrado) é recusado** — e renomear para outra grafia DE SI
/// MESMO é permitido (é assim que se corrige `enemy` para `Enemy`).
///
/// **Mutações que devem sangrar:** conferir a ocupação sem dobrar · contar a própria tag como colisão.
#[test]
fn a_rename_that_collides_with_a_sibling_is_refused_and_a_new_spelling_is_not() {
    let mut t = TagTree::new();
    let enemy = t.create("Enemy").expect("cria");
    let prop = t.create("Prop").expect("cria");
    assert_eq!(
        t.rename(prop, "énemy"),
        Err(TagError::Collision { existing: enemy })
    );
    assert_eq!(t.rename(enemy, "ENEMY"), Ok(()));
    assert_eq!(t.get(enemy).map(|g| g.path.as_str()), Some("ENEMY"));
    assert_eq!(t.rename(prop, "   "), Err(TagError::Empty));
    assert_eq!(t.rename(prop, "A/B"), Err(TagError::HasSeparator));
    assert_eq!(t.rename(TagId(999), "X"), Err(TagError::Missing));
}

/// ⭐⭐ **Mover leva a subárvore, e o ciclo é recusado** — o Blender medido recusa o ciclo.
///
/// **Mutações que devem sangrar:** aceitar mover para dentro de uma descendente · mover sem os filhos.
#[test]
fn moving_a_tag_carries_its_subtree_and_refuses_a_cycle() {
    let mut t = TagTree::new();
    let flying = t.create("Flying").expect("cria");
    let boss = t.create("Flying/Boss").expect("cria");
    let enemy = t.create("Enemy").expect("cria");
    assert_eq!(t.move_under(flying, Some(enemy)), Ok(()));
    assert_eq!(
        caminhos(&t),
        vec!["Enemy", "Enemy/Flying", "Enemy/Flying/Boss"]
    );
    assert_eq!(
        t.move_under(enemy, Some(boss)),
        Err(TagError::IntoOwnSubtree)
    );
    assert_eq!(
        t.move_under(enemy, Some(enemy)),
        Err(TagError::IntoOwnSubtree)
    );
    assert_eq!(t.move_under(boss, None), Ok(()));
    assert_eq!(caminhos(&t), vec!["Boss", "Enemy", "Enemy/Flying"]);
    // Um sítio ocupado (dobrado) recusa.
    let outro = t.create("Enemy/boss").expect("cria");
    assert_eq!(
        t.move_under(outro, None),
        Err(TagError::Collision { existing: boss })
    );
}

/// ⭐ **Apagar devolve a subárvore inteira** — quem chama tira essa pertença dos objectos.
#[test]
fn deleting_returns_the_whole_subtree_and_leaves_the_neighbour() {
    let mut t = TagTree::new();
    let enemy = t.create("Enemy").expect("cria");
    let flying = t.create("Enemy/Flying").expect("cria");
    let statue = t.create("Statue").expect("cria");
    assert_eq!(t.delete(enemy), BTreeSet::from([enemy, flying]));
    assert_eq!(caminhos(&t), vec!["Statue"]);
    assert!(t.get(statue).is_some());
    assert!(t.delete(TagId(999)).is_empty());
}

/// ⭐⭐⭐ **O que `Enemy` alcança é a própria e as descendentes** — o `all_objects` do Blender (medido).
///
/// **Mutação que deve sangrar:** devolver só `{enemy}`, ou casar `Enemyx` por prefixo de texto.
#[test]
fn a_subtree_is_itself_and_its_descendants_and_never_a_text_neighbour() {
    let mut t = TagTree::new();
    let enemy = t.create("Enemy").expect("cria");
    let flying = t.create("Enemy/Flying").expect("cria");
    let boss = t.create("Enemy/Flying/Boss").expect("cria");
    let enemyx = t.create("Enemyx").expect("cria");
    assert_eq!(t.subtree(enemy), BTreeSet::from([enemy, flying, boss]));
    assert_eq!(t.subtree(flying), BTreeSet::from([flying, boss]));
    assert!(!t.subtree(enemy).contains(&enemyx));
    assert!(t.subtree(TagId(999)).is_empty());
}

/// ⭐⭐ **Um id nunca é reciclado** — a lei que o `CatalogTree` pagou na auditoria de 2026-08-30.
#[test]
fn a_restored_tree_never_recycles_an_id() {
    let (mut t, remap) = TagTree::restore(
        vec![Tag {
            id: TagId(1),
            path: "A".into(),
        }],
        // O `B`(2) foi apagado antes de gravar.
        3,
    );
    assert!(remap.is_empty());
    assert_eq!(t.create("Novo"), Ok(TagId(3)));
    let (mut sem_numero, _) = TagTree::restore(
        vec![Tag {
            id: TagId(42),
            path: "Velha".into(),
        }],
        0,
    );
    assert_ne!(sem_numero.create("Nova"), Ok(TagId(42)));
}

/// ⭐⭐⭐ **Um documento com GÉMEOS funde-os no menor id, e diz quem foi para onde** — a forma que o
/// Blender distribui no próprio ficheiro de catálogos (medido: `Brushes/Mesh Sculpt/General/Utilities`
/// e `Geometry Nodes/Generate`, cada um duas vezes com UUIDs diferentes).
///
/// ⚠️ E um gémeo DOBRADO (`Enemy` / `énemy`) é gémeo — a decisão D2 vale também ao ler.
///
/// **Mutações que devem sangrar:** manter os dois · perder o `Remap` (a pertença do gémeo sumia).
#[test]
fn a_document_with_twins_merges_them_and_says_where_each_went() {
    let (t, remap) = TagTree::restore(
        vec![
            Tag {
                id: TagId(7),
                path: "Geometry Nodes/Generate".into(),
            },
            Tag {
                id: TagId(3),
                path: "Geometry Nodes/Generate".into(),
            },
            Tag {
                id: TagId(1),
                path: "Geometry Nodes".into(),
            },
            Tag {
                id: TagId(4),
                path: "Enemy".into(),
            },
            Tag {
                id: TagId(9),
                path: "énemy".into(),
            },
        ],
        10,
    );
    assert_eq!(
        caminhos(&t),
        vec!["Enemy", "Geometry Nodes", "Geometry Nodes/Generate"]
    );
    assert_eq!(remap.get(&TagId(7)), Some(&TagId(3)));
    assert_eq!(remap.get(&TagId(9)), Some(&TagId(4)));
    assert_eq!(remap.len(), 2);
}

/// ⭐⭐ **A ordem é por nível DOBRADO**: `Ártico` antes de `Zebra`, e um pai sempre antes dos filhos.
///
/// **Mutação que deve sangrar:** ordenar pela string crua, ou pelas minúsculas sem dobrar.
#[test]
fn the_tree_sorts_by_folded_level() {
    let mut t = TagTree::new();
    for p in ["Zebra", "Ártico", "A-x", "A/B", "a"] {
        t.create(p).expect("cria");
    }
    // ⚠️ O `"a"` do fim devolve o `"A"` que o `"A/B"` já criou — a grafia é a do primeiro.
    assert_eq!(caminhos(&t), vec!["A", "A/B", "A-x", "Ártico", "Zebra"]);
}

/// ⚠️ **A revisão sobe com uma mutação e NÃO com uma recusa** — senão a cache re-codificava a árvore
/// por um gesto que não a mudou.
#[test]
fn a_refused_gesture_does_not_move_the_revision() {
    let mut t = TagTree::new();
    let a = t.create("A").expect("cria");
    let r = t.revision();
    let _ = t.create("a");
    let _ = t.rename(a, "A/B");
    let _ = t.move_under(a, Some(a));
    assert_eq!(t.revision(), r, "uma recusa mexeu na revisao");
    t.rename(a, "B").expect("renomeia");
    assert!(t.revision() > r);
}

/// ⭐⭐ **Um documento a que falta um ANCESTRAL recebe-o, e a grafia do pai passa aos filhos** — a lei
/// do [`TagTree::create`], agora na porta do load.
///
/// ⚠️ Nasceu com a cura da chave guardada (o `restore` passou a indexar num mapa local): antes dela
/// nenhum gate exercitava esta metade, e uma mutação que a apagasse sobreviveria.
///
/// **Mutações que devem sangrar:** não criar o ancestral em falta · não passar a grafia do pai ao
/// filho (a mesma linha apareceria com duas grafias) · dar ao ancestral criado um id já usado.
#[test]
fn a_document_missing_an_ancestor_gets_it_with_the_parents_spelling() {
    let (t, remap) = TagTree::restore(
        vec![
            Tag {
                id: TagId(1),
                path: "enemy".into(),
            },
            Tag {
                id: TagId(2),
                path: "ENEMY/Flying".into(),
            },
            Tag {
                id: TagId(3),
                path: "Statue/Stone/Big".into(),
            },
        ],
        0,
    );
    assert!(remap.is_empty(), "nenhum gemeo");
    assert_eq!(
        caminhos(&t),
        vec![
            "enemy",
            "enemy/Flying",
            "Statue",
            "Statue/Stone",
            "Statue/Stone/Big"
        ]
    );
    let novos: BTreeSet<TagId> = ["Statue", "Statue/Stone"]
        .iter()
        .map(|p| t.find(p).expect("o ancestral nasceu"))
        .collect();
    assert_eq!(
        novos,
        BTreeSet::from([TagId(4), TagId(5)]),
        "ids novos, a partir do maior lido"
    );
    assert_eq!(t.next_id(), 6);
}

// ─────────────────────────────────────────────────────────────────────────────
// W4 — as duas perguntas que o PAINEL faz e que a W1 não precisou de responder
// ─────────────────────────────────────────────────────────────────────────────

/// ⭐⭐⭐ **A ancestralidade de uma tag, da raiz até ela** — a porta que faz a contagem do painel
/// caber numa passagem só pelo mundo.
///
/// ⚠️ **A ordem é da RAIZ para baixo**, e não é estética: quem soma contagens quer o pai antes do
/// filho, e quem desenha um caminho lê-o na mesma direcção.
///
/// **Mutações que devem sangrar:** devolver só o próprio id · devolver a ordem invertida · incluir
/// os IRMÃOS (a chave do irmão não é prefixo da minha).
#[test]
fn the_ancestry_of_a_tag_is_the_root_down_to_itself() {
    let mut t = TagTree::new();
    let boss = t.create("Enemy/Flying/Boss").expect("cria");
    let voador = t.find("Enemy/Flying").expect("existe");
    let inimigo = t.find("Enemy").expect("existe");
    // ⚠️ O irmão existe DE PROPÓSITO: sem ele, uma implementação que devolvesse «tudo o que vem
    // antes de mim na ordem da chave» passaria.
    let _ = t.create("Enemy/Ground").expect("cria");

    assert_eq!(t.ancestry(boss), vec![inimigo, voador, boss]);
    assert_eq!(t.ancestry(inimigo), vec![inimigo]);
    assert_eq!(
        t.ancestry(TagId(9_999)),
        Vec::<TagId>::new(),
        "um id que não existe não ascende a nada"
    );
}

/// ⚠️ **A ancestralidade nunca inventa um elo que a árvore não tem.**
///
/// Uma tag de raiz ao lado de outra é a dela própria e nada mais — a fixtura tem as duas raízes
/// porque é assim que o painel as desenha, e a mutação que devolvesse «todas as de profundidade
/// menor» passaria sem ela.
///
/// **Mutação que deve sangrar:** derivar a ancestralidade da PROFUNDIDADE em vez do caminho.
#[test]
fn a_sibling_root_is_never_an_ancestor() {
    let mut t = TagTree::new();
    let statue = t.create("Statue").expect("cria");
    let enemy = t.create("Enemy").expect("cria");
    let flying = t.create("Enemy/Flying").expect("cria");
    assert_eq!(t.ancestry(flying), vec![enemy, flying]);
    assert_eq!(t.ancestry(statue), vec![statue]);
}

/// ⭐⭐ **Toda recusa da árvore tem uma frase, e ela vive AO LADO DA LEI que a produz.**
///
/// ⛔ O painel não traduz um `TagError` — se traduzisse, a segunda superfície que mostrasse a mesma
/// recusa escreveria outra frase, e as duas divergiriam na primeira vez que alguém mexesse numa.
///
/// **Mutações que devem sangrar:** duas variantes a partilharem a frase · uma frase vazia.
#[test]
fn every_refusal_says_why_in_words_and_no_two_say_the_same() {
    let todas = [
        TagError::Empty,
        TagError::HasSeparator,
        TagError::Collision { existing: TagId(1) },
        TagError::IntoOwnSubtree,
        TagError::Missing,
    ];
    // ⚠️ Desde 2026-09-19 esta folha publica a CHAVE e não a frase, logo o que este controlo
    //    afirma é que as chaves são distintas e não vazias. ⭐ Quem garante que cada uma tem uma
    //    frase do outro lado é a `ph2d-i18n` (uma chave sem entrada sai CRUA no idioma de teste, e
    //    há gate a dizê-lo) — *a recusa muda continua impossível, e agora por duas réguas.*
    let chaves: BTreeSet<&str> = todas.iter().map(|e| e.message_key()).collect();
    assert_eq!(chaves.len(), todas.len(), "duas recusas com a mesma chave");
    assert!(
        todas
            .iter()
            .all(|e| e.message_key().starts_with("tags.error.")
                && !e.message_key().trim_start_matches("tags.error.").is_empty()),
        "uma recusa sem chave é uma recusa muda"
    );
}
