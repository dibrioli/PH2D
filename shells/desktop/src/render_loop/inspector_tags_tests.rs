//! **Os gates da costura da secção TAGS** — irmão de [`crate::render_loop::inspector_tags`] por CAP
//! de LOC.
//!
//! ⚠️ **Só a shell vê os DOIS documentos**: a árvore vive numa folha sem ECS, o conjunto do objecto
//! vive no `ph2d-ecs`, e o painel não conhece nenhum dos dois. É aqui que se prova que o snapshot
//! diz o que o par tem e que o commit escreve nos dois no mesmo gesto.

use super::*;
use ph2d_ecs::scene::{EditorCommandQueue, apply_editor_commands, register_ecs_components};
use ph2d_ecs::tags::{TAGS_MAX, Tags};
use ph2d_ecs::{SimWorld, Transform};
use ph2d_editor_core::TagsFieldEdit;
use ph2d_tags::{TagId, TagTree};

fn registry() -> ComponentRegistry {
    let mut r = ComponentRegistry::new();
    register_ecs_components(&mut r);
    r
}

/// A árvore das fixturas: `Boss` · `Enemy` · `Enemy/Flying` · `Prop`, na ordem da ÁRVORE.
///
/// ⛔⛔ **A ordem de CRIAÇÃO é load-bearing, e foi uma mutação SOBREVIVENTE que o mostrou.** Os ids
/// saem da criação e o `Tags` guarda-os num `BTreeSet` ⇒ derivar os chips do conjunto do componente
/// dá a ordem dos **ids**. Criada por ordem alfabética, a árvore faz as duas ordens **coincidirem**,
/// e a mutação que troca uma pela outra fica verde sobre um gate que parecia medi-la: *a fixtura não
/// continha o fenómeno*. Criar o `Prop` primeiro e o `Boss` por último fá-las discordar:
///
/// | | ordem |
/// |---|---|
/// | ids (criação) | `Prop(1)` · `Enemy(2)` · `Enemy/Flying(3)` · `Boss(4)` |
/// | árvore (dobrada) | `Boss` · `Enemy` · `Enemy/Flying` · `Prop` |
fn arvore() -> (TagTree, [TagId; 4]) {
    let mut t = TagTree::new();
    let prop = t.create("Prop").expect("cria");
    let enemy = t.create("Enemy").expect("cria");
    let flying = t.create("Enemy/Flying").expect("cria");
    let boss = t.create("Boss").expect("cria");
    (t, [enemy, flying, boss, prop])
}

fn objecto(sim: &mut SimWorld, tags: impl IntoIterator<Item = TagId>) -> ph2d_ecs::Entity {
    sim.world_mut()
        .spawn((Transform::default(), Tags::from_ids(tags)))
        .id()
}

/// Corre uma edição pela porta e aplica a fila, como o quadro faz. Devolve se a árvore mudou.
fn edit(
    sim: &mut SimWorld,
    tree: &mut TagTree,
    e: ph2d_ecs::Entity,
    reg: &ComponentRegistry,
    ed: TagsFieldEdit,
) -> bool {
    let queue = EditorCommandQueue::new();
    let mexeu = apply_tags_edit(sim.world(), tree, e.to_bits(), &ed, &queue, reg);
    apply_editor_commands(sim.world_mut(), &queue, reg).expect("o commit aplica");
    mexeu
}

/// ⚠️ **O painel desenha um chip por tag que o modelo aceita** — a lei que o `ANIM_TAGS_MAX` já
/// pagou, e que só a SHELL pode afirmar: a crate do painel é chrome e não depende do `ph2d-ecs`.
///
/// **Mutação que deve sangrar:** o `TAGS_MAX` a subir sem o array de ids o acompanhar.
#[test]
fn the_tag_chip_ids_cover_the_model_cap() {
    assert_eq!(
        ph2d_panel_inspector::ids::INSP_TAGS_CHIP.len(),
        TAGS_MAX,
        "o painel desenha {} chips para um modelo de {TAGS_MAX} tags — as que sobram seriam \
         inalcancaveis por gesto nenhum",
        ph2d_panel_inspector::ids::INSP_TAGS_CHIP.len(),
    );
}

