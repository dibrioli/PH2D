//! **Os gates de PRÉ-VISUALIZAÇÃO contra DOCUMENTO** ([`super`]).
//!
//! ⚠️ **Todos nasceram do mesmo pedido** (Enio, 2026-08-23: *«precisamos corrigir o CtrlZ para
//! ambas»*), e a auditoria
//! [`21 §4`](../../../docs/Sprite_projeto/21_auditoria_da_animacao_2026-08-23.md) mediu porquê: o
//! passo de undo nasce **por clique**, não por quadro, e o conteúdo dele era só o relógio.
//!
//! ⚠️ **Cada gate tem CONTROLO POSITIVO.** Um gate que afirma *«a captura não mudou»* fica verde
//! sozinho se nada tiver acontecido — por isso todos eles asseguram primeiro que o mundo VIVO de
//! facto andou. *Um gate sem a metade que prova que havia fenómeno mede o instrumento.*

use super::{Driven, PreviewDrive};
use crate::undo::ProjectState;
use ph2d_ecs::scene::{ComponentRegistry, register_ecs_components};
use ph2d_ecs::{AnimationTag, Entity, Name, SimWorld, SpriteAnimations, SpriteAnimator, Transform};
use ph2d_render::{Sprite, register_render_components};

fn registry() -> ComponentRegistry {
    let mut reg = ComponentRegistry::new();
    register_ecs_components(&mut reg);
    register_render_components(&mut reg);
    reg
}

/// O que o `post_frame_undo` fotografa — a MESMA porta do produto, com o ledger no 1.º argumento.
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
        None,
    )
}

/// Uma sprite com grelha de 8 células e uma animação `walk` que as percorre, a tocar.
fn playing_sprite(sim: &mut SimWorld) -> Entity {
    let mut lib = SpriteAnimations::new();
    lib.insert(AnimationTag::new("walk", 0, 7)).unwrap();
    let mut animator = SpriteAnimator::new("walk");
    animator.playing = true;
    sim.world_mut()
        .spawn((
            Transform::default(),
            Name::new("Hero"),
            Sprite::atlas(0, [1.0, 1.0], [1.0; 4]),
            // ⭐ A grelha é um componente (ADR-0164 F1 passo 6) — e sem ela o tique nem entra
            // no laço, porque uma sprite sem células não tem o que animar.
            ph2d_ecs::SpriteGrid {
                hframes: 8,
                vframes: 1,
                frame: 0,
            },
            animator,
            lib,
        ))
        .id()
}

/// Um quadro do app, na ordem em que ele corre: o tique declara, e o `post_frame_undo` faz a
/// `settle` **antes** de fotografar.
fn frame(sim: &mut SimWorld, drive: &mut PreviewDrive, ticks: u32) {
    crate::render_loop::sprite_anim_tick::tick_sprite_animations(
        sim,
        ticks,
        1.0 / 60.0,
        &[],
        drive,
    );
    drive.settle();
}

/// O que o DOCUMENTO diz desta sprite — o mundo posto no autorado, lido, e devolvido ao vivo.
fn document(drive: &PreviewDrive, sim: &mut SimWorld, e: Entity) -> (u32, u64) {
    let live = drive.substitute_authored(sim);
    let f = sim
        .world()
        .get::<ph2d_ecs::SpriteGrid>(e)
        .map_or(u32::MAX, |g| g.frame);
    let t = sim
        .world()
        .get::<SpriteAnimator>(e)
        .map_or(u64::MAX, |a| a.elapsed_ticks);
    PreviewDrive::restore_live(sim, &live);
    (f, t)
}

/// O que o MUNDO VIVO diz — o que o artista vê.
fn live(sim: &SimWorld, e: Entity) -> (u32, u64) {
    (
        sim.world().get::<ph2d_ecs::SpriteGrid>(e).unwrap().frame,
        sim.world().get::<SpriteAnimator>(e).unwrap().elapsed_ticks,
    )
}

/// **UMA ANIMAÇÃO A TOCAR NÃO É UMA EDIÇÃO** — o defeito reportado, na sua forma mais curta.
///
/// A captura é a unidade do undo: se ela muda, um clique naquele quadro vira um passo. Aqui o
/// relógio anda 30 vezes e a célula percorre a tira inteira, e a fotografia do documento é a
/// MESMA — byte a byte.
///
/// **Mutação que deve sangrar:** tirar o `drive.driven(...)` do fim do `tick_sprite_animations`.
#[test]
fn a_playing_animation_is_not_an_edit() {
    let reg = registry();
    let mut sim = SimWorld::new();
    let e = playing_sprite(&mut sim);
    let mut drive = PreviewDrive::default();

    let before = capture(&drive, &mut sim, &reg);
    let mut cells_seen = std::collections::BTreeSet::new();
    for _ in 0..30 {
        frame(&mut sim, &mut drive, 4);
        cells_seen.insert(live(&sim, e).0);
        assert_eq!(
            capture(&drive, &mut sim, &reg),
            before,
            "o relogio a andar mudou a captura — cada clique viraria um passo de Ctrl+Z vazio"
        );
    }
    // CONTROLO POSITIVO, as duas metades: o relogio andou E a celula mudou de facto.
    assert!(
        live(&sim, e).1 > 0,
        "o relogio nao andou — o gate acima nao mediu nada"
    );
    assert!(
        cells_seen.len() > 1,
        "a celula nunca mudou ({cells_seen:?}) — o gate acima nao mediu o indice"
    );
}

