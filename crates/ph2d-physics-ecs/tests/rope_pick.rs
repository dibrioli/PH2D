//! **Apontar uma CORDA no canvas** (W-Pulley W1) — o alvo do eyedropper da §13.
//!
//! ⚠️ **O item aberto prometia *"o irmão exato do eyedropper da §12"*, e ele não
//! pode ser** (`measure_rope_pick`): o da §12 resolve o alvo com
//! `pick_sprites_at_world`, que exige um **sprite** sob o cursor. Um corpo tem um;
//! uma corda é uma **LINHA** e a entidade dela não tem nenhum — copiar aquele gesto
//! daria `None` sobre a corda para sempre, um botão que arma e nunca acerta.
//!
//! O alvo que existe é a **ROTA**, e ela é a que o overlay DESENHA. Medido: sobre a
//! rota **0,00000 m**; afastar `d` pela normal dá `d` ao quinto decimal; entre duas
//! cordas paralelas a mais próxima ganha em TODA separação (3,0 · 1,0 · 0,4 · 0,1 ·
//! 0,02 m), com a razão das distâncias saindo exatamente **2**.
//!
//! ⚠️ **E o mesmo defeito de fixture morde DUAS vezes nesta wave, uma na sonda e
//! uma AQUI:** um ponto que se chama *"sobre a corda"* tem de ser derivado da
//! geometria de uma rota que passa por onde se calcula. Na sonda eu cravei `(0, 5)`
//! e ele estava a **0,46 m** da rota (que desvia para a roldana); aqui a roldana
//! tinha raio, então a rota toca a **TANGENTE** e não o centro, e o gate nasceu
//! vermelho. A cura é o modelo de PONTO (raio 0) — a geometria fica exata no papel,
//! e o oráculo segue sem chamar o produto.

use ph2d_core::Vec2;
use ph2d_ecs::{Entity, Name, SimWorld, Transform, stable_name_id};
use ph2d_physics_ecs::{
    BodyKind, Collider, ColliderShape, JointKind, PhysicsBridge, PhysicsJoint, PulleyWheel,
    RigidBody,
};

/// Uma corda com uma roldana fora do eixo, deslocada por `x`.
///
/// ⚠️ **A roldana FORA do eixo é a fixture conter o fenômeno:** com ela alinhada a
/// rota é uma reta vertical, e um gate que aponta o meio dela não distingue *"a
/// rota"* de *"a reta entre as âncoras"* — que é precisamente a segunda geometria
/// que esta porta existe para não ser.
fn rope(sim: &mut SimWorld, tag: &str, x: f32) {
    let mut ball = |name: String, y: f32, kind: BodyKind| {
        sim.world_mut().spawn((
            Name::new(name),
            RigidBody { kind },
            Collider {
                shape: ColliderShape::Ball { radius: 0.2 },
                density: 1.0,
                ..Collider::default()
            },
            Transform::from_translation(Vec2::new(x, y)),
        ));
    };
    ball(format!("{tag} Dead"), 6.0, BodyKind::Static);
    ball(format!("{tag} Load"), 2.0, BodyKind::Dynamic);
    sim.world_mut().spawn((
        Name::new(format!("{tag} Rope")),
        PhysicsJoint {
            body_a: stable_name_id(&format!("{tag} Dead")),
            body_b: stable_name_id(&format!("{tag} Load")),
            kind: JointKind::Pulley,
            ..PhysicsJoint::of_kind(JointKind::Pulley)
        },
        Transform::from_translation(Vec2::new(x, 6.0)),
    ));
    sim.world_mut().spawn((
        Name::new(format!("{tag} Rope Wheel 1")),
        PulleyWheel {
            rope: stable_name_id(&format!("{tag} Rope")),
            order: 0,
            // ⚠️ **Raio ZERO — o modelo de PONTO**, e não é economia: com raio a
            // rota toca a **TANGENTE** do disco, não o centro, então um ponto
            // derivado à mão da fixture **não fica sobre a linha**. Foi
            // exatamente assim que a 1ª versão deste arquivo nasceu VERMELHA
            // (`afastado 0.05 m com tolerância 0.10 m: esperado true, deu false`
            // — o erro de base da tangente somado ao afastamento passava do
            // limite). Com raio 0 o vértice da rota **é** o centro, a geometria
            // é exata no papel, e o oráculo continua independente do produto.
            radius: 0.0,
            ..Default::default()
        },
        Transform::from_translation(Vec2::new(x + 0.5, 4.5)),
    ));
}

fn named(sim: &mut SimWorld, name: &str) -> Entity {
    let mut q = sim.world_mut().query::<(Entity, &Name)>();
    q.iter(sim.world())
        .find(|(_, n)| n.as_str() == name)
        .map(|(e, _)| e)
        .expect("entidade viva")
}

fn rig(taps: &[(&str, f32)]) -> (SimWorld, PhysicsBridge) {
    let mut sim = SimWorld::new();
    for (tag, x) in taps {
        rope(&mut sim, tag, *x);
    }
    let mut bridge = PhysicsBridge::new();
    bridge.dispatch(&mut sim, false, 0);
    (sim, bridge)
}

