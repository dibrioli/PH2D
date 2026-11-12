//! A cena 24 (`PH2D_GAME_SMOKE=24`, ficha GEF 05): o Top-Down Player montado pelas peças do produto, e a
//! pergunta da ficha — a imagem GIRA com a pose do corpo quando Facing ≠ None?

use ph2d_app_game::smoke::topdown;
use ph2d_ecs::{Entity, GlobalTransform, Name, SimWorld, SpriteAnimator, Transform};
use ph2d_physics_ecs::{BodyKind, PhysicsBridge, PlayerInput, RigidBody, TopDownPlayer};
use ph2d_render::{RenderInstance, Sprite};
use ph2d_topdown::rotation::{self, RotationMode};

fn por_nome(sim: &mut SimWorld, nome: &str) -> Option<Entity> {
    let w = sim.world_mut();
    let mut q = w.query::<(Entity, &Name)>();
    q.iter(w).find(|(_, n)| n.0 == nome).map(|(e, _)| e)
}

/// A cena tem o herói de fábrica VISTO DE CIMA com o mover de vista de cima, corpo Kinematic com forma, Facing = Move, e as
/// três paredes estáticas.
#[test]
fn a_cena_24_monta_o_heroi_de_vista_de_cima_pelas_pecas() {
    let mut sim = SimWorld::new();
    let heroi = crate::comp::montar(&mut sim, topdown::CENA);
    assert_eq!(por_nome(&mut sim, topdown::HEROI), Some(heroi));
    let w = sim.world();
    let td = w.get::<TopDownPlayer>(heroi).expect("o Top-Down Player");
    assert_eq!(
        rotation::from_wire(td.rotation_mode),
        RotationMode::ToMovement
    );
    assert_eq!(
        w.get::<RigidBody>(heroi).map(|r| r.kind),
        Some(BodyKind::Kinematic)
    );
    assert!(ph2d_physics_ecs::tem_forma(w, heroi), "o corpo tem forma");
    assert_eq!(
        w.get::<Sprite>(heroi).map(|s| s.source),
        Some(ph2d_render::SpriteSource::Atlas {
            key: ph2d_render::TOPDOWN_HERO_KEY
        })
    );
    assert_eq!(
        w.get::<ph2d_physics_ecs::AnimRules>(heroi),
        Some(&topdown::regras_de_andar())
    );
    for (nome, _, _) in topdown::PAREDES {
        let e = por_nome(&mut sim, nome).unwrap_or_else(|| panic!("{nome}"));
        assert_eq!(
            sim.world().get::<RigidBody>(e).map(|r| r.kind),
            Some(BodyKind::Static),
            "{nome}"
        );
    }
}

/// O ângulo com que o renderer desenha `e` (a base do `GlobalTransform`; o herói está na raiz).
fn angulo_desenhado(sim: &SimWorld, e: Entity) -> f32 {
    let t = *sim.world().get::<Transform>(e).expect("pose");
    let (_, b) = RenderInstance::pose_de(&GlobalTransform::from_transform(t));
    b[1].atan2(b[0])
}

/// ⭐ **A imagem gira com o corpo**: ↗ ⇒ 45°, ↑ ⇒ 90°, → ⇒ 0°, no `Transform` e na base que o renderer desenha.
/// (Mutação: o herói com Facing = None ⇒ fica a 0° e reprova.)
#[test]
fn com_facing_move_a_imagem_desenhada_vira_para_onde_anda() {
    let mut sim = SimWorld::new();
    let heroi = crate::comp::montar(&mut sim, topdown::CENA);
    let mut bridge = PhysicsBridge::new();
    let mut tick = 0_u64;
    let mut anda = |sim: &mut SimWorld, drive: f32, drive_y: f32| {
        for _ in 0..30 {
            tick += 1;
            let input = PlayerInput {
                drive,
                drive_y,
                ..PlayerInput::default()
            };
            bridge.set_player_input(heroi, input);
            bridge.dispatch(sim, true, tick);
        }
        let t = sim.world().get::<Transform>(heroi).expect("pose");
        (t.translation, t.rotation, angulo_desenhado(sim, heroi))
    };
    let inicio = topdown::INICIO;
    for (drive, drive_y, graus) in [(1.0, 1.0, 45.0), (0.0, 1.0, 90.0), (1.0, 0.0, 0.0)] {
        let (onde, rot, desenhado) = anda(&mut sim, drive, drive_y);
        println!(
            "[topdown-giro] dedo ({drive}, {drive_y}) ⇒ em ({:.2}, {:.2}), Transform.rotation {:.2}°, desenhado {:.2}°",
            onde.x,
            onde.y,
            rot.to_degrees(),
            desenhado.to_degrees()
        );
        assert!(
            (rot.to_degrees() - graus).abs() < 0.5,
            "o Transform devia olhar a {graus}° e olha a {:.3}°",
            rot.to_degrees()
        );
        assert!(
            (desenhado.to_degrees() - graus).abs() < 0.5,
            "o renderer devia desenhar a {graus}° e desenha a {:.3}°",
            desenhado.to_degrees()
        );
    }
    let fim = sim
        .world()
        .get::<Transform>(heroi)
        .expect("pose")
        .translation;
    assert!(
        fim.x > inicio[0] + 1.0 && fim.y > inicio[1] + 1.0,
        "ele andou: de {inicio:?} a ({}, {})",
        fim.x,
        fim.y
    );
}

/// O tocador do herói depois de um quadro das regras com `teclas` seguras.
fn toca(sim: &mut SimWorld, heroi: Entity, teclas: &[&str]) -> String {
    let premidas = teclas.iter().map(|t| (*t).to_owned()).collect();
    let outbox = ph2d_runtime::SignalOutbox::new();
    ph2d_app_game::personagem::anima_personagens(
        sim.world_mut(),
        &PhysicsBridge::new(),
        &premidas,
        &outbox,
    );
    let a = sim.world().get::<SpriteAnimator>(heroi).expect("tocador");
    a.current.clone()
}

