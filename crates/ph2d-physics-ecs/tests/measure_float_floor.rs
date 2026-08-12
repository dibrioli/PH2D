//! **ONDE A PERNA FICA CURTA DEMAIS** — o piso geométrico da altura de
//! flutuação, e por que ele NÃO é uma constante.
//!
//! A sonda do repouso (`measure_idle`) mede, numa rampa de 30°, o personagem a
//! **descer 0,59 m em 10 s com amplitude de 0,27 m** — no `float_height = 0,50`,
//! que é o **default que shipa**. De 0,60 para cima ela mede 0,0000 em tudo.
//!
//! A suspeita tem forma fechada e é o que esta sonda existe para confirmar ou
//! matar: o pé de uma cápsula fica mais longe da origem do raio quando a
//! superfície se inclina, então o mínimo geométrico **cresce com a rampa** —
//!
//! ```text
//! float_min(θ) = half_height + radius / cos θ
//! ```
//!
//! — e a lane do `physics_ecs_c9` já traz esse número escrito à mão para a rampa
//! dela (*"o piso geométrico desta cápsula com folga (`half + radius / cos 45°`
//! ≈ 0,82): sem ela o personagem nasceria tangente e a rampa o faria penetrar"*).
//! O que ninguém tinha perguntado é se o **DEFAULT** o respeita.
//!
//! # O que ela mediu (2026-08-09), e o veredito é *não há defeito*
//!
//! ```text
//!   rampa   previsto   onset medido   |  no default 0,50
//!      10°     0.5031        0.5100   |  deriva  +0.0200  amp 0.0035
//!      20°     0.5128        0.5200   |  deriva  +0.0393  amp 0.0134
//!      30°     0.5309        0.5400   |  deriva  -0.6823  amp 0.3406
//!      40°     0.5611        0.5700   |  deriva  -5.1431  amp 3.3004
//!      45°     0.5828        0.5900   |  deriva  -9.5340  amp 6.7303
//! ```
//!
//! O onset segue a previsão com **um passo de varredura** de folga em toda
//! linha ⇒ o piso é geométrico, confirmado. E o **gesto do artista já o
//! honra**: `apply_player_edit(Add)` chama `RideConfig::min_float_height` e
//! multiplica por **1,2**, o que a 45° dá `0,6994` contra o onset de `0,59` —
//! **19% de folga**, medida em vez de escolhida.
//!
//! ⚠️ **A forma da falha abaixo do piso é o que torna essa margem
//! load-bearing, e ela não estava escrita em lugar nenhum:** não é uma deriva
//! que cresce devagar — a 45° o personagem **cai 9,5 m oscilando 6,7 m**. Um
//! piso que se erra por um centímetro não custa um centímetro.
//!
//! ⚠️ **E o `RideConfig::STARTING_POINT` fica em `0,50` de propósito** — ele é
//! o ponto de partida do MODELO, e quem o veste com a geometria de um corpo
//! concreto é o gesto. A linha `float 0,50` da tabela de rampa do
//! `measure_idle` mostra a patologia sem dizer isso, e foi ela que me mandou
//! nesta caçada: *um número medido fora do piso do seu próprio domínio lê como
//! defeito de produto*.
//!
//! ⚠️ Esta sonda **não conserta nada** — ela mede pelas portas do produto, para
//! que a decisão venha de um número.
//!
//! Rodar: `cargo test -p ph2d-physics-ecs --release --test measure_float_floor
//! -- --ignored --nocapture`

#[path = "platform_scene.rs"]
mod scene_fixture;

use ph2d_core::Vec2;
use ph2d_ecs::{Entity, Name, SimWorld, Transform};
use ph2d_physics_ecs::{
    BodyKind, Collider, ColliderShape, LockRotation, PhysicsBridge, PlatformPlayer, RigidBody,
};
use scene_fixture::pose;

/// A cápsula do player em toda cena e todo gate deste módulo.
const HALF_HEIGHT: f32 = 0.3;
const RADIUS: f32 = 0.2;
/// O afastamento dos pés de fora, em fração da meia-largura — o default do
/// produto, lido da porta e não copiado.
const FOOT_SPREAD: f32 = ph2d_platformer::RideConfig::STARTING_POINT.spread;

