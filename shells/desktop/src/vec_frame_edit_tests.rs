//! Os gates da metade que MOSTRA e EDITA a moldura selecionada.
//!
//! ⚠️ A fixture central seleciona pela **porta do produto** ([`crate::vec_selection::sync_selection`])
//! e não monta a lista de caminhos à mão: o defeito reportado vive exatamente na FORMA que aquela
//! porta produz (a sub-árvore inteira), e uma lista escrita à mão teria o comprimento que o gate
//! quisesse — verde sobre o bug.

use super::*;
use crate::vec_entities::{VecEntityMap, sync};
use crate::vec_selection::{VecSelSync, sync_selection};
use ph2d_ecs::{Transform, VecPathRef};
use ph2d_editor::screens::hero::GizmoStateGroup;
use ph2d_vec_scene::{VecScene, rectangle};
use std::collections::BTreeMap;

/// Um mundo com um caminho `id` que é (ou não) moldura, e o mapa `VecPathId → entidade`.
fn world(frame: Option<bool>) -> (SimWorld, VecEntityMap, VecPathId) {
    let mut sim = SimWorld::new();
    let e = sim
        .world_mut()
        .spawn((Transform::default(), VecPathRef(7)))
        .id();
    if let Some(clip) = frame {
        sim.world_mut().entity_mut(e).insert(VecFrame { clip });
    }
    let mut map: VecEntityMap = BTreeMap::new();
    map.insert(7, e.to_bits());
    (sim, map, 7)
}

/// Uma cena real: uma moldura com `kids` filhos, mais `loose` formas soltas na raiz.
///
/// Devolve tudo o que um gate precisa para SELECIONAR como o produto seleciona.
struct Scene {
    sim: SimWorld,
    scene: VecScene,
    map: VecEntityMap,
    frame: VecPathId,
    kids: Vec<VecPathId>,
    loose: Vec<VecPathId>,
}

fn build(kids: usize, loose: usize, clip: bool) -> Scene {
    let mut sim = SimWorld::default();
    let mut scene = VecScene::new();
    let mut map = VecEntityMap::new();

    let frame = scene.push_path(rectangle([0.0, 0.0], [8.0, 6.0]));
    let kid_ids: Vec<VecPathId> = (0..kids)
        .map(|k| {
            let x = k as f64;
            scene.push_path(rectangle([x, 0.0], [x + 0.5, 0.5]))
        })
        .collect();
    let loose_ids: Vec<VecPathId> = (0..loose)
        .map(|k| {
            let x = 20.0 + k as f64;
            scene.push_path(rectangle([x, 0.0], [x + 0.5, 0.5]))
        })
        .collect();
    sync(&mut sim, &mut scene, &mut map);

    let fe = Entity::from_bits(map[&frame]);
    sim.world_mut().entity_mut(fe).insert(VecFrame { clip });
    for id in &kid_ids {
        let e = Entity::from_bits(map[id]);
        sim.world_mut().entity_mut(e).insert(ChildOf(fe));
    }
    Scene {
        sim,
        scene,
        map,
        frame,
        kids: kid_ids,
        loose: loose_ids,
    }
}

impl Scene {
    fn entity(&self, id: VecPathId) -> Entity {
        Entity::from_bits(self.map[&id])
    }

    /// A seleção que o PRODUTO publica quando a Hierarquia acende `entities` — a rota que
    /// **expande** para a sub-árvore.
    fn select(&self, entities: &[Entity]) -> Vec<VecPathId> {
        let mut gizmo = GizmoStateGroup::default();
        let mut pen = ph2d_vec_edit::PenTool::default();
        let mut state = VecSelSync::default();
        let mut it = entities.iter();
        if let Some(&first) = it.next() {
            gizmo.replace_selection(Some(first.to_bits()));
        }
        for &e in it {
            gizmo.add_to_selection(e.to_bits());
        }
        sync_selection(
            &mut gizmo,
            &self.sim,
            &self.scene,
            &self.map,
            &mut pen,
            &mut state,
            false,
        );
        pen.selected_paths().to_vec()
    }

    fn clip_of(&self, sel: &[VecPathId]) -> Option<bool> {
        selected_frame_clip(&self.sim, &self.map, sel)
    }
}

/// **O REPRO** (Enio 2026-08-01: *"não vejo em lugar nenhum a seção Frame"*).
///
/// Selecionar uma moldura COM CONTEÚDO pela Hierarquia expande a seleção para a sub-árvore
/// inteira; a seção tem de aparecer na mesma. Nasceu VERMELHO: a regra antiga exigia UM caminho
/// selecionado e a moldura com três filhos chega como quatro.
#[test]
fn a_frame_with_children_still_reports_its_clip() {
    let s = build(3, 0, true);
    let sel = s.select(&[s.entity(s.frame)]);
    assert_eq!(
        sel.len(),
        4,
        "premissa da fixture: a seleção de um contêiner EXPANDE (moldura + 3 filhos); \
         sem isso este gate não contém o fenômeno"
    );
    assert_eq!(s.clip_of(&sel), Some(true));
}

