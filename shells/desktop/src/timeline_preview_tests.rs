//! **O TERCEIRO membro da família pré-visualização↔documento** — a timeline.
//!
//! ⚠️ A auditoria [21 §4](../../../docs/Sprite_projeto/21_auditoria_da_animacao_2026-08-23.md)
//! curou dois casos (a §11 Animation e o solver) e **mediu** um terceiro: enquanto o playhead toca,
//! a timeline escreve `Transform` a partir das curvas, e um clique naquele quadro empilha um passo
//! de undo cujo conteúdo é só a pose que o documento já sabe calcular.
//!
//! ⚠️ **E a nota que a deixou de fora estava ERRADA num ponto que decide o preço.** Ela dizia que a
//! alternativa do lado do shell era *«um censo de TODAS as poses por quadro de reprodução, e esse
//! custo é um número que ninguém mediu»*. Não é: o `TimelineDoc` **nomeia** as entidades que anima
//! (`doc.bindings()`), então o censo é `O(bindings)` — a dúzia de objetos que o artista keyou — e
//! não `O(mundo)`. *Uma ausência afirmada sem olhar a API é um palpite com cara de medição.*

use crate::preview_drive::PreviewDrive;
use crate::undo::ProjectState;
use ph2d_anim::{AnimValue, Interp, RationalTime};
use ph2d_ecs::scene::{ComponentRegistry, register_ecs_components};
use ph2d_ecs::{Name, SimWorld, Transform};
use ph2d_timeline::{PropKind, TimelineDoc};

fn registry() -> ComponentRegistry {
    let mut reg = ComponentRegistry::new();
    register_ecs_components(&mut reg);
    ph2d_render::register_render_components(&mut reg);
    reg
}

fn capture(drive: &PreviewDrive, sim: &mut SimWorld, reg: &ComponentRegistry) -> ProjectState {
    ProjectState::capture(
        drive,
        sim,
        &ph2d_vec_scene::VecScene::new(),
        &ph2d_flip::FlipDoc::new(),
        &ph2d_guides::GuideSet::default(),
        &ph2d_ui_state::StateSets::default(),
        &crate::project_library::LibraryDoc::default(),
        reg,
        &mut ph2d_ecs::scene::incremental::CaptureCache::new(),
    )
}

/// Um objeto keyado: `x` vai de 0 a 10 em 4 s.
fn rig() -> (SimWorld, ph2d_ecs::Entity, TimelineDoc) {
    let mut sim = SimWorld::new();
    let e = sim
        .world_mut()
        .spawn((Transform::default(), Name::new("Keyed")))
        .id();
    let mut doc = TimelineDoc::new();
    for (t, v) in [(0.0, 0.0_f32), (4.0, 10.0)] {
        doc.insert_key(
            e.to_bits(),
            PropKind::TranslationX,
            RationalTime::from_seconds(t),
            AnimValue::Float(v),
            Interp::Linear,
        );
    }
    (sim, e, doc)
}

/// **UM QUADRO DA TIMELINE A TOCAR NÃO É UMA EDIÇÃO** — o terceiro caso da família, curado pelo
/// MESMO ledger que serve a §11 e o solver.
///
/// ⚠️ **Ele corre o `apply` REAL do `ph2d-timeline`**, que é o produtor do produto — o
/// `timeline_bridge::run` é que não é alcançável (nove argumentos de estado do shell), e o que ele
/// faz de relevante para este gate é chamar esta função.
///
/// **Mutação que deve sangrar:** tirar o `declare_timeline_writes`.
#[test]
fn a_playing_timeline_is_not_an_edit() {
    let reg = registry();
    let (mut sim, e, mut doc) = rig();
    let mut drive = PreviewDrive::default();
    let at_rest = capture(&drive, &mut sim, &reg);

    let mut t = 0.0;
    for _ in 0..24 {
        t += 1.0 / 60.0;
        let before = crate::timeline_preview::state_of_bindings(sim.world(), &doc);
        ph2d_timeline::apply_from_doc(sim.world_mut(), &mut doc, t);
        crate::timeline_preview::declare_timeline_writes(sim.world(), &before, &mut drive);
        drive.settle();
        assert_eq!(
            capture(&drive, &mut sim, &reg),
            at_rest,
            "a curva a tocar mudou a captura — cada clique viraria um passo de Ctrl+Z vazio"
        );
    }
    // CONTROLO POSITIVO: o objeto ANDOU de facto.
    assert!(
        sim.world().get::<Transform>(e).unwrap().translation.x > 0.9,
        "a curva nao moveu o objeto — o gate acima nao mediu nada"
    );

    // E PARAR deixa UM passo: a pose passa a ser documento, e o Ctrl+Z desfaz a reprodução.
    drive.settle();
    assert_ne!(
        capture(&drive, &mut sim, &reg),
        at_rest,
        "parar a reproducao tem de deixar UM passo"
    );
}