/// ⭐ **Parado respira, a andar anda**: com qualquer tecla de andar segura (e na diagonal, duas) o herói toca
/// `walk`; soltas, volta a `idle` no mesmo quadro — só as regras «Key held» da cena, sem código novo.
/// (Mutação: sem as regras ⇒ fica em `idle` e reprova.)
#[test]
fn o_heroi_de_cima_anda_com_a_tecla_e_para_sem_ela() {
    let mut sim = SimWorld::new();
    let heroi = crate::comp::montar(&mut sim, topdown::CENA);
    let (idle, walk) = (
        ph2d_render::TOPDOWN_HERO_IDLE,
        ph2d_render::TOPDOWN_HERO_WALK,
    );
    assert_eq!(toca(&mut sim, heroi, &[]), idle);
    for teclas in topdown::ACOES_DE_ANDAR
        .iter()
        .map(|a| vec![*a])
        .chain([vec![
            ph2d_input::PLAYER_MOVE_RIGHT,
            ph2d_input::PLAYER_MOVE_UP,
        ]])
    {
        assert_eq!(toca(&mut sim, heroi, &teclas), walk, "{teclas:?}");
        assert_eq!(toca(&mut sim, heroi, &[]), idle, "soltar {teclas:?}");
    }
}

/// Um quadro do horário do Play: as setas seguras, a pose e o ângulo desenhado.
struct Quadro {
    quadro: u32,
    setas: std::collections::BTreeSet<ph2d_app_game::smoke::Seta>,
    onde: ph2d_core::Vec2,
    graus: f32,
}

/// Corre o horário `HORARIO_24` (o do `PH2D_GAME_SMOKE_DRIVE=editor`) pela ponte, um tique por quadro desde o Play.
fn corre_o_horario(sim: &mut SimWorld, heroi: Entity) -> Vec<Quadro> {
    use ph2d_app_game::smoke::{Gesto, Seta, gesto_no_quadro};
    use ph2d_editor_core::game_stage::GameStage;
    let mut bridge = PhysicsBridge::new();
    let mut setas = std::collections::BTreeSet::new();
    let mut fora = Vec::new();
    for quadro in 61..=600_u32 {
        match gesto_no_quadro(topdown::CENA, GameStage::Editor, quadro) {
            Some(Gesto::Para) => break,
            Some(g) => {
                if let Some((s, desce)) = g.seta() {
                    if desce {
                        setas.insert(s);
                    } else {
                        setas.remove(&s);
                    }
                }
            }
            None => {}
        }
        let tem = |s| f32::from(u8::from(setas.contains(&s)));
        bridge.set_player_input(
            heroi,
            PlayerInput {
                drive: tem(Seta::Direita) - tem(Seta::Esquerda),
                drive_y: tem(Seta::Cima) - tem(Seta::Baixo),
                ..PlayerInput::default()
            },
        );
        bridge.dispatch(sim, true, u64::from(quadro));
        let onde = sim
            .world()
            .get::<Transform>(heroi)
            .expect("pose")
            .translation;
        fora.push(Quadro {
            quadro,
            setas: setas.clone(),
            onde,
            graus: angulo_desenhado(sim, heroi).to_degrees(),
        });
    }
    fora
}

/// ⭐ **O Play do smoke anda SÓ para cima (e só para a esquerda)**, livre, depois da diagonal: em cada trecho de uma
/// seta só, passado o giro, a imagem olha para onde anda e cada quadro avança — nenhuma parede o pára antes de se ver o
/// andar. (Mutação: sem os trechos ← e ↑ do `HORARIO_24` ⇒ nenhum quadro com uma seta só e reprova.)
#[test]
fn o_horario_do_play_anda_so_para_cima_e_so_para_a_esquerda() {
    use ph2d_app_game::smoke::Seta;
    let mut sim = SimWorld::new();
    let heroi = crate::comp::montar(&mut sim, topdown::CENA);
    let q = corre_o_horario(&mut sim, heroi);
    let encostou = q
        .iter()
        .filter(|f| f.setas.len() == 2)
        .map(|f| f.onde.y)
        .fold(f32::MIN, f32::max);
    assert!(
        encostou > 1.0,
        "a diagonal chega à parede de cima: y = {encostou}"
    );
    for (seta, graus, passo) in [
        (Seta::Cima, 90.0_f32, ph2d_core::Vec2::new(0.0, 1.0)),
        (Seta::Esquerda, 180.0, ph2d_core::Vec2::new(-1.0, 0.0)),
    ] {
        let trecho: Vec<_> = q
            .iter()
            .filter(|f| f.setas.len() == 1 && f.setas.contains(&seta))
            .collect();
        assert!(trecho.len() >= 45, "{seta:?}: {} quadros", trecho.len());
        let (a, b) = (trecho[15].quadro, trecho[trecho.len() - 1].quadro);
        for par in trecho[15..].windows(2) {
            let (f0, f1) = (par[0], par[1]);
            let d = (f1.onde - f0.onde).dot(passo);
            assert!(
                d > 0.05,
                "{seta:?} q{}: parado ou ao contrário ({:?} → {:?})",
                f1.quadro,
                f0.onde,
                f1.onde
            );
            let erro = ((f1.graus - graus + 540.0).rem_euclid(360.0) - 180.0).abs();
            assert!(erro < 0.5, "{seta:?} q{}: olha a {}°", f1.quadro, f1.graus);
        }
        println!("[horario-24] só {seta:?}: livre e a {graus}° de q{a} a q{b}");
    }
}