/// ⛔ **Um objecto SEM `Tags` não tem secção** — ADR-0166. Um `Some` aqui poria uma secção vazia em
/// todo objecto do projecto, e a paleta deixaria de ter o que anexar.
///
/// **Mutação que deve sangrar:** o `?` do `world.get::<Tags>` trocado por um `unwrap_or_default`.
#[test]
fn an_object_without_the_component_has_no_section() {
    let (tree, _) = arvore();
    let mut sim = SimWorld::default();
    let e = sim.world_mut().spawn((Transform::default(),)).id();
    assert!(
        build_tags_info(sim.world(), &tree, e.to_bits(), 1).is_none(),
        "um objecto sem Tags recebeu a seccao — ela apareceria em TODO objecto, vazia"
    );
}

/// ⭐⭐ **As duas listas do snapshot são duas perguntas, e as duas vêm na ordem da ÁRVORE.**
///
/// ⚠️ A ordem é a que separa esta secção de uma que derivasse os chips do conjunto do componente:
/// aquela daria a ordem dos **ids** (a de criação), e o `Prop`, criado depois, apareceria antes do
/// `Enemy/Flying` na lista de um objecto que tem os dois.
///
/// **Mutação que deve sangrar:** `on_object` construído de `tags.direct_ids()` em vez da travessia.
#[test]
fn the_snapshot_carries_the_object_and_the_project_in_tree_order() {
    let (tree, [enemy, flying, _boss, prop]) = arvore();
    let mut sim = SimWorld::default();
    // ⚠️ Postas pela ordem CONTRÁRIA à da árvore, de propósito.
    let e = objecto(&mut sim, [prop, flying, enemy]);
    let i = build_tags_info(sim.world(), &tree, e.to_bits(), 1).expect("tem Tags");
    // ⛔⛔ **A fixtura CONTÉM o fenómeno** — esta metade é o que faltava quando a mutação
    // sobreviveu: sem ela, uma árvore criada por ordem alfabética faz a ordem dos IDS coincidir com
    // a da ÁRVORE, e a asserção seguinte passa a medir NADA.
    let mut por_id = i.on_object.clone();
    por_id.sort_by_key(|r| r.id);
    assert_ne!(
        por_id.iter().map(|r| r.path.as_str()).collect::<Vec<_>>(),
        i.on_object
            .iter()
            .map(|r| r.path.as_str())
            .collect::<Vec<_>>(),
        "a fixtura nao produz o fenomeno: a ordem dos IDS e a da ARVORE coincidem, logo a asseracao \
         seguinte nao distingue as duas"
    );
    assert_eq!(
        i.on_object
            .iter()
            .map(|r| r.path.as_str())
            .collect::<Vec<_>>(),
        ["Enemy", "Enemy/Flying", "Prop"],
        "os chips nao vieram na ordem da arvore"
    );
    // ⭐ **A ÁRVORE vem pela OUTRA porta** — ela não é dado deste objecto (ver o cabeçalho do
    // modelo), e é a mesma travessia que a alimenta.
    let arv = tag_tree_rows(&tree);
    assert_eq!(
        arv.iter().map(|r| r.path.as_str()).collect::<Vec<_>>(),
        ["Boss", "Enemy", "Enemy/Flying", "Prop"],
        "a lista do projecto nao e' a arvore inteira, na ordem dela"
    );
    // O rótulo é o ÚLTIMO nível e a profundidade vem com ele — é o que o chip desenha.
    let f = arv.iter().find(|r| r.path == "Enemy/Flying").expect("la'");
    assert_eq!((f.label.as_str(), f.depth), ("Flying", 1));
    assert!(!i.full);
}

