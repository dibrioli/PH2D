//! **A POLIA, do lado do ECS** (W-Pulley) — a ponte roteia um tipo que o rapier
//! não tem.
//!
//! O kernel (a corda inextensível, a talha, o clamp de só-puxa, a massa efetiva
//! exata) é gateado em `ph2d-physics/tests/pulley.rs`. Aqui ficam as perguntas
//! que só existem deste lado da fronteira:
//!
//! 1. a ponte entrega ao passe de polias a corda que o componente descreve — e
//!    **não** ao `ImpulseJointSet`;
//! 2. uma polia nova SEMEIA as roldanas e o comprimento da corda da pose de
//!    repouso, pelo mesmo sentinela `anchored` das âncoras;
//! 3. um rewind a re-arma;
//! 4. o `Active` a solta;
//! 5. ela conduz o grupo articulado como qualquer outro joint.

use ph2d_core::Vec2;
use ph2d_ecs::{Entity, Name, SimWorld, Transform, stable_name_id};
use ph2d_physics_ecs::{
    BodyKind, Collider, ColliderShape, JointKind, PhysicsBridge, PhysicsJoint, PulleyWheel,
    RigidBody, WrapSide,
};

/// Onde os dois corpos nascem: lado a lado, na mesma altura.
const START_Y: f32 = 2.0;
const SPAN: f32 = 4.0;

/// Um elevador com contrapeso: a carga à esquerda, o contrapeso à direita,
/// ligados por uma corda que sobe até uma roldana sobre cada um.
fn rig(kind: JointKind, load: f32, counterweight: f32, active: bool) -> SimWorld {
    let mut sim = SimWorld::new();
    let mut body = |name: &str, x: f32, density: f32| {
        sim.world_mut().spawn((
            Name::new(name),
            RigidBody {
                kind: BodyKind::Dynamic,
            },
            Collider {
                shape: ColliderShape::Ball { radius: 0.2 },
                density,
                ..Collider::default()
            },
            Transform::from_translation(Vec2::new(x, START_Y)),
        ));
    };
    // Densidade escolhida para a massa sair no número que o nome promete.
    let area = std::f32::consts::PI * 0.2 * 0.2;
    body("Load", -SPAN / 2.0, load / area);
    body("Counterweight", SPAN / 2.0, counterweight / area);
    sim.world_mut().spawn((
        Name::new("Rope"),
        PhysicsJoint {
            body_a: stable_name_id("Load"),
            body_b: stable_name_id("Counterweight"),
            kind,
            active,
            ..PhysicsJoint::of_kind(kind)
        },
        Transform::from_translation(Vec2::new(-SPAN / 2.0, START_Y)),
    ));
    wheels(&mut sim, 0.0);
    sim
}

/// As duas roldanas, uma sobre cada corpo — **entidades**, como no produto.
///
/// ⚠️ **Raio ZERO por default, e é a âncora de regressão da wave**: uma roldana de
/// raio zero é o modelo de PONTO que a v1 shipou, e a rota o reproduz
/// exatamente. Os gates que medem dinâmica herdam os números de antes; quem quer
/// medir o raio o pede.
fn wheels(sim: &mut SimWorld, radius: f32) {
    let lift = SPAN / 2.0;
    for (i, (name, x)) in [("Wheel A", -SPAN / 2.0), ("Wheel B", SPAN / 2.0)]
        .into_iter()
        .enumerate()
    {
        sim.world_mut().spawn((
            Name::new(name),
            PulleyWheel {
                rope: stable_name_id("Rope"),
                order: u16::try_from(i).expect("two wheels"),
                radius,
                wrap: WrapSide::Auto,
                motor_speed: 0.0,
                break_enabled: false,
                break_force: PulleyWheel::DEFAULT_BREAK_FORCE,
            },
            Transform::from_translation(Vec2::new(x, START_Y + lift)),
        ));
    }
}

