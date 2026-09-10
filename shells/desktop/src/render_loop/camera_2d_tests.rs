//! Os gates da PONTE da câmera de jogo — o que ela DECIDE, medido sem janela.
//!
//! ⚠️ **Eles correm o caminho do PRODUTO** ([`super::update`]), e não uma cópia dele: um gate que
//! reimplementasse o passe mediria o teste. A lei em si já é gateada contra o oráculo do Godot em
//! `ph2d-ecs`; o que se prova aqui é a **costura** — quem manda, para onde ela olha, e o que ela
//! devolve à vista.

use ph2d_core::Vec2;
use ph2d_ecs::{
    CameraFollow, CameraLimits, GameCamera, Name, SimWorld, Transform, assign_missing_stable_ids,
};

use super::update;

const ASPECT: f32 = 16.0 / 9.0;
const DT: f64 = 1.0 / 60.0;

fn mundo() -> SimWorld {
    SimWorld::default()
}

/// Uma câmera na posição `p`, com a config dada.
fn camera(sim: &mut SimWorld, nome: &str, p: [f32; 2], cam: GameCamera) -> ph2d_ecs::Entity {
    let e = sim
        .world_mut()
        .spawn((
            Transform::from_translation(Vec2::new(p[0], p[1])),
            Name::new(nome),
            cam,
        ))
        .id();
    assign_missing_stable_ids(sim.world_mut());
    e
}

/// Um objecto qualquer com nome — o que uma câmera segue.
fn objecto(sim: &mut SimWorld, nome: &str, p: [f32; 2]) -> ph2d_ecs::Entity {
    let e = sim
        .world_mut()
        .spawn((
            Transform::from_translation(Vec2::new(p[0], p[1])),
            Name::new(nome),
        ))
        .id();
    assign_missing_stable_ids(sim.world_mut());
    e
}

fn mover(sim: &mut SimWorld, e: ph2d_ecs::Entity, p: [f32; 2]) {
    if let Some(mut t) = sim.world_mut().get_mut::<Transform>(e) {
        t.translation = Vec2::new(p[0], p[1]);
    }
}

/// ⚠️ **Uma cena sem câmera devolve `None`, e isso é o comportamento de HOJE** — o editor fica com
/// o enquadramento dele. ⛔ Devolver uma vista de omissão faria toda cena existente saltar para a
/// origem no primeiro quadro depois desta wave.
///
/// **Mutação que deve sangrar:** trocar o `else` do `active_camera_of` por uma vista de omissão.
#[test]
fn a_scene_with_no_camera_leaves_the_view_to_the_editor() {
    let mut sim = mundo();
    let (vista, r) = update(&mut sim, ASPECT, 1, DT);
    assert!(
        vista.is_none(),
        "sem camera a vista tem de ficar com o editor"
    );
    assert_eq!(r.cameras, 0);
    assert!(!r.following);
}

/// ⭐ **Uma câmera SEM `follow` é uma câmera FIXA** — a vista é a pose dela, sem componente nenhum
/// a mais. É a metade do desenho que justifica os três componentes separados.
#[test]
fn a_camera_without_follow_frames_its_own_pose() {
    let mut sim = mundo();
    camera(&mut sim, "Cam", [3.0, -2.0], GameCamera::default());
    let (vista, r) = update(&mut sim, ASPECT, 1, DT).clone();
    let v = vista.expect("uma camera activa tem de dar vista");
    assert_eq!(v.center, [3.0, -2.0]);
    assert_eq!(r.cameras, 1);
    assert!(!r.following, "ela nao tem follow");
}

