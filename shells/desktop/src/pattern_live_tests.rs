//! Gates do **Pattern Along Path VIVO** — arquivo irmão de `pattern_live.rs`.
//!
//! O oráculo é a geometria DERIVADA que o `dispatch` desenharia (`PatternLive::live()`): um motivo
//! vinculado a um guia produz N cópias no z dele, e a curva do motivo nunca é tocada. As mutações
//! que estes gates existem para matar: o `recook` não ler o componente (0 cópias), o `detach` não
//! remover o vínculo (as cópias sobrevivem à soltura), o `link` prender a forma a si mesma.

use super::{
    PatternHandle, PatternLive, detach, handle, link, link_candidate, rotation_of,
    set_rotation, spec_of,
};
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
        verts: vec![
            VecVertex::corner([0.0, 0.0]),
            VecVertex::corner([100.0, 0.0]),
        ],
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
    assert!(
        link(&mut sim, &map, motif, guide),
        "prendeu o motivo ao guia"
    );
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
    assert_eq!(
        copies(&live, motif),
        0,
        "solto: sem cópias (o motivo vira fonte)"
    );
    // O guia sobreviveu à soltura — soltar remove a RELAÇÃO, não o objeto.
    assert!(scene.path(guide).is_some(), "o caminho-guia fica");
}

/// **Prender uma forma a si mesma não quer dizer nada** — recusado. Sem isto, o motivo cavalgaria
/// a própria silhueta e o `guide_arc` leria a geometria que ele está a produzir.
#[test]
fn a_shape_cannot_ride_itself() {
    let (_scene, mut sim, map, motif, _guide) = scene();
    assert!(
        !link(&mut sim, &map, motif, motif),
        "recusa prender a si mesma"
    );
    assert!(
        spec_of(&sim, &map, motif).is_none(),
        "nenhum vínculo criado"
    );
}

/// **O guia é o caminho de MAIOR extensão, INDEPENDENTE da ordem de seleção** (Enio, 2026-07-23:
/// *"escolhendo a si mesmo e não a outra curva"*). O motivo é o quadrado (bbox diag ~56,6); o guia
/// é a reta (bbox 100). Mutação que mata: a regra antiga (primário=motivo) invertia os papéis
/// conforme a ordem de clique.
#[test]
fn the_guide_is_the_larger_path_regardless_of_order() {
    let (scene, _sim, _map, motif, guide) = scene();
    // O guia é a RETA (extensão 100 > 56,6 do quadrado) nas DUAS ordens.
    assert_eq!(
        link_candidate(&scene, &[motif, guide]),
        Some((motif, guide))
    );
    assert_eq!(
        link_candidate(&scene, &[guide, motif]),
        Some((motif, guide))
    );
    // Menos/mais de dois, ou o mesmo id duas vezes, não é candidato.
    assert_eq!(link_candidate(&scene, &[motif]), None);
    assert_eq!(link_candidate(&scene, &[motif, motif]), None);
}

/// **`linked_motif` acha o motivo pela seleção, não pelo primário** — depois de prender, o primário
/// pode ser o GUIA (o último clicado), mas quem tem os controles é o motivo. Sem isto, os
/// sliders/alças editariam o guia (sem componente) e não fariam nada.
#[test]
fn the_linked_motif_is_found_in_the_selection_not_the_primary() {
    let (_scene, mut sim, map, motif, guide) = scene();
    link(&mut sim, &map, motif, guide);
    // A seleção lista o GUIA primeiro; o motivo (linkado) é o segundo — e é ele que se acha.
    assert_eq!(
        super::linked_motif(&sim, &map, &[guide, motif]),
        Some(motif)
    );
    assert_eq!(super::linked_motif(&sim, &map, &[guide]), None);
}

