//! **O PERSONAGEM CINEMÁTICO NA ÁGUA** — os gates da W-KinFluid.
//!
//! Report do Enio (2026-08-09): *"testou o kinemátic na água? ou ele não
//! funciona lá?"*. Não funcionava: medido, numa poça funda de 4 s o controle
//! boiava `1,1072 m` acima do ponto de largada, o player dinâmico `1,0893`, e o
//! **cinemático afundava `139,67 m`** — queda livre, com o multiplicador de
//! queda por cima.
//!
//! # ⚠️ O ORÁCULO é o modo DINÂMICO, e a primeira versão dele estava errada
//!
//! O primeiro número que eu reportei foi *"assenta a `0,4140`, três milímetros do
//! dinâmico"* — e ele era **um instante de uma oscilação**: com a poça da fixture
//! o player BOBEIA `1,44 m` de amplitude entre o terceiro e o sexto segundo, e o
//! `t = 4 s` calhou de o apanhar perto do dinâmico. *Uma amostra única de um
//! sistema que oscila não é um repouso.*
//!
//! ⚠️ **E a oscilação não é desta wave:** o corpo DINÂMICO faz `1,4408` na mesma
//! cena contra os `1,4394` do cinemático — eles concordam na quarta decimal, e a
//! cápsula solta (sem lei de player nenhuma) faz `0,8097`. O que bobeia é o
//! *player* na água, nos dois modos, e isso é anterior a esta wave.
//!
//! ⚠️ **E esse `1,44` ficou nomeado como PENDÊNCIA — medido, não é uma.** O
//! excesso inteiro é a modelagem do arco a agir **no AR**, antes do primeiro
//! contacto: com os quatro multiplicadores a `1` a amplitude é o controle ao
//! quarto decimal, e largado JÁ SUBMERSO ele é `1,00×` o controle. Os dois gates
//! do fundo deste arquivo pinam isso; os números e as ablações estão no plano 07
//! §8.4 e na sonda `measure_the_bobbing`.
//!
//! Por isso o oráculo é a **PARIDADE ENTRE MODOS**: a mesma cena, o mesmo tique,
//! e a espécie do corpo a não ser uma pergunta que a água faça. Um literal de
//! linha d'água seria um espelho da fórmula; a paridade é uma propriedade.
//!
//! Rodar: `cargo test -p ph2d-physics-ecs --release --test player_in_water`

use ph2d_core::Vec2;
use ph2d_ecs::{Entity, Name, SimWorld, Transform};
use ph2d_physics_ecs::{
    AreaBuoyancy, AreaDrag, BodyKind, Collider, ColliderShape, LockRotation, PhysicsBridge,
    PlatformPlayer, PlayerInput, PlayerMode, RigidBody,
};

/// A cápsula das fixtures do player, e a poça 4× mais densa que ela.
const HALF_H: f32 = 0.3;
const RADIUS: f32 = 0.2;
const FLOAT: f32 = 0.9;
const FLUID: f32 = 4.0;
const START: f32 = 1.5;

/// A poça funda: nada ao alcance do sensor de chão.
///
/// ⚠️ **O arrasto não é enfeite** — empuxo sem resistência é uma mola sem
/// amortecimento, e a fixture irmã já carregava essa frase. Medido nesta cena:
/// com `AreaDrag 0` a amplitude do cinemático na segunda metade é **2,90 m**,
/// contra **1,44** com o `0,6` daqui.
fn pool(sim: &mut SimWorld, drag: f32) {
    let mut e = sim.world_mut().spawn((
        Name::new("Pool"),
        RigidBody {
            kind: BodyKind::Static,
        },
        Collider {
            is_sensor: true,
            shape: ColliderShape::Cuboid {
                half_x: 20.0,
                half_y: 3.0,
            },
            ..Collider::default()
        },
        AreaBuoyancy(FLUID),
        Transform::from_translation(Vec2::new(0.0, -3.0)),
    ));
    if drag > 0.0 {
        e.insert(AreaDrag(drag));
    }
}

