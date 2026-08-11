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
    ProbeMark::ray(kind, [0.0, 0.0], [1.0, 0.0], 2.0, hit, 0.0)
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
    assert_eq!(cp.len(), 4, "sem acerto: a linha mais a PONTA do alcance");
    assert_eq!(hp.len(), 6, "com acerto: a linha, a ponta, e o tique do acerto");

    // O tique cruza o raio na distancia do acerto: o ponto MEDIO dos dois
    // ultimos pontos e' o ponto de acerto projetado. (Os dois anteriores sao a
    // PONTA, que fica no fim do alcance.)
    let mid = ((hp[4].0 + hp[5].0) * 0.5, (hp[4].1 + hp[5].1) * 0.5);
    let cam = camera();
    let (wx, wy) = (0.5_f32, 0.0_f32);
    let (sx, sy) = cam.world_to_screen([wx, wy], window());
    assert!(
        (mid.0 - f64::from(sx)).abs() < 0.5 && (mid.1 - f64::from(sy)).abs() < 0.5,
        "o tique senta no acerto: {mid:?} != ({sx}, {sy})"
    );
}

/// **O raio desenhado começa na BORDA do corpo, não na origem do cast.**
///
/// ⚠️ **RED-first, e o número é o do report:** o sensor de parede nasce no
/// CENTRO (o `exclude_body` precisa disso) e alcança `meia_largura + wall_reach`
/// — medido na cena 108, **35 px na tela dos quais 20 ficam por baixo do
/// contorno do collider**, deixando um toco de 15 px como tudo o que o artista
/// vê do número que ele está a mexer. O `skin` vem da MESMA porta que lançou o
/// raio; derivá-lo aqui seria a segunda resposta a *"onde acaba este corpo?"*.
#[test]
fn the_drawn_ray_starts_at_the_body_edge_not_at_the_cast_origin() {
    let sim = no_world();
    let (reach, skin) = (0.35_f32, 0.20_f32);
    let m = ProbeMark::ray(ProbeKind::Wall, [0.0, 0.0], [1.0, 0.0], reach, None, skin);
    let drawn = probe_marks(true, &[m], &sim, &camera(), window());
    let p = points(&drawn[0].0);

    let cam = camera();
    let want = cam.world_to_screen([skin, 0.0], window());
    assert!(
        (p[0].0 - f64::from(want.0)).abs() < 0.5,
        "a linha nasce na borda do corpo: {:?} != {want:?}",
        p[0]
    );
    // E o que se VE mede o alcance UTIL, nao o alcance do cast.
    let drawn_px = (p[1].0 - p[0].0).abs();
    let want_px = f64::from(reach - skin) * 100.0; // a camera do harness: 100 px/m
    assert!(
        (drawn_px - want_px).abs() < 0.5,
        "o desenho mede o alcance ALEM do corpo: {drawn_px:.1} != {want_px:.1} px"
    );
}

