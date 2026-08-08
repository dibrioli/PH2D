//! **A ÁGUA NÃO ALIMENTA O PERSONAGEM** (W-Submerged).
//!
//! A cena `=100` fechou com o item que este arquivo cura: largado dentro da
//! poça, o personagem **bombeia** — medido antes de uma linha ser escrita,
//! `−1,05 / +4,71 / +12,08 / −20,31` — e sai de quadro.
//!
//! # ⚠️ O mecanismo, e por que ele NÃO é do empuxo
//!
//! A modelagem do arco de um pulo é **não-conservativa por construção**: subir
//! com `g` e descer com `fall_gravity·g` devolve o corpo ao mesmo nível com
//! `√fall_gravity` da velocidade. Num platformer isso é inofensivo porque todo
//! arco acaba absorvido pelo CHÃO; sobre uma superfície que RESTAURA a ficção
//! passa a acumular, ciclo após ciclo.
//!
//! ⚠️ **A ablação nomeia o culpado sem ambiguidade** (`measure_submersion::
//! measure_which_multiplier_pumps`): com `fall_gravity = 1` a amplitude cai de
//! **14,55 para 0,11**, e `peak`/`takeoff` não movem um centímetro.
//!
//! ⚠️ **E a metade que uma fração instantânea não alcança é o AR:** desvanecer
//! a modelagem só enquanto submerso cura o repouso (0,0046 contra 0,0050 do
//! controle) e deixa o corpo largado de 1,5 m **divergindo a 11 m**, porque a
//! energia é ganha entre dois mergulhos, onde não há fluido nenhum a medir. Daí
//! a lei ser uma TRAVA (`JumpState::waterborne`) e não um fade.
//!
//! **O CONTROLE de todo gate deste arquivo é a cápsula idêntica sem
//! `PlatformPlayer`** — mesma forma, mesma densidade, mesma poça. O que os dois
//! fizerem igual não é do player, e é por isso que nenhum destes números é um
//! literal escolhido.

#[path = "platform_water_scene.rs"]
mod water;

use ph2d_ecs::SimWorld;
use ph2d_physics_ecs::PhysicsBridge;
use water::{pool, subject, y_of};

/// Larga o sujeito de `y0`, corre 20 s e devolve `(y final, amplitude dos
/// últimos 5 s)`.
///
/// ⚠️ **A amplitude é medida na CAUDA**, nunca na corrida inteira: a entrada é
/// um transiente legítimo (o corpo mergulha e volta), e o que estes gates
/// perguntam é se ele **assenta**.
fn settle(player: bool, y0: f32) -> (f32, f32) {
    let mut sim = SimWorld::new();
    pool(&mut sim, 0.0);
    let _ = subject(&mut sim, player, y0);
    let mut bridge = PhysicsBridge::new();
    let (mut lo, mut hi) = (f32::INFINITY, f32::NEG_INFINITY);
    for t in 1..=1200u64 {
        bridge.dispatch(&mut sim, true, t);
        if t > 900 {
            let y = y_of(&sim, "Subject");
            lo = lo.min(y);
            hi = hi.max(y);
        }
    }
    (y_of(&sim, "Subject"), hi - lo)
}

/// **Largado DENTRO da poça, o personagem é indistinguível de uma cápsula.**
///
/// ⚠️ Este é o gate mais afiado do arquivo, e a razão é a fixture: começando os
/// dois **submersos**, a condição inicial é idêntica — nada do que a modelagem
/// do arco faz no ar entra na comparação, então qualquer diferença é a bomba.
/// Nasceu VERMELHO com o player em `11,01 m` e amplitude `17,11` contra `0,28`.
#[test]
fn a_player_dropped_in_the_pool_behaves_like_a_plain_capsule() {
    let (py, pa) = settle(true, -2.0);
    let (cy, ca) = settle(false, -2.0);
    assert!(
        (py - cy).abs() < 0.02,
        "o player assenta em {py:.4} e a capsula identica em {cy:.4}"
    );
    assert!(
        (pa - ca).abs() < 0.02,
        "a amplitude do player e' {pa:.4} e a da capsula {ca:.4} -- \
         mesma condicao inicial, mesma resposta"
    );
}

/// **Entrando de cima, ele ASSENTA** — em toda altura de queda que a cena
/// oferece, e não só na suave.
///
/// ⚠️ **A barra é RELATIVA ao controle, e a folga tem mecanismo:** o player cai
/// com `fall_gravity` (é o que o artista autorou), então ele entra na água mais
/// depressa que a cápsula e o mergulho é maior. O que a wave promete não é
/// *"igual ao controle"*, é ***assenta na linha do controle e a oscilação
/// DECAI*** — e é isso que a barra afirma.
#[test]
fn entering_from_above_always_settles_on_the_control_waterline() {
    let (_, line) = (0.0, settle(false, 0.5).0);
    for y0 in [0.5_f32, 1.5, 3.0, 6.0] {
        let (y, amp) = settle(true, y0);
        assert!(
            (y - line).abs() < 0.1,
            "largado de {y0:.1} ele assenta em {y:.4}, e a linha do controle e' {line:.4}"
        );
        assert!(
            amp < 2.0,
            "largado de {y0:.1} a oscilacao final e' {amp:.4} m -- \
             sem a trava ela era 11 a 18 m e crescia"
        );
    }
}

/// **Uma cena SEM poça é byte-idêntica** — o personagem cai, pula e pousa
/// exactamente como antes desta wave.
///
/// ⚠️ Sem este gate a cura seria indistinguível de *"desligamos a modelagem do
/// arco"*: a trava só é uma adição honesta se nada a arma numa cena seca.
#[test]
fn a_dry_scene_is_untouched() {
    use ph2d_core::Vec2;
    use ph2d_ecs::{Name, Transform};
    use ph2d_physics_ecs::{BodyKind, Collider, ColliderShape, RigidBody};

    let mut sim = SimWorld::new();
    // Chão sólido a y = 0, e o personagem caindo de 3 m.
    sim.world_mut().spawn((
        Name::new("Floor"),
        RigidBody {
            kind: BodyKind::Static,
        },
        Collider {
            shape: ColliderShape::Cuboid {
                half_x: 20.0,
                half_y: 0.5,
            },
            ..Default::default()
        },
        Transform::from_translation(Vec2::new(0.0, -0.5)),
    ));
    let who = water::subject(&mut sim, true, 3.0);
    let mut bridge = PhysicsBridge::new();
    for t in 1..=300u64 {
        bridge.dispatch(&mut sim, true, t);
    }
    let y = y_of(&sim, "Subject");
    assert!(
        (y - water::FLOAT).abs() < 0.05,
        "numa cena seca ele tem de pairar a {:.2} do chao, e ficou em {y:.4}",
        water::FLOAT
    );
    assert_eq!(
        bridge.buoyed(who),
        0.0,
        "e nada numa cena seca pode carregar peso nenhum"
    );
}
