//! Os gates da [`super`] — o que a lei pede CHEGA ao mundo, e não ao `Ctrl+Z`.

use super::*;
use ph2d_ecs::{Timer, TimerRuntime, Timers, Tweens};
use ph2d_tween::{AoAcabar, Tween};

const BRANCO: [f32; 4] = [1.0, 1.0, 1.0, 1.0];

/// Um sprite com `n` timers de um segundo (o primeiro a correr) e os tweens dados.
fn cena(tweens: Vec<Tween>, n: usize) -> (SimWorld, Entity) {
    let mut sim = SimWorld::new();
    let cfg = Timers(
        (0..n)
            .map(|_| Timer {
                duration_us: 1_000_000,
                ..Timer::default()
            })
            .collect(),
    );
    let rt = TimerRuntime(cfg.0.iter().map(ph2d_ecs::timer::born).collect());
    let e = sim
        .world_mut()
        .spawn((
            Transform::default(),
            Sprite::atlas(0, [1.0, 1.0], BRANCO),
            Tweens(tweens),
            cfg,
            rt,
        ))
        .id();
    (sim, e)
}

fn anda(sim: &mut SimWorld, e: Entity, us: u64) {
    let mut rt = sim.world_mut().get_mut::<TimerRuntime>(e).unwrap();
    for s in &mut rt.0 {
        s.elapsed_us = us;
    }
}

/// **O que a lei pede CHEGA ao `Sprite`.**
#[test]
fn um_fade_chega_a_alfa_do_sprite() {
    let (mut sim, e) = cena(vec![Tween::linear(Canal::Opacity, 1.0, 0.0)], 1);
    anda(&mut sim, e, 500_000);
    let mut drive = PreviewDrive::default();
    assert_eq!(drive_tweens(&mut sim, &mut drive, &[]), 1);
    let a = sim.world().get::<Sprite>(e).unwrap().tint[3];
    assert!((a - 0.5).abs() < 1e-6, "a alfa leu {a}");
}

/// ⭐⭐⭐ **E NÃO chega ao documento** — sem o ledger, um fade de meio segundo seriam trinta passos
/// de `Ctrl+Z`.
///
/// ⚠️ **O CONTROLO é metade do gate:** a primeira asserção prova que o valor de facto mudou no
/// mundo, senão um passe inerte passaria por vacuidade.
#[test]
fn o_fade_nao_entra_no_undo() {
    let (mut sim, e) = cena(vec![Tween::linear(Canal::Opacity, 1.0, 0.0)], 1);
    anda(&mut sim, e, 500_000);
    let mut drive = PreviewDrive::default();
    drive_tweens(&mut sim, &mut drive, &[]);
    assert!(
        (sim.world().get::<Sprite>(e).unwrap().tint[3] - 0.5).abs() < 1e-6,
        "controlo: o valor VIVO tem de ter mudado"
    );
    assert!(
        drive.still_driving(e, Driver::SpriteAlpha),
        "o ledger tem de estar a conduzir a alfa"
    );
    // ⭐ O AUTORADO — o que o `Ctrl+Z` e o save veem — continua a ser o do artista.
    let autorado = drive.authored(e.to_bits(), Driver::SpriteAlpha);
    assert_eq!(
        autorado,
        Some(Driven::SpriteAlpha(1.0)),
        "o autorado deixou de ser o do artista: {autorado:?}"
    );
}

/// ⭐⭐ **DOIS tweens de pose no mesmo objecto COMPÕEM** — e o autorado continua a ser um só.
///
/// ⚠️ É o gate da decisão *«um censo por ENTIDADE, não um por escrita»*: com a fotografia tirada
/// duas vezes, a segunda declaração leria a saída da primeira como *«outra mão escreveu»*.
#[test]
fn dois_tweens_de_pose_compoem_e_o_autorado_e_um_so() {
    let (mut sim, e) = cena(
        vec![
            Tween::linear(Canal::PositionX, 0.0, 10.0),
            Tween::linear(Canal::PositionY, 0.0, 4.0),
        ],
        2,
    );
    anda(&mut sim, e, 500_000);
    let mut drive = PreviewDrive::default();
    assert_eq!(drive_tweens(&mut sim, &mut drive, &[]), 2);
    let t = *sim.world().get::<Transform>(e).unwrap();
    assert!(
        (t.translation.x - 5.0).abs() < 1e-6,
        "x leu {}",
        t.translation.x
    );
    assert!(
        (t.translation.y - 2.0).abs() < 1e-6,
        "y leu {}",
        t.translation.y
    );
    assert_eq!(
        drive.authored(e.to_bits(), Driver::TweenPose),
        Some(Driven::TweenPose(Transform::default())),
        "o autorado tem de ser a pose do artista, nao a saida do primeiro tween"
    );
}

/// ⭐⭐⭐ **A SILHUETA acende o `tint_fill`, e o `Rewind` devolve os DOIS campos** — a arte não volta
/// por interpolação, ela volta por o motor **deixar de escrever**.
#[test]
fn a_silhueta_acende_o_interruptor_e_o_rewind_devolve_os_dois() {
    let t = Tween {
        ao_acabar: AoAcabar::Rewind,
        ..Tween::cor(Canal::Silhueta, [1.0, 0.0, 0.0, 1.0], BRANCO)
    };
    let (mut sim, e) = cena(vec![t], 1);
    anda(&mut sim, e, 100_000);
    let mut drive = PreviewDrive::default();
    drive_tweens(&mut sim, &mut drive, &[]);
    let s = *sim.world().get::<Sprite>(e).unwrap();
    assert!(s.tint_fill, "a silhueta nao acendeu o interruptor");
    assert!(s.self_tint[1] < 0.5, "a cor nao chegou: {:?}", s.self_tint);

    // …e o AUTORADO tem os dois campos como o artista os deixou.
    assert_eq!(
        drive.authored(e.to_bits(), Driver::TweenTint),
        Some(Driven::TweenTint {
            self_tint: BRANCO,
            fill: false,
        }),
        "o autorado perdeu um dos dois campos"
    );
}

