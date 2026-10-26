use super::*;
use ph2d_ecs::ChildOf;
use ph2d_physics_ecs::{AnimRule, AnimWhen, LifeEvent};

/// O animador é o do próprio personagem; sem ele, o do 1.º filho que tem um (a imagem dentro do corpo).
#[test]
fn o_alvo_e_o_proprio_ou_o_filho_com_animador() {
    let mut w = World::new();
    let so = w.spawn(SpriteAnimator::new("idle")).id();
    assert_eq!(alvo(&w, so), Some(so));
    let corpo = w.spawn_empty().id();
    let _vazio = w.spawn(ChildOf(corpo)).id();
    let imagem = w.spawn((SpriteAnimator::new("idle"), ChildOf(corpo))).id();
    assert_eq!(alvo(&w, corpo), Some(imagem));
    let nada = w.spawn_empty().id();
    assert_eq!(alvo(&w, nada), None);
}

/// ⭐ Report do dono 25/10 («ao pular não volta mais a correr»): o salto toca UMA vez e, ao acabar, o
/// animador pára; a troca para correr tem de o pôr a tocar outra vez, do início.
#[test]
fn depois_do_salto_que_acabou_a_corrida_toca() {
    let mut a = SpriteAnimator::new("jump");
    a.playing = false;
    a.repeat_count = 1;
    a.elapsed_ticks = 9_999;
    troca(&mut a, "run".into());
    assert_eq!(
        (
            a.current.as_str(),
            a.playing,
            a.elapsed_ticks,
            a.repeat_count
        ),
        ("run", true, 0, 0)
    );
}

/// Sem vista do player (a física desarmada), a ponte não escreve nada.
#[test]
fn sem_vista_nao_escreve() {
    let mut w = World::new();
    let e = w
        .spawn((CharacterAnimations::default(), SpriteAnimator::new("idle")))
        .id();
    anima_personagens(
        &mut w,
        &PhysicsBridge::new(),
        &BTreeSet::new(),
        &SignalOutbox::new(),
    );
    assert_eq!(
        w.get::<SpriteAnimator>(e).map(|a| a.current.as_str()),
        Some("idle")
    );
}

fn regra(when: AnimWhen, play: &str) -> AnimRule {
    AnimRule {
        when,
        play: play.into(),
    }
}

/// As regras do teste: ↓ premida → crouch · levou dano → hurt · sinal «boom» → fall · e as 4 poses.
fn regras() -> Vec<AnimRule> {
    let mut r = vec![
        regra(AnimWhen::Action("down".into()), "crouch"),
        regra(AnimWhen::Life(LifeEvent::Damaged), "hurt"),
        regra(AnimWhen::Signal("boom".into()), "fall"),
    ];
    r.extend(AnimRules::of_character(&CharacterAnimations::default()).0);
    r
}

struct Mundo {
    premidas: BTreeSet<String>,
    vida: Vec<LifeEvent>,
    ouvidos: BTreeSet<String>,
    pose: Option<CharacterPose>,
    agachado: bool,
}

impl Mundo {
    fn parado() -> Self {
        Self {
            premidas: BTreeSet::new(),
            vida: Vec::new(),
            ouvidos: BTreeSet::new(),
            pose: Some(CharacterPose::Run),
            agachado: false,
        }
    }
    fn passo(&self, seguro: &mut Option<Segura>, a: &mut SpriteAnimator) -> Passo {
        let f = Factos {
            pose: self.pose,
            agachado: self.agachado,
            premidas: &self.premidas,
            vida: &self.vida,
            ouvidos: &self.ouvidos,
        };
        let p = decide(&regras(), &f, seguro, a);
        if let Passo::Toca(t) | Passo::Recomeca(t) = &p {
            troca(a, t.clone());
        }
        p
    }
}

/// ⭐ A de cima ganha: ↓ premida vence a pose; soltá-la devolve à regra seguinte que vale.
#[test]
fn a_regra_de_cima_ganha_e_soltar_devolve() {
    let mut a = SpriteAnimator::new("idle");
    let mut seguro = None;
    let mut m = Mundo::parado();
    assert_eq!(m.passo(&mut seguro, &mut a), Passo::Toca("run".into()));
    m.premidas.insert("down".into());
    assert_eq!(m.passo(&mut seguro, &mut a), Passo::Toca("crouch".into()));
    m.premidas.clear();
    assert_eq!(m.passo(&mut seguro, &mut a), Passo::Toca("run".into()));
    m.pose = None;
    assert_eq!(
        m.passo(&mut seguro, &mut a),
        Passo::Fica,
        "nenhuma vale = fica"
    );
}

