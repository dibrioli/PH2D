//! Os gates do modo GERAL da secção *Prefab* (F4.6c, waves 1 a 3 — desde 2026-09-06 ele é o
//! caminho de OMISSÃO).
//!
//! ⚠️ **O oráculo é o que a SECÇÃO oferece e o que o clique FAZ** — nunca *«a função devolveu
//! algo»*. Uma secção que oferecesse *Detach* sobre uma forma comum, ou um *Place* que não põe
//! cópia nenhuma, passaria num gate escrito sobre a chamada.

use super::{dispatch, state_of};
use ph2d_app_components::instance_docs::OwnedDocs;
use crate::vec_component_edit::ComponentEdit;
use ph2d_ecs::{Children, Entity, MasterRoot, Name, SimWorld, Transform};
use ph2d_vec_scene::VecPathId;

fn reg() -> ph2d_ecs::scene::ComponentRegistry {
    crate::init::build_component_registry()
}

/// Um mundo com UMA forma, e o mapa `path ⟺ entidade` que o painel usa.
pub(super) fn scene() -> (
    SimWorld,
    ph2d_ecs::scene::ComponentRegistry,
    ph2d_vec_entities::entities::VecEntityMap,
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
    let mut map = ph2d_vec_entities::entities::VecEntityMap::default();
    map.insert(id, e.to_bits());
    (sim, r, map, id, e)
}

pub(super) fn run(
    verb: ComponentEdit,
    sim: &mut SimWorld,
    r: &ph2d_ecs::scene::ComponentRegistry,
    subject: Entity,
    toasts: &mut ph2d_editor::ToastQueue,
) -> (bool, Option<u64>) {
    run_full(verb, sim, r, subject, toasts).0
}

/// O mesmo dreno, devolvendo também **se o conta-gotas armou** — é a saída que o gesto de duas
/// mãos usa, e a que separa *«o verbo não fez nada»* de *«o verbo abriu um gesto»*.
pub(super) fn run_full(
    verb: ComponentEdit,
    sim: &mut SimWorld,
    r: &ph2d_ecs::scene::ComponentRegistry,
    subject: Entity,
    toasts: &mut ph2d_editor::ToastQueue,
) -> ((bool, Option<u64>), bool) {
    let (mut sc, mut mp) = ph2d_app_components::instance_docs::empty_docs();
    let mut echo = ph2d_app_components::instance_sync::MasterEcho::default();
    let mut select_out = None;
    let mut arm_pick = false;
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
        &mut arm_pick,
    );
    ((changed, select_out), arm_pick)
}

/// ⭐⭐⭐ **A secção descreve o modelo GERAL: uma forma comum só oferece PROMOVER.**
#[test]
fn a_plain_shape_offers_only_create() {
    let (mut sim, _r, map, id, _e) = scene();
    let s = state_of(&mut sim, &map, &[id], None, false).expect("uma forma tem seccao");
    assert!(
        !s.is_main && !s.is_instance,
        "uma forma comum nao e' nenhum dos dois"
    );
    assert!(!s.has_overrides && !s.main_missing);
}