fn player(sim: &mut SimWorld, kinematic: bool) -> Entity {
    let mut e = sim.world_mut().spawn((
        Name::new("Subject"),
        RigidBody {
            kind: if kinematic {
                BodyKind::Kinematic
            } else {
                BodyKind::Dynamic
            },
        },
        Collider {
            shape: ColliderShape::Capsule {
                half_height: HALF_H,
                radius: RADIUS,
            },
            density: 1.0,
            ..Collider::default()
        },
        LockRotation,
        PlatformPlayer {
            float_height: FLOAT,
            ..PlatformPlayer::default()
        },
        Transform::from_translation(Vec2::new(0.0, START)),
    ));
    if kinematic {
        e.insert(PlayerMode::Kinematic);
    }
    e.id()
}

fn y_of(sim: &SimWorld) -> f32 {
    let mut q = sim.world().try_query::<(&Name, &Transform)>().unwrap();
    for (n, t) in q.iter(sim.world()) {
        if n.as_str() == "Subject" {
            return t.translation.y;
        }
    }
    panic!("o sujeito tem de existir");
}

/// A média e a amplitude de `y` na **segunda metade** de seis segundos.
///
/// ⚠️ A primeira metade contém a entrada na água, que não é oscilação — medi-la
/// junto misturaria o transiente com o regime.
fn settle_stats(kinematic: bool, drag: f32) -> (f32, f32) {
    let mut sim = SimWorld::new();
    pool(&mut sim, drag);
    let who = player(&mut sim, kinematic);
    let mut bridge = PhysicsBridge::new();
    bridge.set_player_input(who, PlayerInput::default());
    let (mut lo, mut hi, mut sum, mut n) = (f32::MAX, f32::MIN, 0.0f64, 0u32);
    for t in 1..=360u64 {
        bridge.dispatch(&mut sim, true, t);
        if t > 180 {
            let y = y_of(&sim);
            lo = lo.min(y);
            hi = hi.max(y);
            sum += f64::from(y);
            n += 1;
        }
    }
    #[allow(clippy::cast_possible_truncation)]
    ((sum / f64::from(n)) as f32, hi - lo)
}

/// **A ÁGUA EXISTE PARA O MODO CINEMÁTICO, e ela faz com ele o que faz com o
/// dinâmico** — o gate que a wave serve.
///
/// ⚠️ **Nasceu VERMELHO com `−138,17` contra `+0,41`.** O empuxo e o arrasto de
/// uma zona chegam por `apply_impulse` no corpo, e um corpo cinemático tem massa
/// infinita para o solver: é a mesma frase que explicava por que o `move_shape`
/// não empurrava um caixote antes da W-KinPush, agora do outro lado — ele não
/// RECEBIA.
///
/// ⚠️ **As duas metades são precisas:** a média diz *onde ele vive* e a amplitude
/// diz *como ele se move*. Só a média deixaria passar uma lei que o pusesse no
/// lugar certo por dois caminhos errados que se cancelam; só a amplitude deixaria
/// passar uma que o afogasse suavemente.
#[test]
fn the_kinematic_player_floats_like_the_dynamic_one() {
    let (dyn_mean, dyn_amp) = settle_stats(false, 0.6);
    let (kin_mean, kin_amp) = settle_stats(true, 0.6);

    assert!(
        (kin_mean - dyn_mean).abs() < 0.1,
        "o cinemático tem de viver onde o dinâmico vive: {kin_mean:.4} contra {dyn_mean:.4}"
    );
    assert!(
        (kin_amp - dyn_amp).abs() < 0.1,
        "e mover-se como ele: amplitude {kin_amp:.4} contra {dyn_amp:.4}"
    );
    // E o número absoluto, para o gate não passar com os DOIS afogados.
    assert!(
        kin_mean > 0.0,
        "os dois têm de estar ACIMA da superfície (y = 0), e o cinemático está em {kin_mean:.4}"
    );
}

