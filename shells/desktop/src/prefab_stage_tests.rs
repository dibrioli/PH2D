//! Os gates do PALCO da receita.
//!
//! ⚠️ A porta do quadro ([`super::run`]) precisa de uma `HeroScreen` com store — o que se prende
//! aqui é a **lei** (pura) e o **contrato com o ledger**, que é onde o defeito caro nasce: uma pose
//! de palco que escape para o documento é um passo de undo e bytes no ficheiro por um gesto de
//! *ver*.

use super::delta_to_centre;
use ph2d_ecs::{SimWorld, Transform};
use ph2d_editor::zones::Rect;
use ph2d_host::WindowSize;
use ph2d_preview_drive::{Driven, PreviewDrive};
use ph2d_render::Camera2d;

const WINDOW: WindowSize = WindowSize::new(1600, 900);

/// A janela com uma coluna docada de `400 px` à esquerda — a área visível é o resto.
///
/// ⚠️ **É a fixtura do REPORT**: com as duas colunas iguais o defeito seria invisível (o centro da
/// área e o da janela coincidem), e o gate ficaria verde sobre o que o Enio viu.
const AREA: Rect = Rect {
    x: 400.0,
    y: 0.0,
    w: 1200.0,
    h: 900.0,
};

/// ⭐⭐⭐ **A receita aterra no centro da ÁREA VISÍVEL, e a CÂMERA não se mexe.**
///
/// Com uma coluna docada de 400 px, os dois centros estão a **200 px** um do outro — e a diferença
/// é exactamente a metade da coluna, que é o que põe a receita debaixo dela.
///
/// **Mutação que deve sangrar:** o `usable` devolver sempre a janela, ou o alvo passar a ser o
/// centro do ecrã.
#[test]
fn the_recipe_lands_at_the_centre_of_the_visible_area_not_the_window() {
    let cam = Camera2d::new([0.0, 0.0], 10.0);
    // Uma receita pequena, longe da vista — ela é que tem de vir.
    let (min, max) = ([20.0, -8.0], [21.0, -7.0]);
    let d = delta_to_centre(&cam, WINDOW, AREA, min, max);
    let centre = [
        (min[0] + max[0]) * 0.5 + d[0],
        (min[1] + max[1]) * 0.5 + d[1],
    ];
    let (px, py) = cam.world_to_screen(centre, WINDOW);
    let want = (AREA.x + AREA.w * 0.5, AREA.y + AREA.h * 0.5);
    assert!(
        (px - want.0).abs() < 0.5 && (py - want.1).abs() < 0.5,
        "a receita caiu em ({px:.1}, {py:.1}) e o centro da area visivel e' ({:.1}, {:.1}) — \
         a janela teria dito ({:.1}, {:.1})",
        want.0,
        want.1,
        WINDOW.width as f32 * 0.5,
        WINDOW.height as f32 * 0.5,
    );
}

/// ⚠️ **Uma receita que já está no centro não se mexe** — abrir o que já está à vista não pode dar
/// um salto de um pixel que seja.
#[test]
fn a_recipe_already_at_the_centre_does_not_move() {
    let cam = Camera2d::new([0.0, 0.0], 10.0);
    let centre = cam.screen_to_world((AREA.x + AREA.w * 0.5, AREA.y + AREA.h * 0.5), WINDOW);
    let min = [centre[0] - 0.5, centre[1] - 0.5];
    let max = [centre[0] + 0.5, centre[1] + 0.5];
    let d = delta_to_centre(&cam, WINDOW, AREA, min, max);
    assert!(
        d[0].abs() < 1e-5 && d[1].abs() < 1e-5,
        "a receita ja' centrada saltou {d:?}"
    );
}

/// ⛔ **Uma área ainda não publicada não produz um deslocamento absurdo.** O primeiro quadro publica
/// `0 x 0` (o painel ainda não pintou), e centrar numa caixa de lado zero poria a receita no canto.
#[test]
fn an_area_that_was_not_published_yet_falls_back_to_the_window() {
    let cam = Camera2d::new([0.0, 0.0], 10.0);
    let d = delta_to_centre(
        &cam,
        WINDOW,
        Rect::new(0.0, 0.0, 0.0, 0.0),
        [4.0, 4.0],
        [5.0, 5.0],
    );
    let centre = [4.5 + d[0], 4.5 + d[1]];
    let (px, py) = cam.world_to_screen(centre, WINDOW);
    assert!(
        (px - 800.0).abs() < 0.5 && (py - 450.0).abs() < 0.5,
        "sem area publicada o centro tem de ser o da JANELA: ({px:.1}, {py:.1})"
    );
}

