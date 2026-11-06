//! Gates do undo global (captura · restore · canonicalização · a fila). Irmão do [`super`] pelo teto
//! de LOC (HR-18). ⚠️ **A PARTILHA entre passos vive no [`super::sharing_tests`]** — identidade e
//! igualdade são perguntas diferentes.

use super::*;
use ph2d_ecs::scene::{ComponentRegistry, register_ecs_components};
use ph2d_ecs::{Name, VecPathRef};
use ph2d_render::register_render_components;
use ph2d_vec_scene::rectangle;

pub(super) fn registry() -> ComponentRegistry {
    let mut reg = ComponentRegistry::new();
    register_ecs_components(&mut reg);
    register_render_components(&mut reg);
    reg
}

/// Uma cena de teste: 1 sprite nomeado + 1 forma vetorial com a entidade dela.
pub(super) fn scene() -> (SimWorld, VecScene) {
    let mut sim = SimWorld::new();
    let mut vec = VecScene::new();
    sim.world_mut().spawn((
        Transform::from_translation(ph2d_core::Vec2::new(3.0, 4.0)),
        Name::new("Spr"),
    ));
    let id = vec.push_path(rectangle([0.0, 0.0], [2.0, 2.0]));
    sim.world_mut()
        .spawn((Transform::default(), Name::new("Path 0"), VecPathRef(id)));
    (sim, vec)
}

/// ⚠️ **Sem passo anterior** (`None`): estes gates medem a captura e o restauro, e a partilha da
/// cena entre passos vive no [`super::sharing_tests`], que passa o dele.
pub(super) fn capture(sim: &mut SimWorld, vec: &VecScene, reg: &ComponentRegistry) -> ProjectState {
    ProjectState::capture(
        // Nada sob condução: estes gates são do diff e do restore, não do ledger
        // (`crate::preview_drive` tem os dele).
        &ph2d_preview_drive::PreviewDrive::default(),
        sim,
        vec,
        &FlipDoc::new(),
        &ph2d_guides::GuideSet::default(),
        &ph2d_ui_state::StateSets::default(),
        &crate::project_library::LibraryDoc::default(),
        &[],
        reg,
        &mut ph2d_ecs::scene::incremental::CaptureCache::new(),
        None,
    )
}

/// Capturar → mexer no mundo E na geometria → restaurar devolve o estado exato,
/// e a ponte path↔entidade fica reconstruída (uma entrada por forma).
#[test]
fn capture_then_restore_round_trips_world_and_geometry() {
    let reg = registry();
    let (mut sim, mut vec) = scene();
    let before = capture(&mut sim, &vec, &reg);

    // Muda tudo: move o sprite, apaga a forma, desenha outra.
    {
        let mut q = sim.world_mut().query::<&mut Transform>();
        for mut t in q.iter_mut(sim.world_mut()) {
            t.translation.x += 100.0;
        }
    }
    vec = VecScene::new();
    vec.push_path(rectangle([9.0, 9.0], [10.0, 10.0]));

    let (rvec, map, _rflip, _fmap) = before.restore(&mut sim, &reg);
    vec = rvec;

    // A geometria voltou ao original.
    assert_eq!(vec.paths().len(), 1);
    assert_eq!(vec.paths()[0].verts[0].anchor, [0.0, 0.0]);
    // A ponte tem exatamente a forma restaurada, e aponta uma entidade viva.
    assert_eq!(map.len(), 1);
    let (&pid, &bits) = map.iter().next().unwrap();
    assert_eq!(pid, vec.paths()[0].id);
    assert!(sim.world().get_entity(Entity::from_bits(bits)).is_ok());
    // O sprite voltou à posição original (a captura preservou o Transform).
    let names: Vec<f32> = {
        let mut q = sim.world_mut().query::<(&Name, &Transform)>();
        q.iter(sim.world())
            .filter(|(n, _)| n.as_str() == "Spr")
            .map(|(_, t)| t.translation.x)
            .collect()
    };
    assert_eq!(names, vec![3.0]);
}

