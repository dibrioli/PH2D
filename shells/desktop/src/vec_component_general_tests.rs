//! Os gates do modo GERAL da secção *Prefab* (F4.6c, waves 1 e 2).
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
    run_full(verb, sim, r, subject, toasts).0
}

/// O mesmo dreno, devolvendo também **se o conta-gotas armou** — é a saída que o gesto de duas
/// mãos usa, e a que separa *«o verbo não fez nada»* de *«o verbo abriu um gesto»*.
fn run_full(
    verb: ComponentEdit,
    sim: &mut SimWorld,
    r: &ph2d_ecs::scene::ComponentRegistry,
    subject: Entity,
    toasts: &mut ph2d_editor::ToastQueue,
) -> ((bool, Option<u64>), bool) {
    let (mut sc, mut mp) = crate::instance_docs::empty_docs();
    let mut echo = crate::instance_sync::MasterEcho::default();
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
    let s = state_of(&mut sim, &map, &[id], false).expect("uma forma tem seccao");
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
    let s = state_of(&mut sim, &map, &[id], false).expect("a seccao existe");
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
    let s = state_of(&mut sim, &map, &[copy_id], false).expect("a seccao existe sobre a copia");

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
        !state_of(&mut sim, &map, &[id], false)
            .expect("a seccao existe")
            .can_make_variant,
        "uma forma comum nao e' candidata a VERSAO de nada"
    );
    let (_, out) = run(ComponentEdit::Create, &mut sim, &r, e, &mut toasts);
    let copy = out.map(Entity::from_bits).expect("a copia");
    let copy_id: VecPathId = 2;
    map.insert(copy_id, copy.to_bits());
    assert!(
        state_of(&mut sim, &map, &[copy_id], false)
            .expect("a seccao existe sobre a copia")
            .can_make_variant,
        "o painel nao oferece fazer uma versao nova a partir da copia — o gesto so' existe no menu"
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

/// ⭐⭐ **O botão do *Swap* ARMA o conta-gotas — ele não age e não fala.**
///
/// ⚠️ **`changed == false` aqui NÃO é um clique comido:** o gesto é de duas mãos, e a resposta ao
/// primeiro clique é o painel trocar de rótulo para *Click a copy of the prefab* (o `swap_armed`
/// shell publica). *Um toast a dizer «agora clique noutra coisa» seria a terceira maneira de dizer
/// o que o botão já diz.*
///
/// (Mutação: `*arm_pick = true` a virar `false` ⇒ RED.)
#[test]
fn the_swap_button_arms_the_eyedropper_instead_of_acting() {
    let (mut sim, r, _map, _id, e) = scene();
    let mut toasts = ph2d_editor::ToastQueue::default();
    let ((changed, _), armed_pick) = run_full(ComponentEdit::Swap, &mut sim, &r, e, &mut toasts);
    assert!(!changed, "o Swap mudou o mundo no primeiro clique");
    assert!(
        armed_pick,
        "o botao do Swap nao armou o conta-gotas — o gesto de duas maos nao chega a abrir"
    );
}

/// ⭐⭐ **E a secção DIZ que está à espera** — é o `swap_armed` que troca o rótulo do botão.
#[test]
fn the_section_says_the_eyedropper_is_waiting() {
    let (mut sim, _r, map, id, _e) = scene();
    let idle = state_of(&mut sim, &map, &[id], false).expect("a seccao existe");
    let waiting = state_of(&mut sim, &map, &[id], true).expect("a seccao existe");
    assert!(!idle.swap_armed && waiting.swap_armed);
}

/// ⭐⭐⭐ **O SEGUNDO clique troca o prefab da cópia — e o alvo é uma CÓPIA do prefab que se quer.**
///
/// ⚠️ **É a lei que devolveu o `Instantiate`, aplicada ao gesto:** a receita está escondida no
/// canvas, então clicar nela é impossível; clicar numa cópia dela quer dizer *«esta também passa a
/// ser um destes»*. A fixtura monta a família como o artista a monta — um prefab, e uma **variante**
/// dele — porque sem parentesco o mapa determinístico não existe e a troca recusa (por desenho).
///
/// (Mutação: `master_subject` a virar `clicked` ⇒ RED, porque a cópia clicada não é um `MasterRoot`.)
#[test]
fn the_second_click_makes_the_copy_a_copy_of_the_clicked_prefab() {
    let (mut sim, r, _map, _id, e) = scene();
    let mut toasts = ph2d_editor::ToastQueue::default();
    // A é a receita, `copy_a` é a cópia que fica no lugar.
    let (_, out) = run(ComponentEdit::Create, &mut sim, &r, e, &mut toasts);
    let copy_a = out.map(Entity::from_bits).expect("a copia de A");
    // Promover uma cópia faz uma VARIANTE — é assim que a família nasce.
    let (_, out) = run(ComponentEdit::Create, &mut sim, &r, copy_a, &mut toasts);
    let copy_b = out.map(Entity::from_bits).expect("a copia da variante B");
    let b_id = master_id(&mut sim, copy_b).expect("o elo da copia de B");
    // E uma segunda cópia de A, que é quem vai trocar.
    let (_, out) = run(ComponentEdit::Place, &mut sim, &r, e, &mut toasts);
    let mine = out.map(Entity::from_bits).expect("a segunda copia de A");
    assert_ne!(master_id(&mut sim, mine), Some(b_id), "ja' nasceu como B");

    let mut echo = crate::instance_sync::MasterEcho::default();
    let did = super::swap_by_pick(&mut sim, &mut echo, &mut toasts, mine, copy_b);

    assert!(did, "o segundo clique nao trocou nada");
    assert_eq!(
        master_id(&mut sim, mine),
        Some(b_id),
        "a copia continua a seguir a receita antiga — o alvo nao foi resolvido pela copia clicada"
    );
}

/// ⛔⛔ **E clicar numa forma comum RECUSA em voz alta, sem desarmar.**
///
/// *Um gesto de duas mãos que se desarma no primeiro clique fora do alvo faz o artista pensar que a
/// troca aconteceu* — a mesma lei que o motor velho já escrevia no comentário dele.
#[test]
fn clicking_a_plain_shape_with_the_eyedropper_refuses_out_loud() {
    let (mut sim, r, _map, _id, e) = scene();
    let mut toasts = ph2d_editor::ToastQueue::default();
    let (_, out) = run(ComponentEdit::Create, &mut sim, &r, e, &mut toasts);
    let copy = out.map(Entity::from_bits).expect("a copia");
    let plain = sim
        .world_mut()
        .spawn((Transform::IDENTITY, Name::new("Loose")))
        .id();
    ph2d_ecs::assign_missing_stable_ids(sim.world_mut());
    let before = master_id(&mut sim, copy);

    let mut echo = crate::instance_sync::MasterEcho::default();
    let did = super::swap_by_pick(&mut sim, &mut echo, &mut toasts, copy, plain);

    assert!(!did, "trocou por uma forma que nao e' prefab nenhum");
    assert_eq!(master_id(&mut sim, copy), before, "a copia mexeu-se");
    assert!(
        toasts.iter().any(|t| t.message.contains("not a copy")),
        "a recusa nao diz o que clicar"
    );
}

/// O `StableId` do mestre que esta cópia segue.
fn master_id(sim: &mut SimWorld, e: Entity) -> Option<u64> {
    let root = crate::instance_verbs::instance_root_of(sim, e)?;
    sim.world()
        .get::<ph2d_ecs::InstanceOf>(root)
        .map(|l| l.master)
}