/// **O CENSO É `O(bindings)`, e não `O(mundo)`** — a medição que desbloqueou esta wave.
///
/// ⛔ A nota que deixou a timeline de fora dizia que o custo era desconhecido porque a alternativa
/// era varrer todas as poses. O documento **nomeia** quem ele anima.
///
/// **Mutação que deve sangrar:** o censo varrer o mundo em vez das bindings.
#[test]
fn the_census_only_looks_at_what_the_document_animates() {
    let (mut sim, _e, doc) = rig();
    // Mais cem objetos que a timeline não conhece.
    for i in 0..100 {
        sim.world_mut()
            .spawn((Transform::default(), Name::new(format!("Extra {i}"))));
    }
    let poses = crate::timeline_preview::state_of_bindings(sim.world(), &doc);
    assert_eq!(
        poses.len(),
        1,
        "o censo apanhou {} entidades num mundo de 101 — ele tem de olhar so' as keyadas",
        poses.len()
    );
}

/// **Uma entidade com VÁRIAS props keyadas entra UMA vez** — `TranslationX` e `TranslationY` são
/// duas bindings do mesmo objeto, e declarar a mesma pose duas vezes é trabalho a dobrar por nada.
#[test]
fn an_entity_keyed_on_two_axes_is_censused_once() {
    let (sim, e, mut doc) = rig();
    doc.insert_key(
        e.to_bits(),
        PropKind::TranslationY,
        RationalTime::from_seconds(0.0),
        AnimValue::Float(0.0),
        Interp::Linear,
    );
    assert!(
        doc.bindings().len() >= 2,
        "a fixtura precisa de duas bindings"
    );
    assert_eq!(
        crate::timeline_preview::state_of_bindings(sim.world(), &doc).len(),
        1
    );
}

/// **Uma binding cuja entidade já não existe não derruba o censo** — um objeto apagado deixa a
/// binding pendurada, e o documento mostra-a a vermelho de propósito (a §12 tomou a mesma decisão).
#[test]
fn a_dangling_binding_does_not_break_the_census() {
    let (mut sim, e, doc) = rig();
    sim.world_mut().despawn(e);
    assert!(crate::timeline_preview::state_of_bindings(sim.world(), &doc).is_empty());
}

/// **UMA CURVA DE OPACIDADE TAMBÉM NÃO É UMA EDIÇÃO** — o caso que a 1.ª versão desta wave deixou
/// de fora, e que uma animação de *fade* produz o tempo todo.
///
/// ⚠️ **A colisão que o justificava DISSOLVEU-SE, e a cura foi olhar o que a curva escreve:**
/// `sprite.tint[3] = f` é **um número**, não o `Sprite` inteiro. O facto do ledger passou a ser tão
/// estreito quanto a escrita, e deixou de disputar o componente com o `frame` da §11. *Quando duas
/// granularidades colidem, a pergunta é qual delas é grosseira demais para o que o motor faz.*
///
/// **Mutação que deve sangrar:** tirar o braço do `alpha` do `declare_timeline_writes`.
#[test]
fn a_fading_opacity_is_not_an_edit_either() {
    let reg = registry();
    let mut sim = SimWorld::new();
    let e = sim
        .world_mut()
        .spawn((
            Transform::default(),
            Name::new("Fading"),
            ph2d_render::Sprite::atlas(0, [1.0, 1.0], [1.0; 4]),
        ))
        .id();
    let mut doc = TimelineDoc::new();
    for (t, v) in [(0.0, 1.0_f32), (2.0, 0.0)] {
        doc.insert_key(
            e.to_bits(),
            PropKind::Opacity,
            RationalTime::from_seconds(t),
            AnimValue::Float(v),
            Interp::Linear,
        );
    }
    let mut drive = PreviewDrive::default();
    let at_rest = capture(&drive, &mut sim, &reg);
    let mut t = 0.0;
    for _ in 0..24 {
        t += 1.0 / 60.0;
        let before = crate::timeline_preview::state_of_bindings(sim.world(), &doc);
        ph2d_timeline::apply_from_doc(sim.world_mut(), &mut doc, t);
        crate::timeline_preview::declare_timeline_writes(sim.world(), &before, &mut drive);
        drive.settle();
        assert_eq!(
            capture(&drive, &mut sim, &reg),
            at_rest,
            "o fade mudou a captura — cada clique viraria um passo de Ctrl+Z vazio"
        );
    }
    // CONTROLO POSITIVO: a opacidade DESCEU de facto.
    assert!(
        sim.world().get::<ph2d_render::Sprite>(e).unwrap().tint[3] < 0.9,
        "a curva nao mexeu na opacidade — o gate acima nao mediu nada"
    );
}