/// ⛔⛔⛔ **UM GRUPO é sujeito, e ele não é um path** (report do Enio, 2026-09-07: *«com a pasta do
/// grupo seleccionada não aparecem as opções de prefab; aparecem ao clicar nos filhos»*).
///
/// # O mecanismo, e por que é a QUARTA vez
///
/// A secção perguntava **só à caneta**, e um grupo é uma entidade **comum com filhos** — sem
/// `VecPathRef`, logo não é path nenhum. Os verbos gerais trabalham sobre ENTIDADES, e o menu da
/// Hierarquia já o aceitava — *que é, aliás, o único sítio por onde se faz um prefab de várias
/// formas*. ⇒ a lente do painel outra vez mais estreita que a do verbo.
///
/// ⚠️ **As duas metades:** com o grupo na mão o sujeito é o GRUPO (não um filho), e com duas formas
/// soltas — nenhum objecto único seleccionado — não há secção nenhuma.
///
/// **Mutação que deve sangrar:** o `subject_of` a devolver `None` em vez do `single_selected`.
#[test]
fn a_group_is_a_subject_even_though_it_is_not_a_path() {
    let mut sim = SimWorld::new();
    let group = sim
        .world_mut()
        .spawn((Transform::IDENTITY, Name::new("Group 2")))
        .id();
    let mut map = ph2d_vec_entities::entities::VecEntityMap::default();
    for (i, id) in [(1u64, 10 as VecPathId), (2, 11)] {
        let child = sim
            .world_mut()
            .spawn((
                Transform::IDENTITY,
                Name::new(format!("Shape {i}")),
                ph2d_ecs::ChildOf(group),
            ))
            .id();
        map.insert(id, child.to_bits());
    }
    ph2d_ecs::assign_missing_stable_ids(sim.world_mut());

    // A caneta tem os DOIS filhos; o gizmo tem o grupo.
    let both = [10 as VecPathId, 11];
    assert_eq!(
        super::subject_of(&map, &both, Some(group.to_bits())),
        Some(group),
        "o sujeito nao e' o grupo — a seccao falaria de um filho"
    );
    let s = state_of(&mut sim, &map, &both, Some(group.to_bits()), false)
        .expect("a seccao existe sobre o grupo");
    assert!(
        !s.is_main && !s.is_instance,
        "um grupo comum nao e' receita nem copia — ele oferece PROMOVER"
    );
    // E sem objecto único na mão (duas formas soltas), não há secção.
    assert!(
        state_of(&mut sim, &map, &both, None, false).is_none(),
        "a seccao apareceu sobre uma seleccao de duas coisas"
    );
}

/// ⭐⭐ **E promover o grupo dá um prefab cuja cópia é uma SUB-ÁRVORE** — que é o que o artista vê
/// como «pastinha que abre».
#[test]
fn promoting_a_group_gives_a_copy_with_the_pieces_inside() {
    let mut sim = SimWorld::new();
    let r = reg();
    let group = sim
        .world_mut()
        .spawn((Transform::IDENTITY, Name::new("Group 2")))
        .id();
    for i in 1..=2 {
        sim.world_mut().spawn((
            Transform::IDENTITY,
            Name::new(format!("Shape {i}")),
            ph2d_render::Sprite::atlas(0, [1.0, 1.0], [1.0; 4]),
            ph2d_ecs::ChildOf(group),
        ));
    }
    ph2d_ecs::assign_missing_stable_ids(sim.world_mut());
    let mut toasts = ph2d_editor::ToastQueue::default();

    let (changed, out) = run(ComponentEdit::Create, &mut sim, &r, group, &mut toasts);

    assert!(changed, "promover o grupo nao fez nada");
    let copy = out.map(Entity::from_bits).expect("a copia do grupo");
    assert_eq!(
        sim.world().get::<Children>(copy).map(|k| k.len()),
        Some(2),
        "a copia do grupo nasceu sem as pecas — nao ha' pastinha que abra"
    );
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
    let s = state_of(&mut sim, &map, &[id], None, false).expect("a seccao existe");
    assert!(s.is_main, "a receita nao se anuncia como mestre");
}

/// ⛔⛔⛔ **E ela oferece-o ONDE A SELECÇÃO FICOU — na CÓPIA** (report do Enio, 2026-09-06:
/// *«não existe mais a opção de instanciar»*).
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
    let s =
        state_of(&mut sim, &map, &[copy_id], None, false).expect("a seccao existe sobre a copia");

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

