//! The contact marks: where they sit, and what their size means.

use super::*;
use ph2d_ecs::Entity;

fn window() -> WindowSize {
    WindowSize {
        width: 1000,
        height: 1000,
    }
}

fn camera() -> Camera2d {
    Camera2d {
        center: [0.0, 0.0],
        height_world: 10.0,
        ..Camera2d::default()
    }
}

/// A contact with **no beginning** — the state a pair is in after a re-baseline, and
/// the one these load/placement tests want: they measure the standing `+`, and a pair
/// that is flashing would put a second cross in the same picture.
///
/// The premise is declared rather than inherited: a fixture that reaches its state by
/// leaning on a default silently changes meaning the day the default moves, and stays
/// green while testing the opposite.
fn contact(point: [f32; 2], impulse: f32) -> BodyContact {
    BodyContact {
        a: Entity::from_bits(1),
        b: Entity::from_bits(2),
        point,
        impulse,
        // Default the impact peak to the load — a standing contact's peak is at
        // least its load, and these placement/load tests do not exercise impact.
        // The flash tests that DO set it explicitly via `contact_impact`.
        impact: impulse,
        age_ticks: None,
    }
}

/// The same pair, `age` ticks after it BEGAN touching — what flashes.
fn contact_aged(point: [f32; 2], impulse: f32, age: u64) -> BodyContact {
    BodyContact {
        age_ticks: Some(age),
        ..contact(point, impulse)
    }
}

/// A fresh contact with a chosen IMPACT peak — for the flash tests, where the
/// size rides impact, not load. `age 0` and load fixed, so only the impact varies.
fn contact_impact(impact: f32, age: u64) -> BodyContact {
    BodyContact {
        impact,
        age_ticks: Some(age),
        ..contact([0.0, 0.0], 0.01)
    }
}

/// Half the width of a mark's horizontal arm, in screen px — the size the load maps
/// to. Read off the path so the test measures what is DRAWN, not what the constant
/// says.
fn arm_px(path: &BezPath) -> f64 {
    let pts: Vec<Point> = path
        .elements()
        .iter()
        .filter_map(|el| match el {
            ph2d_vector::PathEl::MoveTo(p) | ph2d_vector::PathEl::LineTo(p) => Some(*p),
            _ => None,
        })
        .collect();
    (pts[1].x - pts[0].x) / 2.0
}

/// The half-diagonal a flash spans, in screen px — measured off the drawn path, like
/// [`arm_px`], and converted back to the arm the code speaks in.
fn flash_arm_px(path: &BezPath) -> f64 {
    let pts: Vec<Point> = path
        .elements()
        .iter()
        .filter_map(|el| match el {
            ph2d_vector::PathEl::MoveTo(p) | ph2d_vector::PathEl::LineTo(p) => Some(*p),
            _ => None,
        })
        .collect();
    let half_diag = (pts[1].x - pts[0].x) / 2.0;
    half_diag * std::f64::consts::SQRT_2
}

/// A pair that BEGAN this tick gets a flash; one adopted at a re-baseline does not.
///
/// ⚠️ This is the overlay half of the wave's central rule. `age_ticks: None` means
/// "already touching when the timeline jumped" — a scrub, or re-arming the Physics
/// toggle — and flashing those would light up the whole scene every time the artist
/// drags the ruler, announcing collisions that never happened.
#[test]
fn a_fresh_contact_flashes_and_a_re_baselined_one_never_does() {
    let fresh = contact_aged([0.0, 0.0], 0.01, 0);
    let adopted = contact([0.0, 0.0], 0.01);

    assert_eq!(
        contact_flashes(true, &[fresh], &camera(), window()).len(),
        1,
        "a contact that began this tick flashes"
    );
    assert!(
        contact_flashes(true, &[adopted], &camera(), window()).is_empty(),
        "a pair with no beginning has nothing to announce"
    );
    // And the standing cross is drawn either way — the flash is an ADDITION, never a
    // replacement, or a scrub would erase the marks it must keep showing.
    assert_eq!(contact_marks(true, &[fresh], &camera(), window()).len(), 1);
    assert_eq!(
        contact_marks(true, &[adopted], &camera(), window()).len(),
        1
    );
}