/// **CONTROLE: sem poça, ele CAI** — o gate acima não pode passar sobre um
/// personagem que simplesmente parou de obedecer à gravidade.
///
/// ⚠️ A ablação é pela ENTRADA (a poça deixa de existir), nunca por
/// instrumentação: é a mesma cena, o mesmo corpo, o mesmo relógio.
#[test]
fn without_a_pool_the_kinematic_player_still_falls() {
    let mut sim = SimWorld::new();
    let who = player(&mut sim, true);
    let mut bridge = PhysicsBridge::new();
    bridge.set_player_input(who, PlayerInput::default());
    for t in 1..=240u64 {
        bridge.dispatch(&mut sim, true, t);
    }
    let y = y_of(&sim);
    // Queda livre de 4 s são ~78 m de gravidade nominal; o multiplicador de
    // queda da lei do pulo a aumenta. O que se afirma é a ORDEM, não o número.
    assert!(
        y < START - 50.0,
        "sem água ele tem de cair, e caiu só até {y:.4}"
    );
}

/// Como a [`settle_stats`], mas largando de onde se pedir e podendo tirar a lei
/// do player do caminho (`law = false` ⇒ o CONTROLE, uma cápsula solta).
fn stats_from(start: f32, law: bool, kinematic: bool) -> (f32, f32) {
    let mut sim = SimWorld::new();
    pool(&mut sim, 0.6);
    let mut e = sim.world_mut().spawn((
        Name::new("Subject"),
        RigidBody {
            kind: if kinematic {
                BodyKind::Kinematic
            } else {
                BodyKind::Dynamic
            },
        },
        Collider {
            shape: ColliderShape::Capsule {
                half_height: HALF_H,
                radius: RADIUS,
            },
            density: 1.0,
            ..Collider::default()
        },
        LockRotation,
        Transform::from_translation(Vec2::new(0.0, start)),
    ));
    if law {
        e.insert(PlatformPlayer {
            float_height: FLOAT,
            ..PlatformPlayer::default()
        });
    }
    if kinematic {
        e.insert(PlayerMode::Kinematic);
    }
    let who = e.id();
    let mut bridge = PhysicsBridge::new();
    if law {
        bridge.set_player_input(who, PlayerInput::default());
    }
    let (mut lo, mut hi, mut sum, mut n) = (f32::MAX, f32::MIN, 0.0f64, 0u32);
    for t in 1..=360u64 {
        bridge.dispatch(&mut sim, true, t);
        if t > 180 {
            let y = y_of(&sim);
            lo = lo.min(y);
            hi = hi.max(y);
            sum += f64::from(y);
            n += 1;
        }
    }
    #[allow(clippy::cast_possible_truncation)]
    ((sum / f64::from(n)) as f32, hi - lo)
}

