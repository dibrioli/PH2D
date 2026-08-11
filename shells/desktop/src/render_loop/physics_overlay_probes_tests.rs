//! **Os gates do desenho dos sensores** (`W-Probes`).
//!
//! ⚠️ **Eles julgam o DESENHO, e a leitura tem os dela** (`ph2d-physics-ecs`,
//! `tests/player_probe_view.rs`): aqui a pergunta é *"o que a marca publicada
//! vira na tela?"*, não *"a marca está certa?"*.

use super::probe_marks;
use crate::render_loop::physics_overlay::tests::{camera, points, window};
use ph2d_core::Vec2;
use ph2d_ecs::{Name, SimWorld, Transform};
use ph2d_physics_ecs::{
    BodyKind, Collider, ColliderShape, ProbeKind, ProbeMark, ProbeState, RigidBody,
};

fn no_world() -> SimWorld {
    SimWorld::new()
}

fn ray(kind: ProbeKind, hit: Option<f32>) -> ProbeMark {
    ProbeMark::ray(kind, [0.0, 0.0], [1.0, 0.0], 2.0, hit)
}

/// **O interruptor manda, e uma lista vazia não desenha nada.**
///
/// Os sensores acompanham o MESMO toggle do contorno — é a mesma pergunta
/// (*mostre-me a física que não se vê*), e um segundo interruptor seria a
/// segunda porta para ela.
#[test]
fn the_toggle_governs_and_an_empty_reading_draws_nothing() {
    let sim = no_world();
    let m = [ray(ProbeKind::Ground, None)];
    assert!(
        probe_marks(false, &m, &sim, &camera(), window()).is_empty(),
        "desligado nao desenha"
    );
    assert!(
        probe_marks(true, &[], &sim, &camera(), window()).is_empty(),
        "sem leitura nao desenha"
    );
    assert!(
        !probe_marks(true, &m, &sim, &camera(), window()).is_empty(),
        "ligado, com leitura, desenha"
    );
}

/// **Os TRÊS estados são distinguíveis, e na ordem certa.**
///
/// ⚠️ Este é o gate da metade nova: sem o `Idle` mais apagado que o `Clear`, as
/// respostas *"a capacidade não é a hora"* e *"perguntou e não achou"* leem
/// igual — e são vereditos opostos para quem está a afinar o alcance.
#[test]
fn the_three_states_read_differently_and_in_order() {
    let sim = no_world();
    let alpha = |st: ProbeState| {
        let mut m = ray(ProbeKind::Wall, None);
        m.state = st;
        probe_marks(true, &[m], &sim, &camera(), window())[0].1[3]
    };
    let (idle, clear, hit) = (
        alpha(ProbeState::Idle),
        alpha(ProbeState::Clear),
        alpha(ProbeState::Hit),
    );
    assert!(
        idle < clear && clear < hit,
        "inerte < perguntou < achou: {idle} / {clear} / {hit}"
    );
    assert!(idle > 0.0, "inerte ainda tem de aparecer: {idle}");
}

/// **Um acerto ganha um TIQUE, e ele fica ONDE o sensor achou.**
///
/// ⚠️ O tique é a metade que responde *quanto* — o número que o artista está a
/// afinar. Uma diferença só de brilho diria *"achou"* e não *"achou aqui"*.
#[test]
fn a_hit_gets_a_tick_where_it_was_found() {
    let sim = no_world();
    let clear = probe_marks(
        true,
        &[ray(ProbeKind::Ground, None)],
        &sim,
        &camera(),
        window(),
    );
    let hit = probe_marks(
        true,
        &[ray(ProbeKind::Ground, Some(0.5))],
        &sim,
        &camera(),
        window(),
    );
    let (cp, hp) = (points(&clear[0].0), points(&hit[0].0));
    assert_eq!(cp.len(), 2, "sem acerto e' so' a linha");
    assert_eq!(hp.len(), 4, "com acerto, a linha mais o tique");

    // O tique cruza o raio na distancia do acerto: o ponto MEDIO dos dois
    // ultimos pontos e' o ponto de acerto projetado.
    let mid = ((hp[2].0 + hp[3].0) * 0.5, (hp[2].1 + hp[3].1) * 0.5);
    let cam = camera();
    let (wx, wy) = (0.5_f32, 0.0_f32);
    let (sx, sy) = cam.world_to_screen([wx, wy], window());
    assert!(
        (mid.0 - f64::from(sx)).abs() < 0.5 && (mid.1 - f64::from(sy)).abs() < 0.5,
        "o tique senta no acerto: {mid:?} != ({sx}, {sy})"
    );
}