/// The flash EXPANDS as it ages, and dies. Expansion is what makes it read as a
/// burst instead of as a second, brighter contact sitting there.
#[test]
fn the_flash_expands_with_age_and_then_stops() {
    let cam = camera();
    let at = |age: u64| {
        contact_flashes(true, &[contact_aged([0.0, 0.0], 0.01, age)], &cam, window())
            .first()
            .map(flash_arm_px)
    };

    let young = at(0).expect("a brand-new contact flashes");
    let older = at(FLASH_TICKS - 1).expect("it is still alive on its last tick");
    assert!(
        older > young + 1.0,
        "the flash must visibly grow: {young:.1} px -> {older:.1} px"
    );
    assert!(
        at(FLASH_TICKS).is_none(),
        "and it must stop, or every settled contact in the scene stays lit"
    );
    assert!(at(FLASH_TICKS + 50).is_none(), "long-standing contacts too");
}

/// ⚠️ The flash must NOT ride the arm length, because arm length already means LOAD.
///
/// Two meanings on one channel is how a brand-new light touch becomes
/// indistinguishable from an old heavy one. The gate states it as the property rather
/// than as "it is diagonal": the flash of a LIGHT fresh contact has to be bigger than
/// the standing mark of a HEAVY one, which can only be true if they are separate
/// marks.
#[test]
fn the_flash_does_not_steal_the_channel_that_means_load() {
    let cam = camera();
    let light_fresh = contact_aged([0.0, 0.0], 0.0, 0);
    let heavy_old = contact([0.0, 0.0], LOAD_FULL_NS * 4.0);

    let mark_light = contact_marks(true, &[light_fresh], &cam, window());
    let mark_heavy = contact_marks(true, &[heavy_old], &cam, window());
    assert!(
        arm_px(&mark_heavy[0]) > arm_px(&mark_light[0]),
        "the upright cross still says load, and only load"
    );

    let flash = contact_flashes(true, &[light_fresh], &cam, window());
    assert!(
        flash_arm_px(&flash[0]) > arm_px(&mark_heavy[0]),
        "and a light NEW contact still announces itself over a heavy old one"
    );
}

/// ⚠️ The flash size reads the IMPACT peak (W-ImpactForce): a hard hit flashes
/// bigger than a gentle one, at the SAME age. This is the visible half of the wave —
/// without it the captured peak would be a number no one can see.
///
/// The gate fixes age (both fresh) and load (both the helper's default) so only the
/// impact differs, and asserts the hard flash out-reads the gentle one by a real
/// margin. Mutating the boost away (flash ignores impact) makes them equal — RED.
#[test]
fn a_harder_impact_flashes_bigger() {
    let cam = camera();
    let gentle = contact_impact(0.05, 0); // barely a tap
    let hard = contact_impact(IMPACT_FULL_NS, 0); // a full-scale slam

    let gentle_arm = flash_arm_px(&contact_flashes(true, &[gentle], &cam, window())[0]);
    let hard_arm = flash_arm_px(&contact_flashes(true, &[hard], &cam, window())[0]);
    assert!(
        hard_arm > gentle_arm + FLASH_IMPACT_BOOST_PX * 0.5,
        "a hard impact must flash visibly bigger than a gentle one: \
         gentle {gentle_arm:.1} px vs hard {hard_arm:.1} px"
    );
    // And the floor still holds: even the gentlest flash clears the biggest standing
    // load cross, so the event is never hidden inside the load it announces.
    assert!(
        gentle_arm > MARK_MAX_PX,
        "the flash floor survives the impact ruler"
    );
}

/// The overlay toggle switches the flashes off with everything else — a refusal that
/// lives in the data, not in the paint loop.
#[test]
fn the_toggle_switches_the_flashes_off() {
    let fresh = contact_aged([0.0, 0.0], 0.01, 0);
    assert!(contact_flashes(false, &[fresh], &camera(), window()).is_empty());
}

#[test]
fn a_heavier_load_draws_a_bigger_mark() {
    // The size IS the reading. A stack shows a gradient because the bottom joint
    // carries everything above it, and if every mark were the same size the overlay
    // would be answering "are these touching" while pretending to answer "how hard".
    let light = &contact_marks(true, &[contact([0.0, 0.0], 0.005)], &camera(), window())[0];
    let heavy = &contact_marks(true, &[contact([0.0, 0.0], 0.045)], &camera(), window())[0];
    assert!(
        arm_px(heavy) > arm_px(light) * 1.5,
        "a 9x load should draw a visibly bigger mark ({} vs {})",
        arm_px(heavy),
        arm_px(light)
    );
    // And it saturates rather than growing without bound — past the ruler, "very
    // loaded" is the useful reading and a mark the size of the screen is not.
    let huge = &contact_marks(true, &[contact([0.0, 0.0], 500.0)], &camera(), window())[0];
    assert!(
        (arm_px(huge) - arm_px(heavy)).abs() < 1.0,
        "the mark must saturate at the ruler, got {} vs {}",
        arm_px(huge),
        arm_px(heavy)
    );
}