fn y_of(sim: &mut SimWorld, name: &str) -> f32 {
    let mut q = sim.world_mut().query::<(&Name, &Transform)>();
    q.iter(sim.world())
        .find(|(n, _)| n.as_str() == name)
        .map(|(_, t)| t.translation.y)
        .expect("body alive")
}

fn joint_of(sim: &mut SimWorld) -> PhysicsJoint {
    let mut q = sim.world_mut().query::<(&Name, &PhysicsJoint)>();
    q.iter(sim.world())
        .find(|(n, _)| n.as_str() == "Rope")
        .map(|(_, j)| *j)
        .expect("joint alive")
}

fn entity_named(sim: &mut SimWorld, name: &str) -> Entity {
    let mut q = sim.world_mut().query::<(Entity, &Name)>();
    q.iter(sim.world())
        .find(|(_, n)| n.as_str() == name)
        .map(|(e, _)| e)
        .expect("entity alive")
}

fn run(sim: &mut SimWorld, bridge: &mut PhysicsBridge, ticks: u64) {
    for t in 1..=ticks {
        bridge.dispatch(sim, false, t);
    }
}

/// **A ponte entrega a polia — e a corda ergue o contrapeso.**
///
/// O controle é o MESMO rig com o mesmo par de massas e sem corda nenhuma: ali o
/// contrapeso apenas cai.
#[test]
fn the_bridge_folds_a_pulley_and_the_counterweight_rises() {
    let mut sim = rig(JointKind::Pulley, 4.0, 1.0, true);
    let mut bridge = PhysicsBridge::new();
    run(&mut sim, &mut bridge, 60);
    let load = y_of(&mut sim, "Load");
    let cw = y_of(&mut sim, "Counterweight");
    assert!(load < START_Y - 0.5, "a carga tinha de descer: {load:.4}");
    assert!(cw > START_Y + 0.5, "o contrapeso tinha de SUBIR: {cw:.4}");
    // O que um lado desce, o outro sobe.
    let fell = START_Y - load;
    let rose = cw - START_Y;
    assert!(
        (fell - rose).abs() < 0.02,
        "a corda esticou {:.4} m em {fell:.4} m de percurso",
        (fell - rose).abs()
    );
}

/// **Uma polia nova semeia as roldanas e a corda da pose de REPOUSO.**
///
/// Mesmo sentinela das âncoras (`anchored`): a polia é *montada* onde o artista
/// pôs os corpos, uma vez, e depois disso mover um corpo não re-deriva nada.
#[test]
fn a_fresh_pulley_seeds_its_rope_from_the_route_the_wheels_draw() {
    let mut sim = rig(JointKind::Pulley, 1.0, 1.0, true);
    assert!(
        !joint_of(&mut sim).anchored,
        "uma polia nova nasce por semear"
    );

    let mut bridge = PhysicsBridge::new();
    run(&mut sim, &mut bridge, 1);
    let j = joint_of(&mut sim);
    assert!(j.anchored, "o primeiro reconcile tinha de semear");
    // ⚠️ **O comprimento inclui o trecho ENTRE as roldanas**, que o modelo de
    // ponto da v1 ignorava (ele somava só os dois ramos). É uma CONSTANTE para
    // roldanas paradas, então a dinâmica é a mesma — o que muda é que o número
    // agora descreve a corda inteira, que é o que uma corda é.
    let lift = SPAN / 2.0;
    let want = 2.0 * lift + SPAN;
    assert!(
        (j.max_length - want).abs() < 1.0e-3,
        "a corda nasceu com {:.4} m, e a rota mede {want:.4}",
        j.max_length
    );
}

