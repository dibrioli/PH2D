//! **Os gates de [`super::anchor_overlay`]** — irmão por CAP de LOC (HR-18, 600 no shell).
//!
//! ⚠️ **Corte mecânico, conteúdo verbatim.** O módulo de testes saiu inteiro quando os
//! quatro pedidos do Enio de 2026-08-23 levaram o ficheiro acima do teto. A regra da casa é
//! cortar para o IRMÃO, nunca declarar exceção — e o idioma é o do `anchor_gizmo_tests.rs`.

use super::*;

use ph2d_ecs::{AnchorMount, AnchorVisibility, ChildOf, NamedAnchorList};

fn owner(w: &mut World, names: &[&str]) -> Entity {
    let mut l = NamedAnchorList::new();
    for n in names {
        l.insert(NamedAnchor::socket(*n)).unwrap();
    }
    w.spawn((Transform::IDENTITY, l)).id()
}

/// **O default não mudou: sem seleção e sem caixa, o canvas fica limpo.**
///
/// ⚠️ É o controlo positivo das três passagens. Sem ele, um plano que devolvesse tudo sempre
/// passaria em todos os testes abaixo — eles só verificam que o que devia lá estar está.
#[test]
fn nothing_is_drawn_by_default() {
    let mut w = World::new();
    owner(&mut w, &["muzzle"]);
    assert!(marks_plan(&w, true, None, None).is_empty());
    assert!(
        marks_plan(&w, false, None, None).is_empty(),
        "a seccao fechada nao pode acender nada"
    );
}

/// **(pedido 2) A âncora que o filho MONTA aparece — e mesmo com a §12 FECHADA.**
///
/// ⚠️ A seção fechada é metade do pedido: mover um filho ancorado é um gesto de CANVAS, e
/// exigir o painel aberto para ver a referência tornaria a marca inútil no momento em que
/// ela é precisa.
#[test]
fn the_ridden_anchor_shows_up_even_with_the_section_closed() {
    let mut w = World::new();
    let host = owner(&mut w, &["hand_r", "head"]);
    let rider = w
        .spawn((
            Transform::IDENTITY,
            ChildOf(host),
            AnchorMount::new("hand_r"),
        ))
        .id();
    let plan = marks_plan(&w, false, Some(rider.to_bits()), None);
    assert_eq!(
        plan,
        vec![(host, PlanMode::RiddenAnchor("hand_r".into()))],
        "a ancora de que o filho parte tem de aparecer, e SO' ela"
    );
}

/// Um filho **sem** montagem não acende âncora nenhuma do pai.
#[test]
fn an_unmounted_child_shows_nothing_of_the_parent() {
    let mut w = World::new();
    let host = owner(&mut w, &["hand_r"]);
    let plain = w.spawn((Transform::IDENTITY, ChildOf(host))).id();
    assert!(marks_plan(&w, false, Some(plain.to_bits()), None).is_empty());
}

/// **(pedido 3) A caixa «Always show anchors» desenha sem seleção e com a seção fechada.**
#[test]
fn the_always_visible_box_draws_without_selection() {
    let mut w = World::new();
    let host = owner(&mut w, &["muzzle"]);
    assert!(marks_plan(&w, false, None, None).is_empty());
    w.entity_mut(host).insert(AnchorVisibility {
        in_editor: true,
        at_runtime: false,
    });
    assert_eq!(
        marks_plan(&w, false, None, None),
        vec![(host, PlanMode::AlwaysVisible)]
    );
}

/// ⚠️ **`at_runtime` sozinho NÃO acende nada no editor.** As duas caixas são intenções
/// diferentes, e confundi-las faria marcar «em runtime» encher o editor de cruzes.
#[test]
fn the_runtime_box_alone_changes_nothing_in_the_editor() {
    let mut w = World::new();
    let host = owner(&mut w, &["muzzle"]);
    w.entity_mut(host).insert(AnchorVisibility {
        in_editor: false,
        at_runtime: true,
    });
    assert!(marks_plan(&w, false, None, None).is_empty());
}

