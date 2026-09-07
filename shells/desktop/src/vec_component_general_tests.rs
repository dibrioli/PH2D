//! Os gates do modo GERAL da secção *Component* (F4.6c, wave 1).
//!
//! ⚠️ **O oráculo é o que a SECÇÃO oferece e o que o clique FAZ** — nunca *«a função devolveu
//! algo»*. Uma secção que oferecesse *Detach* sobre uma forma comum, ou um *Place* que não põe
//! cópia nenhuma, passaria num gate escrito sobre a chamada.

use super::{armed, dispatch, state_of};
use crate::instance_docs::OwnedDocs;
use crate::vec_component_edit::ComponentEdit;
use ph2d_ecs::{Children, Entity, MasterRoot, Name, SimWorld, Transform};
use ph2d_vec_scene::VecPathId;

fn reg() -> ph2d_ecs::scene::ComponentRegistry {
    crate::init::build_component_registry()
}

/// Um mundo com UMA forma, e o mapa `path ⟺ entidade` que o painel usa.
fn scene() -> (
    SimWorld,
    ph2d_ecs::scene::ComponentRegistry,
    crate::vec_entities::VecEntityMap,
    VecPathId,
    Entity,
) {
    let mut sim = SimWorld::new();
    let r = reg();
    let e = sim
        .world_mut()
        .spawn((Transform::IDENTITY, Name::new("Badge")))
        .id();
    ph2d_ecs::assign_missing_stable_ids(sim.world_mut());
    let id: VecPathId = 1;
    let mut map = crate::vec_entities::VecEntityMap::default();
    map.insert(id, e.to_bits());
    (sim, r, map, id, e)
}

fn run(
    verb: ComponentEdit,
    sim: &mut SimWorld,
    r: &ph2d_ecs::scene::ComponentRegistry,
    subject: Entity,
    toasts: &mut ph2d_editor::ToastQueue,
) -> (bool, Option<u64>) {
    let (mut sc, mut mp) = crate::instance_docs::empty_docs();
    let mut echo = crate::instance_sync::MasterEcho::default();
    let mut select_out = None;
    let changed = dispatch(
        verb,
        sim,
        r,
        &mut echo,
        subject,
        toasts,
        &mut OwnedDocs {
            vec_scene: &mut sc,
            vec_entities: &mut mp,
        },
        [0.25, 0.0],
        &mut select_out,
    );
    (changed, select_out)
}

/// ⛔⛔⛔ **A PORTA É UMA SÓ, e sem a env var ela está FECHADA.**
///
/// É o que torna esta wave incapaz de regredir: os dois sítios que decidem (o que a secção MOSTRA e
/// o que o clique FAZ) lêem a mesma função, e sem a variável o caminho de omissão é o de sempre.
///
/// ⚠️ **Este gate corre no processo de teste**, onde a env var não está posta — ele mede a
/// omissão, que é exactamente o estado em que o dono vai receber a build.
#[test]
fn the_new_mode_is_closed_unless_the_env_var_opens_it() {
    assert!(
        !armed(),
        "o modo novo esta' ARMADO por omissao — a wave passa a poder regredir o editor vetorial"
    );
}

/// ⭐⭐⭐ **A secção descreve o modelo GERAL: uma forma comum só oferece PROMOVER.**
#[test]
fn a_plain_shape_offers_only_create() {
    let (mut sim, _r, map, id, _e) = scene();
    let s = state_of(&mut sim, &map, &[id]).expect("uma forma tem seccao");
    assert!(
        !s.is_main && !s.is_instance,
        "uma forma comum nao e' nenhum dos dois"
    );
    assert!(!s.has_overrides && !s.main_missing);
}