/// **O RAIO acrescenta ARCO à corda** — o pedido (3) do artista, no número que a
/// polia guarda.
///
/// Duas roldanas de meia-volta cada (a corda sobe, atravessa, desce) somam
/// `2·π·r/2 = π·r` de corda que não existia no modelo de ponto. É esse mesmo arco
/// que o desenho vai mostrar e que a roda vai girar.
#[test]
fn a_wheel_with_a_radius_puts_arc_into_the_rope() {
    let mut point = rig(JointKind::Pulley, 1.0, 1.0, true);
    let mut with_radius = rig_with_wheels(1.0, 1.0, 0.4);
    let mut b1 = PhysicsBridge::new();
    let mut b2 = PhysicsBridge::new();
    run(&mut point, &mut b1, 1);
    run(&mut with_radius, &mut b2, 1);
    let (thin, thick) = (
        joint_of(&mut point).max_length,
        joint_of(&mut with_radius).max_length,
    );
    assert!(
        thick > thin,
        "a roldana com raio tem de somar arco: {thin:.4} vs {thick:.4}"
    );
}

/// O rig com roldanas de raio `r`.
fn rig_with_wheels(load: f32, counterweight: f32, radius: f32) -> SimWorld {
    let mut sim = rig(JointKind::Pulley, load, counterweight, true);
    // Trocar o raio das que o rig já criou.
    let mut q = sim.world_mut().query::<(Entity, &Name)>();
    let ids: Vec<Entity> = q
        .iter(sim.world())
        .filter(|(_, n)| n.as_str().starts_with("Wheel"))
        .map(|(e, _)| e)
        .collect();
    for e in ids {
        if let Some(mut w) = sim.world_mut().get_mut::<PulleyWheel>(e) {
            w.radius = radius;
        }
    }
    sim
}

/// **Um rewind re-arma a polia.** A tabela é reconstruída do estado autorado
/// todo dispatch, então um scrub para trás e de volta reproduz o percurso.
#[test]
fn a_rewind_re_arms_the_pulley() {
    let mut sim = rig(JointKind::Pulley, 4.0, 1.0, true);
    let mut bridge = PhysicsBridge::new();
    run(&mut sim, &mut bridge, 45);
    let straight = y_of(&mut sim, "Counterweight");

    bridge.dispatch(&mut sim, false, 0);
    run(&mut sim, &mut bridge, 45);
    let replayed = y_of(&mut sim, "Counterweight");
    assert!(
        (straight - replayed).abs() < 1.0e-3,
        "o replay tinha de reproduzir: {straight:.5} vs {replayed:.5}"
    );
}

/// **`Active` desmarcado solta a corda** — sem apagar nada dela.
#[test]
fn an_inactive_pulley_lets_go() {
    let mut sim = rig(JointKind::Pulley, 4.0, 1.0, false);
    let mut bridge = PhysicsBridge::new();
    run(&mut sim, &mut bridge, 45);
    let cw = y_of(&mut sim, "Counterweight");
    assert!(
        cw < START_Y,
        "com a corda solta o contrapeso apenas cai: {cw:.4}"
    );
    // E os números seguem autorados: soltar não é apagar.
    assert!(joint_of(&mut sim).max_length > 0.0);
}

/// **Virar uma polia tira o joint do SOLVER.**
///
/// Um Pin segura os dois corpos juntos rigidamente; trocado para Pulley ele tem
/// de sair do `ImpulseJointSet` — senão o vínculo antigo continuaria valendo por
/// baixo, e o artista teria dois vínculos onde autorou um.
#[test]
fn switching_a_pin_to_a_pulley_takes_it_out_of_the_solver() {
    let mut sim = rig(JointKind::Pin, 4.0, 1.0, true);
    let mut bridge = PhysicsBridge::new();
    run(&mut sim, &mut bridge, 30);
    // Um Pin junta os dois corpos no mesmo ponto: eles caem JUNTOS.
    let pinned = (y_of(&mut sim, "Load") - y_of(&mut sim, "Counterweight")).abs();
    assert!(pinned < 0.5, "o pin tinha de juntar os dois: {pinned:.4}");

    let e = entity_named(&mut sim, "Rope");
    if let Some(mut j) = sim.world_mut().get_mut::<PhysicsJoint>(e) {
        j.kind = JointKind::Pulley;
        // Re-semear, como o gesto de trocar o tipo faz.
        j.anchored = false;
    }
    run(&mut sim, &mut bridge, 60);
    let apart = (y_of(&mut sim, "Load") - y_of(&mut sim, "Counterweight")).abs();
    assert!(
        apart > 1.0,
        "a polia manda os dois para lados opostos; distância {apart:.4}"
    );
}