/// **As duas alças (W4) ficam nas pontas do trecho, e arrastá-las edita Start/End.** A guia é a
/// reta [0,0]→[100,0], então a ficha de Start cai em ~(0,0) e a de End em ~(100,0). Mutação que
/// mata: o `drag` escrever o campo errado (Start em vez de End) muda `start_offset`, não `end`.
#[test]
fn the_handles_sit_at_the_ends_and_dragging_edits_them() {
    let (scene, mut sim, map, motif, guide) = scene();
    link(&mut sim, &map, motif, guide);
    let sel = [motif, guide];
    let (s, e) = handle::world(&sim, &scene, &map, &sel).expect("vinculado -> alcas");
    assert!(s[0].abs() < 1e-3, "Start na origem do arco: {s:?}");
    assert!((e[0] - 100.0).abs() < 1e-2, "End no fim do arco: {e:?}");

    // Pressão sobre a ficha do END arma o End.
    let mut armed = None;
    assert!(handle::press(
        &sim,
        &scene,
        &map,
        &sel,
        [100.0, 0.0],
        5.0,
        &mut armed
    ));
    assert_eq!(
        armed,
        Some(PatternHandle::End),
        "a ficha sob o cursor é a de End"
    );

    // Arrastá-la para o arco 50 escreve end_offset = 0.5 (e NÃO mexe no start).
    assert!(handle::drag(
        &mut sim,
        &scene,
        &map,
        &sel,
        [50.0, 0.0],
        armed
    ));
    let spec = spec_of(&sim, &map, motif).unwrap();
    assert!(
        (spec.end_offset - 0.5).abs() < 0.02,
        "End arrastado p/ 0.5: {}",
        spec.end_offset
    );
    assert!(spec.start_offset.abs() < 1e-6, "o Start não se moveu");

    // Uma seleção SEM pattern ⇒ sem alças.
    assert!(handle::world(&sim, &scene, &map, &[guide]).is_none());
}

/// **Sob FLIP, as alças acompanham as cópias (o bug do Enio).** O motor espelha o arco
/// (`total - s`), então a ficha de Start salta para o FIM físico do arco e a de End para o começo,
/// e arrastar escreve a fração LÓGICA (invertida). Mutação que mata: ignorar o flip na alça deixa
/// as fichas num lado e as cópias no outro.
#[test]
fn the_handles_follow_the_copies_under_flip() {
    let (scene, mut sim, map, motif, guide) = scene();
    link(&mut sim, &map, motif, guide);
    super::edit(&mut sim, &map, motif, |l| l.flip = true);
    let sel = [motif, guide];

    // Start (frac 0) cai no arco físico 100; End (frac 1) no arco físico 0 — ESPELHADOS.
    let (s, e) = handle::world(&sim, &scene, &map, &sel).expect("alças sob flip");
    assert!(
        (s[0] - 100.0).abs() < 1e-2,
        "Start no fim físico sob flip: {s:?}"
    );
    assert!(e[0].abs() < 1e-2, "End no começo físico sob flip: {e:?}");

    // Arrastar a ficha de Start para o arco físico 25 escreve start_offset = 0.75 (o lógico).
    assert!(handle::drag(
        &mut sim,
        &scene,
        &map,
        &sel,
        [25.0, 0.0],
        Some(PatternHandle::Start)
    ));
    let start = spec_of(&sim, &map, motif).unwrap().start_offset;
    assert!(
        (start - 0.75).abs() < 0.02,
        "Start lógico sob flip: {start}"
    );
}

// ── A ATITUDE do motivo (`VecPatternRotation`) ───────────────────────────────────

