//! **UMA ESCADA DE PRANCHAS APERTADA** — as duas bordas da descida (W12/W20).
//!
//! O irmão `platform_drop.rs` mede uma prancha SOLTA com muito espaço embaixo, e
//! é por isso que ele nunca viu isto: ali a caixa inteira do personagem chega a
//! ficar abaixo da prancha. Numa escada apertada ela nunca chega.
//!
//! # A lei que estes gates pinam (W20, fechada na W27)
//!
//! ```text
//!   aposenta  ⇔  estou a SUBIR  ∨  (já passei  ∧  a prancha parou de me pegar)
//! ```
//!
//! As duas últimas metades são obrigatórias e cada uma cura o defeito da outra;
//! a primeira fechou a borda de baixo. O mecanismo, com os números, está no
//! aviso de `bridge::player::retire_drops`.
//!
//! * **a borda de CIMA fechou na W20** — o vão em que a prancha voltava a ser
//!   sólida a meio da queda e o **arremessava de volta** (prancha 0,15, vãos
//!   1,75–1,85) desce um degrau;
//! * **a borda de BAIXO fechou na W27** — e o que a fechou foi medir o preço
//!   dela. O item estava registado como *"as pranchas ficam fantasma"*, que é o
//!   sintoma; medido (`measure_what_an_armed_drop_costs`), o preço era o
//!   personagem descer um degrau e **ficar lá para sempre** (`−0,598 → −0,598` a
//!   1,60, em toda célula da janela). Uma **ARMADILHA**, não um contorno.
//!
//! ⚠️ **A cura NÃO foi a que este aviso prescrevia.** Ele dizia que a saída
//! conhecida era a descida por-PLATAFORMA — que a W21 construiu, mediu (**nenhuma
//! diferença**: quem segura o personagem é a MOLA, e o raio dela já ignora só a
//! plataforma da travessia) e reverteu. A que fechou é uma cláusula de
//! **intenção**: uma descida travada existe para deixar passar para BAIXO, e
//! quem decide na subida já é o **cone** do one-way.
//!
//! ⚠️ **E o gate do fantasma fez exactamente o que foi escrito para fazer:** ele
//! afirmava o DEFEITO com a instrução de ficar vermelho no dia em que a lei
//! mudasse, e ficou — com o número que ele próprio previu. Foi reescrito de
//! propósito, não contornado.

use ph2d_core::Vec2;
use ph2d_ecs::{Name, SimWorld, Transform};
use ph2d_physics_ecs::{
    BodyKind, Collider, ColliderShape, LockRotation, OneWayPlatform, PhysicsBridge, PlatformPlayer,
    PlayerInput, RigidBody,
};

#[path = "platform_scene.rs"]
mod scene_fixture;

use scene_fixture::{FLOAT_HEIGHT, pose};

/// Meia-espessura de uma prancha.
const PLANK_HALF_Y: f32 = 0.1;
/// O topo da prancha de CIMA (a que ele atravessa).
const UPPER_TOP: f32 = PLANK_HALF_Y;
/// Meia-altura da cápsula do personagem (`half_height 0,3 + radius 0,2`).
const BODY_HALF: f32 = 0.5;

fn rest_over(top: f32) -> f32 {
    top + FLOAT_HEIGHT
}

struct Rig {
    sim: SimWorld,
    bridge: PhysicsBridge,
    player: ph2d_ecs::Entity,
}

fn plank(sim: &mut SimWorld, centre_y: f32, half_y: f32, name: &str) {
    sim.world_mut().spawn((
        Name::new(name),
        RigidBody {
            kind: BodyKind::Static,
        },
        Collider {
            shape: ColliderShape::Cuboid {
                half_x: 40.0,
                half_y,
            },
            ..Collider::default()
        },
        Transform::from_translation(Vec2::new(0.0, centre_y)),
        OneWayPlatform,
    ));
}

/// **Duas pranchas jump-through separadas por `gap`**, com o personagem em pé
/// na de cima e um chão sólido bem lá em baixo.
fn ladder(gap: f32) -> Rig {
    ladder_of(gap, PLANK_HALF_Y)
}