/// ⭐⭐⭐ **CRIAR pelo painel faz um componente de VERDADE — e deixa uma cópia no lugar.**
///
/// ⚠️ **As duas metades são a lei do `Make` geral:** a receita esconde-se (ela é a biblioteca) e a
/// cópia fica onde a forma estava, **seleccionada**. Sem a segunda, o artista fica a olhar para o
/// nada com a receita invisível na mão — o defeito que o `select_out` do dreno geral existe para
/// curar.
///
/// (Mutação: `general_verb(Create)` a devolver `None` ⇒ RED.)
#[test]
fn create_through_the_panel_makes_a_real_prefab_and_leaves_a_copy() {
    let (mut sim, r, _map, _id, e) = scene();
    let mut toasts = ph2d_editor::ToastQueue::default();
    let (changed, select_out) = run(ComponentEdit::Create, &mut sim, &r, e, &mut toasts);
    assert!(changed, "o Create pelo painel nao mudou nada");
    assert!(
        sim.world().get::<MasterRoot>(e).is_some(),
        "a forma escolhida nao virou RECEITA"
    );
    let copy = select_out
        .map(Entity::from_bits)
        .expect("a seleccao segue a copia");
    assert_ne!(copy, e, "a seleccao ficou na receita, que e' invisivel");
    assert!(
        sim.world().get::<ph2d_ecs::InstanceOf>(copy).is_some(),
        "o que ficou no lugar nao e' uma copia da receita"
    );
}

/// ⭐⭐ **E a secção passa a oferecer *Place*** — porque agora a forma É um mestre no modelo geral.
#[test]
fn after_create_the_section_offers_place() {
    let (mut sim, r, map, id, e) = scene();
    let mut toasts = ph2d_editor::ToastQueue::default();
    run(ComponentEdit::Create, &mut sim, &r, e, &mut toasts);
    let s = state_of(&mut sim, &map, &[id]).expect("a seccao existe");
    assert!(s.is_main, "a receita nao se anuncia como mestre");
}

/// ⛔⛔⛔ **E ela oferece-o ONDE A SELECÇÃO FICOU — na CÓPIA** (report do Enio, 2026-09-06:
/// *«não existe mais a opção instanciate»*).
///
/// # Porque o gate acima estava VERDE por cima deste defeito
///
/// Ele mede `state_of` sobre `id`, o path **original**, que o mapa ainda liga ao mestre — ou seja,
/// **a porta em que o artista não está**. O teste imediatamente antes dele AFIRMA que o
/// `select_out` aponta para outra entidade, e nenhum dos dois compõe o outro: *duas metades certas
/// que nunca se encontram*. No app, o `select_out` vai ao gizmo e o gizmo volta ao pen, então a
/// pergunta seguinte da secção é feita sobre a **cópia** — e ali `is_main` era `false`, com o
/// botão a desaparecer para sempre (a receita fica invisível no canvas).
///
/// **Mutação que deve sangrar:** `is_main` voltar a ser `get::<MasterRoot>(e).is_some()`.
#[test]
fn the_section_still_offers_place_where_the_selection_landed() {
    let (mut sim, r, mut map, _id, e) = scene();
    let mut toasts = ph2d_editor::ToastQueue::default();
    let (_, select_out) = run(ComponentEdit::Create, &mut sim, &r, e, &mut toasts);
    let copy = select_out
        .map(Entity::from_bits)
        .expect("a seleccao segue a copia");

    // É o que a shell faz: o pen passa a ter o path da cópia seleccionado.
    let copy_id: VecPathId = 2;
    map.insert(copy_id, copy.to_bits());
    let s = state_of(&mut sim, &map, &[copy_id]).expect("a seccao existe sobre a copia");

    assert!(
        s.is_main,
        "a seccao nao oferece Instantiate sobre a copia — o gesto seguinte da fila e' inalcancavel"
    );
    assert!(s.is_instance, "a copia deixou de se anunciar como copia");
}

/// ⭐⭐⭐ **E o botão que voltou não é um botão morto: o verbo FUNCIONA a partir da cópia.**
///
/// ⚠️ **A metade justa.** Alargar a condição que PINTA sem provar que o consumidor aceita o mesmo
/// sujeito seria trocar um botão ausente por um botão mudo — o defeito que este repo caça. O
/// `Verb::Place` resolve a receita a partir de uma cópia desde 2026-08-31
/// (`instance_verbs_walk::master_subject`), e é isso que este gate fixa.
#[test]
fn instantiating_from_the_copy_adds_another_copy() {
    let (mut sim, r, _map, _id, e) = scene();
    let mut toasts = ph2d_editor::ToastQueue::default();
    let (_, select_out) = run(ComponentEdit::Create, &mut sim, &r, e, &mut toasts);
    let copy = select_out.map(Entity::from_bits).expect("a copia");
    let before = copies_of(&mut sim, e);

    let (changed, _) = run(ComponentEdit::Place, &mut sim, &r, copy, &mut toasts);

    assert!(changed, "instanciar a partir da copia nao fez nada");
    assert_eq!(
        copies_of(&mut sim, e),
        before + 1,
        "o numero de copias da receita nao subiu — o botao seria pintado sobre um verbo mudo"
    );
}