#[test]
fn the_mark_sits_on_the_contact_point_in_screen_space() {
    // A camera 10 world units tall over a 1000 px window is 100 px per unit, and the
    // centre of the world is the centre of the screen. A contact 2 units to the right
    // is therefore 200 px right of centre — the arithmetic a mark drawn at the body's
    // centre, or in world units, would get wrong.
    let marks = contact_marks(true, &[contact([2.0, 0.0], 0.01)], &camera(), window());
    let pts: Vec<Point> = marks[0]
        .elements()
        .iter()
        .filter_map(|el| match el {
            ph2d_vector::PathEl::MoveTo(p) | ph2d_vector::PathEl::LineTo(p) => Some(*p),
            _ => None,
        })
        .collect();
    let cx = (pts[0].x + pts[1].x) / 2.0;
    let cy = (pts[2].y + pts[3].y) / 2.0;
    assert!(
        (cx - 700.0).abs() < 1.0 && (cy - 500.0).abs() < 1.0,
        "the mark should be centred at (700, 500) px, got ({cx}, {cy})"
    );
}

#[test]
fn the_toggle_switches_the_marks_off() {
    assert!(
        contact_marks(false, &[contact([0.0, 0.0], 0.01)], &camera(), window()).is_empty(),
        "a painter or vector user must never see physics chrome over their artwork"
    );
    assert!(
        contact_marks(true, &[], &camera(), window()).is_empty(),
        "a scene where nothing touches draws nothing"
    );
}

#[test]
fn the_waterline_is_drawn_across_the_pool_and_obeys_the_toggle() {
    // A metade visível do empuxo. A geometria vem PRONTA da física, então o que este
    // gate cobre é a projeção: a linha tem de sair na altura certa da tela, atravessar a
    // poça, e sumir com o toggle.
    //
    // Câmera de 10 unidades de mundo sobre 1000 px = 100 px por unidade, centro do mundo
    // no centro da tela. Uma superfície em y = 0 cai em 500 px; de x = −3,5 a +3,5 vira
    // 150..850 px, mais a sobra de 6 px em cada ponta.
    let marks = waterline_marks(true, &[([-3.5, 0.0], [3.5, 0.0])], &camera(), window());
    assert_eq!(marks.len(), 1);
    let pts: Vec<Point> = marks[0]
        .elements()
        .iter()
        .filter_map(|el| match el {
            ph2d_vector::PathEl::MoveTo(p) | ph2d_vector::PathEl::LineTo(p) => Some(*p),
            _ => None,
        })
        .collect();
    assert!(
        (pts[0].y - 500.0).abs() < 1.0 && (pts[1].y - 500.0).abs() < 1.0,
        "a superfície em y = 0 tem de cair no meio da tela, e caiu em {pts:?}"
    );
    assert!(
        (pts[0].x - 144.0).abs() < 1.0 && (pts[1].x - 856.0).abs() < 1.0,
        "a linha tem de atravessar a poça inteira mais a sobra de 6 px, e ficou em {pts:?}"
    );

    assert!(
        waterline_marks(false, &[([-1.0, 0.0], [1.0, 0.0])], &camera(), window()).is_empty(),
        "a linha d'água obedece ao mesmo toggle que o resto do chrome de física"
    );
    assert!(
        waterline_marks(true, &[], &camera(), window()).is_empty(),
        "uma cena sem poça não desenha nada"
    );
}

#[test]
fn a_tilted_pool_draws_a_level_waterline() {
    // ⚠️ O gate que exige que a linha venha da FÍSICA e não do topo da caixa. Água é
    // horizontal mesmo numa poça torta, e é isso que a `surface_level` (perpendicular à
    // gravidade) devolve — um overlay que desenhasse a aresta de cima do collider daria
    // uma linha INCLINADA, e as duas só discordam aqui.
    //
    // A entrada é o que a física de fato produz para uma poça rotacionada: dois pontos
    // na MESMA altura, mas afastados do centro.
    let marks = waterline_marks(true, &[([-2.0, 1.4], [2.6, 1.4])], &camera(), window());
    let pts: Vec<Point> = marks[0]
        .elements()
        .iter()
        .filter_map(|el| match el {
            ph2d_vector::PathEl::MoveTo(p) | ph2d_vector::PathEl::LineTo(p) => Some(*p),
            _ => None,
        })
        .collect();
    assert!(
        (pts[0].y - pts[1].y).abs() < 1e-6,
        "os dois extremos têm de sair na MESMA altura de tela ({pts:?})"
    );
}