/// **Uma polia conduz o grupo articulado** — assar uma ponta puxa a outra, como
/// em qualquer outro joint.
#[test]
fn a_pulley_conducts_the_jointed_group() {
    let mut sim = rig(JointKind::Pulley, 4.0, 1.0, true);
    let load = entity_named(&mut sim, "Load");
    let group = ph2d_physics_ecs::jointed_group(sim.world_mut(), &[load]);
    assert_eq!(group.len(), 2, "a corda liga os dois corpos: {group:?}");
}

/// **A view de uma polia diz que ela NÃO PARTE, e `∞` é como isso se diz.**
///
/// O campo é lido por `is_finite()` rio abaixo (o readout do overlay), então
/// `0.0` não significa *sem limiar* — significa **parte a 0 N**, e era isso que
/// punha um `0 / 0 N` permanente sobre toda corda na tela. É a mesma lei que o
/// `joint_desc` aplica a um checkbox desarmado, agora no tipo que não tem
/// checkbox nenhum.
#[test]
fn a_pulley_view_reports_no_break_threshold() {
    let mut sim = rig(JointKind::Pulley, 4.0, 1.0, true);
    let mut bridge = PhysicsBridge::new();
    run(&mut sim, &mut bridge, 1);
    let v = bridge
        .joint_views()
        .find(|v| v.kind == JointKind::Pulley)
        .expect("a polia tem view");
    assert!(
        v.break_force.is_infinite() && v.break_torque.is_infinite(),
        "os dois tetos: {} / {}",
        v.break_force,
        v.break_torque
    );
}

/// **A roldana GIRA, e a grande gira MAIS DEVAGAR** — a segunda metade do pedido
/// (3) do artista (*"nem a representação da polia e sua rotação"*).
///
/// ⚠️ **O oráculo é a RAZÃO entre os dois ângulos, e não *"girou"***: uma corda
/// inextensível corre na mesma taxa por todas as suas roldanas, então o que o
/// diâmetro decide é `ω = s/r` — e sem esta razão o raio seria um número que muda
/// o desenho e não muda nada mais. Duas rodas de raios 2:1 na MESMA corda têm de
/// medir ângulos na razão 1:2.
#[test]
fn the_wheels_spin_and_the_big_one_spins_slower() {
    let mut sim = rig(JointKind::Pulley, 4.0, 1.0, true);
    // Raios diferentes na mesma corda: 0,4 e 0,2 — dois para um.
    let ids: Vec<Entity> = {
        let mut q = sim.world_mut().query::<(Entity, &Name)>();
        let mut v: Vec<_> = q
            .iter(sim.world())
            .filter(|(_, n)| n.as_str().starts_with("Wheel"))
            .map(|(e, n)| (n.as_str().to_string(), e))
            .collect();
        v.sort();
        v.into_iter().map(|(_, e)| e).collect()
    };
    for (e, r) in ids.iter().zip([0.4_f32, 0.2]) {
        if let Some(mut w) = sim.world_mut().get_mut::<PulleyWheel>(*e) {
            w.radius = r;
        }
    }
    let mut bridge = PhysicsBridge::new();
    run(&mut sim, &mut bridge, 20);
    let spins = bridge.pulley_wheel_spins().to_vec();
    assert_eq!(spins.len(), 2, "duas roldanas, dois ângulos");
    let (big, small) = (spins[0].abs(), spins[1].abs());
    assert!(
        big > 1.0e-3 && small > 1.0e-3,
        "as duas tinham de girar: {big:.4} e {small:.4}"
    );
    let ratio = small / big;
    assert!(
        (ratio - 2.0).abs() < 0.05,
        "a roda de raio METADE gira o DOBRO: mediu {ratio:.4} (grande {big:.4}, pequena {small:.4})"
    );
}