/// ⛔⛔⛔ **A secção oferece FAZER UMA VERSÃO NOVA a partir de uma cópia** (report do Enio,
/// 2026-09-06: *«Make Prefab só aparece no menu da hierarchy e não no painel vector»*).
///
/// O menu da Hierarquia já o oferecia porque a **tabela dele é plana** — ela não sabe o que a linha
/// é. O painel sabe, e a regra que ele herdou do motor vetorial dizia *«uma cópia não é candidata a
/// promover»*, que ali era verdade: lá não há variantes. ⇒ *a terceira vez, nesta secção, em que a
/// lente do painel era mais estreita que a do verbo.*
///
/// **Mutação que deve sangrar:** `can_make_variant: link.is_some()` a virar `false`.
#[test]
fn the_section_offers_making_a_variant_out_of_a_copy() {
    let (mut sim, r, mut map, id, e) = scene();
    let mut toasts = ph2d_editor::ToastQueue::default();
    assert!(
        !state_of(&mut sim, &map, &[id], None, false)
            .expect("a seccao existe")
            .can_make_variant,
        "uma forma comum nao e' candidata a VERSAO de nada"
    );
    let (_, out) = run(ComponentEdit::Create, &mut sim, &r, e, &mut toasts);
    let copy = out.map(Entity::from_bits).expect("a copia");
    let copy_id: VecPathId = 2;
    map.insert(copy_id, copy.to_bits());
    assert!(
        state_of(&mut sim, &map, &[copy_id], None, false)
            .expect("a seccao existe sobre a copia")
            .can_make_variant,
        "o painel nao oferece fazer uma versao nova a partir da copia — o gesto so' existe no menu"
    );
}

/// ⭐⭐ **E o IRMÃO do *Instantiate* põe uma cópia que DIVIDE a arte** (2026-09-07).
///
/// ⚠️ **A diferença entre os dois só se vê no gesto SEGUINTE** — pintar, mover um nó — e por isso
/// ela é medida pela marca que o motor deixa em cada peça (`LinkedArt`), e não pelo toast. *Um gate
/// escrito sobre a voz mediria a frase, não a lei.*
///
/// ⚠️ Ele vivia só no menu da Hierarquia: *um verbo cujo irmão está noutro sítio do app não se usa*,
/// porque o artista não sabe que a escolha existe.
///
/// **Mutação que deve sangrar:** `general_verb(PlaceLinked)` a devolver `Verb::Place`.
#[test]
fn the_linked_twin_shares_the_art_and_the_plain_one_does_not() {
    let (mut sim, r, _map, _id, e) = scene();
    // A receita precisa de uma peça: a marca viaja peça a peça, não na raiz.
    sim.world_mut().spawn((
        Transform::IDENTITY,
        Name::new("Box"),
        ph2d_render::Sprite::atlas(0, [1.0, 1.0], [1.0; 4]),
        ph2d_ecs::ChildOf(e),
    ));
    ph2d_ecs::assign_missing_stable_ids(sim.world_mut());
    let mut toasts = ph2d_editor::ToastQueue::default();
    run(ComponentEdit::Create, &mut sim, &r, e, &mut toasts);

    let (plain_changed, plain_out) = run(ComponentEdit::Place, &mut sim, &r, e, &mut toasts);
    let plain = plain_out.map(Entity::from_bits).expect("a copia comum");
    let (linked_changed, linked_out) =
        run(ComponentEdit::PlaceLinked, &mut sim, &r, e, &mut toasts);
    let linked = linked_out.map(Entity::from_bits).expect("a copia ligada");

    assert!(plain_changed && linked_changed, "um dos dois nao pos copia");
    let marked = |sim: &SimWorld, root: Entity| {
        sim.world()
            .get::<Children>(root)
            .into_iter()
            .flat_map(|k| k.iter().copied().collect::<Vec<_>>())
            .any(|p| sim.world().get::<ph2d_ecs::LinkedArt>(p).is_some())
    };
    assert!(
        marked(&sim, linked),
        "a copia LIGADA nasceu sem a marca — ela nao divide a arte, e o botao faz o mesmo que o irmao"
    );
    assert!(
        !marked(&sim, plain),
        "a copia comum nasceu LIGADA — os dois botoes deixaram de ser uma escolha"
    );
}