/// O piso geométrico previsto: onde o PÉ da cápsula toca a superfície inclinada.
///
/// ⚠️ **Ela PERGUNTA à porta do produto** (`RideConfig::min_float_height`) em vez
/// de reescrever a forma fechada — e a 1ª versão deste arquivo a reescrevia, o
/// que fazia do gate um espelho de uma SEGUNDA cópia: ele continuaria verde no
/// dia em que a fórmula do produto se movesse, que é precisamente o dia que ele
/// existe para pegar. A forma fechada fica no doc do módulo, para o leitor.
fn predicted_floor(slope_deg: f32) -> f32 {
    ph2d_platformer::RideConfig::min_float_height(HALF_HEIGHT, RADIUS, slope_deg.to_radians().cos())
}

fn rig(slope_deg: f32, float_height: f32, feet: u16) -> (SimWorld, PhysicsBridge, Entity) {
    let slope = slope_deg.to_radians();
    let mut sim = SimWorld::new();
    sim.world_mut().spawn((
        Name::new("Ramp"),
        RigidBody {
            kind: BodyKind::Static,
        },
        Collider {
            shape: ColliderShape::Cuboid {
                half_x: 40.0,
                half_y: 0.25,
            },
            ..Collider::default()
        },
        Transform {
            rotation: slope,
            ..Transform::from_translation(Vec2::new(0.0, 0.0))
        },
    ));
    let who = sim
        .world_mut()
        .spawn((
            Name::new("Player"),
            RigidBody {
                kind: BodyKind::Dynamic,
            },
            Collider {
                shape: ColliderShape::Capsule {
                    half_height: HALF_HEIGHT,
                    radius: RADIUS,
                },
                ..Collider::default()
            },
            LockRotation,
            PlatformPlayer {
                float_height,
                foot_samples: feet,
                ..PlatformPlayer::default()
            },
            Transform::from_translation(Vec2::new(0.0, 0.25 + float_height + 0.4)),
        ))
        .id();
    let bridge = PhysicsBridge::new();
    (sim, bridge, who)
}

/// O mesmo, com a contagem de raios da perna dita — é ela que move o piso —,
/// e com a FOLGA PERPENDICULAR assentada, que é o que separa *flutuar* de
/// *estar deitado na rampa*.
fn idle_with(slope_deg: f32, float_height: f32, feet: u16) -> (f32, f32, f32) {
    let (mut sim, mut bridge, who) = rig(slope_deg, float_height, feet);
    // Assenta.
    for t in 1..=120u64 {
        bridge.dispatch(&mut sim, true, t);
    }
    let p0 = pose(&sim);
    let (mut lo, mut hi) = (f32::MAX, f32::MIN);
    for t in 121..=720u64 {
        bridge.dispatch(&mut sim, true, t);
        let y = pose(&sim).1;
        lo = lo.min(y);
        hi = hi.max(y);
    }
    let p1 = pose(&sim);
    let _ = who;
    let (c, sn) = (slope_deg.to_radians().cos(), slope_deg.to_radians().sin());
    let along = (p1.0 - p0.0) * c + (p1.1 - p0.1) * sn;
    // Distância PERPENDICULAR do centro à face de cima da rampa (meia-espessura
    // 0,25, e ela passa pela origem).
    let perp = p1.1 * c - p1.0 * sn - 0.25;
    (along, hi - lo, perp)
}

/// **Onde a patologia ACABA**, perguntado ao simulador em passos de 1 cm.
///
/// ⚠️ **A varredura começa em 0,20 e não em 0,45, e isso é o `W-FootFan` a
/// morder:** uma varredura não consegue ver um piso ABAIXO do início dela — ela
/// devolve o próprio início e passa por medição. Com a perna em leque o piso de
/// uma rampa desce, e a 20° ele passou para debaixo do 0,45 de antes.
fn onset(slope_deg: f32, feet: u16) -> f32 {
    let c = slope_deg.to_radians().cos();
    // O SUPORTE da cápsula na direção normal da rampa: abaixo disto ela está
    // ENCOSTADA. (Vertical, é o `half + radius / cos θ` do doc — a mesma
    // grandeza, medida no eixo em que ela é geometria da cápsula e mais nada.)
    let touching = HALF_HEIGHT * c + RADIUS + 1.0e-3;
    let mut f = 0.20f32;
    while f <= 0.9005 {
        let (drift, amp, perp) = idle_with(slope_deg, f, feet);
        if drift.abs() < 1.0e-3 && amp < 1.0e-3 && perp > touching {
            return f;
        }
        f += 0.01;
    }
    f32::NAN
}