/// A ponte reconstruída faz o `sync` seguinte virar NO-OP — não duplica.
#[test]
fn the_rebuilt_bridge_makes_the_next_sync_a_noop() {
    let reg = registry();
    let (mut sim, vec) = scene();
    let snap = capture(&mut sim, &vec, &reg);
    let (mut rvec, mut map, _rflip, _fmap) = snap.restore(&mut sim, &reg);

    let entities_before = map.len();
    ph2d_vec_entities::entities::sync(&mut sim, &mut rvec, &mut map);
    assert_eq!(map.len(), entities_before, "sync não spawnou nada");
    // Uma entidade por path, e nenhuma órfã (a contagem de VecPathRef == paths).
    let vecref_count = {
        let mut q = sim.world_mut().query::<&VecPathRef>();
        q.iter(sim.world()).count()
    };
    assert_eq!(vecref_count, rvec.paths().len());
}

/// push_undo/undo/redo alternam corretamente (o registro por-diff é testado no
/// nível do App; aqui é a pilha pura).
#[test]
fn push_undo_then_undo_redo_alternate() {
    let reg = registry();
    let (mut sim, vec) = scene();
    let s0 = capture(&mut sim, &vec, &reg);
    let mut stack = ProjectUndo::default();
    assert!(!stack.can_undo() && !stack.can_redo());

    // Um passo: s0 é o estado-pré.
    let mut vec1 = vec.clone();
    vec1.push_path(rectangle([5.0, 5.0], [6.0, 6.0]));
    let s1 = ProjectState {
        world: s0.world.clone(),
        vec: std::sync::Arc::new(vec1),
        flip: std::sync::Arc::new(FlipDoc::new()),
        guides: ph2d_guides::GuideSet::default(),
        ui_states: ph2d_ui_state::StateSets::default(),
        library: crate::project_library::LibraryDoc::default(),
        tags: Vec::new(),
        regras: Default::default(),
        formato_do_jogo: None,
    };
    // ⭐⭐ **A marca da SELEÇÃO viaja com o passo** (2026-09-04) — duas diferentes, para o gate
    // poder dizer qual voltou. Ver [`crate::undo::SelectionMark`].
    let m0 = crate::undo::SelectionMark {
        field: vec![ph2d_ecs::StableId(7)],
    };
    let m1 = crate::undo::SelectionMark {
        field: vec![ph2d_ecs::StableId(9)],
    };
    stack.push_undo(s0.clone(), m0.clone());
    assert!(stack.can_undo() && !stack.can_redo());

    // Undo devolve o pré (s0) **com a selecção dele** e arma o redo com o atual (s1, m1).
    let back = stack.undo(s1.clone(), m1.clone()).unwrap();
    assert_eq!(back, (s0.clone(), m0.clone()));
    assert!(stack.can_redo());
    // ⭐⭐⭐ **Redo devolve s1 E a mão que estava nele** — era exactamente isto que faltava: refazer
    // uma criação devolvia o objecto e deixava a selecção vazia, e o gizmo não voltava mais
    // (report do Enio, 2026-09-04).
    let fwd = stack.redo(s0.clone(), m0.clone()).unwrap();
    assert_eq!(fwd, (s1.clone(), m1));
    assert!(stack.can_undo());
    // push_undo limpa o redo.
    let _ = stack.undo(s1.clone(), crate::undo::SelectionMark::default());
    stack.push_undo(s0.clone(), m0.clone());
    assert!(!stack.can_redo());
    // ⭐ **A transação viaja com o passo** (W8, a etapa editada de uma macro e a imagem dela): o topo
    // do desfazer tem-na; desfeito, o topo do refazer; refeito, de volta ao desfazer.
    stack.push_undo_em(s1.clone(), m0.clone(), Some(42));
    let topos = |p: &ProjectUndo| (p.transacao_do_topo(false), p.transacao_do_topo(true));
    assert_eq!(topos(&stack), (Some(42), None));
    let _ = stack.undo(s0.clone(), m0.clone());
    assert_eq!(topos(&stack), (None, Some(42)));
    let _ = stack.redo(s1.clone(), m0);
    assert_eq!(topos(&stack), (Some(42), None));
}