/// **Uma roldana de raio ZERO não gira** — e não é caso especial, é o que um
/// PONTO é: não há superfície para a corda arrastar, e `s/0` seria o infinito que
/// envenena a pose de desenho.
#[test]
fn a_point_wheel_does_not_spin() {
    let mut sim = rig(JointKind::Pulley, 4.0, 1.0, true);
    let mut bridge = PhysicsBridge::new();
    run(&mut sim, &mut bridge, 20);
    assert!(
        bridge.pulley_wheel_spins().iter().all(|a| *a == 0.0),
        "raio zero é um ponto: {:?}",
        bridge.pulley_wheel_spins()
    );
}

/// **A `motor_speed` de uma roldana chega à corda como `ω·r` — e as roldanas
/// SOMAM** (W2).
///
/// A conversão mora na colheita porque o RAIO é da roldana e o kernel só conhece
/// a corda: somar `ω·r` aqui é o que torna *"as taxas somam"* uma soma de metros
/// por segundo, e não de radianos por segundo de rodas de tamanhos diferentes.
///
/// ⚠️ O oráculo é o que a CORDA faz, não o campo — um gate que lesse
/// `motor_speed` de volta seria o espelho do componente, e o componente não é
/// quem move a carga.
#[test]
fn a_drivens_wheel_speed_reaches_the_rope_as_metres_per_second() {
    /// O elevador com a roldana `idx` dirigida a `omega`, com o raio `radius`.
    fn driven(radius: f32, omega: [f32; 2]) -> f32 {
        let mut sim = rig(JointKind::Pulley, 1.0, 1.0, true);
        let mut q = sim.world_mut().query::<(&Name, &mut PulleyWheel)>();
        for (n, mut w) in q.iter_mut(sim.world_mut()) {
            let i = usize::from(n.as_str() == "Wheel B");
            w.radius = radius;
            w.motor_speed = omega[i];
        }
        let mut bridge = PhysicsBridge::new();
        run(&mut sim, &mut bridge, 60);
        y_of(&mut sim, "Counterweight") - START_Y
    }

    // Um tambor parado deixa o elevador em equilíbrio (massas iguais).
    let still = driven(0.3, [0.0, 0.0]);
    assert!(
        still.abs() < 0.02,
        "sem motor o elevador andou {still:.4} m"
    );

    // Um tambor recolhendo ergue o contrapeso; DOIS recolhem o dobro.
    //
    // ⚠️ Cada corpo sobe cerca de METADE do recolhido, e isso é o rig, não o
    // guincho: aqui os DOIS lados são dinâmicos e de massa igual, então encurtar
    // a corda os puxa para as respectivas roldanas em partes iguais (0,3 m/s de
    // corda ⇒ ~0,15 m por lado). O que este gate afirma são as RAZÕES, que não
    // dependem disso.
    let one = driven(0.3, [1.0, 0.0]);
    let two = driven(0.3, [1.0, 1.0]);
    assert!(one > 0.1, "um tambor ergueu só {one:.4} m em 1 s");
    assert!(
        (two / one - 2.0).abs() < 0.15,
        "dois tambores ergueram {two:.4} contra {one:.4} de um só (razão {:.3})",
        two / one
    );

    // E o RAIO é o câmbio: o mesmo ω num tambor duas vezes maior recolhe o dobro.
    let big = driven(0.6, [1.0, 0.0]);
    assert!(
        (big / one - 2.0).abs() < 0.15,
        "o tambor de raio dobrado ergueu {big:.4} contra {one:.4} (razão {:.3})",
        big / one
    );

    // E sentidos opostos se ANULAM — dois guinchos brigando pela mesma corda.
    let fighting = driven(0.3, [1.0, -1.0]);
    assert!(
        fighting.abs() < 0.02,
        "dois tambores opostos moveram {fighting:.4} m"
    );
}

