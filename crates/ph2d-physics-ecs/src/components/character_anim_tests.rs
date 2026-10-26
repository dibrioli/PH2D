use super::*;

fn view(footing: FootingKind, v: [f32; 2], ground: [f32; 2]) -> PlayerView {
    PlayerView {
        footing,
        velocity: v,
        ground_velocity: ground,
        ..PlayerView::default()
    }
}

#[test]
fn no_chao_quieto_e_parado_e_a_andar_e_correr() {
    assert_eq!(
        CharacterPose::of(&view(FootingKind::Ground, [0.1, 0.0], [0.0, 0.0]), 0.5),
        CharacterPose::Idle
    );
    assert_eq!(
        CharacterPose::of(&view(FootingKind::Ground, [-3.0, 0.0], [0.0, 0.0]), 0.5),
        CharacterPose::Run
    );
}

/// Em pé numa plataforma que anda, o personagem está PARADO: a velocidade conta-se contra o chão.
#[test]
fn numa_plataforma_que_anda_quieto_e_parado() {
    let v = view(FootingKind::Ground, [2.0, 0.0], [2.0, 0.0]);
    assert_eq!(CharacterPose::of(&v, 0.5), CharacterPose::Idle);
}

#[test]
fn no_ar_subir_e_saltar_e_descer_e_cair() {
    assert_eq!(
        CharacterPose::of(&view(FootingKind::Airborne, [1.0, 4.0], [0.0, 0.0]), 0.5),
        CharacterPose::Jump
    );
    assert_eq!(
        CharacterPose::of(&view(FootingKind::Airborne, [1.0, -0.1], [0.0, 0.0]), 0.5),
        CharacterPose::Fall
    );
    assert_eq!(
        CharacterPose::of(&view(FootingKind::Steep, [0.0, -2.0], [0.0, 0.0]), 0.5),
        CharacterPose::Fall
    );
}

#[test]
fn cada_estado_tem_o_seu_nome() {
    let a = CharacterAnimations::default();
    let names: Vec<&str> = [
        CharacterPose::Idle,
        CharacterPose::Run,
        CharacterPose::Jump,
        CharacterPose::Fall,
    ]
    .into_iter()
    .map(|p| a.tag(p).expect("nomeada"))
    .collect();
    assert_eq!(names, ["idle", "run", "jump", "fall"]);
    assert_eq!(a.tag(CharacterPose::Crouch), None);
}