/// **E O QUE ELA ESCREVE VÊ-SE.** A metade oposta da mesma lei: o documento não muda, mas o MUNDO
/// muda — senão a cura teria simplesmente parado a animação.
///
/// **Mutação que deve sangrar:** fazer a `substitute_authored` esquecer-se de repor o vivo
/// (`restore_live` no-op) — a sprite congelaria na célula em que a primeira captura a apanhou.
#[test]
fn the_document_stands_still_and_the_picture_does_not() {
    let mut sim = SimWorld::new();
    let e = playing_sprite(&mut sim);
    let mut drive = PreviewDrive::default();
    for _ in 0..20 {
        frame(&mut sim, &mut drive, 4);
    }
    let doc = document(&drive, &mut sim, e);
    let seen = live(&sim, e);
    assert_eq!(doc.0, 0, "o documento tem de ficar na celula autorada");
    assert_eq!(doc.1, 0, "e no relogio autorado");
    assert!(
        seen.1 > 0,
        "o mundo vivo tem de ter andado — a cura nao pode ser parar a animacao"
    );
}

/// **REPOR O AUTORADO E DEVOLVER O VIVO NÃO DEIXA MARCA.** A fotografia mexe no mundo por um
/// instante; se ela deixasse alguma coisa para trás, o artista veria a animação saltar de volta.
///
/// **Mutação que deve sangrar:** `restore_live` reescrever só o relógio e não o `frame`.
#[test]
fn the_capture_gives_the_world_back_exactly_as_it_found_it() {
    let reg = registry();
    let mut sim = SimWorld::new();
    let e = playing_sprite(&mut sim);
    let mut drive = PreviewDrive::default();
    for _ in 0..7 {
        frame(&mut sim, &mut drive, 4);
    }
    let before = live(&sim, e);
    let _ = capture(&drive, &mut sim, &reg);
    let _ = document(&drive, &mut sim, e);
    assert_eq!(
        live(&sim, e),
        before,
        "a captura deixou o mundo noutro sitio"
    );
}

/// **MAS ESCOLHER UMA CÉLULA À MÃO É UMA EDIÇÃO** — e o caminho é o do produto: arrastar a barra
/// de frames **pausa** (§11, F12), o motor larga a sprite, e a `settle` devolve o vivo ao documento
/// no mesmo quadro.
///
/// ⛔ Sem esta metade a cura seria pior que o defeito: o undo deixaria de registar o que o artista
/// faz, que é exactamente a saída (suprimir enquanto toca) que a auditoria mediu e recusou.
///
/// **Mutação que deve sangrar:** a `settle` reter tudo (`retain(|_, _| true)`).
#[test]
fn scrubbing_the_frame_bar_is_an_edit() {
    let reg = registry();
    let mut sim = SimWorld::new();
    let e = playing_sprite(&mut sim);
    let mut drive = PreviewDrive::default();
    for _ in 0..5 {
        frame(&mut sim, &mut drive, 4);
    }
    let playing_doc = capture(&drive, &mut sim, &reg);

    // O gesto: a barra pausa e escreve a celula (o que o `AnimFieldEdit::SetFrame` faz).
    if let Some(mut a) = sim.world_mut().get_mut::<SpriteAnimator>(e) {
        a.playing = false;
        a.elapsed_ticks = 0;
    }
    if let Some(mut s) = sim.world_mut().get_mut::<ph2d_ecs::SpriteGrid>(e) {
        s.frame = 5;
    }
    frame(&mut sim, &mut drive, 4); // o tique ja' nao declara: parada e sem ferramenta

    assert_ne!(
        capture(&drive, &mut sim, &reg),
        playing_doc,
        "arrastar a barra tem de ser um passo de undo"
    );
    assert_eq!(
        document(&drive, &mut sim, e).0,
        5,
        "e o documento tem de guardar a celula escolhida"
    );
}