/// **Um rewind rebobina o guincho junto com o mundo.**
///
/// O recolhido é uma INTEGRAL, então ele é a única parte da polia que um Reset
/// tem de esquecer — e ele esquece de graça, porque `rebuild_from_rest` constrói
/// um `PhysicsWorld` novo. Este gate existe para que ninguém "otimize" o rebuild
/// preservando o mundo antigo e deixe o guincho a meio caminho.
#[test]
fn a_rewind_puts_the_winch_back_where_it_started() {
    let mut sim = rig(JointKind::Pulley, 1.0, 1.0, true);
    {
        let mut q = sim.world_mut().query::<(&Name, &mut PulleyWheel)>();
        for (n, mut w) in q.iter_mut(sim.world_mut()) {
            if n.as_str() == "Wheel A" {
                w.radius = 0.3;
                w.motor_speed = 1.5;
            }
        }
    }
    let mut bridge = PhysicsBridge::new();
    run(&mut sim, &mut bridge, 60);
    let lifted = y_of(&mut sim, "Counterweight") - START_Y;
    assert!(
        lifted > 0.1,
        "a fixture não continha o fenômeno: subiu {lifted:.4}"
    );

    bridge.dispatch(&mut sim, false, 0);
    let back = y_of(&mut sim, "Counterweight") - START_Y;
    assert!(
        back.abs() < 1.0e-3,
        "depois do Reset o contrapeso ficou {back:.4} m acima do repouso"
    );

    // E re-simular do zero reproduz a MESMA subida — o guincho re-arma.
    run(&mut sim, &mut bridge, 60);
    let again = y_of(&mut sim, "Counterweight") - START_Y;
    assert!(
        (again - lifted).abs() < 1.0e-3,
        "o replay ergueu {again:.4}, o run original ergueu {lifted:.4}"
    );
}

/// **Um eixo que cede sai da ARENA, e não só da conta** (W2).
///
/// ⚠️ Duas camadas, e esta é a de fora. O passe tem o próprio filtro — é o que
/// mantém o `PhysicsWorld` correto sozinho —, mas a **arena é a lista que o
/// DESENHO lê**: sem esta metade o overlay desenharia a corda passando por uma
/// roldana que o solver já ignorou, e o doc do `physics_overlay_pulley` proíbe
/// exatamente isso em voz alta (*"a corda desenhada e a corda que segura são a
/// mesma corda"*). Camada própria, gate próprio.
#[test]
fn a_broken_axle_leaves_the_arena_the_drawing_reads() {
    let mut sim = rig(JointKind::Pulley, 4.0, 1.0, true);
    {
        // A roldana A com um eixo que não aguenta o que ela vai carregar.
        let mut q = sim.world_mut().query::<(&Name, &mut PulleyWheel)>();
        for (n, mut w) in q.iter_mut(sim.world_mut()) {
            if n.as_str() == "Wheel A" {
                w.radius = 0.3;
                w.break_enabled = true;
                w.break_force = 1.0;
            }
        }
    }
    let mut bridge = PhysicsBridge::new();
    // ⚠️ O canal de eventos é uma TRANSIÇÃO — esvaziado no topo de cada dispatch
    // —, então ele tem de ser colhido ENQUANTO a corrida anda; ler
    // `joint_breaks()` no fim seria ler uma lista já esvaziada sessenta vezes.
    // ⚠️ E a colheita começa no PRIMEIRO tique: com este eixo (1 N contra uma
    // carga de 4 kg) o rompimento acontece já ali, e um laço que começasse no
    // segundo perderia o evento e falharia sobre um produto correto.
    let mut reported = 0;
    let mut arena_at_first = 0;
    for t in 1..=60 {
        bridge.dispatch(&mut sim, false, t);
        if t == 1 {
            arena_at_first = bridge.pulley_wheel_arena().len();
        }
        reported += bridge.joint_breaks().len();
    }
    assert_eq!(arena_at_first, 2, "as duas roldanas têm de nascer na arena");
    assert_eq!(
        bridge.pulley_wheel_arena().len(),
        1,
        "o eixo que cedeu tinha de sair da arena que o desenho lê"
    );
    assert_eq!(
        reported, 1,
        "o rompimento tinha de ser reportado UMA vez no canal de eventos"
    );
}

