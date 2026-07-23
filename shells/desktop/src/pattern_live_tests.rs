//! Gates do **Pattern Along Path VIVO** — arquivo irmão de `pattern_live.rs`.
//!
//! O oráculo é a geometria DERIVADA que o `dispatch` desenharia (`PatternLive::live()`): um motivo
//! vinculado a um guia produz N cópias no z dele, e a curva do motivo nunca é tocada. As mutações
//! que estes gates existem para matar: o `recook` não ler o componente (0 cópias), o `detach` não
//! remover o vínculo (as cópias sobrevivem à soltura), o `link` prender a forma a si mesma.

use super::{PatternLive, detach, link, link_candidate, spec_of};
use crate::vec_entities::VecEntityMap;
use ph2d_ecs::{Name, SimWorld, Transform, VecPathRef};
use ph2d_vec_scene::{VecPath, VecPathId, VecScene, VecVertex};

/// Um motivo (quadrado de 40) + um guia (reta de 100), cada um uma entidade na identidade.
fn scene() -> (VecScene, SimWorld, VecEntityMap, VecPathId, VecPathId) {
    let mut sim = SimWorld::default();
    let mut map = VecEntityMap::new();
    let mut scene = VecScene::new();

    let motif = scene.push_path(VecPath {
        verts: [[0.0, 0.0], [40.0, 0.0], [40.0, 40.0], [0.0, 40.0]]
            .map(VecVertex::corner)
            .to_vec(),
        closed: true,
        ..VecPath::default()
    });
    let guide = scene.push_path(VecPath {
        verts: vec![VecVertex::corner([0.0, 0.0]), VecVertex::corner([100.0, 0.0])],
        closed: false,
        ..VecPath::default()
    });
    for (id, name) in [(motif, "Motif"), (guide, "Guide")] {
        let e = sim
            .world_mut()
            .spawn((Transform::IDENTITY, Name::new(name), VecPathRef(id)))
            .id();
        map.insert(id, e.to_bits());
    }
    (scene, sim, map, motif, guide)
}

/// Nº de cópias que o `dispatch` desenharia para `id` neste frame.
fn copies(live: &PatternLive, id: VecPathId) -> usize {
    live.live().get(&id).map_or(0, Vec::len)
}

/// **Um motivo vinculado a um guia re-cozinha em CÓPIAS.** Reta de 100, quadrado de 40, spacing
/// 1.0 ⇒ 2 fatias cabem ⇒ 2 cópias. É a metade visível da feature.
///
/// Mutação que mata: o `recook` ignorar o componente (`spec_of` → 0) deixa `copies == 0`.
#[test]
fn a_linked_motif_recooks_into_copies() {
    let (scene, mut sim, map, motif, guide) = scene();
    assert!(link(&mut sim, &map, motif, guide), "prendeu o motivo ao guia");
    assert!(spec_of(&sim, &map, motif).is_some(), "o vínculo existe");

    let mut live = PatternLive::default();
    live.recook(&scene, &sim, &map);
    assert_eq!(copies(&live, motif), 2, "duas cópias tilam a reta de 100");
    // Cada cópia é um quadrado (4 verts), no z do motivo.
    for c in live.live().get(&motif).unwrap() {
        assert_eq!(c.verts.len(), 4, "a cópia é o quadrado do motivo");
    }
    // O GUIA não é redesenhado como pattern (só o motivo carrega o componente).
    assert_eq!(copies(&live, guide), 0, "o guia não é um pattern");
}

/// **Soltar PARA as cópias** — o motivo volta a ser desenhado como fonte (ausente da `live`), o
/// caminho fica. Mutação que mata: o `detach` não remover o componente (as cópias sobrevivem).
#[test]
fn detaching_stops_the_copies() {
    let (scene, mut sim, map, motif, guide) = scene();
    link(&mut sim, &map, motif, guide);
    let mut live = PatternLive::default();
    live.recook(&scene, &sim, &map);
    assert_eq!(copies(&live, motif), 2, "vinculado: cópias");

    assert!(detach(&mut sim, &map, motif), "soltou");
    live.recook(&scene, &sim, &map);
    assert_eq!(copies(&live, motif), 0, "solto: sem cópias (o motivo vira fonte)");
    // O guia sobreviveu à soltura — soltar remove a RELAÇÃO, não o objeto.
    assert!(scene.path(guide).is_some(), "o caminho-guia fica");
}

/// **Prender uma forma a si mesma não quer dizer nada** — recusado. Sem isto, o motivo cavalgaria
/// a própria silhueta e o `guide_arc` leria a geometria que ele está a produzir.
#[test]
fn a_shape_cannot_ride_itself() {
    let (_scene, mut sim, map, motif, _guide) = scene();
    assert!(!link(&mut sim, &map, motif, motif), "recusa prender a si mesma");
    assert!(spec_of(&sim, &map, motif).is_none(), "nenhum vínculo criado");
}

/// **O primário é o MOTIVO, o outro é o guia** — a lei de disambiguação do painel (W3). Sem ela, o
/// gesto não saberia qual dos dois selecionados cavalga qual.
#[test]
fn the_primary_is_the_motif_and_the_other_is_the_guide() {
    let (_scene, _sim, _map, motif, guide) = scene();
    // Primário = motif ⇒ (motif, guide).
    assert_eq!(
        link_candidate(&[motif, guide], Some(motif)),
        Some((motif, guide))
    );
    // Primário = guide ⇒ os papéis se invertem (o primário É sempre o motivo).
    assert_eq!(
        link_candidate(&[motif, guide], Some(guide)),
        Some((guide, motif))
    );
    // Um só selecionado, ou sem primário, não é candidato.
    assert_eq!(link_candidate(&[motif], Some(motif)), None);
    assert_eq!(link_candidate(&[motif, guide], None), None);
}