/// **MEXER NUM PARÂMETRO DURANTE A REPRODUÇÃO TAMBÉM É UMA EDIÇÃO.** É a razão de a granularidade
/// ser o CAMPO e não o componente: o `SpriteAnimator` guarda o relógio ao lado da velocidade, e
/// repor o componente inteiro engoliria este gesto em silêncio.
///
/// **Mutação que deve sangrar:** a `Driven::write` do braço `SpriteAnim` repor também o
/// `speed_q16`.
#[test]
fn changing_a_param_while_it_plays_is_still_an_edit() {
    let reg = registry();
    let mut sim = SimWorld::new();
    let e = playing_sprite(&mut sim);
    let mut drive = PreviewDrive::default();
    for _ in 0..5 {
        frame(&mut sim, &mut drive, 4);
    }
    let before = capture(&drive, &mut sim, &reg);
    if let Some(mut a) = sim.world_mut().get_mut::<SpriteAnimator>(e) {
        a.speed_q16 = 1 << 15; // meia velocidade
    }
    frame(&mut sim, &mut drive, 4);
    assert_ne!(
        capture(&drive, &mut sim, &reg),
        before,
        "a velocidade e' autoria: mexer nela durante a reproducao tem de dar um passo"
    );
}

/// **OUTRA MÃO A ESCREVER ADOPTA O AUTORADO.** Se o que o motor encontra não é o que ele deixou,
/// alguém autorou por cima entre os dois quadros — e o documento passa a ser essa mão.
///
/// ⚠️ Sem esta lei, uma edição feita a meio de uma corrida ficaria **para sempre** por baixo do
/// memo tirado no início dela: o artista escreveria, e o valor voltaria sozinho ao antigo.
///
/// **Mutação que deve sangrar:** tirar o `if e.last_written != before { e.authored = before; }`.
#[test]
fn another_hand_writing_takes_over_the_authored_value() {
    let mut sim = SimWorld::new();
    let e = playing_sprite(&mut sim);
    let mut drive = PreviewDrive::default();
    for _ in 0..5 {
        frame(&mut sim, &mut drive, 4);
    }
    assert_eq!(
        document(&drive, &mut sim, e).0,
        0,
        "o autorado ainda e' o do inicio da corrida"
    );
    // A outra mão, sem pausar: escreve a célula e deixa a animação a tocar.
    if let Some(mut s) = sim.world_mut().get_mut::<ph2d_ecs::SpriteGrid>(e) {
        s.frame = 6;
    }
    frame(&mut sim, &mut drive, 1); // o tique volta a conduzir, e VÊ a mão alheia
    assert_eq!(
        document(&drive, &mut sim, e).0,
        6,
        "o documento tinha de adoptar o que a outra mao escreveu"
    );
}

/// **QUANDO O MOTOR LARGA, A CORRIDA INTEIRA VIRA UM PASSO.** É o passo que faz sentido pedir —
/// *«desfaz a corrida»* —, e é a `settle` que o produz: sem ela, parar não mudaria nada e a corrida
/// seria indesfazível.
///
/// **Mutação que deve sangrar:** a `settle` não limpar o `seen` (`for … { e.seen = false }` fora).
#[test]
fn when_the_engine_lets_go_the_whole_run_is_one_step() {
    let reg = registry();
    let mut sim = SimWorld::new();
    let e = playing_sprite(&mut sim);
    let mut drive = PreviewDrive::default();
    let pre_run = capture(&drive, &mut sim, &reg);
    for _ in 0..9 {
        frame(&mut sim, &mut drive, 4);
    }
    assert_eq!(
        capture(&drive, &mut sim, &reg),
        pre_run,
        "durante a corrida nao ha' passo nenhum"
    );
    // Parar: a caixa «Playing» desmarcada. O tique deixa de declarar.
    if let Some(mut a) = sim.world_mut().get_mut::<SpriteAnimator>(e) {
        a.playing = false;
    }
    frame(&mut sim, &mut drive, 4);
    assert!(drive.is_empty(), "a `settle` tinha de esquecer quem parou");
    assert_ne!(
        capture(&drive, &mut sim, &reg),
        pre_run,
        "parar tem de deixar UM passo — a corrida"
    );
}

