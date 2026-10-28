//! W11 do Kanban pelo hero REAL: o cartão que abre um objecto selecciona-o (o mesmo pedido de um
//! clique, que a Hierarquia segue) e traz a aba da cena; o trecho vai ao quadro de início; o
//! catálogo publicado tem os objectos e os scripts do projecto.

use super::*;
use ph2d_editor_core::NodeId;
use ph2d_editor_core::kanban_links::{catalog as project_catalog, open};
use ph2d_script::ScriptLibrary;

fn scene() -> (SimWorld, HeroScreen, u64) {
    let mut sim = SimWorld::new();
    let e = sim
        .world_mut()
        .spawn((StableId(41), Name("Hero".into())))
        .id();
    let mut lib = ScriptLibrary::new();
    lib.insert("jump", "-- jump");
    ProjectScripts::install(sim.world_mut(), lib);
    (sim, HeroScreen::new(NodeId(1)), e.to_bits())
}

fn selected(hero: &mut HeroScreen) -> Vec<u64> {
    hero.bus
        .drain()
        .filter_map(|a| match a {
            EditorAction::SelectSprite {
                entity_bits,
                modifier: SelectModifier::Replace,
            } => Some(entity_bits),
            _ => None,
        })
        .collect()
}

/// ⭐ Abrir o objecto selecciona-o pela porta do clique e activa a cena; o catálogo tem-no.
#[test]
fn opening_an_object_selects_it_and_the_catalog_names_it() {
    let (mut sim, mut hero, bits) = scene();
    let tl = TimelineDoc::default();
    let mut intents = Vec::new();
    hero.apply_event(ph2d_editor_core::interaction::WidgetEvent::Click(
        ph2d_editor_core::ids::DOC_TAB_NEW,
    ));
    assert!(hero.documents.active().is_some(), "um Quadro à frente");
    open(&Target::Object(41));
    frame(&mut sim, &mut hero, &tl, &mut intents);
    assert_eq!(selected(&mut hero), vec![bits]);
    assert_eq!(hero.documents.active(), None, "a aba da cena");
    let cat = project_catalog(&hero.documents);
    assert_eq!(cat.objects.get(&41).map(String::as_str), Some("Hero"));
    assert_eq!(
        cat.scripts.values().map(String::as_str).collect::<Vec<_>>(),
        ["jump"]
    );
    assert!(intents.is_empty());
}

/// O trecho vai ao quadro de início e selecciona o dono; um objecto apagado não selecciona nada.
#[test]
fn a_span_seeks_its_first_frame_and_a_gone_object_selects_nothing() {
    let (mut sim, mut hero, bits) = scene();
    let tl = TimelineDoc::default();
    let mut intents = Vec::new();
    open(&Target::Span {
        object: Some(41),
        strip: None,
        from: 12,
        to: 48,
    });
    open(&Target::Object(99));
    frame(&mut sim, &mut hero, &tl, &mut intents);
    assert!(matches!(intents[..], [TimelineIntent::SeekFrame(12)]));
    assert_eq!(selected(&mut hero), vec![bits], "só o dono do trecho");
    assert!(
        project_catalog(&hero.documents).objects.get(&99).is_none(),
        "o verso mostra-a quebrada"
    );
}