/// ⭐ Gravity Y com Auto Key (smoke 30/10): o passo global e a chave do MESMO gesto são uma transação
/// — o Ctrl+Z da timeline cede-o à fila global, e o global desfaz/refaz a chave com ele.
#[test]
fn o_passo_global_leva_a_chave_do_auto_key_do_mesmo_gesto() {
    let reg = registry();
    let (mut sim, vec) = scene();
    let s0 = capture(&mut sim, &vec, &reg);
    let mark = crate::undo::SelectionMark::default;
    let mut tl = ph2d_timeline::TimelineState::new();
    tl.history.begin(&tl.doc);
    tl.doc.upsert_key(
        1,
        ph2d_timeline::PropKind::Rotation,
        Default::default(),
        ph2d_anim::AnimValue::Float(1.0),
        ph2d_timeline::Interp::Hold,
    );
    assert!(tl.history.commit_da_cena(&tl.doc));
    let mut stack = ProjectUndo::default();
    stack.push_linked(
        s0.clone(),
        mark(),
        None,
        None,
        tl.history.liga_gesto_da_cena(),
    );
    for redo in [false, true] {
        let t = tl.history.transacao_do_topo(redo);
        assert!(
            t.is_some() && stack.linha_do_topo(redo) == t,
            "ligados (redo={redo})"
        );
        let _ = if redo {
            stack.redo(s0.clone(), mark())
        } else {
            stack.undo(s0.clone(), mark())
        };
        let _ = if redo { tl.redo() } else { tl.undo() };
    }
}

/// ⭐ **O elo do marcador viaja com o degrau** (CodeEditor I10, `ph2d_editor_core::undo_link`): o
/// degrau desfeito leva-o para o refazer e volta com ele; um degrau novo que limpa o refazer deixa-o
/// de guardar (a família fica a saber pelos `links`).
#[test]
fn the_undo_link_travels_with_its_step_and_dies_with_the_redo() {
    use ph2d_editor_core::undo_link::UndoLink;
    let reg = registry();
    let (mut sim, vec) = scene();
    let s0 = capture(&mut sim, &vec, &reg);
    let mark = crate::undo::SelectionMark::default;
    let l = UndoLink::fresh();
    let mut stack = ProjectUndo::default();
    stack.push_undo(s0.clone(), mark());
    stack.push_linked(s0.clone(), mark(), None, Some(l), None);
    assert_eq!(stack.top_link(false), Some(l));
    assert_eq!(stack.top_link(true), None);
    let _ = stack.undo(s0.clone(), mark()).unwrap();
    assert_eq!(
        (stack.top_link(false), stack.top_link(true)),
        (None, Some(l))
    );
    let _ = stack.redo(s0.clone(), mark()).unwrap();
    assert_eq!(
        (stack.top_link(false), stack.top_link(true)),
        (Some(l), None)
    );
    let _ = stack.undo(s0.clone(), mark()).unwrap();
    assert_eq!(stack.links().collect::<Vec<_>>(), [l]);
    stack.push_undo(s0, mark());
    assert_eq!(stack.links().count(), 0, "o refazer limpo leva o elo");
}
/// O diff de undo depende disto: capturar o MESMO estado duas vezes tem de dar
/// snapshots IGUAIS. Se não, o `post_frame_undo` registraria um passo espúrio a
/// cada frame com input (Enio 2026-07-09: "precisa de mais de 1 Ctrl+Z").
#[test]
fn capturing_the_same_state_twice_is_identical() {
    let reg = registry();
    let (mut sim, vec) = scene();
    let a = capture(&mut sim, &vec, &reg);
    let b = capture(&mut sim, &vec, &reg);
    assert_eq!(a, b, "captura não-determinística -> diff espúrio");
}