/// **A fronteira, varrida finamente** — e a previsão ao lado dela.
#[test]
#[ignore = "sonda"]
fn measure_where_the_leg_runs_short() {
    println!("\n=== O PISO DA PERNA: onde a rampa comeca a bater na capsula ===");
    println!("capsula half {HALF_HEIGHT} / radius {RADIUS}; previsao = half + radius/cos(theta)\n");
    println!("  rampa   previsto   |  onset n=1   onset n=3   |  desce  |  0,2*tan(theta)");
    for slope in [0.0f32, 10.0, 20.0, 30.0, 40.0, 45.0] {
        let pred = predicted_floor(slope);
        let one = onset(slope, 1);
        let fan = onset(slope, 3);
        // A previsao do quanto o leque baixa o piso: o pe' de CIMA acha chao
        // `meia-largura * spread * tan(theta)` mais perto.
        let lift = RADIUS * FOOT_SPREAD * slope.to_radians().tan();
        println!(
            "   {slope:>4.0}°   {pred:>8.4}   |   {one:>8.4}    {fan:>8.4}   | \
             {:>7.4} |  {lift:>7.4}",
            one - fan
        );
    }
    println!(
        "\nLEITURA: se a coluna do meio SEGUIR a previsao, o piso e' geometrico\n\
         e o default de 0,50 (`RideConfig::STARTING_POINT`) fica ABAIXO dele em\n\
         toda rampa que o limite autorado de 45 graus permite.\n"
    );
}

/// **O PISO PREVISTO É O PISO QUE O SIMULADOR MOSTRA** — a fórmula é medida
/// contra o comportamento, não contra a tabela dela mesma.
///
/// ⚠️ **Por que este gate não é redundante com o `ride_tests`:** aquele confere
/// `min_float_height` contra a **tabela do próprio doc** — um espelho da
/// fórmula, que continuaria verde se as duas se movessem juntas. Este pergunta
/// ao **SIMULADOR**, que não conhece fórmula nenhuma.
///
/// ⚠️ **E a 1ª versão dele não tinha dentes, com este mesmo nome:** ela afirmava
/// que `piso × 1,2` fica QUIETO, e a margem de 20% **absorve a fórmula errada**
/// — com o piso colapsado numa constante (sem o `cos`), a 45° ele dá
/// `0,50 × 1,2 = 0,60` contra um onset de `0,59`, e o gate passava. Um gate cujo
/// nome fala de PISO tem de medir o piso: o que ele afirma agora é que o
/// **onset do simulador SEGUE a fórmula**, varrido.
#[test]
fn the_predicted_floor_is_the_floor_the_simulator_shows() {
    // Um passo de varredura (0,01) mais o viés medido (~0,008) mais folga.
    const TOL: f32 = 0.03;
    // A margem que o gesto do artista aplica (`fitted_float`, na shell).
    const MARGIN: f32 = 1.2;

    for slope in [20.0f32, 30.0, 45.0] {
        let floor = predicted_floor(slope);

        // ⚠️ **UM raio, porque é isso que a fórmula descreve.** Com a perna em
        // leque (`W-FootFan`, o default de hoje) o piso DESCE — e é o gate
        // seguinte que afirma isso. Medir o leque contra esta fórmula seria
        // pedir-lhe que descrevesse uma perna que ela não conhece.
        let onset = onset(slope, 1);
        assert!(
            onset.is_finite(),
            "a fixture tem de CONTER o fenomeno: a {slope}° a varredura nao \
             achou onde a patologia acaba"
        );
        assert!(
            (onset - floor).abs() <= TOL,
            "a {slope}° o simulador para de brigar em {onset:.4} e a formula do \
             produto prevê {floor:.4} — a formula deixou de descrever o piso"
        );
        // E a margem do gesto o limpa: é ela que faz de um personagem novo um
        // personagem quieto, e o número dela é medido em vez de escolhido.
        assert!(
            floor * MARGIN > onset,
            "a {slope}° a margem de {MARGIN} sobre o piso ({:.4}) nao limpa o \
             onset medido ({onset:.4})",
            floor * MARGIN
        );
    }
}