fn ladder_of(gap: f32, half: f32) -> Rig {
    let mut sim = SimWorld::new();
    sim.world_mut().spawn((
        Name::new("Floor"),
        RigidBody {
            kind: BodyKind::Static,
        },
        Collider {
            shape: ColliderShape::Cuboid {
                half_x: 40.0,
                half_y: 0.5,
            },
            ..Collider::default()
        },
        Transform::from_translation(Vec2::new(0.0, -6.5)),
    ));
    plank(&mut sim, 0.0, half, "Upper");
    plank(&mut sim, -gap, half, "Lower");

    let player = sim
        .world_mut()
        .spawn((
            Name::new("Player"),
            RigidBody {
                kind: BodyKind::Dynamic,
            },
            Collider {
                shape: ColliderShape::Capsule {
                    half_height: 0.3,
                    radius: 0.2,
                },
                ..Collider::default()
            },
            LockRotation,
            PlatformPlayer {
                float_height: FLOAT_HEIGHT,
                // A quina e a memória do chão fora do caminho: esta cena não tem
                // teto nem plataforma móvel, e desligá-las explicitamente é o
                // que mantém o gate a medir UMA coisa.
                corner_reach: 0.0,
                lift_momentum: 0.0,
                ..PlatformPlayer::default()
            },
            Transform::from_translation(Vec2::new(0.0, rest_over(half))),
        ))
        .id();

    Rig {
        sim,
        bridge: PhysicsBridge::new(),
        player,
    }
}

fn settle(r: &mut Rig, ticks: u64, from: u64) -> u64 {
    let mut t = from;
    for _ in 0..ticks {
        t += 1;
        r.bridge.dispatch(&mut r.sim, true, t);
    }
    t
}

fn press(r: &mut Rig, input: PlayerInput, hold: u64, then: u64, from: u64) -> u64 {
    r.bridge.set_player_input(r.player, input);
    let t = settle(r, hold, from);
    r.bridge.set_player_input(r.player, PlayerInput::default());
    settle(r, then, t)
}

fn down_jump() -> PlayerInput {
    PlayerInput {
        drive: 0.0,
        jump: true,
        down: true,
        dash: false,
        grab: false,
    }
}

fn jump_only() -> PlayerInput {
    PlayerInput {
        drive: 0.0,
        jump: true,
        down: false,
        dash: false,
        grab: false,
    }
}

/// **O personagem CABE numa escada apertada** — a metade da nota antiga que a
/// medição derrubou.
///
/// Vão de 1,20 m: ele sobe 1,40 m acima do apoio, então a cabeça atravessa a
/// prancha de cima. Isso é o idioma de uma jump-through, não uma cena partida —
/// e ele fica **quieto** ali.
#[test]
fn a_short_gap_ladder_is_a_scene_the_character_fits_in() {
    let gap = 1.20_f32;
    let lower_top = -gap + PLANK_HALF_Y;

    let mut r = ladder(gap);
    // Posto no degrau de baixo: é essa a cena que a nota antiga chamava de
    // impossível.
    if let Some(mut t) = r.sim.world_mut().get_mut::<Transform>(r.player) {
        t.translation.y = rest_over(lower_top);
    }
    let mut t = settle(&mut r, 120, 0);
    let (_, settled) = pose(&r.sim);
    assert!(
        (settled - rest_over(lower_top)).abs() < 0.05,
        "ele tem de descansar no degrau de baixo ({:.3}), e esta' em {settled:.3}",
        rest_over(lower_top)
    );
    assert!(
        settled + BODY_HALF > UPPER_TOP,
        "premissa: a cabeca ({:.3}) atravessa a prancha de cima ({UPPER_TOP:.3})",
        settled + BODY_HALF
    );

    // E fica QUIETO — sem tremor, sem afundar.
    let (mut lo, mut hi) = (f32::MAX, f32::MIN);
    for _ in 0..60 {
        t = settle(&mut r, 1, t);
        let (_, y) = pose(&r.sim);
        lo = lo.min(y);
        hi = hi.max(y);
    }
    assert!(
        hi - lo < 0.001,
        "a pose tem de ser estavel, e variou {:.4} m em 60 tiques",
        hi - lo
    );
}