/// ⛔⛔ **A árvore do projecto é publicada MESMO sem objecto com `Tags`** — é a razão de ela ter
/// porta própria: a caixa de escolha do alvo de uma *Signal Action* vive num objecto que pode não
/// ter o componente, e com a lista dentro do instantâneo por objecto ela abria vazia.
///
/// **Mutação que deve sangrar:** a `tag_tree_rows` a devolver vazio quando ninguém está escolhido
/// (ou a publicação a mudar-se para dentro do `and_then` da selecção).
#[test]
fn the_project_tree_is_published_even_with_no_tagged_object() {
    let (tree, _) = arvore();
    let mut sim = SimWorld::default();
    // Um objecto SEM o componente — o instantâneo dele é `None`…
    let e = sim.world_mut().spawn((Transform::default(),)).id();
    assert!(build_tags_info(sim.world(), &tree, e.to_bits(), 1).is_none());
    // …e a árvore continua a ter as quatro.
    assert_eq!(
        tag_tree_rows(&tree).len(),
        4,
        "a arvore do projecto veio vazia para um objecto sem `Tags` — a caixa de escolha do alvo          de uma accao abriria sem nada para oferecer"
    );
}

/// ⭐ **Escolher uma tag da lista marca o objecto — e só isso.**
///
/// **Mutação que deve sangrar:** o `queue_set` fora do `if mudou` (escreveria sempre) ou a ausência
/// dele (o chip nasceria e morria no mesmo quadro).
#[test]
fn choosing_a_tag_marks_the_object_and_leaves_the_tree_alone() {
    let (mut tree, [enemy, _f, boss, _p]) = arvore();
    let reg = registry();
    let mut sim = SimWorld::default();
    let e = objecto(&mut sim, [enemy]);
    let rev = tree.revision();
    assert!(!edit(
        &mut sim,
        &mut tree,
        e,
        &reg,
        TagsFieldEdit::Add(boss.0)
    ));
    assert_eq!(
        sim.world().get::<Tags>(e),
        Some(&Tags::from_ids([enemy, boss])),
        "a escolha nao marcou o objecto"
    );
    assert_eq!(tree.revision(), rev, "marcar um objecto mexeu na arvore");
    // Tirar devolve o objecto ao que era, e continua sem tocar na árvore — ⛔ *Remove* é do CHIP,
    // nunca da tag (apagar a tag é do painel Tags, W4).
    assert!(!edit(
        &mut sim,
        &mut tree,
        e,
        &reg,
        TagsFieldEdit::Remove(boss.0)
    ));
    assert_eq!(sim.world().get::<Tags>(e), Some(&Tags::from_ids([enemy])));
    assert_eq!(tree.len(), 4, "o Remove apagou a tag da arvore");
}

/// ⭐⭐⭐ **`Create` escreve nos DOIS documentos no mesmo gesto** — e cria os ancestrais que faltarem.
///
/// **Mutação que deve sangrar:** o `t.insert(id)` apagado (a tag nasceria na lista e o objecto
/// ficaria sem o chip — exactamente o meio-gesto que o cabeçalho do módulo proíbe).
#[test]
fn creating_a_tag_from_the_section_writes_both_documents() {
    let (mut tree, _) = arvore();
    let reg = registry();
    let mut sim = SimWorld::default();
    let e = objecto(&mut sim, []);
    assert!(
        edit(
            &mut sim,
            &mut tree,
            e,
            &reg,
            TagsFieldEdit::Create("Enemy/Ground/Heavy".into())
        ),
        "a arvore nao acusou a mudanca"
    );
    let novo = tree.find("Enemy/Ground/Heavy").expect("a tag existe");
    assert_eq!(
        sim.world().get::<Tags>(e),
        Some(&Tags::from_ids([novo])),
        "criou a tag e nao marcou o objecto"
    );
    // O ancestral que faltava nasceu com ela, e o que já existia manteve a GRAFIA dele.
    assert!(tree.find("Enemy/Ground").is_some());
    assert_eq!(tree.get(novo).expect("la'").path, "Enemy/Ground/Heavy");
}