/// ⭐⭐⭐ **PLACE põe uma cópia a mais, e ela é uma sub-árvore de ENTIDADES** — a diferença que o
/// motor velho não tem (lá a cópia é um rectângulo de suporte com o desenho derivado).
///
/// (Mutação: `general_verb(Place)` a devolver `None` ⇒ RED.)
#[test]
fn place_through_the_panel_adds_a_second_real_copy() {
    let (mut sim, r, _map, _id, e) = scene();
    // A receita precisa de uma peça, senão «a cópia é uma sub-árvore» não mede nada.
    sim.world_mut().spawn((
        Transform::IDENTITY,
        Name::new("Box"),
        ph2d_render::Sprite::atlas(0, [1.0, 1.0], [1.0; 4]),
        ph2d_ecs::ChildOf(e),
    ));
    ph2d_ecs::assign_missing_stable_ids(sim.world_mut());
    let mut toasts = ph2d_editor::ToastQueue::default();
    run(ComponentEdit::Create, &mut sim, &r, e, &mut toasts);
    let before = copies_of(&mut sim, e);
    let (changed, _) = run(ComponentEdit::Place, &mut sim, &r, e, &mut toasts);
    assert!(changed, "o Place pelo painel nao pos copia nenhuma");
    let after = copies_of(&mut sim, e);
    assert_eq!(after, before + 1, "o numero de copias nao subiu");
    // ⚠️ A metade que separa os dois motores: a cópia tem a PEÇA, como entidade.
    let last = last_copy(&mut sim, e).expect("a copia nova");
    assert!(
        sim.world()
            .get::<Children>(last)
            .is_some_and(|k| !k.is_empty()),
        "a copia nasceu sem pecas — ela nao e' uma sub-arvore"
    );
}

/// Quantas cópias vivas este mestre tem.
fn copies_of(sim: &mut SimWorld, master: Entity) -> usize {
    let Some(id) = sim.world().get::<ph2d_ecs::StableId>(master).map(|s| s.0) else {
        return 0;
    };
    let mut q = sim.world_mut().query::<&ph2d_ecs::InstanceOf>();
    q.iter(sim.world()).filter(|l| l.master == id).count()
}

fn last_copy(sim: &mut SimWorld, master: Entity) -> Option<Entity> {
    let id = sim.world().get::<ph2d_ecs::StableId>(master).map(|s| s.0)?;
    let mut q = sim.world_mut().query::<(Entity, &ph2d_ecs::InstanceOf)>();
    let mut all: Vec<Entity> = q
        .iter(sim.world())
        .filter(|(_, l)| l.master == id)
        .map(|(e, _)| e)
        .collect();
    all.sort_by_key(|e| e.to_bits());
    all.pop()
}

/// ⛔⛔ **O conta-gotas do *Swap* FALA em vez de morrer.**
///
/// O painel pinta aquele botão para toda instância, e no modo novo ele não tem consumidor. *Um
/// controlo que come o clique em silêncio é pior que um ausente* — e a voz nomeia a saída que
/// **existe**: o *Replace selection with this* da biblioteca.
///
/// (Mutação: apagar o braço do `Swap` ⇒ RED.)
#[test]
fn the_eyedropper_swap_says_where_to_do_it_instead() {
    let (mut sim, r, _map, _id, e) = scene();
    let mut toasts = ph2d_editor::ToastQueue::default();
    let (changed, _) = run(ComponentEdit::Swap, &mut sim, &r, e, &mut toasts);
    assert!(!changed, "o Swap mudou o mundo no modo novo");
    assert!(
        toasts.iter().any(|t| t.message.contains("Replace")),
        "o Swap comeu o clique em silencio — nao ha' voz a dizer onde fazer"
    );
}