/// E depois de um restore (ids de entidade NOVOS), capturar de novo tem de dar o
/// mesmo snapshot — senão o primeiro frame pós-undo registra um passo fantasma.
#[test]
fn capture_after_restore_matches_the_restored_state() {
    let reg = registry();
    let (mut sim, vec) = scene();
    let snap = capture(&mut sim, &vec, &reg);
    let (rvec, _, _, _) = snap.restore(&mut sim, &reg);
    let again = capture(&mut sim, &rvec, &reg);
    assert_eq!(snap, again, "restore->capture != capture original");
}
/// ADR-0114: o `FlipDoc` entra na captura como 3º campo. Round-trip por
/// capture→restore, a ponte objeto↔entidade é reconstruída, e capturar o
/// mesmo estado 2× é idêntico (o flip é determinístico — sem diff espúrio).
#[test]
fn flip_survives_capture_restore_and_rebuilds_bridge() {
    use ph2d_flip::{Hold, KeyKind};
    let reg = registry();
    let mut sim = SimWorld::new();
    let vec = VecScene::new();
    // Documento Flip: um objeto (camada + frame + traço) + a entidade-ponte.
    let mut flip = FlipDoc::new();
    let oid = flip.push_object("Char");
    {
        let obj = flip.object_mut(oid).unwrap();
        let l = obj.add_layer("L");
        let d = obj
            .insert_frame(l, 0, Hold::Implicit, KeyKind::Keyframe)
            .unwrap();
        obj.drawing_mut(d).unwrap().strokes.push(Default::default());
    }
    sim.world_mut().spawn((
        Transform::default(),
        Name::new("Char"),
        ph2d_ecs::FlipObjectRef(oid.0),
    ));

    let snap = {
        ProjectState::capture(
            &ph2d_preview_drive::PreviewDrive::default(),
            &mut sim,
            &vec,
            &flip,
            &ph2d_guides::GuideSet::default(),
            &ph2d_ui_state::StateSets::default(),
            &crate::project_library::LibraryDoc::default(),
            &[],
            &reg,
            &mut ph2d_ecs::scene::incremental::CaptureCache::new(),
            None,
        )
    };

    // Muda o flip (adiciona objeto) e restaura ao capturado.
    flip.push_object("Extra");
    let (_rvec, _vmap, rflip, fmap) = snap.restore(&mut sim, &reg);
    assert_eq!(rflip.objects().len(), 1, "o flip voltou ao capturado");
    assert_eq!(fmap.len(), 1, "a ponte flip reconstruída tem o objeto");
    let (&fid, &bits) = fmap.iter().next().unwrap();
    assert_eq!(fid, oid);
    assert!(sim.world().get_entity(Entity::from_bits(bits)).is_ok());

    // Capturar o mesmo estado 2× = idêntico (sem passo espúrio de undo).
    let a = ProjectState::capture(
        &ph2d_preview_drive::PreviewDrive::default(),
        &mut sim,
        &vec,
        &rflip,
        &ph2d_guides::GuideSet::default(),
        &ph2d_ui_state::StateSets::default(),
        &crate::project_library::LibraryDoc::default(),
        &[],
        &reg,
        &mut ph2d_ecs::scene::incremental::CaptureCache::new(),
        None,
    );
    let b = ProjectState::capture(
        &ph2d_preview_drive::PreviewDrive::default(),
        &mut sim,
        &vec,
        &rflip,
        &ph2d_guides::GuideSet::default(),
        &ph2d_ui_state::StateSets::default(),
        &crate::project_library::LibraryDoc::default(),
        &[],
        &reg,
        &mut ph2d_ecs::scene::incremental::CaptureCache::new(),
        None,
    );
    assert_eq!(a, b, "flip determinístico -> sem diff espúrio");
}

/// REGRESSÃO (Enio 2026-07-09: "ao deletar não volta no mesmo lugar"). O
/// `RootOrder` (a posição de z / linha na hierarquia) tem de sobreviver ao
/// ciclo capturar → deletar → restaurar.
#[test]
fn delete_then_restore_preserves_z_order() {
    use ph2d_ecs::RootOrder;
    let reg = registry();
    let mut sim = SimWorld::new();
    let vec = VecScene::new();
    // Três sprites com ordens de z distintas.
    let a = sim
        .world_mut()
        .spawn((Transform::default(), Name::new("A"), RootOrder(0)))
        .id();
    let b = sim
        .world_mut()
        .spawn((Transform::default(), Name::new("B"), RootOrder(1)))
        .id();
    let _c = sim
        .world_mut()
        .spawn((Transform::default(), Name::new("C"), RootOrder(2)))
        .id();
    let _ = (a, b);

    let snap = capture(&mut sim, &vec, &reg);
    // Deleta o do meio (B).
    sim.world_mut().despawn(b);
    // Restaura o estado pré-delete.
    let _ = snap.restore(&mut sim, &reg);

    // Cada nome volta com o RootOrder original.
    let mut got: Vec<(String, u32)> = {
        let mut q = sim.world_mut().query::<(&Name, &RootOrder)>();
        q.iter(sim.world())
            .map(|(n, r)| (n.as_str().to_owned(), r.0))
            .collect()
    };
    got.sort();
    assert_eq!(
        got,
        vec![("A".into(), 0), ("B".into(), 1), ("C".into(), 2)],
        "o objeto deletado volta na MESMA posição de z"
    );
}