/// ⭐⭐⭐ **A secção oferece ABRIR A RECEITA de uma cópia** (2026-09-07).
///
/// A receita é escondida do canvas e da Hierarquia enquanto ninguém a edita, e o único caminho até
/// ela era o cartão do navegador de assets — que exige saber o nome dela e ter aquele painel
/// aberto. ⚠️ **E três recusas deste app já mandavam o artista *«editar no prefab»*** sem lhe dar um
/// gesto para lá chegar: *um app que nomeia um sítio inalcançável ensina que a feature está
/// partida.*
///
/// **Mutação que deve sangrar:** `can_edit_prefab: link.is_some()` a virar `false`.
#[test]
fn the_section_offers_opening_the_prefab_of_a_copy() {
    let (mut sim, r, mut map, id, e) = scene();
    let mut toasts = ph2d_editor::ToastQueue::default();
    assert!(
        !state_of(&mut sim, &map, &[id], None, false)
            .expect("a seccao existe")
            .can_edit_prefab,
        "uma forma comum nao tem receita para abrir"
    );
    let (_, out) = run(ComponentEdit::Create, &mut sim, &r, e, &mut toasts);
    let copy = out.map(Entity::from_bits).expect("a copia");
    let copy_id: VecPathId = 2;
    map.insert(copy_id, copy.to_bits());
    assert!(
        state_of(&mut sim, &map, &[copy_id], None, false)
            .expect("a seccao existe sobre a copia")
            .can_edit_prefab,
        "o painel nao oferece abrir a receita — ela so' e' alcancavel pelo cartao da biblioteca"
    );
}

/// ⭐⭐⭐ **E o clique SELECCIONA a receita — que é o que a faz aparecer.**
///
/// ⚠️ **O `MasterEditing` é DERIVADO da selecção**, então não há modo nem janela: pôr a selecção na
/// raiz do mestre acende o canvas, arma o gizmo e enche o Inspector. *O verbo que faltava não era um
/// modo: era um acesso.*
///
/// ⚠️ **E ele devolve `false`** — seleccionar não é editar, e devolver `true` poria um passo de undo
/// sobre um gesto de *ver*.
#[test]
fn opening_the_prefab_selects_the_recipe_and_is_not_an_edit() {
    let (mut sim, r, _map, _id, e) = scene();
    let mut toasts = ph2d_editor::ToastQueue::default();
    let (_, out) = run(ComponentEdit::Create, &mut sim, &r, e, &mut toasts);
    let copy = out.map(Entity::from_bits).expect("a copia");

    let (changed, out) = run(ComponentEdit::Edit, &mut sim, &r, copy, &mut toasts);

    assert!(
        !changed,
        "abrir a receita registou-se como edicao do documento"
    );
    assert_eq!(
        out.map(Entity::from_bits),
        Some(e),
        "a seleccao nao foi para a receita — o gesto nao leva o artista a lado nenhum"
    );
}

/// ⛔ **E sobre uma forma comum ele RECUSA com voz** — a tabela do painel não o oferece ali, mas o
/// dreno é partilhado com o menu da Hierarquia, cuja tabela é plana.
#[test]
fn opening_the_prefab_of_a_plain_shape_refuses_out_loud() {
    let (mut sim, r, _map, _id, e) = scene();
    let mut toasts = ph2d_editor::ToastQueue::default();
    let (changed, out) = run(ComponentEdit::Edit, &mut sim, &r, e, &mut toasts);
    assert!(!changed);
    assert!(out.is_none(), "seleccionou alguma coisa sem haver receita");
    assert!(
        toasts.iter().any(|t| t.message.contains("not a copy")),
        "a recusa nao diz o que escolher"
    );
}