/// A mesma moldura pela OUTRA rota (o clique de canvas publica só ela) dá a MESMA resposta — é
/// isso que impede a seção de depender de por onde o artista selecionou.
#[test]
fn the_answer_does_not_depend_on_how_the_frame_was_selected() {
    let s = build(3, 0, false);
    assert_eq!(s.clip_of(&[s.frame]), Some(false));
    assert_eq!(s.clip_of(&s.select(&[s.entity(s.frame)])), Some(false));
}

/// **Um filho sozinho não é a moldura.** O artista selecionou a FORMA; oferecer ali os controles
/// do contêiner editaria outro objeto.
#[test]
fn a_child_alone_is_not_the_frame() {
    let s = build(2, 0, true);
    let sel = s.select(&[s.entity(s.kids[0])]);
    assert_eq!(sel, vec![s.kids[0]], "premissa: uma folha não expande");
    assert_eq!(s.clip_of(&sel), None);
}

/// **Moldura + forma solta não tem UMA resposta** — a moldura não contém a forma de fora.
#[test]
fn a_frame_plus_an_outsider_reports_no_frame() {
    let s = build(2, 1, true);
    let sel = s.select(&[s.entity(s.frame), s.entity(s.loose[0])]);
    assert!(
        sel.contains(&s.loose[0]) && sel.contains(&s.frame),
        "premissa: os dois estão na seleção"
    );
    assert_eq!(s.clip_of(&sel), None);
}

/// **Duas molduras irmãs também não** — nenhuma contém a outra.
#[test]
fn two_sibling_frames_report_no_frame() {
    let mut s = build(1, 1, true);
    let other = s.entity(s.loose[0]);
    s.sim
        .world_mut()
        .entity_mut(other)
        .insert(VecFrame { clip: false });
    let sel = s.select(&[s.entity(s.frame), other]);
    assert_eq!(s.clip_of(&sel), None);
}

/// **Aninhadas: a de FORA vence**, porque é ela que contém tudo — e é nela que o artista clicou.
#[test]
fn the_outer_frame_wins_when_frames_nest() {
    let mut s = build(1, 0, true);
    // O único filho vira ele próprio uma moldura, com o recorte OPOSTO: assim a resposta certa é
    // distinguível da errada por VALOR, não só por presença.
    let inner = s.entity(s.kids[0]);
    s.sim
        .world_mut()
        .entity_mut(inner)
        .insert(VecFrame { clip: false });
    let sel = s.select(&[s.entity(s.frame)]);
    assert_eq!(
        sel.len(),
        2,
        "premissa: a de fora expandiu para a de dentro"
    );
    assert_eq!(s.clip_of(&sel), Some(true), "a de FORA");
    assert_eq!(
        s.clip_of(&[s.kids[0]]),
        Some(false),
        "e a de dentro sozinha"
    );
}

/// A seção só existe sobre uma moldura — e é este `None` que a esconde.
#[test]
fn a_plain_shape_is_not_a_frame() {
    let (sim, map, id) = world(None);
    assert_eq!(selected_frame_clip(&sim, &map, &[id]), None);
    assert_eq!(selected_frame_clip(&sim, &map, &[]), None);
}

/// Sobre uma moldura, o chip mostra o que o componente diz.
#[test]
fn a_frame_reports_its_clip() {
    for clip in [false, true] {
        let (sim, map, id) = world(Some(clip));
        assert_eq!(selected_frame_clip(&sim, &map, &[id]), Some(clip));
    }
}

/// O chip escreve — e escrever o MESMO valor não muda nada (o undo é por diff; um passo por
/// clique repetido encheria a fila com estados idênticos).
#[test]
fn the_chip_writes_the_clip_and_a_no_op_changes_nothing() {
    let (mut sim, map, id) = world(Some(true));
    assert!(
        !set_selected_frame_clip(&mut sim, &map, &[id], true),
        "no-op"
    );
    assert!(set_selected_frame_clip(&mut sim, &map, &[id], false));
    assert_eq!(selected_frame_clip(&sim, &map, &[id]), Some(false));
}

/// E o chip alcança a moldura **através da seleção expandida** — senão ele mostraria o estado
/// certo e não editaria nada, que é a metade do defeito que um gate de leitura não pega.
#[test]
fn the_chip_writes_through_an_expanded_selection() {
    let mut s = build(3, 0, true);
    let sel = s.select(&[s.entity(s.frame)]);
    assert!(set_selected_frame_clip(&mut s.sim, &s.map, &sel, false));
    assert_eq!(s.clip_of(&sel), Some(false));
}

/// ⚠️ **Escrever numa forma que não é moldura CRIARIA uma** — um chip de opção viraria um gesto
/// de criação, e o artista ganharia um contêiner que nunca desenhou.
#[test]
fn the_chip_never_turns_a_plain_shape_into_a_frame() {
    let (mut sim, map, id) = world(None);
    assert!(!set_selected_frame_clip(&mut sim, &map, &[id], true));
    assert_eq!(selected_frame_clip(&sim, &map, &[id]), None);
}