/// **A TRAVA DO FLUIDO CONTÉM O ARCO — um corpo largado JÁ DENTRO da água é a
/// cápsula solta, ao dígito.**
///
/// # ⚠️ Este gate existe porque a nota aberta era FALSA
///
/// A W-KinFluid deixou escrito, como pendência, que *"o player bobeia 1,44 m nos
/// dois modos e a cápsula solta faz 0,81"* — lido como um defeito por fechar.
/// Medido (`measure_the_bobbing`), ele é o **transiente de uma entrada mais
/// rápida**, e as duas ablações independentes dizem a mesma coisa:
///
/// | ablação | amplitude | vs controle |
/// |---|---|---|
/// | player default, largado de `+1,5` (no ar) | `1,4408` | `1,78×` |
/// | os quatro multiplicadores a `1` | `0,8097` | **`1,00×`** (o controle ao 4.º decimal) |
/// | largado de `−0,5` (já submerso) | `0,8326` | **`1,00×`** |
/// | largado de `−1,5` (submerso fundo) | `3,3214` | **`0,99×`** |
///
/// Ou seja: **a modelagem do arco actua no AR, que é onde ela é autorada para
/// actuar**, e a trava a cala no instante em que o fluido toma o corpo. O
/// personagem entra na água a `1,299×` a velocidade do controle (`1,687×` de
/// energia) porque `fall_gravity = 2.0` — e isso não é um defeito, é a queda
/// pesada que o artista pediu, a chegar à água com o momento que ela dá.
///
/// ⚠️ **O gate larga SUBMERSO de propósito:** é a única largada em que a trava
/// arma no tique 1, logo a única que isola *a trava contém* de *a entrada foi
/// mais rápida*. Um gate largado no ar mediria a soma das duas e não poderia
/// falhar pelo motivo que alega.
///
/// ⚠️ **E os três gates que já viviam neste arquivo ficam VERDES nas duas
/// mutações abaixo** — o de paridade entre modos por construção, porque a trava
/// é comum aos dois e uma razão entre dois doentes não a vê. Era esse o buraco.
///
/// | mutação | amplitude aos 30 s |
/// |---|---|
/// | a trava não cala (`extra = scale − 1`) | **857 m** — sai de quadro |
/// | a fração instantânea em vez da trava | **15,3 m**, a crescer |
///
/// A segunda é a alternativa que o doc da `waterborne` já REJEITAVA com números
/// (*"a energia é ganha entre dois mergulhos, onde não há fluido nenhum a
/// medir"*) — as duas sangram os dois gates desta wave.
#[test]
fn the_water_lock_contains_the_arc_shaping() {
    for start in [-0.5f32, -1.5] {
        let (c_mean, c_amp) = stats_from(start, false, false);
        let (p_mean, p_amp) = stats_from(start, true, false);
        assert!(
            (p_amp - c_amp).abs() < 0.1 * c_amp,
            "largado submerso de {start} a lei não pode acrescentar oscilação: \
             {p_amp:.4} contra {c_amp:.4} do controle"
        );
        assert!(
            (p_mean - c_mean).abs() < 0.1,
            "nem mudar onde ele vive: {p_mean:.4} contra {c_mean:.4}"
        );
    }
}

/// **E O BOBEIO DECAI — ele não bombeia.**
///
/// A modelagem do arco é não-conservativa por construção (o doc de
/// [`ph2d_platformer::JumpState::waterborne`] tem a aritmética), e é isso que a
/// trava existe para conter. **Uma janela só não distingue** um transiente de
/// uma bomba — as duas podem medir `1,44` no mesmo instante; o que as separa é a
/// SEQUÊNCIA.
///
/// Medido em 30 s, amplitude por janela de 3 s:
///
/// * controle `1,927 · 0,810 · 0,329 · 0,139 · 0,059 · 0,021 · 0,009 · 0,004 · 0,002 · 0,001`
/// * player  `2,172 · 1,441 · 0,594 · 0,221 · 0,093 · 0,039 · 0,017 · 0,006 · 0,003 · 0,001`
///
/// As duas caem monotonicamente e **convergem no mesmo valor**. O modo de falha
/// que este gate compra é o que o `Buoyed` documenta com números do produto
/// (`−1,05 / +4,71 / +12,08 / −20,31`, e o personagem sai de quadro).
#[test]
fn the_bobbing_decays_it_does_not_pump() {
    let mut sim = SimWorld::new();
    pool(&mut sim, 0.6);
    let who = player(&mut sim, false);
    let mut bridge = PhysicsBridge::new();
    bridge.set_player_input(who, PlayerInput::default());

    let mut windows = Vec::new();
    let (mut lo, mut hi) = (f32::MAX, f32::MIN);
    for t in 1..=1800u64 {
        bridge.dispatch(&mut sim, true, t);
        let y = y_of(&sim);
        lo = lo.min(y);
        hi = hi.max(y);
        if t % 180 == 0 {
            windows.push(hi - lo);
            lo = f32::MAX;
            hi = f32::MIN;
        }
    }

    // ⚠️ A comparação é entre janelas NÃO-ADJACENTES: o transiente da entrada
    // faz a 1.ª janela ser menor que a 2.ª (ele ainda está a cair no ar durante
    // parte dela), e exigir monotonia estrita ali afirmaria algo que a física
    // não promete. O que se afirma é que a energia SAI do sistema.
    for w in windows.windows(2).skip(1) {
        assert!(
            w[1] < w[0],
            "a amplitude tem de decair depois do transiente, e foi {:.4} → {:.4}: \
             a modelagem do arco está a bombear energia dentro do fluido. Janelas: {windows:?}",
            w[0],
            w[1]
        );
    }
    let last = *windows.last().expect("dez janelas");
    assert!(
        last < 0.01,
        "e ao fim de 30 s ele tem de estar parado, e a última janela mede {last:.4}"
    );
}

