use super::*;

/// Ficheiro antigo: só `CharacterAnimations` ⇒ as 4 regras de hoje, pela ordem dos estados.
#[test]
fn sem_regras_gravadas_o_personagem_da_as_quatro() {
    let c = CharacterAnimations {
        jump: "salto".into(),
        ..CharacterAnimations::default()
    };
    let r = AnimRules::effective(None, Some(&c), |_| false).expect("regras");
    let pares: Vec<_> =
        r.0.iter()
            .map(|r| (r.when.clone(), r.play.as_str()))
            .collect();
    assert_eq!(
        pares,
        [
            (AnimWhen::Pose(CharacterPose::Idle), "idle"),
            (AnimWhen::Pose(CharacterPose::Run), "run"),
            (AnimWhen::Pose(CharacterPose::Jump), "salto"),
            (AnimWhen::Pose(CharacterPose::Fall), "fall"),
        ]
    );
    assert_eq!(AnimRules::effective(None, None, |_| true), None);
    let gravadas = AnimRules(vec![]);
    assert_eq!(
        AnimRules::effective(Some(&gravadas), Some(&c), |_| true),
        Some(gravadas)
    );
}

/// ⭐ Plano 10: as do agachar entram no TOPO só se a tag de convenção existe — a andar agachado por cima de
/// agachado — e um objecto sem elas (todo ficheiro antigo) dá as 4, bit a bit.
#[test]
fn as_do_agachar_so_entram_com_a_sua_tag() {
    let c = CharacterAnimations::default();
    let quatro = AnimRules::of_character(&c);
    assert_eq!(quatro.0.len(), 4);
    assert_eq!(AnimRules::of_character_with(&c, |n| n == "salto"), quatro);
    let whens = |r: AnimRules| -> Vec<(AnimWhen, String)> {
        r.0.into_iter().map(|r| (r.when, r.play)).collect()
    };
    let seis = whens(AnimRules::of_character_with(&c, |n| {
        n == "crouch" || n == "crouch_walk"
    }));
    assert_eq!(
        seis[..2],
        [
            (
                AnimWhen::Pose(CharacterPose::CrouchWalk),
                "crouch_walk".into()
            ),
            (AnimWhen::Pose(CharacterPose::Crouch), "crouch".into()),
        ]
    );
    assert_eq!(seis[2..], whens(quatro.clone())[..]);
    let so_agachado = whens(AnimRules::of_character_with(&c, |n| n == "crouch"));
    assert_eq!(
        so_agachado[0],
        (AnimWhen::Pose(CharacterPose::Crouch), "crouch".into())
    );
    assert_eq!(so_agachado.len(), 5);
}

#[test]
fn renomear_leva_as_regras_e_os_factos_de_um_quadro_seguram() {
    let mut r = AnimRules::of_character(&CharacterAnimations::default());
    assert!(r.rename_play("run", "corre"));
    assert_eq!(r.0[1].play, "corre");
    assert!(!r.rename_play("nada", "x"));
    assert!(AnimWhen::Life(LifeEvent::Damaged).is_instant());
    assert!(AnimWhen::Signal("x".into()).is_instant());
    assert!(!AnimWhen::Action("down".into()).is_instant());
    assert_eq!(
        LifeEvent::of(HealthEventKind::Shielded { amount: 1.0 }),
        None,
        "escudo não é dano na vida"
    );
}

/// O dado viaja pelo postcard (é o que o ficheiro grava) e volta igual.
#[test]
fn as_regras_voltam_iguais_do_postcard() {
    let r = AnimRules(vec![
        AnimRule {
            when: AnimWhen::Action("down".into()),
            play: "crouch".into(),
        },
        AnimRule {
            when: AnimWhen::Life(LifeEvent::Damaged),
            play: "hurt".into(),
        },
        AnimRule {
            when: AnimWhen::Signal("boom".into()),
            play: "fall".into(),
        },
    ]);
    let bytes = postcard::to_allocvec(&r).unwrap();
    assert_eq!(postcard::from_bytes::<AnimRules>(&bytes).unwrap(), r);
}
