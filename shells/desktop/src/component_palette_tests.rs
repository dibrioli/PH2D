//! Os gates da paleta de componentes (ADR-0166 / plano F3).
//!
//! ⚠️ Eles medem o **MODELO**, não a pintura — o widget é o `command_palette`, que já tem os
//! gates dele. O que é desta linha é *o que a paleta oferece a quem*, e é isso que se afirma aqui.

use super::*;

/// Tudo se constrói, para os gates isolarem o filtro do `can_build`.
fn buildable(_: &str) -> bool {
    true
}

fn labels(m: &PaletteModel) -> Vec<String> {
    m.groups
        .iter()
        .flat_map(|g| &g.subs)
        .flat_map(|s| &s.items)
        .map(|i| i.label.clone())
        .collect()
}

/// ⭐ **O EXEMPLO DO ENIO, medido:** *"9-slice provavelmente não se aplica a nada além de uma
/// sprite de imagem"*. Ele é oferecido a uma imagem e **não** a um objeto vetorial.
///
/// (Mutação: ignorar o `applies_to` ⇒ o vetor passa a oferecê-lo — RED.)
#[test]
fn nine_slice_is_offered_to_an_image_and_not_to_a_vector() {
    let img = build(ObjectKind::Image, &[], &buildable, false);
    assert!(
        labels(&img).iter().any(|l| l == "9-Slice"),
        "uma imagem tem de poder receber 9-Slice; ofereceu: {:?}",
        labels(&img)
    );
    let vec = build(ObjectKind::Vector, &[], &buildable, false);
    assert!(
        !labels(&vec).iter().any(|l| l.starts_with("9-Slice")),
        "um objeto vetorial NAO pode receber 9-Slice; ofereceu: {:?}",
        labels(&vec)
    );
}

/// ⚠️ **O inaplicável NÃO some — ele fica sob *Show all*, esmaecido e COM A RAZÃO.**
///
/// ⛔ Nem apagar da lista (um componente que existe e é invisível lê-se como defeito), nem no-op
/// silencioso ao clique (DIRETIVA §2). O rótulo carrega o porquê.
#[test]
fn show_all_reveals_the_inapplicable_with_the_reason_named() {
    let hidden = build(ObjectKind::Vector, &[], &buildable, false);
    let shown = build(ObjectKind::Vector, &[], &buildable, true);
    assert!(
        labels(&shown).len() > labels(&hidden).len(),
        "o Show all tem de REVELAR alguma coisa"
    );
    let nine = labels(&shown)
        .into_iter()
        .find(|l| l.starts_with("9-Slice"))
        .expect("o 9-Slice tem de aparecer sob Show all");
    assert!(
        nine.contains("not for this object type"),
        "o item inaplicavel tem de dizer PORQUE: {nine:?}"
    );
    // …e num sub-grupo próprio, depois dos aplicáveis.
    let sub = shown
        .groups
        .iter()
        .flat_map(|g| &g.subs)
        .find(|s| s.items.iter().any(|i| i.label.starts_with("9-Slice")))
        .expect("o sub-grupo");
    assert_eq!(sub.title.as_deref(), Some("Not for this object type"));
}

/// **O que o objeto JÁ TEM não é oferecido** — anexar o que já existe é um clique que não faz nada.
#[test]
fn a_component_already_on_the_object_is_not_offered() {
    let before = build(ObjectKind::Image, &[], &buildable, false);
    let after = build(
        ObjectKind::Image,
        &["ph2d::ecs::SliceNine"],
        &buildable,
        false,
    );
    assert!(labels(&before).iter().any(|l| l == "9-Slice"));
    assert!(
        !labels(&after).iter().any(|l| l == "9-Slice"),
        "ja' esta' no objeto e continuou a ser oferecido"
    );
}

/// ⚠️ **O que a paleta não consegue CONSTRUIR não pode estar nela.** Sem `insert_default` não há
/// valor inicial, e um item que aceita o clique e não anexa nada é o defeito que o `+` existe para
/// não ter.
#[test]
fn a_component_the_registry_cannot_build_is_never_offered() {
    let all = build(ObjectKind::Image, &[], &buildable, true);
    let none = build(ObjectKind::Image, &[], &|_| false, true);
    assert!(
        !labels(&all).is_empty(),
        "o controle positivo tem de oferecer"
    );
    assert!(
        labels(&none).is_empty(),
        "sem construtor, a paleta tem de ficar VAZIA; ofereceu: {:?}",
        labels(&none)
    );
}

/// ⭐ **Só `Authored` é oferecido.** A `Sprite` é `Intrinsic` (chega pelo gesto que cria a imagem)
/// e as pontes são `Machinery` — nenhuma é uma escolha do artista.
#[test]
fn only_authored_components_reach_the_palette() {
    let all = build(ObjectKind::Image, &[], &buildable, true);
    let ls = labels(&all);
    assert!(
        !ls.iter().any(|l| l.starts_with("Sprite Pixels")),
        "uma Machinery chegou a' paleta: {ls:?}"
    );
    assert!(
        !ls.iter().any(|l| l == "Sprite"),
        "a Sprite e' Intrinsic — ela chega pelo gesto, nao pelo +"
    );
}

/// **O pick volta a ser um componente** — o inverso do `item_id`, e a única rota de volta.
///
/// ⚠️ Ele varre o CATÁLOGO: uma segunda lista à mão envelheceria no primeiro componente novo, e o
/// sintoma seria *"o item aparece e não faz nada"*.
#[test]
fn every_offered_item_maps_back_to_its_component() {
    let m = build(ObjectKind::Image, &[], &buildable, true);
    let items: Vec<_> = m
        .groups
        .iter()
        .flat_map(|g| &g.subs)
        .flat_map(|s| &s.items)
        .collect();
    assert!(!items.is_empty());
    for it in items {
        let name = name_of_pick(it.id)
            .unwrap_or_else(|| panic!("o item {:?} nao volta a ser um componente", it.label));
        assert!(
            it.label.starts_with(
                ph2d_component_desc::desc_for(name)
                    .expect("o descritor")
                    .display_name
            ),
            "o id do item {:?} nomeia {name}, que tem outro rotulo",
            it.label
        );
    }
}

/// **Um objeto VAZIO ainda tem o que receber** — senão o `+` num objeto novo abriria uma paleta
/// vazia, que é a primeira coisa que o smoke do Enio faz.
#[test]
fn an_empty_object_still_has_something_to_offer() {
    let m = build(ObjectKind::Empty, &[], &buildable, false);
    assert!(
        !labels(&m).is_empty(),
        "o + num objeto vazio abriu uma paleta VAZIA"
    );
}

/// ⚠️ **Toda categoria com item tem um título e uma cor** — um grupo sem título é uma faixa muda
/// no modal.
#[test]
fn every_group_is_named_and_tinted() {
    let m = build(ObjectKind::Image, &[], &buildable, true);
    for g in &m.groups {
        assert!(!g.title.is_empty(), "grupo sem titulo");
        assert!(
            !g.subs.iter().all(|s| s.items.is_empty()),
            "grupo {:?} sem itens — ele nao devia existir",
            g.title
        );
    }
}
