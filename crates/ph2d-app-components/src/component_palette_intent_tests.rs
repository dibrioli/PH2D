//! ⭐ **A busca por INTENÇÃO do *Add Component*** (doc ObjectsAndComponents 01 §7.3 e §13, F6).
//!
//! O artista escreve o que quer que o objecto FAÇA e a paleta acha o componente — pelas palavras da
//! tabela `component.*.keywords`, em inglês (escolha do dono, 08/10).

use super::*;
use ph2d_editor_core::widget::command_palette::top_match;

fn buildable(_: &str) -> bool {
    true
}

/// ⛔ **Nenhum componente oferecível sem palavras** — um que faltasse só se acharia pelo nome, e a
/// busca por intenção passaria a responder «não existe» por um componente que existe.
#[test]
fn every_offered_component_has_intent_words() {
    let offered: Vec<&ComponentDesc> = ph2d_component_desc::all()
        .filter(|d| matches!(d.attach, Attach::Authored { .. }))
        .collect();
    assert!(
        offered.len() >= 50,
        "o catálogo encolheu? {}",
        offered.len()
    );
    let missing: Vec<&str> = offered
        .iter()
        .filter(|d| intent_words(d).is_empty())
        .map(|d| d.canonical_name)
        .collect();
    assert!(missing.is_empty(), "sem palavras de intenção: {missing:?}");
}

/// ⭐ **Os exemplos do plano, pelo `Enter` da paleta de uma imagem:** «fall» e «jump» ⇒ *Physics Body*,
/// «play sound» ⇒ *Audio Source 2D*, «die» ⇒ *Health*, «smoke» ⇒ *Particles*.
/// (Mutação: `make_item` sem `intent_words` ⇒ nenhum casa — RED.)
#[test]
fn the_artist_finds_a_component_by_what_it_should_do() {
    let model = build(ObjectKind::Image, &[], &buildable, false);
    for (query, canonical) in [
        ("fall", "ph2d::physics::RigidBody"),
        ("jump", "ph2d::physics::RigidBody"),
        ("play sound", "ph2d::ecs::AudioSource2D"),
        ("die", "ph2d::physics::Health"),
        ("smoke", "ph2d::ecs::ParticleEmitter"),
    ] {
        assert_eq!(
            top_match(&model, query),
            Some(item_id(canonical)),
            "«{query}» tinha de achar {canonical}"
        );
    }
}

/// ⚠️ **As palavras nunca se pintam:** o rótulo do item continua a ser só o nome (e a cascata).
#[test]
fn intent_words_never_reach_the_painted_label() {
    let model = build(ObjectKind::Image, &[], &buildable, false);
    let body = model
        .groups
        .iter()
        .flat_map(|g| &g.subs)
        .flat_map(|s| &s.items)
        .find(|i| i.id == item_id("ph2d::physics::RigidBody"))
        .expect("Physics Body é oferecido a uma imagem");
    assert!(!body.keywords.is_empty());
    assert!(!body.label.contains("gravity"), "{:?}", body.label);
}