/// ⭐⭐⭐ **O NASCIMENTO não amortece** — senão a câmera viaja da origem até ao alvo no primeiro
/// segundo de jogo, que é o report que o `Timer` e o som já pagaram.
///
/// **Mutação que deve sangrar:** apagar o ramo do `settled` (a câmera passaria a arrancar em `(0,0)`
/// e a amortecer até `[40, 0]`).
#[test]
fn the_first_frame_settles_instead_of_travelling() {
    let mut sim = mundo();
    objecto(&mut sim, "Heroi", [40.0, 0.0]);
    camera(&mut sim, "Cam", [0.0, 0.0], GameCamera::default());
    let cam_e = ph2d_ecs::active_camera_of(sim.world_mut()).unwrap();
    sim.world_mut().entity_mut(cam_e).insert(CameraFollow {
        target: "Heroi".into(),
        ..CameraFollow::default()
    });
    let (vista, r) = update(&mut sim, ASPECT, 1, DT);
    assert!(r.target_found, "o alvo chama-se `Heroi` e existe");
    assert_eq!(
        vista.unwrap().center,
        [40.0, 0.0],
        "a camera tem de NASCER no alvo, nunca viajar ate' ele"
    );
}

/// ⭐⭐ **Depois de assentar, ela AMORTECE** — e o número sai da lei, que é a do oráculo.
#[test]
fn once_settled_it_damps_towards_the_target() {
    let mut sim = mundo();
    let heroi = objecto(&mut sim, "Heroi", [0.0, 0.0]);
    camera(&mut sim, "Cam", [0.0, 0.0], GameCamera::default());
    let cam_e = ph2d_ecs::active_camera_of(sim.world_mut()).unwrap();
    sim.world_mut().entity_mut(cam_e).insert(CameraFollow {
        target: "Heroi".into(),
        damping: [5.0, 5.0],
        ..CameraFollow::default()
    });
    // Quadro 1: assenta em (0,0).
    let _ = update(&mut sim, ASPECT, 1, DT);
    // O herói salta para 300; um tique tem de dar `300 · 5/60 = 25`.
    mover(&mut sim, heroi, [300.0, 0.0]);
    let (vista, _) = update(&mut sim, ASPECT, 1, DT);
    let c = vista.unwrap().center;
    assert!(
        (c[0] - 25.0).abs() < 1e-3,
        "um tique a speed 5 devia dar 25, deu {c:?}"
    );
}

/// ⭐⭐⭐ **Um alvo que não existe é RELATADO, nunca engolido.**
///
/// ⚠️ *«segue ninguém»* e *«segue alguém parado»* dão exactamente a mesma câmera imóvel, e só uma
/// delas é um defeito da cena. Sem esta coluna o dono lê as duas como *«a câmera não funciona»*.
///
/// **Mutação que deve sangrar:** pôr `target_found = true` incondicionalmente.
#[test]
fn a_target_that_does_not_exist_is_reported_not_swallowed() {
    let mut sim = mundo();
    camera(&mut sim, "Cam", [7.0, 1.0], GameCamera::default());
    let cam_e = ph2d_ecs::active_camera_of(sim.world_mut()).unwrap();
    sim.world_mut().entity_mut(cam_e).insert(CameraFollow {
        target: "NaoExiste".into(),
        ..CameraFollow::default()
    });
    let (vista, r) = update(&mut sim, ASPECT, 1, DT);
    assert!(r.following, "ela TEM follow");
    assert!(
        !r.target_found,
        "e o alvo NAO foi encontrado — a coluna tem de o dizer"
    );
    assert_eq!(r.target_name, "NaoExiste");
    assert_eq!(
        vista.unwrap().center,
        [7.0, 1.0],
        "sem alvo ela fica na pose dela, e nao na origem"
    );
}