/// **O LEQUE BAIXA O PISO, e a fórmula do produto fica CONSERVADORA** — o
/// `W-FootFan` a 2026-08-11.
///
/// ⚠️ **Isto é o §0 a morder:** `RideConfig::min_float_height` descreve o piso de
/// uma perna que casta **UM raio no centro**, e essa premissa deixou de ser a do
/// produto. Com o leque o pé de CIMA acha chão `meia-largura × tan θ` mais
/// perto, o corpo cavalga essa altura acima do que cavalgava, e o piso desce a
/// mesma coisa:
///
/// ```text
///   rampa    formula   n=1 medido   n=3 medido   desce   r·tan θ
///     20°     0.5128       0.5200       0.4500  0.0700    0.0728
///     30°     0.5309       0.5400       0.4200  0.1200    0.1155
///     45°     0.5828       0.5900       0.3900  0.2000    0.2000
/// ```
///
/// ⚠️ **A fórmula NÃO foi mudada, e a decisão está nomeada:** ela é lida pelo
/// rótulo do gesto (*"Fit to Collider (needs > X m)"*) e pelo `fitted_float`, que
/// ainda aplica margem de 1,2 — então o que ela custa hoje é um personagem novo
/// que **paira mais alto do que precisaria** (0,70 contra 0,46 a 45°). Isso é
/// seguro e nada regride: o valor que ela dá **hoje** é o mesmo que dava ontem.
/// Ensiná-la sobre o leque muda a assinatura dela e o número que o artista lê em
/// dois painéis ⇒ **é wave própria**, e este gate é onde ela começa.
///
/// ⚠️ **Ele não é uma segunda cópia da fórmula:** o que compara são duas
/// MEDIÇÕES (a mesma rampa, o mesmo corpo, um raio contra três) contra uma
/// grandeza que é a definição do leque — onde o pé de fora pousa.
#[test]
fn the_fan_lowers_the_floor_by_exactly_the_uphill_foots_rise() {
    // Um passo de varredura (0,01) mais o viés medido, mais folga.
    const TOL: f32 = 0.02;
    for slope in [20.0f32, 30.0, 45.0] {
        let one = onset(slope, 1);
        let fan = onset(slope, 3);
        assert!(
            one.is_finite() && fan.is_finite(),
            "a fixture tem de CONTER o fenomeno a {slope}°: {one} / {fan}"
        );
        // O pé de fora nasce em `meia-largura × spread`, e para esta cápsula a
        // meia-largura É o raio. ⚠️ O `spread` entra: com o default de 0,9 o pé
        // fica DENTRO da pegada (ver `RideConfig::spread`), e usar o raio nu
        // aqui seria prever um pé que o produto não casta.
        let lift = RADIUS * FOOT_SPREAD * slope.to_radians().tan();
        assert!(
            (one - fan - lift).abs() <= TOL,
            "a {slope}° o leque tinha de baixar o piso {lift:.4} m (o pe' de cima \
             acha chao mais perto) e baixou {:.4} ({one:.4} -> {fan:.4})",
            one - fan
        );
        // ⚠️ E a metade que impede o gate de ficar verde sobre um leque INERTE:
        // com `lift` pequeno as duas colunas quase coincidem, e a diferença
        // sozinha não distingue *"desceu o certo"* de *"nao desceu"*.
        assert!(
            fan < one - TOL,
            "a {slope}° o leque tem de ficar ABAIXO do raio unico: \
             {fan:.4} contra {one:.4}"
        );
    }
}