/// **A corda publica a TENSÃO que ela segura, e o overlay a lê pela mesma
/// porta dos outros tipos** (W2).
///
/// Antes desta wave a view da polia nascia com `load: 0` e `break_force: ∞`
/// porque nada media a reação de um vínculo fora do `ImpulseJointSet`. Agora
/// mede — e este gate é o que impede a view de voltar a mentir em silêncio.
#[test]
fn the_rope_view_carries_the_tension_it_is_holding() {
    let mut sim = rig(JointKind::Pulley, 4.0, 1.0, true);
    let mut bridge = PhysicsBridge::new();
    run(&mut sim, &mut bridge, 60);
    let view = bridge
        .joint_views()
        .find(|v| v.wheel_count > 0)
        .expect("a polia tem uma view");
    assert!(
        view.load.force > 1.0,
        "a corda está segurando 4 kg contra 1 e a view lê {:.4} N",
        view.load.force
    );
    assert_eq!(view.load.torque, 0.0, "uma corda não transmite torque");
    assert!(
        view.peak.force >= view.load.force,
        "o pico não pode ser menor"
    );
    assert!(!view.broken, "ela não devia ter partido");
}

/// **O limiar AUTORADO da corda chega ao passe** — e o checkbox o gateia (W2).
///
/// ⚠️ Este gate nasceu de uma mutação SOBREVIVENTE: cortar o `break_force` do
/// componente na ponte (mandando `∞` sempre) deixava os treze gates ao lado
/// verdes. O kernel prova que o passe honra *um* limiar; nada provava que o
/// limiar que o ARTISTA digitou é o que chega lá — e as duas frases são
/// diferentes exatamente onde a fiação pode se perder.
#[test]
fn the_authored_rope_threshold_reaches_the_pass() {
    /// `Some(carga)` no tique em que a corda partiu.
    fn run_until_break(enabled: bool, force: f32) -> Option<f32> {
        let mut sim = rig(JointKind::Pulley, 4.0, 1.0, true);
        {
            let mut q = sim.world_mut().query::<(&Name, &mut PhysicsJoint)>();
            for (n, mut j) in q.iter_mut(sim.world_mut()) {
                if n.as_str() == "Rope" {
                    j.break_enabled = enabled;
                    j.break_force = force;
                }
            }
        }
        let mut bridge = PhysicsBridge::new();
        for t in 1..=60 {
            bridge.dispatch(&mut sim, false, t);
            if let Some(b) = bridge.joint_breaks().first() {
                return Some(b.force);
            }
        }
        None
    }

    // Uma corda de 5 N não segura 4 kg contra 1 (a tensão passa de 10 N).
    let load = run_until_break(true, 5.0).expect("a corda tinha de partir");
    assert!(
        load > 5.0,
        "o rompimento reportou {load:.4} N, abaixo do próprio limiar"
    );
    // O MESMO limiar com a caixa desmarcada não parte nada — é o checkbox
    // gateando, e não o número sendo alto.
    assert!(
        run_until_break(false, 5.0).is_none(),
        "com o Breakable desmarcado a corda não pode partir"
    );
    // E um limiar generoso também não: o número é lido, não ignorado.
    assert!(
        run_until_break(true, 1.0e6).is_none(),
        "um limiar de 1 MN não pode ser cruzado por 4 kg"
    );
}