/// **O perfil do teto desenha o vão que a LEI varre.**
///
/// ⚠️ O gate compara contra [`ph2d_physics_ecs::corner_offsets`] — a porta —, não
/// contra uma aritmética repetida aqui. Meia célula de desvio poria o desenho ao
/// lado do que a assistência de facto leu, e nada na tela diria isso.
#[test]
fn the_ceiling_profile_spans_what_the_law_scans() {
    let sim = no_world();
    let (half_width, reach, rise) = (0.2_f32, 0.12_f32, 0.3_f32);
    let m = ProbeMark::profile([0.0, 1.0], half_width, reach, rise, None);
    let drawn = probe_marks(true, &[m], &sim, &camera(), window());
    assert_eq!(drawn.len(), 1, "sem obstrucao, so' o vao");

    let offs = ph2d_physics_ecs::corner_offsets(half_width, reach);
    let cam = camera();
    let want_lo = cam.world_to_screen([offs[0], 1.0 + rise], window());
    let want_hi = cam.world_to_screen([offs[offs.len() - 1], 1.0 + rise], window());
    let p = points(&drawn[0].0);
    assert!(
        (p[0].0 - f64::from(want_lo.0)).abs() < 0.5,
        "a ponta esquerda do vao sai da porta: {:?} != {want_lo:?}",
        p[0]
    );
    assert!(
        (p[1].0 - f64::from(want_hi.0)).abs() < 0.5,
        "a ponta direita do vao sai da porta: {:?} != {want_hi:?}",
        p[1]
    );
}

/// **Uma célula TAPADA é desenhada com o brilho de acerto.**
///
/// Sem obstrução o perfil não tem nada a dizer além do vão — e é isso que o
/// gate acima pina; aqui a metade oposta.
#[test]
fn a_blocked_cell_is_drawn_at_hit_intensity() {
    let sim = no_world();
    let mut blocked = [false; ph2d_platformer_samples()];
    blocked[10] = true;
    let m = ProbeMark::profile([0.0, 1.0], 0.2, 0.12, 0.3, Some(blocked));
    let drawn = probe_marks(true, &[m], &sim, &camera(), window());
    assert_eq!(drawn.len(), 2, "o vao e as celulas tapadas");
    assert!(
        drawn[1].1[3] > drawn[0].1[3],
        "a celula tapada e' mais forte que o vao: {} vs {}",
        drawn[1].1[3],
        drawn[0].1[3]
    );
    assert_eq!(
        m.state,
        ProbeState::Hit,
        "com celula tapada, o perfil ACHOU"
    );
}

/// Quantas amostras o perfil tem — lido da porta, nunca escrito à mão.
const fn ph2d_platformer_samples() -> usize {
    ph2d_physics_ecs::CORNER_SAMPLES
}

/// **A varredura do agachar é o CORPO desenhado onde ele quer ficar de pé.**
///
/// ⚠️ Um overlay que só soubesse linhas desenharia este sensor como um raio que
/// ele não é, e o artista afinaria o `crouch_height` contra um desenho que
/// mente sobre o que o produto mede.
#[test]
fn the_crouch_sweep_is_the_body_drawn_where_it_wants_to_stand() {
    let mut sim = SimWorld::new();
    let body = sim
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
            Transform::from_translation(Vec2::new(0.0, 0.5)),
        ))
        .id();

    let rise = 0.4_f32;
    let m = ProbeMark::sweep(body, [0.0, rise], Some(false));
    let drawn = probe_marks(true, &[m], &sim, &camera(), window());
    assert_eq!(drawn.len(), 1, "um corpo, uma silhueta fantasma");

    // O fantasma e' o contorno VIVO deslocado: a diferenca de altura media, em
    // px de tela, e' exatamente `rise` projetado.
    let ghost_y: f64 =
        points(&drawn[0].0).iter().map(|p| p.1).sum::<f64>() / points(&drawn[0].0).len() as f64;
    let m0 = ProbeMark::sweep(body, [0.0, 0.0], Some(false));
    let base = probe_marks(true, &[m0], &sim, &camera(), window());
    let base_y: f64 =
        points(&base[0].0).iter().map(|p| p.1).sum::<f64>() / points(&base[0].0).len() as f64;

    let cam = camera();
    let a = cam.world_to_screen([0.0, 0.0], window());
    let b = cam.world_to_screen([0.0, rise], window());
    let want = f64::from(b.1 - a.1);
    assert!(
        ((ghost_y - base_y) - want).abs() < 0.5,
        "o fantasma sobe exatamente a subida: {} != {want}",
        ghost_y - base_y
    );
}