/// ⭐⭐⭐ **A pose de PALCO é pré-visualização: a captura vê o BASTIDOR.**
///
/// É a metade que impede o palco de ser uma edição — sem ela, abrir uma receita empilha um passo de
/// undo e escreve uma posição nova no ficheiro por um gesto de *ver*.
///
/// **Mutação que deve sangrar:** o braço `StagePose` do `Driven::read`/`write`, ou a declaração no
/// [`super::run`] (que o censo irmão prende).
#[test]
fn a_staged_pose_is_preview_and_the_capture_sees_the_backstage_one() {
    let mut sim = SimWorld::new();
    let e = sim.world_mut().spawn((Transform::default(),)).id();
    let authored = *sim.world().get::<Transform>(e).unwrap();
    let mut placed = authored;
    placed.translation.x = 42.0;
    Driven::StagePose(placed).write(&mut sim, e);
    let mut drive = PreviewDrive::default();
    drive.driven(e, Driven::StagePose(authored), Driven::StagePose(placed));

    assert_eq!(
        sim.world().get::<Transform>(e).unwrap().translation.x,
        42.0,
        "a receita nao foi para o palco — o artista nao a veria"
    );
    let live = drive.substitute_authored(&mut sim);
    assert_eq!(
        sim.world().get::<Transform>(e).unwrap().translation.x,
        0.0,
        "a captura viu a pose de PALCO — ela viraria um passo de undo e bytes no ficheiro"
    );
    PreviewDrive::restore_live(&mut sim, &live);
    assert_eq!(
        sim.world().get::<Transform>(e).unwrap().translation.x,
        42.0,
        "a captura nao devolveu o vivo — a receita saltaria de volta a meio da edicao"
    );
}

/// ⭐⭐ **O palco e o SOLVER conduzem a mesma entidade sem se apagarem.**
///
/// Uma receita cuja raiz seja também um corpo dinâmico é conduzida pelos dois. ⚠️ A chave do ledger
/// é `(entidade, driver)` — reusar o driver do solver faria o `authored` de um apagar o do outro, e
/// o sintoma seria a receita voltar do palco para a pose da simulação.
#[test]
fn the_stage_and_the_solver_are_two_entries_not_one() {
    let mut sim = SimWorld::new();
    let e = sim.world_mut().spawn((Transform::default(),)).id();
    let a = *sim.world().get::<Transform>(e).unwrap();
    let mut b = a;
    b.translation.x = 1.0;
    let mut drive = PreviewDrive::default();
    drive.driven(e, Driven::SolverPose(a), Driven::SolverPose(b));
    drive.driven(e, Driven::StagePose(b), Driven::StagePose(b));
    assert_eq!(
        drive.len(),
        2,
        "o palco e o solver colidiram numa entrada so'"
    );
}

/// Uma receita ABERTA (a marca é a mesma que o quadro carimba) com a raiz já no palco.
fn on_stage() -> (SimWorld, super::Stage, Transform, Transform) {
    let mut sim = SimWorld::new();
    let root = sim
        .world_mut()
        .spawn((
            Transform::default(),
            ph2d_ecs::MasterRoot,
            ph2d_ecs::MasterEditing,
            ph2d_ecs::StableId(7777),
        ))
        .id();
    let authored = *sim.world().get::<Transform>(root).unwrap();
    let mut placed = authored;
    placed.translation.x = 42.0;
    Driven::StagePose(placed).write(&mut sim, root);
    let stage = super::Stage {
        id: 7777,
        root: root.to_bits(),
        authored,
        last_written: placed,
    };
    (sim, stage, authored, placed)
}

/// ⭐⭐⭐ **SEGURAR é obrigatório em todo quadro** — sem isso a `settle` esquece a condução e a pose
/// de palco vira DOCUMENTO no quadro seguinte (um passo de undo e bytes no ficheiro por um gesto de
/// *ver*).
///
/// **Mutação que deve sangrar:** apagar o `drive.driven(...)` do [`super::hold`].
#[test]
fn a_recipe_on_stage_stays_preview_frame_after_frame() {
    let (mut sim, stage, authored, placed) = on_stage();
    let mut st = Some(stage);
    let mut drive = PreviewDrive::default();
    // O quadro da abertura declarou; daí em diante é o `hold` que segura.
    drive.driven(
        ph2d_ecs::Entity::from_bits(stage.root),
        Driven::StagePose(authored),
        Driven::StagePose(placed),
    );
    for quadro in 0..5 {
        drive.settle();
        super::hold(&mut st, &mut sim, &mut drive);
        assert!(
            !drive.is_empty(),
            "a conducao morreu no quadro {quadro} — a pose de palco vira documento"
        );
    }
    // E o que a captura vê continua a ser o bastidor.
    let live = drive.substitute_authored(&mut sim);
    let e = ph2d_ecs::Entity::from_bits(stage.root);
    assert_eq!(
        sim.world().get::<Transform>(e).unwrap().translation.x,
        authored.translation.x,
        "cinco quadros depois, a captura ja' via a pose de PALCO"
    );
    PreviewDrive::restore_live(&mut sim, &live);
}