/// **A ORDEM é a de pintura, e a selecionada vai por ÚLTIMO.**
///
/// ⚠️ O que se pode agarrar tem de estar por cima: alças desenhadas debaixo de uma marca
/// esmaecida ainda agarram, e um alvo que se agarra sem se ver é pior que um que não existe.
#[test]
fn the_selected_entity_paints_last() {
    let mut w = World::new();
    let other = owner(&mut w, &["a"]);
    w.entity_mut(other).insert(AnchorVisibility {
        in_editor: true,
        at_runtime: false,
    });
    let host = owner(&mut w, &["hand_r"]);
    let rider = w
        .spawn((
            Transform::IDENTITY,
            ChildOf(host),
            AnchorMount::new("hand_r"),
            NamedAnchorList::new(),
        ))
        .id();
    let plan = marks_plan(&w, true, Some(rider.to_bits()), Some(0));
    assert_eq!(
        plan,
        vec![
            (other, PlanMode::AlwaysVisible),
            (host, PlanMode::RiddenAnchor("hand_r".into())),
            (rider, PlanMode::Editing(Some(0))),
        ]
    );
}

/// **⭐ «Always show anchors» NÃO pode tirar o destaque da selecionada** (Enio, 2026-08-23).
///
/// ⚠️ **Este gate existe porque a versão anterior dele afirmava o CONTRÁRIO.** Ela pinava
/// `AlwaysVisible` sobre a entidade selecionada, com a justificação de que desenhar duas vezes
/// soma o alfa e finge destaque — a observação estava certa e a **cura estava ao contrário**.
/// `Editing` é **superset** de `AlwaysVisible`: as mesmas âncoras, mais o realce da linha aberta,
/// mais as alças. Quem tem de sair é a outra.
///
/// *Um gate verde pode pinar um defeito de produto* — e quem o apanhou foi um smoke.
#[test]
fn the_always_visible_box_never_steals_the_highlight_from_the_selection() {
    let mut w = World::new();
    let host = owner(&mut w, &["muzzle", "head"]);
    w.entity_mut(host).insert(AnchorVisibility {
        in_editor: true,
        at_runtime: false,
    });

    // Selecionada ⇒ **Editing**, com a linha aberta. Uma entrada só: nada se repete.
    assert_eq!(
        marks_plan(&w, true, Some(host.to_bits()), Some(1)),
        vec![(host, PlanMode::Editing(Some(1)))],
        "a caixa roubou o destaque a` selecionada"
    );

    // ⭐ A outra metade do pedido: **desselecionar faz o destaque sumir**, e a caixa assume.
    assert_eq!(
        marks_plan(&w, true, None, None),
        vec![(host, PlanMode::AlwaysVisible)],
        "sem seleccao a caixa tem de manter as marcas — esmaecidas"
    );

    // ⚠️ Com a §12 FECHADA não há linha aberta nem alças: não há destaque que se possa roubar,
    // e a caixa manda mesmo na selecionada.
    assert_eq!(
        marks_plan(&w, false, Some(host.to_bits()), Some(1)),
        vec![(host, PlanMode::AlwaysVisible)]
    );
}

/// **A dedup nunca deixa cair a metade que falta.**
///
/// Um pai que a passagem (2) apanhou desenhou **uma** âncora; se o filho selecionado tiver
/// âncoras suas, a (3) tem de correr na mesma sobre ele.
#[test]
fn the_dedup_drops_the_repeat_but_never_the_missing_half() {
    // Um objeto que monta numa âncora do PRÓPRIO pai e está selecionado, tendo âncoras
    // suas: as duas passagens correm — a (2) sobre o pai, a (3) sobre ele.
    let mut w = World::new();
    let parent = owner(&mut w, &["slot"]);
    let child = w
        .spawn((
            Transform::IDENTITY,
            ChildOf(parent),
            AnchorMount::new("slot"),
            NamedAnchorList::new(),
        ))
        .id();
    let plan = marks_plan(&w, true, Some(child.to_bits()), None);
    assert_eq!(plan.len(), 2, "faltou uma das duas metades: {plan:?}");
    assert_eq!(plan[1].0, child);
}