/// ⭐⭐ **A cerca prende a JANELA, não o centro** — e o relatório diz quando ela mordeu.
#[test]
fn the_limits_hold_the_window_and_the_report_says_so() {
    let mut sim = mundo();
    let heroi = objecto(&mut sim, "Heroi", [0.0, 0.0]);
    camera(
        &mut sim,
        "Cam",
        [0.0, 0.0],
        GameCamera {
            height_world: 10.0,
            ..GameCamera::default()
        },
    );
    let cam_e = ph2d_ecs::active_camera_of(sim.world_mut()).unwrap();
    sim.world_mut().entity_mut(cam_e).insert(CameraFollow {
        target: "Heroi".into(),
        damping: [0.0, 0.0], // instantâneo: isola a cerca do amortecimento
        ..CameraFollow::default()
    });
    sim.world_mut().entity_mut(cam_e).insert(CameraLimits {
        min: [-100.0, -100.0],
        max: [100.0, 100.0],
    });
    let _ = update(&mut sim, ASPECT, 1, DT);
    mover(&mut sim, heroi, [1000.0, 0.0]);
    let (vista, r) = update(&mut sim, ASPECT, 1, DT);
    // meia-largura = 5 · (16/9) = 8,888…  ⇒  o centro pára em 100 − 8,888… = 91,111…
    let esperado = 100.0 - 5.0 * ASPECT;
    let c = vista.unwrap().center;
    assert!(
        (c[0] - esperado).abs() < 1e-3,
        "a JANELA e' que tem de parar na cerca: esperado {esperado}, deu {c:?}"
    );
    assert!(r.limited, "a cerca mordeu e o relatorio tem de o dizer");
}

/// ⭐ **A prioridade escolhe quem manda, e a desligada não concorre** — pelo caminho do produto.
#[test]
fn the_bridge_uses_the_camera_that_commands() {
    let mut sim = mundo();
    camera(
        &mut sim,
        "Baixa",
        [1.0, 0.0],
        GameCamera {
            priority: 1,
            ..GameCamera::default()
        },
    );
    camera(
        &mut sim,
        "Alta",
        [9.0, 0.0],
        GameCamera {
            priority: 5,
            ..GameCamera::default()
        },
    );
    let (vista, r) = update(&mut sim, ASPECT, 1, DT);
    assert_eq!(r.cameras, 2);
    assert_eq!(vista.unwrap().center[0], 9.0, "a de maior prioridade manda");
}

/// ⭐⭐⭐ **O `offset` NÃO realimenta o estado vivo.**
///
/// ⚠️ Se ele fosse somado ao centro guardado, a zona morta passaria a medir a distância ao ALVO
/// DESLOCADO e a câmera afastar-se-ia um pouco mais a cada quadro — uma deriva lenta que só se vê
/// depois de minutos, e que nenhum gate de um quadro apanharia.
///
/// **Mutação que deve sangrar:** somar o `offset` a `rt.center` em vez de à vista.
#[test]
fn the_offset_frames_the_view_without_feeding_back() {
    let mut sim = mundo();
    let heroi = objecto(&mut sim, "Heroi", [0.0, 0.0]);
    camera(
        &mut sim,
        "Cam",
        [0.0, 0.0],
        GameCamera {
            offset: [0.0, 3.0],
            ..GameCamera::default()
        },
    );
    let cam_e = ph2d_ecs::active_camera_of(sim.world_mut()).unwrap();
    sim.world_mut().entity_mut(cam_e).insert(CameraFollow {
        target: "Heroi".into(),
        damping: [0.0, 0.0],
        ..CameraFollow::default()
    });
    let mut ultimo = [0.0_f32, 0.0];
    for _ in 0..240 {
        mover(&mut sim, heroi, [0.0, 0.0]);
        let (v, _) = update(&mut sim, ASPECT, 1, DT);
        ultimo = v.unwrap().center;
    }
    assert!(
        (ultimo[1] - 3.0).abs() < 1e-3,
        "depois de 240 quadros parados a vista tem de continuar a 3 m acima do alvo, e esta' em \
         {ultimo:?} — o offset esta' a realimentar"
    );
}

/// ⚠️ **Um quadro SEM passo fixo ainda arma a câmera** — senão uma câmera anexada pela paleta ficava
/// por nascer até calhar um tique, e o sintoma é *«às vezes a câmera não pega»*.
#[test]
fn a_frame_with_no_fixed_tick_still_gives_the_camera_a_life() {
    let mut sim = mundo();
    camera(&mut sim, "Cam", [4.0, 4.0], GameCamera::default());
    let (vista, _) = update(&mut sim, ASPECT, 0, DT);
    assert_eq!(
        vista
            .expect("mesmo com ticks=0 a camera activa da' vista")
            .center,
        [4.0, 4.0]
    );
}