/// **A POSE QUE O SOLVER ESCREVE É A OUTRA METADE DO MESMO PEDIDO** (Enio: *«para ambas»*), e
/// atravessa o MESMO ledger — a lei é uma, os motores é que são dois.
///
/// ⚠️ Este gate encena o solver pela porta pública do ledger (declarar o antes e o depois), que é
/// exactamente o que o `physics_bridge` faz: a rota de lá pede um `PhysicsBridge` vivo e não é
/// alcançável de um teste.
///
/// **Mutação que deve sangrar:** o braço `SolverPose` da `Driven::write` não escrever nada.
#[test]
fn the_pose_the_solver_writes_is_preview_too() {
    let reg = registry();
    let mut sim = SimWorld::new();
    let e = sim
        .world_mut()
        .spawn((Transform::default(), Name::new("Ball")))
        .id();
    let mut drive = PreviewDrive::default();
    let at_rest = capture(&drive, &mut sim, &reg);

    let mut y = 0.0_f32;
    for _ in 0..12 {
        let was = *sim.world().get::<Transform>(e).unwrap();
        y -= 0.5;
        if let Some(mut t) = sim.world_mut().get_mut::<Transform>(e) {
            t.translation.y = y;
        }
        let now = *sim.world().get::<Transform>(e).unwrap();
        drive.driven(e, Driven::SolverPose(was), Driven::SolverPose(now));
        drive.settle();
        assert_eq!(
            capture(&drive, &mut sim, &reg),
            at_rest,
            "a bola a cair mudou a captura — cada clique viraria um passo vazio"
        );
    }
    assert!(
        sim.world().get::<Transform>(e).unwrap().translation.y < -5.0,
        "a bola nao caiu — o gate acima nao mediu nada"
    );
    // E parar a corrida deixa UM passo: a pose caída passa a ser documento.
    drive.settle();
    assert_ne!(
        capture(&drive, &mut sim, &reg),
        at_rest,
        "parar a simulacao tem de deixar um passo — a corrida"
    );
}

/// **SEM CONDUÇÃO, A CAPTURA É EXACTAMENTE A DE ANTES.** A metade que prova a inércia: um projeto
/// em que nada se move sozinho não paga uma varredura nem muda um byte.
///
/// **Mutação que deve sangrar:** a `is_empty` mentir (`false` constante) não muda nada aqui — de
/// propósito: o que este gate prende é o RESULTADO, e o atalho é só velocidade.
#[test]
fn with_nothing_running_the_capture_is_untouched() {
    let reg = registry();
    let mut sim = SimWorld::new();
    let e = playing_sprite(&mut sim);
    if let Some(mut a) = sim.world_mut().get_mut::<SpriteAnimator>(e) {
        a.playing = false;
    }
    let drive = PreviewDrive::default();
    let a = capture(&drive, &mut sim, &reg);
    let b = capture(&drive, &mut sim, &reg);
    assert_eq!(a, b, "capturar duas vezes o mesmo estado tem de dar igual");
    assert!(drive.is_empty());
}

/// **UMA ENTIDADE PODE ESTAR SOB DOIS MOTORES**, e as duas entradas não se pisam — um corpo rígido
/// com sprite animada é o caso normal, não a excepção.
///
/// **Mutação que deve sangrar:** a chave do ledger ser só os bits da entidade.
#[test]
fn one_entity_can_be_driven_by_two_engines_at_once() {
    let mut sim = SimWorld::new();
    let e = playing_sprite(&mut sim);
    let mut drive = PreviewDrive::default();
    frame(&mut sim, &mut drive, 4);
    let was = *sim.world().get::<Transform>(e).unwrap();
    if let Some(mut t) = sim.world_mut().get_mut::<Transform>(e) {
        t.translation.y = -3.0;
    }
    let now = *sim.world().get::<Transform>(e).unwrap();
    drive.driven(e, Driven::SolverPose(was), Driven::SolverPose(now));
    assert_eq!(
        drive.len(),
        2,
        "as duas conducoes da mesma entidade tem de coexistir"
    );
    let saved = drive.substitute_authored(&mut sim);
    assert_eq!(sim.world().get::<ph2d_ecs::SpriteGrid>(e).unwrap().frame, 0);
    assert_eq!(sim.world().get::<Transform>(e).unwrap().translation.y, 0.0);
    PreviewDrive::restore_live(&mut sim, &saved);
    assert_eq!(sim.world().get::<Transform>(e).unwrap().translation.y, -3.0);
}

/// **UM MEMO SOBREVIVE À MORTE DA ENTIDADE SEM DERRUBAR A CAPTURA.** O restore do undo despawna
/// tudo e respawna com bits novos; um memo do quadro anterior fica a apontar para o vazio, e a
/// `settle` seguinte limpa-o.
///
/// **Mutação que deve sangrar:** a `substitute_authored` fazer `unwrap` do `Driven::read`.
#[test]
fn a_memo_that_outlives_its_entity_does_not_break_the_capture() {
    let reg = registry();
    let mut sim = SimWorld::new();
    let e = playing_sprite(&mut sim);
    let mut drive = PreviewDrive::default();
    frame(&mut sim, &mut drive, 4);
    assert!(!drive.is_empty());
    sim.world_mut().despawn(e);
    let _ = capture(&drive, &mut sim, &reg); // nao pode entrar em panico
    drive.settle();
    assert!(drive.is_empty(), "a `settle` tinha de esquecer o fantasma");
}