/// ⭐⭐⭐ **FECHAR devolve a receita ao bastidor.**
///
/// ⚠️ Sem isto ela fica onde o palco a deixou — e como a `settle` a esquece no quadro seguinte,
/// essa posição vira o documento: abrir uma receita passaria a MOVÊ-LA para sempre.
///
/// **Mutação que deve sangrar:** apagar o `write` do braço de fecho do [`super::hold`].
#[test]
fn closing_the_recipe_returns_it_to_the_backstage_pose() {
    let (mut sim, stage, authored, _) = on_stage();
    let mut st = Some(stage);
    let mut drive = PreviewDrive::default();
    let e = ph2d_ecs::Entity::from_bits(stage.root);
    // O gesto: a selecção mudou, e o carimbo do quadro tirou a marca.
    sim.world_mut()
        .entity_mut(e)
        .remove::<ph2d_ecs::MasterEditing>();
    super::hold(&mut st, &mut sim, &mut drive);
    assert!(st.is_none(), "o palco nao desmontou");
    assert_eq!(
        sim.world().get::<Transform>(e).unwrap().translation.x,
        authored.translation.x,
        "a receita ficou onde o palco a deixou — no quadro seguinte isso vira o documento"
    );
}

/// ⚠️ **O artista pode ARRASTAR a receita pelo palco, e o bastidor não muda.**
///
/// O ledger tem uma lei de *outra mão* (uma escrita estrangeira passa a ser o documento). Aqui ela
/// **não** se aplica, e é deliberado: o `before` que o palco declara é o que ELE escreveu, então o
/// arrasto entra como pré-visualização — a receita volta ao bastidor ao fechar. *A posição de mundo
/// de uma receita nunca foi uma escolha visível.*
#[test]
fn dragging_the_recipe_on_stage_does_not_rewrite_the_backstage_pose() {
    let (mut sim, stage, authored, _) = on_stage();
    let mut st = Some(stage);
    let mut drive = PreviewDrive::default();
    let e = ph2d_ecs::Entity::from_bits(stage.root);
    drive.driven(
        e,
        Driven::StagePose(authored),
        Driven::StagePose(*sim.world().get::<Transform>(e).unwrap()),
    );
    // A mão do artista: arrastar a receita 10 unidades no palco.
    if let Some(mut t) = sim.world_mut().get_mut::<Transform>(e) {
        t.translation.x += 10.0;
    }
    super::hold(&mut st, &mut sim, &mut drive);
    let live = drive.substitute_authored(&mut sim);
    assert_eq!(
        sim.world().get::<Transform>(e).unwrap().translation.x,
        authored.translation.x,
        "o arrasto no palco virou documento — abrir e mexer moveria a receita para sempre"
    );
    PreviewDrive::restore_live(&mut sim, &live);
}

/// ⭐⭐⭐ **O PALCO REMONTA-SE depois de um `Ctrl+Z`.**
///
/// O restauro traz a receita numa entidade NOVA e com a pose de **bastidor** (a captura substitui a
/// pré-visualização pelo autorado). ⛔ Sem esta metade a receita saltava para fora do ecrã a meio da
/// sessão — e, pior, o passo seguinte gravava a pose de palco como documento, porque a entrada do
/// ledger tinha ficado presa a uma entidade morta.
///
/// **Mutação que deve sangrar:** o braço do `bits != st.root` no [`super::hold`].
#[test]
fn the_stage_reassembles_itself_after_an_undo_step() {
    let (mut sim, stage, authored, placed) = on_stage();
    let mut st = Some(stage);
    let mut drive = PreviewDrive::default();
    let old = ph2d_ecs::Entity::from_bits(stage.root);
    drive.driven(old, Driven::StagePose(authored), Driven::StagePose(placed));
    // O `Ctrl+Z`: entidade nova, MESMO `StableId`, pose de bastidor.
    sim.world_mut().entity_mut(old).despawn();
    let novo = sim
        .world_mut()
        .spawn((
            authored,
            ph2d_ecs::MasterRoot,
            ph2d_ecs::MasterEditing,
            ph2d_ecs::StableId(7777),
        ))
        .id();

    super::hold(&mut st, &mut sim, &mut drive);
    assert!(st.is_some(), "o palco desmontou-se ao desfazer");
    assert_eq!(
        sim.world().get::<Transform>(novo).unwrap().translation.x,
        placed.translation.x,
        "a receita saltou para o bastidor a meio da sessao"
    );
    // E continua a ser pré-visualização: a captura vê o bastidor.
    let live = drive.substitute_authored(&mut sim);
    assert_eq!(
        sim.world().get::<Transform>(novo).unwrap().translation.x,
        authored.translation.x,
        "depois do restauro a pose de palco virou documento"
    );
    PreviewDrive::restore_live(&mut sim, &live);
}