/// **Um alcance que nunca sobe continua VISÍVEL.**
///
/// ⚠️ **Este é o gate do report, e ele nasceu VERMELHO com 0,0 px:** o `rise` do
/// perfil vale `rel_up · dt · CORNER_LOOKAHEAD`, ou seja **zero sempre que a
/// cabeça não sobe** — medido, `rise = 0.0000 m` nos três momentos da cena 108,
/// e a barra saía com **0,0 px de altura**. As hastes que o roteiro do smoke
/// manda procurar não estavam fracas: **não existiam**.
///
/// Alongar o leque seria mentir (um sensor parado olha mesmo zero para cima); o
/// que dá corpo à marca são as **PONTAS**, que desenham o vão lateral — o número
/// autorado (`corner_reach`), que não é zero nunca.
#[test]
fn a_profile_that_never_rises_is_still_visible() {
    let sim = no_world();
    let m = ProbeMark::profile([0.0, 1.0], 0.2, 0.12, 0.0, None);
    let drawn = probe_marks(true, &[m], &sim, &camera(), window());
    let p = points(&drawn[0].0);
    let (y0, y1) = p.iter().fold((f64::MAX, f64::MIN), |(a, b), q| (a.min(q.1), b.max(q.1)));
    assert!(
        y1 - y0 >= 4.0,
        "com rise = 0 o vao ainda tem de ter CORPO na tela: {:.1} px de altura",
        y1 - y0
    );
    let (x0, x1) = p.iter().fold((f64::MAX, f64::MIN), |(a, b), q| (a.min(q.0), b.max(q.0)));
    assert!(
        x1 - x0 >= 60.0,
        "e a largura continua a ser o vao autorado: {:.1} px",
        x1 - x0
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

// ---------------------------------------------------------------------------
// SONDAS (`--ignored`) — o que o artista de facto VE, em px de tela.
// ---------------------------------------------------------------------------

/// Quanto mede, em px, o maior lado do que este caminho desenha.
fn extent_px(path: &ph2d_vector::BezPath) -> (f64, f64) {
    let p = points(path);
    let (mut x0, mut x1, mut y0, mut y1) = (f64::MAX, f64::MIN, f64::MAX, f64::MIN);
    for (x, y) in &p {
        x0 = x0.min(*x);
        x1 = x1.max(*x);
        y0 = y0.min(*y);
        y1 = y1.max(*y);
    }
    (x1 - x0, y1 - y0)
}

/// **O QUE O ARTISTA VE** — mede cada marca da cena 108 em px de tela.
///
/// A camera do harness e' 1000 px / 10 m = **100 px/m**, a mesma ordem do app.
#[test]
#[ignore = "sonda: mede, nao afirma"]
fn measure_what_each_mark_measures_on_screen() {
    use ph2d_physics_ecs::{PhysicsBridge, PlayerInput, ProbeShape};

    let run = |label: &str, x: f32, y: f32, hold: PlayerInput, n: u64| {
        let mut sim = SimWorld::new();
        let player = crate::physics_smoke_probes::build_probe_scene(sim.world_mut());
        {
            let mut t = sim.world_mut().get_mut::<Transform>(player).unwrap();
            t.translation.x = x;
            t.translation.y = y;
        }
        let mut bridge = PhysicsBridge::new();
        for i in 1..=n {
            bridge.set_player_input(player, hold);
            bridge.dispatch(&mut sim, true, i);
        }
        let marks = bridge.player_probe_marks().to_vec();
        println!("\n== {label} ==");
        for m in &marks {
            let shape = match m.shape {
                ProbeShape::Ray { reach, hit, .. } => {
                    format!("ray reach={reach:.3}m hit={hit:?}")
                }
                ProbeShape::Profile {
                    half_width,
                    reach,
                    rise,
                    ..
                } => format!("profile half_w={half_width:.3} reach={reach:.3} rise={rise:.4}m"),
                ProbeShape::Sweep { offset, .. } => format!("sweep dy={:.3}m", offset[1]),
            };
            let drawn = probe_marks(true, std::slice::from_ref(m), &sim, &camera(), window());
            for (path, rgba) in &drawn {
                let (w, h) = extent_px(path);
                println!(
                    "  {:<9?} {:<5?} {shape:<46} -> {w:6.1} x {h:6.1} px  alpha {:.2}",
                    m.kind, m.state, rgba[3]
                );
            }
        }
    };

    let idle = PlayerInput::default();
    run("PARADO no chao", 2.0, 0.9, idle, 40);
    run(
        "EMPURRANDO a parede, no ar",
        crate::physics_smoke_probes::WALL_FACE_X - 0.25,
        2.5,
        PlayerInput {
            drive: 1.0,
            ..idle
        },
        20,
    );
    run(
        "SUBINDO junto da quina",
        crate::physics_smoke_probes::LEDGE_EDGE_X - 0.1,
        1.4,
        PlayerInput {
            jump: true,
            ..idle
        },
        4,
    );
}

/// **A LEITURA ACOMPANHA O CORPO FORA DO RUNTIME?** — o report do Enio.
#[test]
#[ignore = "sonda: mede, nao afirma"]
fn measure_whether_the_reading_follows_a_dragged_body() {
    use ph2d_physics_ecs::{PhysicsBridge, PlayerInput, ProbeShape};

    let mut sim = SimWorld::new();
    let player = crate::physics_smoke_probes::build_probe_scene(sim.world_mut());
    {
        let mut t = sim.world_mut().get_mut::<Transform>(player).unwrap();
        t.translation.x = 2.0;
        t.translation.y = 0.9;
    }
    let mut bridge = PhysicsBridge::new();
    for i in 1..=40 {
        bridge.set_player_input(player, PlayerInput::default());
        bridge.dispatch(&mut sim, true, i);
    }
    let leg_x = |b: &PhysicsBridge| {
        b.player_probe_marks()
            .iter()
            .find_map(|m| match m.shape {
                ProbeShape::Ray { origin, .. } if m.kind == ProbeKind::Ground => Some(origin[0]),
                _ => None,
            })
            .unwrap_or(f32::NAN)
    };
    println!("\n== arrastar o corpo com o relogio PARADO ==");
    println!("apos 40 ticks: corpo x=2.000  perna x={:.3}", leg_x(&bridge));

    // O artista arrasta o corpo 3 m para a direita, com o relogio no mesmo tick.
    {
        let mut t = sim.world_mut().get_mut::<Transform>(player).unwrap();
        t.translation.x = 5.0;
    }
    // ⚠️ `playing = FALSE`. Com `true` o ramo `Equal` nao assenta nada (um quadro
    // mais rapido que o tique nao pode mexer no mundo tocando), entao a fixture
    // NAO conteria o fenomeno — foi assim que a 1a versao desta sonda mediu
    // "nao segue" sobre um produto ja corrigido.
    bridge.dispatch(&mut sim, false, 40); // MESMO tick, PAUSADO = o ramo `Equal`
    println!(
        "arrastado p/ x=5.000, mesmo tick: perna x={:.3}  <-- devia ser 5.000",
        leg_x(&bridge)
    );

    bridge.hold(&mut sim, 40);
    println!(
        "com o toggle Physics DESMARCADO (hold): {} marca(s)",
        bridge.player_probe_marks().len()
    );
}
