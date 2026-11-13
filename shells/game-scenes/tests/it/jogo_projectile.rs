//! A cena 25 (`PH2D_GAME_SMOKE=25`, ficha GEF 06): cinco balas montadas pelas peças do produto, cada uma com UM
//! campo do Projectile Motion mudado — o Play mostra o que esse campo faz.

use ph2d_app_game::smoke::projectile;
use ph2d_core::Vec2;
use ph2d_ecs::{Entity, Name, SimWorld, Transform};
use ph2d_physics_ecs::{BodyKind, PhysicsBridge, ProjectileMotion, RigidBody};

fn por_nome(sim: &mut SimWorld, nome: &str) -> Entity {
    let w = sim.world_mut();
    let mut q = w.query::<(Entity, &Name)>();
    let e = q.iter(w).find(|(_, n)| n.0 == nome).map(|(e, _)| e);
    e.unwrap_or_else(|| panic!("{nome}"))
}

fn onde(sim: &SimWorld, e: Entity) -> Vec2 {
    sim.world().get::<Transform>(e).expect("pose").translation
}

/// O míssil abre selecionado, e cada bala tem corpo Kinematic com forma e o Projectile Motion.
#[test]
fn a_cena_25_monta_as_cinco_balas_pelas_pecas() {
    let mut sim = SimWorld::new();
    let missil = crate::comp::montar(&mut sim, projectile::CENA);
    assert_eq!(por_nome(&mut sim, projectile::MISSIL), missil);
    for (nome, _, _, _) in projectile::BALAS {
        let e = por_nome(&mut sim, nome);
        let w = sim.world();
        let pm = w.get::<ProjectileMotion>(e).expect(nome);
        assert_eq!(pm.initial_speed, projectile::RAPIDEZ, "{nome}");
        let kind = w.get::<RigidBody>(e).map(|r| r.kind);
        assert_eq!(kind, Some(BodyKind::Kinematic), "{nome}");
        assert!(ph2d_physics_ecs::tem_forma(w, e), "{nome} tem forma");
    }
}

/// ⭐ **O Play ensina o que diz**: depois de 3 s de relógio, o Missile (lançado para BAIXO) deu a volta e acabou no
/// Target sem Max Speed, Straight parou na parede, Bouncy voltou para trás, Arc pousou no Floor (dentro da arena),
/// Short Range parou a ~5 m. (Mutação: o Pull que soma rapidez ⇒ a volta abre e o Missile passa ao lado.)
#[test]
fn no_play_cada_bala_faz_o_que_o_seu_campo_diz() {
    let mut sim = SimWorld::new();
    crate::comp::montar(&mut sim, projectile::CENA);
    let balas: Vec<(&str, Entity, Vec2)> = projectile::BALAS
        .iter()
        .map(|(nome, at, _, _)| (*nome, por_nome(&mut sim, nome), Vec2::new(at[0], at[1])))
        .collect();
    let mut bridge = PhysicsBridge::new();
    let mut max_x = [f32::MIN; 5];
    for tick in 1..=180_u64 {
        bridge.dispatch(&mut sim, true, tick);
        for (i, (_, e, _)) in balas.iter().enumerate() {
            max_x[i] = max_x[i].max(onde(&sim, *e).x);
        }
    }
    let acabou: Vec<Entity> = bridge.projectiles_finished().collect();
    for (i, (nome, e, a)) in balas.iter().enumerate() {
        let p = onde(&sim, *e);
        println!(
            "[projectile-25] {nome}: de ({:.2}, {:.2}) a ({:.2}, {:.2}), x máx {:.2}, acabou {}",
            a.x,
            a.y,
            p.x,
            p.y,
            max_x[i],
            acabou.contains(e)
        );
    }
    let bala = |nome: &str| {
        let i = balas.iter().position(|(n, _, _)| *n == nome).expect(nome);
        (balas[i].1, balas[i].2, i)
    };
    let alvo = Vec2::new(projectile::ALVO_EM[0], projectile::ALVO_EM[1]);
    let (missil, _, _) = bala(projectile::MISSIL);
    let d = (onde(&sim, missil) - alvo).length();
    println!("[projectile-25] Missile acaba a {d:.2} m do Target");
    assert!(
        acabou.contains(&missil) && d < 0.7,
        "o Missile acerta o Target: a {d} m"
    );
    let parede = 5.5 - 0.3;
    let (reta, _, _) = bala("Straight");
    assert!(acabou.contains(&reta), "Straight acaba na parede");
    assert!(onde(&sim, reta).x < parede, "Straight fica antes da parede");
    let (bouncy, _, ib) = bala("Bouncy");
    assert!(
        max_x[ib] > 0.0 && onde(&sim, bouncy).x < max_x[ib] - 1.0,
        "Bouncy bateu e voltou: x máx {} e agora {}",
        max_x[ib],
        onde(&sim, bouncy).x
    );
    let (c, h) = projectile::PAREDE_ESQ;
    for (nome, e, _) in &balas {
        let x = onde(&sim, *e).x;
        assert!(x > c[0] + h[0], "{nome} fica dentro da arena: x = {x}");
    }
    let (arco, a, _) = bala("Arc");
    assert!(onde(&sim, arco).y < a.y - 1.0, "Arc caiu");
    let (c, h) = projectile::CHAO;
    let pousa = c[1] + h[1];
    let pa = onde(&sim, arco);
    assert!(acabou.contains(&arco), "Arc acaba ao tocar o Floor");
    assert!(
        pa.y > pousa - 0.05 && pa.y < pousa + 0.5 && pa.x.abs() < h[0],
        "Arc pousa no Floor, dentro da arena: ({}, {})",
        pa.x,
        pa.y
    );
    let longe = (onde(&sim, arco) - alvo).length();
    assert!(longe > 2.0, "o Arc não acerta o Target: a {longe} m");
    let (curta, a, _) = bala("Short Range");
    assert!(acabou.contains(&curta), "Short Range acabou");
    let x = onde(&sim, curta).x;
    assert!(
        (x - (a.x + 5.0)).abs() < 0.5,
        "Short Range parou a 5 m: x = {x}"
    );
}