/// **A BORDA DE BAIXO FECHOU: subir APOSENTA a descida, e ele volta.** (W27)
///
/// ⚠️ **Este gate era o inverso dele mesmo, e a reescrita é deliberada** — o
/// aviso do módulo pedia exactamente isto. Ele nascera a afirmar o DEFEITO
/// (*"depois de descer um degrau, um pulo simples não volta a pousar na prancha
/// de cima"*), com a instrução de ficar vermelho no dia em que a lei mudasse; e
/// ficou, com o número que ele próprio previu.
///
/// **O que a medição mostrou, e ela muda a gravidade do item:** o custo do vão
/// preso não era cosmético. Medido (`measure_what_an_armed_drop_costs`), em toda
/// célula da janela o personagem descia e **ficava lá para sempre** —
/// `−0,598 → −0,598` a 1,60, `−0,198 → −0,198` a 1,20. Não é *"as pranchas ficam
/// fantasma"*: é uma **ARMADILHA**.
///
/// **A lei que fechou:** uma descida travada existe para deixar passar **para
/// BAIXO**. No instante em que o corpo SOBE, quem decide já é o **cone** do
/// one-way (`ALLOWED_COS`), que deixa passar por baixo por conta própria —
/// manter o bit ali não protege nada e só o prende. Então o `retire_drops` ganha
/// uma terceira cláusula, em OU com as duas que já tinha:
///
/// ```text
///   aposenta  ⇔  estou a SUBIR  ∨  (já passei  ∧  a prancha parou de me pegar)
/// ```
///
/// ⚠️ **Ela não podia reabrir a borda de CIMA** (o cuspe), e não reabre: aquele
/// defeito é a prancha voltar a ser sólida **com ele a CAIR através dela**, e
/// esta cláusula só dispara com a velocidade para cima. Os gates dessa borda
/// ficam verdes ao lado deste.
///
/// ⚠️ **E o CONTROLE de pé não se moveu:** a tabela do
/// `measure_whether_a_short_gap_is_a_broken_scene` sai **idêntica** antes e
/// depois — inclusive o `+1,002` dos vãos de 1,00 e 0,80, que é a prancha
/// SÓLIDA a cuspi-lo e não tem descida nenhuma envolvida.
#[test]
fn a_short_gap_ladder_lets_him_climb_back_because_rising_retires_the_drop() {
    let gap = 1.20_f32;
    let lower_top = -gap + PLANK_HALF_Y;

    let mut r = ladder(gap);
    let t = settle(&mut r, 30, 0);
    let t = press(&mut r, down_jump(), 4, 120, t);

    let (_, dropped) = pose(&r.sim);
    assert!(
        (dropped - rest_over(lower_top)).abs() < 0.1,
        "premissa: a descida em si funciona, e ele para no degrau de baixo ({:.3}); esta' em {dropped:.3}",
        rest_over(lower_top)
    );

    press(&mut r, jump_only(), 6, 150, t);
    let (_, climbed) = pose(&r.sim);
    assert!(
        (climbed - rest_over(UPPER_TOP)).abs() < 0.1,
        "ele tinha de voltar a pousar na prancha de cima ({:.3}) e parou em \
         {climbed:.3} -- se isto ficou perto de {:.3}, a descida voltou a ser uma \
         ARMADILHA (o vao preso, medido em `measure_what_an_armed_drop_costs`)",
        rest_over(UPPER_TOP),
        rest_over(lower_top)
    );
}

/// **A BORDA DE CIMA FECHOU: o vão que o ARREMESSAVA agora desce um degrau.**
///
/// A lei antiga aposentava a descida assim que a caixa ficava abaixo da prancha,
/// e **isso acontece a meio da queda**: a prancha voltava a ser sólida e o
/// contato o atirava para cima com um pico de **0,3267 N·s** entre sub-passos
/// (`tests/measure_drop_retire.rs`). Numa prancha grossa (meia-espessura 0,15 —
/// a da cena 91) a faixa medida era **1,75 m a 1,85 m**, e ali ele **não descia
/// de todo**.
///
/// ⚠️ **Quem fecha esta borda é o LIVRO-RAZÃO do gancho**, não a geometria: a
/// mutação que remove o `&& !drop_is_catching(..)` do `retire_drops` devolve o
/// arremesso, com a caixa 0,016 m abaixo da prancha e nenhuma sobreposição — que
/// é exactamente o regime em que toda lei geométrica tentada aqui morreu.
#[test]
fn a_thick_plank_ladder_no_longer_throws_him_back() {
    const THICK: f32 = 0.15;
    let gap = 1.80_f32;
    let top = rest_over(THICK);
    let one_rung_down = rest_over(-gap + PLANK_HALF_Y);

    let mut r = ladder_of(gap, THICK);
    let t = settle(&mut r, 30, 0);
    press(&mut r, down_jump(), 4, 130, t);

    let after = pose(&r.sim).1;
    assert!(
        (after - one_rung_down).abs() < 0.1,
        "o vao 1,80 com prancha grossa tem de DESCER um degrau ({one_rung_down:.3}); \
         ele parou em {after:.3} — se isto voltou para {top:.3}, o arremesso voltou"
    );
}