/// O plano é **determinístico**: a ordem de iteração de um arquétipo não é a da cena.
#[test]
fn the_always_visible_sweep_is_ordered() {
    let mut w = World::new();
    let mut ids: Vec<Entity> = (0..4)
        .map(|i| {
            let e = owner(&mut w, &["a"]);
            let _ = i;
            w.entity_mut(e).insert(AnchorVisibility {
                in_editor: true,
                at_runtime: false,
            });
            e
        })
        .collect();
    ids.sort_unstable_by_key(|e| e.to_bits());
    let got: Vec<Entity> = marks_plan(&w, false, None, None)
        .into_iter()
        .map(|(e, _)| e)
        .collect();
    assert_eq!(got, ids);
}

/// ⚠️ **O DEFEITO QUE O SMOKE DO ENIO APANHOU (2026-08-22): a âncora tem de SEGUIR a sprite.**
///
/// A leitura antiga caía em `Vec2::ZERO` e deixava toda âncora cravada na origem do mundo.
#[test]
fn an_anchor_follows_the_sprite_it_belongs_to() {
    let mut a = NamedAnchor::socket("muzzle");
    a.transform.translation = Vec2::new(0.5, 0.25);

    let at_origin = Transform::default();
    let p0 = anchor_world_point(at_origin, &a, [0.0, 0.0], 100.0);
    assert!((p0.x - 0.5).abs() < 1e-6 && (p0.y - 0.25).abs() < 1e-6);

    // A sprite anda 10 m para a direita: a âncora tem de andar com ela.
    let moved = Transform {
        translation: Vec2::new(10.0, 0.0),
        ..Transform::default()
    };
    let p1 = anchor_world_point(moved, &a, [0.0, 0.0], 100.0);
    assert!(
        (p1.x - 10.5).abs() < 1e-6,
        "a ancora nao seguiu a sprite: {p1:?} (ficou cravada no mundo)"
    );
    assert_ne!(p0, p1, "mover a sprite nao mexeu a ancora");
}

/// E segue a ESCALA e a ROTAÇÃO, não só a translação — é o que faz a caixa de dano andar com
/// o objeto quando ele é redimensionado ou rodado.
#[test]
fn the_mark_follows_scale_and_rotation_too() {
    let mut a = NamedAnchor::socket("hand");
    a.transform.translation = Vec2::new(1.0, 0.0);

    let scaled = Transform {
        scale: Vec2::new(3.0, 1.0),
        ..Transform::default()
    };
    let p = anchor_world_point(scaled, &a, [0.0, 0.0], 100.0);
    assert!(
        (p.x - 3.0).abs() < 1e-5,
        "a escala nao alcancou a ancora: {p:?}"
    );

    let turned = Transform {
        rotation: std::f32::consts::FRAC_PI_2,
        ..Transform::default()
    };
    let q = anchor_world_point(turned, &a, [0.0, 0.0], 100.0);
    assert!(
        q.x.abs() < 1e-5 && (q.y - 1.0).abs() < 1e-5,
        "rodar 90 graus tinha de levar (1,0) para (0,1), deu {q:?}"
    );
}

/// O canto de uma área sai em pixels da FONTE, convertido pelo `pixels_per_meter`.
#[test]
fn a_bounds_corner_converts_source_pixels_to_metres() {
    let a = NamedAnchor::socket("box");
    let p = anchor_world_point(Transform::default(), &a, [50.0, -25.0], 100.0);
    assert!(
        (p.x - 0.5).abs() < 1e-6 && (p.y + 0.25).abs() < 1e-6,
        "deu {p:?}"
    );
}

/// A cor é **estável** e **distinta** — é o que permite distinguir duas âncoras
/// sobrepostas, e o que faz o mesmo socket ter a mesma cor amanhã.
#[test]
fn the_colour_is_stable_and_distinguishes_names() {
    assert_eq!(
        color_of("muzzle"),
        color_of("muzzle"),
        "a cor mudou sozinha"
    );
    assert_ne!(
        color_of("muzzle"),
        color_of("face_box"),
        "dois nomes com a mesma cor: duas ancoras sobrepostas ficam indistinguiveis"
    );
    // Opaca o suficiente para ler sobre arte clara e escura.
    for name in ["a", "b", "left_hand", "anchor_63"] {
        let c = color_of(name);
        assert!(c[3] > 0.9, "'{name}' saiu translucido demais para chrome");
        assert!(
            c[0] + c[1] + c[2] > 0.5,
            "'{name}' saiu quase preto — invisivel sobre arte escura"
        );
    }
}
