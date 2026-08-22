//! Os gates da metade que MOSTRA e EDITA a moldura selecionada.
//!
//! ⚠️ A fixture central seleciona pela **porta do produto** ([`crate::vec_selection::sync_selection`])
//! e não monta a lista de caminhos à mão: o defeito reportado vive exatamente na FORMA que aquela
//! porta produz (a sub-árvore inteira), e uma lista escrita à mão teria o comprimento que o gate
//! quisesse — verde sobre o bug.

use super::*;
use crate::vec_entities::{VecEntityMap, sync};
use crate::vec_selection::{VecSelSync, sync_selection};
use ph2d_ecs::{Transform, VecClipContent, VecPathRef};
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
        sim.world_mut().entity_mut(e).insert(VecFrame);
        if clip {
            sim.world_mut().entity_mut(e).insert(VecClipContent);
        }
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
    sim.world_mut().entity_mut(fe).insert(VecFrame);
    if clip {
        sim.world_mut().entity_mut(fe).insert(VecClipContent);
    }
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

    /// A moldura que a seleção nomeia.
    ///
    /// ⚠️ **Compara a ENTIDADE, e é mais forte do que o que estava aqui.** Até 2026-08-21 estes
    /// gates liam o `Option<bool>` do recorte e distinguiam a moldura certa da errada pelo VALOR
    /// do clip — um proxy que só funcionava porque as duas perguntas eram a mesma. Com o recorte
    /// mudado para o `vec_clip_edit`, a resposta passou a ser a própria moldura, que é o que estes
    /// testes sempre quiseram dizer.
    fn frame_of(&self, sel: &[VecPathId]) -> Option<Entity> {
        frame_of_selection(&self.sim, &self.map, sel)
    }
}

/// **O REPRO** (Enio 2026-08-01: *"não vejo em lugar nenhum a seção Frame"*).
///
/// Uma moldura que chega ACOMPANHADA dos filhos tem de reportar o recorte na mesma. Nasceu
/// VERMELHO: a regra antiga exigia UM caminho selecionado e a moldura com três filhos chega
/// como quatro.
///
/// ⚠️ **A fixture passou a selecionar o conjunto EXPLICITAMENTE (2026-08-02).** Antes ela
/// pegava só a moldura e deixava a expansão produzir os quatro — e nesse dia uma moldura
/// deixou de emprestar os filhos, o que faria este gate medir uma seleção de UM e continuar
/// verde sobre outro fenómeno. Frame+filhos continua a ser uma seleção alcançável (box-select,
/// Ctrl+A, a moldura dentro de um grupo), e é ela que este gate julga.
#[test]
fn a_frame_with_children_still_reports_its_clip() {
    let s = build(3, 0, true);
    let mut subjects = vec![s.entity(s.frame)];
    subjects.extend(s.kids.iter().map(|&k| s.entity(k)));
    let sel = s.select(&subjects);
    assert_eq!(
        sel.len(),
        4,
        "premissa da fixture: moldura + 3 filhos numa selecao so'; \
         sem isso este gate não contém o fenômeno"
    );
    assert_eq!(s.frame_of(&sel), Some(s.entity(s.frame)));
}

/// A mesma moldura pela OUTRA rota (o clique de canvas publica só ela) dá a MESMA resposta — é
/// isso que impede a seção de depender de por onde o artista selecionou.
#[test]
fn the_answer_does_not_depend_on_how_the_frame_was_selected() {
    let s = build(3, 0, false);
    let want = Some(s.entity(s.frame));
    assert_eq!(s.frame_of(&[s.frame]), want);
    assert_eq!(s.frame_of(&s.select(&[s.entity(s.frame)])), want);
}

/// **Um filho sozinho não é a moldura.** O artista selecionou a FORMA; oferecer ali os controles
/// do contêiner editaria outro objeto.
#[test]
fn a_child_alone_is_not_the_frame() {
    let s = build(2, 0, true);
    let sel = s.select(&[s.entity(s.kids[0])]);
    assert_eq!(sel, vec![s.kids[0]], "premissa: uma folha não expande");
    assert_eq!(s.frame_of(&sel), None);
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
    assert_eq!(s.frame_of(&sel), None);
}

/// **Duas molduras irmãs também não** — nenhuma contém a outra.
#[test]
fn two_sibling_frames_report_no_frame() {
    let mut s = build(1, 1, true);
    let other = s.entity(s.loose[0]);
    s.sim.world_mut().entity_mut(other).insert(VecFrame);
    let sel = s.select(&[s.entity(s.frame), other]);
    assert_eq!(s.frame_of(&sel), None);
}

/// **Aninhadas: a de FORA vence**, porque é ela que contém tudo — e é nela que o artista clicou.
#[test]
fn the_outer_frame_wins_when_frames_nest() {
    let mut s = build(1, 0, true);
    // O único filho vira ele próprio uma moldura: assim há DUAS respostas possíveis e a certa é
    // distinguível da errada pela identidade, não por acaso de haver só uma.
    let inner = s.entity(s.kids[0]);
    s.sim.world_mut().entity_mut(inner).insert(VecFrame);
    let sel = s.select(&[s.entity(s.frame), inner]);
    assert_eq!(sel.len(), 2, "premissa: as DUAS molduras na seleção");
    assert_eq!(s.frame_of(&sel), Some(s.entity(s.frame)), "a de FORA");
    assert_eq!(
        s.frame_of(&[s.kids[0]]),
        Some(inner),
        "e a de dentro sozinha"
    );
}

/// A seção só existe sobre uma moldura — e é este `None` que a esconde.
#[test]
fn a_plain_shape_is_not_a_frame() {
    let (sim, map, id) = world(None);
    assert_eq!(frame_of_selection(&sim, &map, &[id]), None);
    assert_eq!(frame_of_selection(&sim, &map, &[]), None);
}

/// **Uma forma que RECORTA não vira moldura por isso** — o par deste gate vive em
/// `vec_clip_edit_tests`, e os dois juntos cercam a separação pelos dois lados: lá se prova que
/// ligar o recorte não põe o `VecFrame`, aqui que a presença do recorte não faz a seção Frame
/// (com os presets de dispositivo e o *Show as Panel*) aparecer sobre uma forma comum.
#[test]
fn a_clipping_shape_is_still_not_a_frame() {
    let (mut sim, map, id) = world(None);
    let e = Entity::from_bits(map[&id]);
    sim.world_mut().entity_mut(e).insert(VecClipContent);
    assert_eq!(
        frame_of_selection(&sim, &map, &[id]),
        None,
        "recortar nao e' ser contentor -- a secao Frame nao pode abrir aqui"
    );
}

// ⚠️ **Os gates do CHIP mudaram-se para `vec_clip_edit_tests`** (2026-08-21), junto com o que eles
// testam: escrever o recorte, o no-op que não custa passo de undo, e o alcance através da seleção
// expandida. O que ficou aqui é a pergunta deste módulo — *qual é a MOLDURA desta seleção?*