/// ⭐ **Os efeitos no undo** — irmão por assunto, ver o cabeçalho de lá.
#[path = "undo_effects_tests.rs"]
mod effects_tests;

/// ⭐⭐⭐ **A ÁRVORE DE TAGS no undo** (TOP-20 #9) — irmão por assunto, ver o cabeçalho de lá.
#[path = "undo_tags_tests.rs"]
mod tags_tests;

#[path = "undo_library_tests.rs"]
mod library;

/// ⭐ **As regras do mundo e da mesa no undo** (F41) — irmão por assunto, ver o cabeçalho de lá.
#[path = "undo_regras_tests.rs"]
mod regras_tests;

/// ⭐ O cemitério (doc ObjectsAndComponents 01 §14, item 3) é tão fundo como este undo: mais fundo, o
/// undo global já não devolve o objecto e a cova nunca abriria; mais raso, um objecto apagado há
/// muitos passos voltaria sem a animação.
#[test]
fn the_graveyard_is_as_deep_as_the_global_undo() {
    assert_eq!(ph2d_ecs::GRAVEYARD_DEPTH, UNDO_CAP);
}

/// ⭐⭐ **Com a timeline aberta ou a ferramenta Motion na mão, o Ctrl+Z que encontra uma purga
/// enterrada vai ao undo GLOBAL** (onda 4, item 3). Antes: a fila própria consumia-o, repunha os
/// clipes/nós de um dono morto, a purga tirava-os outra vez (timeline) ou a migração dava-os a outro
/// objecto (Motion) — e o objecto nunca voltava. As leis estão provadas nas crates
/// (`top_is_buried`, `undo_e_do_global`); este portão tranca os dois GANCHOS do teclado.
#[test]
fn a_buried_purge_sends_ctrl_z_to_the_global_undo() {
    let raiz = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src/input_dispatch");
    let tl = std::fs::read_to_string(raiz.join("keyboard_timeline.rs")).unwrap();
    assert!(
        tl.contains("&& !self.timeline.history.top_is_buried()"),
        "o Ctrl+Z da timeline tem de largar a purga enterrada"
    );
    let mo = std::fs::read_to_string(raiz.join("keyboard_cadeia.rs")).unwrap();
    let braco = mo
        .find("KeyCode::KeyZ | KeyCode::KeyY if self.motion_undo_e_do_global(redo)")
        .expect("o Ctrl+Z do Motion tem de perguntar se é do global");
    assert!(
        mo[braco..]
            .lines()
            .take(4)
            .any(|l| l.contains("self.undo_or_redo(redo)")),
        "e mandá-lo DIRETO ao undo global"
    );
}