/// ⭐ GEF 06 — **arrastar um objeto da Hierarquia para o Target da bala** (regra do dono 02/11: arrastar, escrever
/// ou conta-gotas): o campo pintado recebe só com o Homing ligado, o objeto vira o alvo, o campo acende, e a escrita
/// é a MESMA edição do conta-gotas (um passo de desfazer); uma imagem é recusada.
///
/// **Mutações que devem sangrar:** receber com o Homing desligado · o braço da bala devolver o da câmera ·
/// a edição escrever outro campo.
#[test]
fn o_target_da_bala_recebe_um_objeto_arrastado() {
    use ph2d_app_components::asset_drop::{
        DropAction, DropTarget, RefField, edicao_do_campo, field_marks, field_target, resolve,
    };
    use ph2d_editor_core::action_bus::{ComponentEdit, EditorAction};
    use ph2d_editor_core::interaction::drag_payload::DragPayload;
    use ph2d_editor_core::projectile_edits::ProjectileFieldEdit;
    use ph2d_panel_inspector::ids::{INSP_PJ_HOMING_TARGET, INSP_PJ_TARGET_PICK};
    use ph2d_physics_ecs::ProjectileMotion;
    let mut w = ph2d_ecs::World::new();
    let bala = w.spawn(ProjectileMotion::default()).id();
    let bits = bala.to_bits();
    let info = |w: &ph2d_ecs::World| {
        ph2d_app_components::projectile_inspector::build_projectile_info(w, bits, 1, false, false)
    };
    ph2d_panel_inspector::set_current_inspector_projectile(info(&w));
    assert_eq!(
        field_target(INSP_PJ_HOMING_TARGET),
        None,
        "Homing desligado não recebe"
    );
    w.get_mut::<ProjectileMotion>(bala)
        .expect("bala")
        .homing_accel = 30.0;
    ph2d_panel_inspector::set_current_inspector_projectile(info(&w));
    let field = RefField::Homing;
    let alvo = DropTarget::ObjectRef {
        entity_bits: bits,
        field,
    };
    assert_eq!(field_target(INSP_PJ_HOMING_TARGET), Some(alvo));
    assert_eq!(
        field_target(INSP_PJ_TARGET_PICK),
        None,
        "o conta-gotas não é o campo"
    );
    let objeto = DragPayload::Object { stable_id: 42 };
    assert_eq!(
        resolve(objeto, alvo),
        DropAction::SetObjectRef {
            entity_bits: bits,
            field,
            stable_id: 42
        }
    );
    let acende = field_marks([INSP_PJ_HOMING_TARGET], objeto, None);
    assert_eq!(acende.len(), 1, "o campo acende com um objeto na mão");
    let imagem = DragPayload::Image { asset: [0; 32] };
    assert_eq!(resolve(imagem, alvo), DropAction::Refuse);
    assert_eq!(
        edicao_do_campo(bits, field, "Wall".into()),
        EditorAction::InspectorComponentEdit {
            entity_bits: bits,
            edit: ComponentEdit::Projectile(ProjectileFieldEdit::HomingTarget("Wall".into())),
        }
    );
    ph2d_panel_inspector::set_current_inspector_projectile(None);
}