/// **A PARIDADE DE ARRASTO ENTRE OS MODOS VALE `1,15%`, E ELA DECAI.**
///
/// O solver amortece por SUB-PASSO e a lei cinemática uma vez por TIQUE —
/// `(1+d·h)⁻⁴` contra `(1+d·4h)⁻¹` —, e o plano 07 precificava isso por
/// **analogia** (*"a mesma classe que a W-AreaDrag mediu em 1,25%"*). Medido
/// nesta paridade (`measure_the_bobbing`), numa zona de arrasto **puro**:
///
/// | t | divergência relativa |
/// |---|---|
/// | 1 s | **1,149%** (o pico) |
/// | 2 s | 0,257% |
/// | 3 s | 0,056% |
/// | 4 s | 0,018% |
///
/// ⚠️ **A velocidade terminal não serve de oráculo** — ela é `g/d` nos dois por
/// álgebra, então um gate nela seria verde por construção e cego à divergência
/// inteira. O que se afirma é o **PICO**, que é onde ela vive.
///
/// ⚠️ **E o arrasto é PURO de propósito:** com empuxo a oscilação é uma ordem de
/// grandeza maior que este sinal e afogá-lo-ia.
#[test]
fn the_drag_parity_between_modes_stays_within_its_measured_price() {
    let mut runs = Vec::new();
    for kinematic in [false, true] {
        let mut sim = SimWorld::new();
        sim.world_mut().spawn((
            Name::new("Pool"),
            RigidBody {
                kind: BodyKind::Static,
            },
            Collider {
                is_sensor: true,
                shape: ColliderShape::Cuboid {
                    half_x: 20.0,
                    half_y: 40.0,
                },
                ..Collider::default()
            },
            AreaDrag(0.6),
            Transform::from_translation(Vec2::new(0.0, -40.0)),
        ));
        let who = player(&mut sim, kinematic);
        let mut bridge = PhysicsBridge::new();
        bridge.set_player_input(who, PlayerInput::default());
        let mut ys = Vec::new();
        for t in 1..=240u64 {
            bridge.dispatch(&mut sim, true, t);
            if t % 60 == 0 {
                ys.push(y_of(&sim));
            }
        }
        runs.push(ys);
    }

    let mut worst = 0.0f32;
    for (a, b) in runs[0].iter().zip(runs[1].iter()) {
        let travelled = (START - a).abs();
        assert!(travelled > 1.0, "o sujeito tem de ter caído: {travelled:.4}");
        worst = worst.max((b - a).abs() / travelled);
    }
    // Medido `1,149%` no pico; o teto traz a folga e nada mais.
    assert!(
        worst < 0.02,
        "a divergência de arrasto entre os modos tem de ficar no preço medido, e foi {:.3}%",
        worst * 100.0
    );
}

/// **O MEIO é o que o freia — e o gate mede a ablação do knob do artista.**
///
/// ⚠️ Sem o arrasto o empuxo é um oscilador conservativo: a amplitude não decai,
/// porque não há nada que consuma energia. Medido nesta fixture: `2,90 m` com
/// `AreaDrag 0` contra `1,44` com `0,6` — e a diferença é o termo que a lei
/// integra, não uma propriedade do solver (que nem toca num corpo cinemático).
#[test]
fn the_mediums_drag_is_what_damps_the_bobbing() {
    let (_, loose) = settle_stats(true, 0.0);
    let (_, damped) = settle_stats(true, 0.6);
    assert!(
        loose > damped * 1.5,
        "sem arrasto ele tem de oscilar bem mais: {loose:.4} contra {damped:.4}"
    );
}