/// **A OPACIDADE E O FRAME COEXISTEM na mesma sprite** — e é isto que a 1.ª versão não conseguia.
///
/// ⚠️ Duas conduções sobre o **mesmo componente**, com granularidades diferentes: a §11 possui o
/// `frame`, a timeline possui o `tint[3]`. Elas só coexistem porque **nenhuma das duas escreve o
/// `Sprite` inteiro** — se uma o fizesse, a substituição escreveria por cima da outra e a que
/// corresse por último ganhava.
///
/// **Mutação que deve sangrar:** o `SpriteAlpha::write` escrever o `tint` inteiro é benigno, mas
/// fazê-lo escrever também o `frame` (o componente todo) parte este gate.
#[test]
fn the_frame_and_the_alpha_are_driven_side_by_side() {
    use crate::preview_drive::Driven;
    let mut sim = SimWorld::new();
    let e = sim
        .world_mut()
        .spawn((
            Transform::default(),
            Name::new("Both"),
            ph2d_render::Sprite::atlas(0, [1.0, 1.0], [1.0; 4]),
            // ⭐ A grelha é um componente (ADR-0164 F1 passo 6) — e o facto `SpriteAnim` lê-se
            // hoje do par (`SpriteAnimator` + `SpriteGrid`), porque a célula viva mora nela.
            ph2d_ecs::SpriteGrid {
                hframes: 4,
                vframes: 1,
                frame: 0,
            },
            // ⚠️ **O animador faz parte da fixtura**, e o gate ensinou-o: o facto `SpriteAnim` é
            // lido do PAR (`SpriteAnimator` + `Sprite`), então uma sprite sem tocador não é
            // conduzida pela §11 — a substituição salta-a, e o gate reprova a dizer que o
            // documento não guardou a célula. *Uma fixtura só prova o que ela contém.*
            ph2d_ecs::SpriteAnimator::new("walk"),
        ))
        .id();
    let mut drive = PreviewDrive::default();
    // A §11 avançou o frame 0 → 2; a timeline baixou o alfa 1,0 → 0,4.
    drive.driven(
        e,
        Driven::SpriteAnim {
            elapsed_ticks: 0,
            pingpong_reverse: false,
            repeat_count: 0,
            frame: 0,
        },
        Driven::SpriteAnim {
            elapsed_ticks: 100,
            pingpong_reverse: false,
            repeat_count: 0,
            frame: 2,
        },
    );
    drive.driven(e, Driven::SpriteAlpha(1.0), Driven::SpriteAlpha(0.4));
    if let Some(mut g) = sim.world_mut().get_mut::<ph2d_ecs::SpriteGrid>(e) {
        g.frame = 2;
    }
    if let Some(mut s) = sim.world_mut().get_mut::<ph2d_render::Sprite>(e) {
        s.tint[3] = 0.4;
    }
    let live = drive.substitute_authored(&mut sim);
    let doc_sprite = *sim.world().get::<ph2d_render::Sprite>(e).unwrap();
    let doc_grid = *sim.world().get::<ph2d_ecs::SpriteGrid>(e).unwrap();
    assert_eq!(doc_grid.frame, 0, "o documento guarda a celula autorada");
    assert_eq!(doc_sprite.tint[3], 1.0, "…E a opacidade autorada, as DUAS");
    PreviewDrive::restore_live(&mut sim, &live);
    let back = *sim.world().get::<ph2d_render::Sprite>(e).unwrap();
    let back_grid = *sim.world().get::<ph2d_ecs::SpriteGrid>(e).unwrap();
    assert_eq!(
        (back_grid.frame, back.tint[3]),
        (2, 0.4),
        "e o vivo volta inteiro"
    );
}