/// ⚠️ **Criar um caminho que já existe NÃO é uma mudança** — a resposta é a revisão comparada, e
/// não *«chamei o `create`»*.
///
/// **Mutação que deve sangrar:** o `apply_tags_edit` a devolver `mudou` em vez da revisão.
#[test]
fn creating_a_path_that_already_exists_changes_nothing_in_the_tree() {
    let (mut tree, [_e, flying, _b, _p]) = arvore();
    let reg = registry();
    let mut sim = SimWorld::default();
    let e = objecto(&mut sim, []);
    assert!(
        !edit(
            &mut sim,
            &mut tree,
            e,
            &reg,
            // ⚠️ Com OUTRA grafia — a árvore compara dobrado (D2 do dono).
            TagsFieldEdit::Create("enemy/FLYING".into())
        ),
        "criar um caminho que ja' existe foi reportado como mudanca da arvore"
    );
    assert_eq!(tree.len(), 4, "a arvore cresceu num caminho que ja' tinha");
    assert_eq!(
        sim.world().get::<Tags>(e),
        Some(&Tags::from_ids([flying])),
        "o objecto nao ficou com a tag que ja' existia"
    );
}

/// ⛔⛔ **Um objecto cheio não faz a árvore crescer** — meio gesto deixaria na lista uma tag que
/// ninguém pediu e nenhum chip para a mostrar.
///
/// **Mutação que deve sangrar:** o guarda `t.len() >= TAGS_MAX` apagado do braço `Create`.
#[test]
fn a_full_object_does_not_grow_the_tree_with_a_tag_it_cannot_take() {
    let mut tree = TagTree::new();
    let cheio: Vec<TagId> = (0..TAGS_MAX)
        .map(|i| tree.create(&format!("T{i}")).expect("cria"))
        .collect();
    let reg = registry();
    let mut sim = SimWorld::default();
    let e = objecto(&mut sim, cheio);
    let (n, rev) = (tree.len(), tree.revision());
    assert!(!edit(
        &mut sim,
        &mut tree,
        e,
        &reg,
        TagsFieldEdit::Create("Nova".into())
    ));
    assert_eq!(tree.len(), n, "a arvore cresceu para um objecto cheio");
    assert_eq!(tree.revision(), rev);
    assert!(tree.find("Nova").is_none());
    // E o snapshot diz que está cheio — a secção esconde a caixa em vez de oferecer um gesto que o
    // `Tags::insert` vai recusar.
    let i = build_tags_info(sim.world(), &tree, e.to_bits(), 1).expect("tem Tags");
    assert!(i.full, "o snapshot nao disse que o objecto esta' cheio");
    assert_eq!(i.on_object.len(), TAGS_MAX);
}

/// ⚠️ **Um id ÓRFÃO ocupa lugar e não se desenha** — é o que sobra de uma tag apagada sem o
/// `scrub`, e dizer *«cabe»* ali ofereceria um gesto que o `Tags::insert` recusa.
///
/// **Mutação que deve sangrar:** o `full` medido por `on_object.len()` em vez de `tags.len()`.
#[test]
fn an_orphan_id_takes_room_without_painting_a_chip() {
    let mut tree = TagTree::new();
    let ids: Vec<TagId> = (0..TAGS_MAX)
        .map(|i| tree.create(&format!("T{i}")).expect("cria"))
        .collect();
    let mut sim = SimWorld::default();
    let e = objecto(&mut sim, ids.clone());
    // A última é apagada da árvore SEM o `scrub` — o objecto fica com o id órfão.
    let _ = tree.delete(ids[TAGS_MAX - 1]);
    let i = build_tags_info(sim.world(), &tree, e.to_bits(), 1).expect("tem Tags");
    assert_eq!(
        i.on_object.len(),
        TAGS_MAX - 1,
        "um id orfao foi desenhado como chip"
    );
    assert!(
        i.full,
        "o orfao deixou de contar para o tecto — a seccao ofereceria um gesto recusado"
    );
}