/// ⚠️ **Um tween de cor num objecto SEM sprite é inerte, não um erro** — e o gate mede que ele não
/// conta como escrita.
#[test]
fn um_tween_de_cor_sem_sprite_e_inerte() {
    let mut sim = SimWorld::new();
    let cfg = Timers(vec![Timer {
        duration_us: 1_000_000,
        ..Timer::default()
    }]);
    let rt = TimerRuntime(cfg.0.iter().map(ph2d_ecs::timer::born).collect());
    let e = sim
        .world_mut()
        .spawn((
            Transform::default(),
            Tweens(vec![Tween::linear(Canal::Opacity, 1.0, 0.0)]),
            cfg,
            rt,
        ))
        .id();
    {
        let mut rt = sim.world_mut().get_mut::<TimerRuntime>(e).unwrap();
        rt.0[0].elapsed_us = 500_000;
    }
    let mut drive = PreviewDrive::default();
    assert_eq!(drive_tweens(&mut sim, &mut drive, &[]), 0);
}

/// ⭐ **Sem tween nenhum, ZERO custo** — a lista vem vazia e o passe devolve `0` antes de tirar
/// censo nenhum, que é o caso de toda cena que já existe.
#[test]
fn sem_tween_nenhum_o_passe_nao_faz_nada() {
    let mut sim = SimWorld::new();
    sim.world_mut()
        .spawn((Transform::default(), Sprite::atlas(0, [1.0, 1.0], BRANCO)));
    let mut drive = PreviewDrive::default();
    assert_eq!(drive_tweens(&mut sim, &mut drive, &[]), 0);
    assert!(
        drive.is_empty(),
        "um passe sem sujeito declarou alguma coisa"
    );
}

/// ⭐⭐⭐ **UM SEGUIDOR COM PAI POUSA NA PISTA, E NÃO AO LADO DELA** (suplente #23).
///
/// ⚠️⚠️ **A curva vive em MUNDO e o `Transform` é LOCAL**, e escrever um no outro desloca o
/// seguidor **exactamente pela pose do pai** — aqui `(10, 4)`. O gate mede a pose de MUNDO do
/// seguidor (que é o que o artista vê) e não o `Transform` cru, porque *um local certo e um local
/// errado leem-se igual até alguém compor a cadeia*.
#[test]
fn um_seguidor_com_pai_pousa_onde_a_curva_manda() {
    let mut sim = SimWorld::new();
    let pai = sim
        .world_mut()
        .spawn((Transform {
            translation: ph2d_core::Vec2::new(10.0, 4.0),
            ..Transform::IDENTITY
        },))
        .id();
    let e = sim
        .world_mut()
        .spawn((Transform::IDENTITY, ph2d_ecs::ChildOf(pai)))
        .id();
    let mut drive = PreviewDrive::default();
    let pedido = crate::path_follow_bridge::PoseDeCaminho {
        entity: e,
        mundo: [3.0, -1.0],
        angulo: None,
    };
    assert_eq!(drive_tweens(&mut sim, &mut drive, &[pedido]), 1);
    let m = ph2d_ecs::world_transform(sim.world(), e).unwrap();
    assert!(
        (m.translation.x - 3.0).abs() < 1e-5,
        "x = {}",
        m.translation.x
    );
    assert!(
        (m.translation.y + 1.0).abs() < 1e-5,
        "y = {}",
        m.translation.y
    );
}

/// ⚠️ **A ESCALA e a ROTAÇÃO autoradas ficam** quando o seguidor não alinha — este motor conduz
/// *onde* e *para onde*, nunca *quão grande*.
#[test]
fn o_seguidor_nao_toca_na_escala_nem_na_rotacao_que_nao_conduz() {
    let mut sim = SimWorld::new();
    let e = sim
        .world_mut()
        .spawn((Transform {
            rotation: 1.25,
            scale: ph2d_core::Vec2::new(3.0, 0.5),
            ..Transform::IDENTITY
        },))
        .id();
    let mut drive = PreviewDrive::default();
    let pedido = crate::path_follow_bridge::PoseDeCaminho {
        entity: e,
        mundo: [7.0, 7.0],
        angulo: None,
    };
    drive_tweens(&mut sim, &mut drive, &[pedido]);
    let t = *sim.world().get::<Transform>(e).unwrap();
    assert_eq!(t.scale, ph2d_core::Vec2::new(3.0, 0.5));
    assert_eq!(t.rotation, 1.25);
    assert_eq!(t.translation, ph2d_core::Vec2::new(7.0, 7.0));
}

/// ⭐⭐⭐ **O LEDGER conduz o seguidor** — sem isto, um objecto a patrulhar seria **um passo de
/// `Ctrl+Z` por quadro**, que é a razão de ele escrever dentro deste passe e não fora dele.
#[test]
fn um_seguidor_e_pre_visualizacao_e_nao_documento() {
    let mut sim = SimWorld::new();
    let e = sim.world_mut().spawn((Transform::IDENTITY,)).id();
    let mut drive = PreviewDrive::default();
    drive_tweens(
        &mut sim,
        &mut drive,
        &[crate::path_follow_bridge::PoseDeCaminho {
            entity: e,
            mundo: [4.0, 0.0],
            angulo: None,
        }],
    );
    assert!(
        !drive.is_empty(),
        "o seguidor escreveu a pose e NÃO a declarou — o Ctrl+Z ia herdar a corrida"
    );
}