/// ⭐ **A edição de etapa que espera fica fora da fotografia do desfazer** (W14,
/// `ph2d_app_macros::espera`) sobre o `ProjectState` de verdade: o `Macro` editado volta ao de antes no que
/// se compara, e o objecto que o artista moveu entretanto fica movido; restaurar um passo põe a edição de
/// volta como está.
#[test]
fn a_edicao_que_espera_fica_fora_da_fotografia_do_desfazer() {
    use ph2d_app_macros::espera::{EdicaoQueEspera, EstadoComMacros};
    let reg = registry();
    let (mut sim, vec) = scene();
    let m = sim
        .world_mut()
        .spawn((
            Transform::default(),
            Name::new("Tela"),
            ph2d_ecs::Macro {
                doc: b"velho".to_vec(),
                modo: ph2d_ecs::ModoDaMacro::Fiel,
            },
        ))
        .id();
    let base = capture(&mut sim, &vec, &reg);
    let id = sim.world().get::<ph2d_ecs::StableId>(m).expect("o id").0;
    let velho = base.macro_de(id).expect("o Macro na fotografia");
    sim.world_mut()
        .get_mut::<ph2d_ecs::Macro>(m)
        .expect("Macro")
        .doc = b"novo".to_vec();
    let mut espera = EdicaoQueEspera::default();
    assert!(!espera.quadro(&[id], Some(&base)), "a espera começa");
    let editado = capture(&mut sim, &vec, &reg);
    assert!(editado != base, "controlo: a edição muda a fotografia");
    assert!(
        espera.a_comparar(editado.clone()) == base,
        "só a edição que espera mudou: nada a registar"
    );
    let mut q = sim
        .world_mut()
        .query_filtered::<&mut Transform, ph2d_ecs::With<Name>>();
    for mut t in q.iter_mut(sim.world_mut()) {
        t.translation.x += 5.0;
    }
    let movido = espera.a_comparar(capture(&mut sim, &vec, &reg));
    assert!(movido != base, "o movimento é um passo");
    assert_eq!(movido.macro_de(id), Some(velho.clone()), "sem a edição");
    let (aplica, novo_base) = espera.restaurado(&base, &editado);
    assert_eq!(aplica.macro_de(id), editado.macro_de(id), "a edição fica");
    assert!(novo_base == base, "o baseline fica sem ela");
}

/// ⭐ **O formato do jogo é um passo, e não pesa no ficheiro** (onda 18): dois estados que só diferem no formato
/// diferem para a fila (o Ctrl+Z desfá-lo), e os bytes gravados são os mesmos — o blob `jogo` continua a casa
/// dele, sem degrau do `PROJECT_SCHEMA`. A prova pela tecla real é a foto `camera_desfaz`.
#[test]
fn o_formato_do_jogo_e_um_passo_e_nao_muda_o_ficheiro() {
    use crate::project_settings::FormatoDoJogo;
    let reg = registry();
    let (mut sim, vec) = scene();
    let mut a = capture(&mut sim, &vec, &reg);
    let mut b = a.clone();
    let mut p = ph2d_editor_core::project::ProjectSettings::default();
    a.formato_do_jogo = Some(FormatoDoJogo::de(&p));
    p.game_format.preset = ph2d_editor_core::GamePreset::GameBoy;
    b.formato_do_jogo = Some(FormatoDoJogo::de(&p));
    assert_ne!(a, b, "a fila não veria a troca de formato");
    assert_eq!(a.parts_that_differ(&b), vec!["formato_do_jogo"]);
    let mut lido = a.clone();
    lido.formato_do_jogo = None;
    assert_eq!(
        postcard::to_allocvec(&a).unwrap(),
        postcard::to_allocvec(&lido).unwrap(),
        "o formato entrou nos bytes do ficheiro"
    );
    let mut volta = ph2d_editor_core::project::ProjectSettings::default();
    b.formato_do_jogo.unwrap().instala(&mut volta);
    assert_eq!(
        volta.game_format.preset,
        ph2d_editor_core::GamePreset::GameBoy
    );
}

/// ⭐ *Com a edição aberta a captura não corre: o objecto de trabalho e a sprite escondida nunca entram num passo.*
/// *Mutação: tirar o braço do `motivo_da_supressao` ⇒ o documento com a origem escondida vira um passo.*
#[test]
fn com_a_imagem_aberta_no_pixel_nada_entra_no_desfazer() {
    use ph2d_sprite_edit::no_pixel;
    let app = crate::App::new();
    assert_eq!(app.motivo_da_supressao(true), None, "sem edição, regista");
    let mut w = ph2d_ecs::World::new();
    let origem = w.spawn(Transform::default()).id();
    let trabalho = w.spawn(Transform::default()).id();
    let imagem = ph2d_asset::AssetDb::default().insert_image_rgba8(1, 1, vec![0; 4]);
    no_pixel::abre(
        &mut w,
        origem,
        trabalho,
        (1, imagem),
        (1, 1),
        100.0,
        ([0.0, 0.0], 1.0),
    );
    let motivo = app.motivo_da_supressao(true);
    no_pixel::esquece();
    assert_eq!(
        motivo,
        Some(ph2d_i18n::tr("shell.undo_app.edicao_de_imagem"))
    );
}