/// **A rotação autorada chega às CÓPIAS** — o gate ponta-a-ponta da metade shell: escrever pela
/// porta muda o que o `dispatch` desenharia, no MESMO frame.
///
/// O oráculo é a geometria derivada, não o componente: o quadrado de 40 deitado dá 2 cópias na
/// reta de 100; girado 90° ele continua a dar 2 (um quadrado é simétrico — de propósito, para
/// isolar a ATITUDE do avanço), mas cada cópia sai **virada**, e a viragem mede-se pela diagonal.
///
/// ⚠️ Mutação: o `recook` não ler a rotação (passar `0.0` ao `spec_to_motor`) deixa a geometria
/// idêntica à do não-girado ⇒ RED.
#[test]
fn the_authored_attitude_reaches_the_copies() {
    let (scene, mut sim, map, motif, guide) = scene();
    assert!(link(&mut sim, &map, motif, guide));

    let mut live = PatternLive::default();
    live.recook(&scene, &sim, &map);
    let flat: Vec<[f64; 2]> = live.live()[&motif][0].verts.iter().map(|v| v.anchor).collect();

    assert!(set_rotation(&mut sim, &map, motif, 45.0), "escreveu a atitude");
    live.recook(&scene, &sim, &map);
    let turned: Vec<[f64; 2]> = live.live()[&motif][0].verts.iter().map(|v| v.anchor).collect();

    assert_eq!(flat.len(), turned.len(), "o nº de vértices não muda");
    let moved = flat
        .iter()
        .zip(&turned)
        .filter(|(a, b)| (a[0] - b[0]).abs() > 1e-6 || (a[1] - b[1]).abs() > 1e-6)
        .count();
    assert_eq!(
        moved,
        flat.len(),
        "a 45° TODO vértice da cópia devia ter-se movido; moveram-se {moved} de {}",
        flat.len()
    );
}

/// **O neutro DESTACA o componente** — arquivo sem no-op, e *"não tem o componente"* volta a
/// significar exatamente *"não está girado"*, sem uma segunda forma de o dizer.
///
/// ⚠️ Mutação: `set_rotation` inserir `VecPatternRotation(0.0)` em vez de remover deixa o
/// componente vivo com um zero ⇒ RED (e o save passaria a carregar um no-op para sempre).
#[test]
fn the_neutral_attitude_detaches_the_component() {
    let (_scene, mut sim, map, motif, guide) = scene();
    assert!(link(&mut sim, &map, motif, guide));
    assert_eq!(rotation_of(&sim, &map, motif), 0.0, "nasce sem rotação");

    assert!(set_rotation(&mut sim, &map, motif, 30.0));
    assert_eq!(rotation_of(&sim, &map, motif), 30.0);
    let e = ph2d_ecs::Entity::from_bits(map[&motif]);
    assert!(
        sim.world().get::<ph2d_ecs::VecPatternRotation>(e).is_some(),
        "com ângulo autorado o componente existe"
    );

    assert!(set_rotation(&mut sim, &map, motif, 0.0));
    assert!(
        sim.world().get::<ph2d_ecs::VecPatternRotation>(e).is_none(),
        "no neutro o componente tem de ser REMOVIDO, não guardado a zero"
    );
    assert_eq!(rotation_of(&sim, &map, motif), 0.0, "e continua a ler 0");
}

/// **Soltar leva a atitude junto.** Sem isto a rotação vira estado invisível que RESSUSCITA no
/// próximo `link`: o artista prende o motivo a outra curva e as cópias nascem tortas por um ângulo
/// que ele não vê em lado nenhum.
///
/// ⚠️ Mutação: `detach` remover só o `VecPatternPath` ⇒ o re-link lê 45° ⇒ RED.
#[test]
fn detaching_takes_the_attitude_with_it() {
    let (_scene, mut sim, map, motif, guide) = scene();
    assert!(link(&mut sim, &map, motif, guide));
    assert!(set_rotation(&mut sim, &map, motif, 45.0));

    assert!(detach(&mut sim, &map, motif), "soltou");
    assert_eq!(
        rotation_of(&sim, &map, motif),
        0.0,
        "a atitude tem de morrer com o vínculo"
    );

    assert!(link(&mut sim, &map, motif, guide), "prendeu de novo");
    assert_eq!(
        rotation_of(&sim, &map, motif),
        0.0,
        "o re-link não pode ressuscitar o ângulo do vínculo anterior"
    );
}

/// **Girar um motivo SOLTO não quer dizer nada** — e a porta recusa, em vez de deixar um
/// componente órfão que ressuscitaria no primeiro `link`.
#[test]
fn the_attitude_needs_a_link() {
    let (_scene, mut sim, map, motif, _guide) = scene();
    assert!(
        !set_rotation(&mut sim, &map, motif, 45.0),
        "sem vínculo a porta recusa"
    );
    assert_eq!(rotation_of(&sim, &map, motif), 0.0);
}