/// ⭐⭐ **E o botão não é decoração: promover a cópia dá uma VARIANTE de verdade.**
///
/// Uma variante é `MasterRoot` **e** `InstanceOf` ao mesmo tempo — ela segue a base *e* é a receita
/// das cópias dela. *Sem a segunda metade isto seria apenas um segundo prefab solto.*
#[test]
fn promoting_a_copy_gives_a_variant_that_still_follows_its_base() {
    let (mut sim, r, _map, _id, e) = scene();
    let mut toasts = ph2d_editor::ToastQueue::default();
    let (_, out) = run(ComponentEdit::Create, &mut sim, &r, e, &mut toasts);
    let copy = out.map(Entity::from_bits).expect("a copia");
    let base = master_id(&mut sim, copy).expect("o elo da copia");

    let (changed, out) = run(ComponentEdit::Create, &mut sim, &r, copy, &mut toasts);

    assert!(changed, "promover a copia nao fez nada");
    assert!(
        sim.world().get::<MasterRoot>(copy).is_some(),
        "a copia nao virou receita"
    );
    assert_eq!(
        sim.world()
            .get::<ph2d_ecs::InstanceOf>(copy)
            .map(|l| l.master),
        Some(base),
        "a versao nova deixou de seguir a base — isto e' um prefab solto, nao uma variante"
    );
    let mine = out.map(Entity::from_bits).expect("a copia da variante");
    assert_ne!(mine, copy, "nao ficou copia nenhuma no lugar");
}

/// ⛔⛔⛔ **E ela não aterra EM CIMA da anterior** (report do Enio, 2026-09-06: *«instantiate não
/// desloca a cópia, deixa exatamente sobre a outra»*).
///
/// # O mecanismo, que é o mesmo do botão que sumia
///
/// A cascata pergunta *«quantas cópias esta receita já tem?»* para saber de quanto afastar a nova.
/// Ela lia o `StableId` da **linha clicada**, e desde que o *Make* move a selecção para a cópia, a
/// linha clicada é uma **cópia** — cujas instâncias são **zero**. ⇒ `n = 0` e o passo saía nulo,
/// **em silêncio**, sobre um verbo que fez tudo o resto certo.
///
/// ⚠️ **A linha estava certa quando foi escrita**: ela é anterior ao `master_subject`, quando o
/// verbo só aceitava a receita. *Quem alarga a lente de um verbo tem de reconferir tudo o que lia o
/// sujeito antigo.*
///
/// **Mutação que deve sangrar:** `cascade(sim, inst, <id de `entity`>, …)`.
#[test]
fn a_copy_instantiated_from_a_copy_does_not_land_on_top_of_it() {
    let (mut sim, r, _map, _id, e) = scene();
    let mut toasts = ph2d_editor::ToastQueue::default();
    let (_, out) = run(ComponentEdit::Create, &mut sim, &r, e, &mut toasts);
    let first = out.map(Entity::from_bits).expect("a 1a copia");
    let (_, out) = run(ComponentEdit::Place, &mut sim, &r, first, &mut toasts);
    let second = out.map(Entity::from_bits).expect("a 2a copia");

    let at = |sim: &SimWorld, x: Entity| {
        sim.world()
            .get::<Transform>(x)
            .map(|t| t.translation.x)
            .expect("pose")
    };
    assert!(
        (at(&sim, second) - at(&sim, first)).abs() > 1e-6,
        "a copia nova nasceu exactamente sobre a anterior — o artista ve' UMA forma onde ha' duas"
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

/// O `StableId` do mestre que esta cópia segue.
pub(super) fn master_id(sim: &mut SimWorld, e: Entity) -> Option<u64> {
    let root = ph2d_app_components::instance_verbs::instance_root_of(sim, e)?;
    sim.world()
        .get::<ph2d_ecs::InstanceOf>(root)
        .map(|l| l.master)
}