/// **A perna que a roldana desvia** — o ponto do meio do 1º trecho, que existe SÓ
/// na rota e não na reta entre as âncoras.
///
/// Derivado da geometria declarada na fixture (âncora `(x, 6)` → roldana
/// `(x+0.5, 4.5)`), nunca de uma chamada ao produto: um oráculo que usa a função
/// sob teste para computar o que espera é sempre verde.
fn on_the_deflected_leg(x: f32) -> [f32; 2] {
    [x + 0.25, 5.25]
}

/// **O cursor sobre a corda acha a corda.**
#[test]
fn a_click_on_the_rope_finds_it() {
    let (mut sim, bridge) = rig(&[("A", 0.0)]);
    let want = named(&mut sim, "A Rope");
    let p = on_the_deflected_leg(0.0);
    assert_eq!(
        bridge.rope_at_world(p, 0.15),
        Some(want),
        "o ponto {p:?} está sobre a perna desviada pela roldana e a porta não achou \
         a corda"
    );
}

/// **E o cursor LONGE não acha nada** — a metade que impede o pick de aceitar
/// qualquer clique.
///
/// ⚠️ **Duas asserções, e a 1ª é o controle:** sem provar que ALGO é achável a esta
/// tolerância, *"nada foi achado"* passaria com a porta devolvendo `None` sempre.
#[test]
fn a_click_away_from_every_rope_finds_nothing() {
    let (mut sim, bridge) = rig(&[("A", 0.0)]);
    let want = named(&mut sim, "A Rope");
    const TOL: f32 = 0.15;
    assert_eq!(
        bridge.rope_at_world(on_the_deflected_leg(0.0), TOL),
        Some(want),
        "controle: a esta tolerância a corda TEM de ser achável"
    );
    // Cinco metros ao lado — mais que qualquer tolerância de tela em qualquer zoom.
    assert_eq!(
        bridge.rope_at_world([5.0, 5.25], TOL),
        None,
        "um clique no vazio não pode religar coisa nenhuma"
    );
}

/// **A tolerância é uma FRONTEIRA, não uma sugestão** — o número que a mão controla.
///
/// Afastar-se `d` da rota some do alcance exatamente quando `d > tol`. É o que faz
/// do 14 px do editor um alcance previsível em vez de um ímã que às vezes pega.
#[test]
fn the_tolerance_is_the_boundary_the_hand_feels() {
    let (mut sim, bridge) = rig(&[("A", 0.0)]);
    let want = named(&mut sim, "A Rope");
    let on = on_the_deflected_leg(0.0);
    // A perna vai de `(0, 6)` ao centro `(0.5, 4.5)` — o vetor `(0.5, -1.5)`, cujo
    // comprimento é `sqrt(2.5) = 1.5811`. A normal é o giro de 90°, normalizado.
    let len = 2.5_f32.sqrt();
    let n = [1.5 / len, 0.5 / len];
    for (off, tol, expect) in [
        (0.05_f32, 0.10_f32, true),
        (0.15, 0.10, false),
        (0.15, 0.20, true),
    ] {
        let p = [on[0] + n[0] * off, on[1] + n[1] * off];
        let got = bridge.rope_at_world(p, tol).is_some();
        assert_eq!(
            got, expect,
            "afastado {off:.2} m com tolerância {tol:.2} m: esperado {expect}, deu {got}"
        );
        let _ = want;
    }
}

/// **Entre duas cordas, a mais PRÓXIMA ganha.**
///
/// ⚠️ **A fixture põe o cursor SOBRE uma delas e o desloca na direção da outra** —
/// um cursor "entre as duas" pode estar longe das DUAS (foi o defeito da 1ª versão
/// da sonda: um ponto que eu chamei de *em cima da corda* estava a 0,46 m dela, e a
/// coluna saiu não-monotônica). Um pick medido de um ponto que não está na linha não
/// mede pick nenhum.
#[test]
fn the_nearest_rope_wins() {
    const GAP: f32 = 1.0;
    let (mut sim, bridge) = rig(&[("A", 0.0), ("B", GAP)]);
    let (a, b) = (named(&mut sim, "A Rope"), named(&mut sim, "B Rope"));
    let on_a = on_the_deflected_leg(0.0);
    // Um terço do vão na direção de B: mais perto de A.
    assert_eq!(
        bridge.rope_at_world([on_a[0] + GAP / 3.0, on_a[1]], 1.0),
        Some(a),
        "o cursor está a um terço do vão de A e a porta escolheu a outra"
    );
    // E dois terços: mais perto de B. A mesma tolerância, o mesmo rig — só a mão
    // mudou de lado, que é o que um pick tem de honrar.
    assert_eq!(
        bridge.rope_at_world([on_a[0] + GAP * 2.0 / 3.0, on_a[1]], 1.0),
        Some(b),
        "o cursor está a dois terços do vão e a porta não seguiu para B"
    );
}