/// **O CONTROLE: com espaço a mesma escada funciona.**
///
/// ⚠️ Sem ele, *"a prancha fica fantasma"* poderia ser lido como *"a retirada
/// nunca funciona"* — e ela funciona, o que é precisamente o que torna o caso
/// apertado um defeito em vez de uma feature ausente. Este gate fica VERDE
/// antes e depois de qualquer cura.
#[test]
fn a_wide_gap_ladder_retires_the_drop_as_it_should() {
    let gap = 2.0_f32;
    let lower_top = -gap + PLANK_HALF_Y;

    let mut r = ladder(gap);
    let t = settle(&mut r, 30, 0);
    let t = press(&mut r, down_jump(), 4, 120, t);
    let (_, dropped) = pose(&r.sim);
    assert!(
        (dropped - rest_over(lower_top)).abs() < 0.1,
        "vao largo: esperado {:.3}, e parou em {dropped:.3}",
        rest_over(lower_top)
    );

    press(&mut r, jump_only(), 6, 150, t);
    let (_, climbed) = pose(&r.sim);
    assert!(
        (climbed - rest_over(UPPER_TOP)).abs() < 0.1,
        "vao largo: a prancha de cima TEM de voltar a ser chao ({:.3}), e ele parou em {climbed:.3}",
        rest_over(UPPER_TOP)
    );
}

/// **A LEI da EVIDÊNCIA: EM REPOUSO, a descida sobrevive exactamente onde a
/// caixa de repouso ainda sobrepõe a prancha.**
///
/// Não é uma faixa de números escolhida — é um bicondicional, e ele vale célula
/// a célula nas duas espessuras varridas. É isto que torna o que resta
/// **honesto** em vez de arbitrário: onde ele acontece, a prancha de facto
/// pegaria o personagem (o cone do gancho devolve `+1,000`, medido).
///
/// ⚠️ **"EM REPOUSO" é a premissa, e ela ficou load-bearing na W27:** a cláusula
/// da INTENÇÃO aposenta a descida no instante em que o corpo SOBE, então este
/// bicondicional descreve o personagem **parado** — que é o estado em que ele foi
/// medido, e o único em que a pergunta *"a prancha me pegaria?"* tem uma resposta
/// que não depende de para onde ele vai. O gate irmão
/// [`a_short_gap_ladder_lets_him_climb_back_because_rising_retires_the_drop`]
/// mede o outro estado, e os dois passam ao mesmo tempo.
///
/// ⚠️ **É também o gate que impede a lei de regredir para os DOIS lados:**
/// aposentar cedo demais reprova nas células que sobrepõem (volta o arremesso),
/// e aposentar tarde demais reprova nas que não sobrepõem (volta o fantasma
/// largo).
#[test]
fn the_drop_survives_exactly_where_the_resting_box_still_overlaps_the_plank() {
    for (thick, gap) in [
        (0.15_f32, 1.50_f32),
        (0.15, 1.60),
        (0.15, 1.70),
        (0.15, 1.75),
        (0.15, 1.90),
        (0.10, 1.10),
        (0.10, 1.20),
        (0.10, 1.60),
        (0.10, 1.65),
        (0.10, 2.00),
    ] {
        let mut r = ladder_of(gap, thick);
        let t = settle(&mut r, 30, 0);
        press(&mut r, down_jump(), 4, 150, t);

        let rest = pose(&r.sim).1;
        let expected = rest_over(-gap + thick);
        assert!(
            (rest - expected).abs() < 0.1,
            "premissa: ph {thick:.2} vao {gap:.2} tem de descer UM degrau \
             ({expected:.3}); parou em {rest:.3}"
        );
        // A caixa de repouso sobrepõe a prancha de cima?
        let overlaps = rest + BODY_HALF > -thick;
        let ghost = r.bridge.player_is_dropping(r.player);
        assert_eq!(
            ghost,
            overlaps,
            "ph {thick:.2} vao {gap:.2}: repouso {rest:.3} (topo da caixa \
             {:.3}, base da prancha {:.3}) — sobrepoe {overlaps}, mas a descida \
             viva e' {ghost}",
            rest + BODY_HALF,
            -thick
        );
    }
}
