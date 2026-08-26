//! **A SEQUÊNCIA do `+`** (ADR-0166 / F3) — a quarta pergunta da costura, a que as outras três não
//! implicam.
//!
//! O gate do painel prova que o clique chega ao barramento; o do widget prova que a caixa é
//! registada; o do chrome prova que ela vira. **Nenhum dos três prova que o gesto INTEIRO leva a
//! algum lugar** — que abrir a paleta sobre este objeto oferece o que ele pode receber, que virar a
//! caixa revela o inaplicável **sem apagar o que foi escrito na busca**, e que escolher um item
//! deixa o componente na cena.

use ph2d_ecs::{SimWorld, Transform};
use ph2d_editor::{HeroScreen, NodeId};

fn registry() -> ph2d_ecs::scene::ComponentRegistry {
    crate::init::build_component_registry()
}

/// Uma sprite selecionável.
fn image(sim: &mut SimWorld) -> u64 {
    sim.world_mut()
        .spawn((
            Transform::IDENTITY,
            ph2d_ecs::Name::new("Image"),
            ph2d_render::Sprite::atlas(0, [1.0, 1.0], [1.0; 4]),
        ))
        .id()
        .to_bits()
}

fn labels(hero: &HeroScreen) -> Vec<String> {
    hero.store
        .command_palette_model()
        .map(|m| {
            m.groups
                .iter()
                .flat_map(|g| &g.subs)
                .flat_map(|s| &s.items)
                .map(|i| i.label.clone())
                .collect()
        })
        .unwrap_or_default()
}

/// ⭐ **O gesto inteiro: `+` → paleta → *Show all* → o inaplicável aparece, com a razão.**
///
/// ⚠️ **E a BUSCA sobrevive à caixa.** Ligar *Show all* depois de escrever «slice» não pode apagar
/// o que foi escrito — seria um controlo a desfazer o trabalho do controlo do lado. É por isso que
/// a reconstrução usa `set_command_palette_model` e não `open_command_palette`.
#[test]
fn the_whole_gesture_opens_filters_and_reveals() {
    let mut sim = SimWorld::new();
    let bits = image(&mut sim);
    let reg = registry();
    let mut hero = HeroScreen::new(NodeId(1));
    let mut target: Option<u64> = None;

    crate::component_attach::open_palette_if_asked(&mut hero, &sim, &reg, Some(bits), &mut target);
    assert_eq!(target, Some(bits), "a paleta tem de lembrar quem pediu");
    assert!(hero.store.command_palette_open());
    let closed = labels(&hero);
    assert!(
        closed.iter().any(|l| l.starts_with("9-Slice")),
        "uma imagem tem de receber 9-Slice"
    );
    assert!(
        !closed
            .iter()
            .any(|l| l.contains("not for this object type")),
        "com a caixa DESLIGADA o inaplicavel nao aparece"
    );

    // O artista escreve, e depois liga a caixa.
    hero.store.command_palette_push_char('s');
    hero.store.command_palette_push_char('l');
    hero.store.flip_command_palette_toggle();
    crate::component_attach::refresh_palette_on_toggle(&mut hero, &sim, &reg, target);

    assert_eq!(
        hero.store.command_palette_query(),
        "sl",
        "ligar a caixa apagou o que o artista escreveu"
    );
    let open = labels(&hero);
    assert!(
        open.iter().any(|l| l.contains("not for this object type")),
        "o Show all tem de REVELAR o inaplicavel, com a razao: {open:?}"
    );
    assert!(
        open.len() > closed.len(),
        "o Show all nao revelou nada ({} -> {})",
        closed.len(),
        open.len()
    );
}

/// ⚠️ **A caixa é uma PORTA DE DOIS SENTIDOS** — virá-la outra vez volta a esconder.
///
/// Um controlo que só funciona uma vez é a mesma classe do botão morto: o artista liga *Show all*
/// para procurar, e depois quer a lista curta de volta.
#[test]
fn the_box_goes_both_ways() {
    let mut sim = SimWorld::new();
    let bits = image(&mut sim);
    let reg = registry();
    let mut hero = HeroScreen::new(NodeId(1));
    let mut target: Option<u64> = None;
    crate::component_attach::open_palette_if_asked(&mut hero, &sim, &reg, Some(bits), &mut target);
    let closed = labels(&hero).len();

    hero.store.flip_command_palette_toggle();
    crate::component_attach::refresh_palette_on_toggle(&mut hero, &sim, &reg, target);
    let opened = labels(&hero).len();
    assert!(opened > closed);

    hero.store.flip_command_palette_toggle();
    crate::component_attach::refresh_palette_on_toggle(&mut hero, &sim, &reg, target);
    assert_eq!(labels(&hero).len(), closed, "a caixa nao voltou atras");
}

/// ⭐ **E o pick deixa o componente na CENA** — a ponta final da sequência.
///
/// ⚠️ **Pelo `route_pick`, que é o dreno CONDICIONAL:** o canal de pick tem três consumidores (a
/// biblioteca de nós do Motion, o `Ctrl+K` e este), e um `take` incondicional faria quem recebe o
/// pick ser *a ordem dos drenos no quadro*.
#[test]
fn picking_an_item_leaves_the_component_in_the_scene() {
    let mut sim = SimWorld::new();
    let bits = image(&mut sim);
    let reg = registry();
    let mut hero = HeroScreen::new(NodeId(1));
    let mut target: Option<u64> = None;
    let mut toasts = ph2d_editor::ToastQueue::default();
    crate::component_attach::open_palette_if_asked(&mut hero, &sim, &reg, Some(bits), &mut target);

    hero.store
        .set_command_pick(crate::component_palette::item_id("ph2d::ecs::SliceNine"));
    let picked = crate::component_attach::route_pick(&mut hero, &mut target);
    assert!(picked.is_some(), "o dreno nao reconheceu o proprio id");
    crate::component_attach::attach_picked(picked.as_ref(), &mut sim, &reg, &mut toasts);

    let e = ph2d_ecs::Entity::from_bits(bits);
    assert!(
        sim.world().get::<ph2d_ecs::SliceNine>(e).is_some(),
        "o componente escolhido nao chegou a` cena"
    );
    // ⭐ E a seção da §5 passa a existir — que é a razão de tudo isto.
    assert!(
        crate::render_loop::inspector_presence_probe::slice(sim.world(), bits),
        "o artista anexou o 9-Slice e a seccao nao apareceu"
    );
    assert_eq!(target, None, "o alvo tem de ser limpo depois do pick");
}

/// ⚠️ **Um pick que não é MEU fica onde está.** A metade de ausência do dreno condicional: sem ela,
/// este consumidor engoliria o pick da biblioteca de nós e o sintoma seria *«às vezes não faz
/// nada»* — um defeito que depende da ordem dos drenos, que ninguém lê.
#[test]
fn a_pick_that_is_not_mine_is_left_alone() {
    let mut sim = SimWorld::new();
    let bits = image(&mut sim);
    let reg = registry();
    let mut hero = HeroScreen::new(NodeId(1));
    let mut target: Option<u64> = None;
    crate::component_attach::open_palette_if_asked(&mut hero, &sim, &reg, Some(bits), &mut target);

    let foreign = ph2d_tool_registry::hash_node_id("motion.boids");
    hero.store.set_command_pick(foreign);
    assert!(
        crate::component_attach::route_pick(&mut hero, &mut target).is_none(),
        "o dreno reclamou um id que nao e' dele"
    );
    assert_eq!(
        hero.store.take_command_pick(),
        Some(foreign),
        "o pick alheio tem de ficar no canal para quem o sabe executar"
    );
}