/// ⭐ «Levou dano» dura um quadro, mas a animação dele toca até fechar uma volta; só uma regra ACIMA
/// a interrompe.
#[test]
fn levar_dano_segura_a_animacao_ate_ao_fim_da_volta() {
    let mut a = SpriteAnimator::new("run");
    let mut seguro = None;
    let mut m = Mundo::parado();
    m.vida.push(LifeEvent::Damaged);
    assert_eq!(m.passo(&mut seguro, &mut a), Passo::Recomeca("hurt".into()));
    m.vida.clear();
    assert_eq!(
        m.passo(&mut seguro, &mut a),
        Passo::Fica,
        "a corrida (abaixo) não a interrompe"
    );
    m.ouvidos.insert("boom".into());
    assert_eq!(
        m.passo(&mut seguro, &mut a),
        Passo::Fica,
        "o sinal (abaixo) também não"
    );
    m.ouvidos.clear();
    a.repeat_count = 1;
    assert_eq!(
        m.passo(&mut seguro, &mut a),
        Passo::Toca("run".into()),
        "acabou a volta: solta"
    );

    m.vida.push(LifeEvent::Damaged);
    m.passo(&mut seguro, &mut a);
    m.vida.clear();
    m.premidas.insert("down".into());
    assert_eq!(
        m.passo(&mut seguro, &mut a),
        Passo::Toca("crouch".into()),
        "a de cima interrompe"
    );
    assert_eq!(seguro, None);
}

/// Um ficheiro antigo (só `CharacterAnimations`): as regras derivadas dão a tag que a ponte de antes dava.
#[test]
fn as_quatro_derivadas_dao_a_tag_de_cada_pose() {
    let c = CharacterAnimations::default();
    let rules = AnimRules::effective(None, Some(&c), |_| false).unwrap();
    let nada = BTreeSet::new();
    for (pose, agachado) in CharacterPose::DERIVED
        .into_iter()
        .flat_map(|p| [(p, false), (p, true)])
    {
        let f = Factos {
            pose: Some(pose),
            agachado,
            premidas: &nada,
            vida: &[],
            ouvidos: &nada,
        };
        let mut a = SpriteAnimator::new("x");
        assert_eq!(
            decide(&rules.0, &f, &mut None, &a),
            Passo::Toca(c.tag(pose).expect("nomeada").to_owned()),
            "{pose:?} agachado={agachado}"
        );
        a.current = c.tag(pose).expect("nomeada").to_owned();
    }
}

/// Plano 10: com as tags do agachar (a folha de fábrica) as derivadas tocam «crouch_walk» a andar agachado,
/// «crouch» parado agachado, e levantar devolve as de pé.
#[test]
fn as_derivadas_com_o_agachar_tocam_as_celulas_dele() {
    let c = CharacterAnimations::default();
    let tags: Vec<&str> = ph2d_render::HERO_SHEET_TAGS
        .iter()
        .map(|t| t.name)
        .collect();
    let rules = AnimRules::effective(None, Some(&c), |n| tags.contains(&n)).unwrap();
    let nada = BTreeSet::new();
    let a = SpriteAnimator::new("idle");
    for (pose, agachado, tag) in [
        (CharacterPose::Run, true, "crouch_walk"),
        (CharacterPose::Idle, true, "crouch"),
        (CharacterPose::Run, false, "run"),
        (CharacterPose::Idle, false, "idle"),
    ] {
        let f = Factos {
            pose: Some(pose),
            agachado,
            premidas: &nada,
            vida: &[],
            ouvidos: &nada,
        };
        assert_eq!(
            decide(&rules.0, &f, &mut None, &a),
            Passo::Toca(tag.into()),
            "{pose:?} agachado={agachado}"
        );
    }
}

/// Plano 09: «Crouching» por cima das poses ganha enquanto agachado — a pose base continua a valer por
/// baixo, e levantar devolve-a.
#[test]
fn agachado_ganha_e_levantar_devolve_a_pose() {
    let mut rules = vec![regra(AnimWhen::Pose(CharacterPose::Crouch), "crouch")];
    rules.extend(AnimRules::of_character(&CharacterAnimations::default()).0);
    let nada = BTreeSet::new();
    let f = |agachado| Factos {
        pose: Some(CharacterPose::Idle),
        agachado,
        premidas: &nada,
        vida: &[],
        ouvidos: &nada,
    };
    let a = SpriteAnimator::new("idle");
    assert_eq!(
        decide(&rules, &f(true), &mut None, &a),
        Passo::Toca("crouch".into())
    );
    assert_eq!(
        decide(&rules, &f(false), &mut None, &a),
        Passo::Toca("idle".into())
    );
}
