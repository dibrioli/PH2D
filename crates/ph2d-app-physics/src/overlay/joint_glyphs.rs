//! **Os GLIFOS de um joint — uma figura por tipo (W-J1, vocabulário do RUBE).**
//!
//! Comprimento é MUNDO projetado (o anel cresce com o zoom); ângulo e ornamento
//! são px de TELA constantes (a lei de [`crate::overlay::annotations`]).
//! ⚠️ Figura angular usa a BASE da câmera (`û`, `ŵ`), nunca ângulos de tela à mão:
//! assim honra qualquer y-flip.

use ph2d_host::WindowSize;
use ph2d_render::Camera2d;
use ph2d_vector::{BezPath, Point};

/// Cordas num arco (como o glifo de torque). ⚠️ Partilhado com a corda na
/// roldana (`physics_overlay_pulley`): o mesmo círculo.
pub const ARC_SEGS: u32 = 24;

/// Raio do anel do PINO, px de tela (maior que a ponta `JOINT_DOT_PX`).
pub const PIN_RING_PX: f64 = 6.5; // LITERAL-PX-OK: chrome de overlay

/// Meia-diagonal do quadrado do WELD, px de tela.
pub const WELD_HALF_PX: f64 = 5.5; // LITERAL-PX-OK: chrome de overlay
/// Raio do hexágono do CUSTOM, px de tela (distinguível do pino e da solda).
pub const CUSTOM_HEX_PX: f64 = 7.0; // LITERAL-PX-OK: chrome de overlay

/// Meia-largura da BARRA, px de tela (< `PIN_RING_PX`: sai de dentro dos anéis).
pub const ROD_HALF_PX: f64 = 2.5; // LITERAL-PX-OK: chrome de overlay

/// Raio do arco de LIMITE, px de tela — fora do anel do pino, que ele cerca.
pub const LIMIT_ARC_PX: f64 = 21.0; // LITERAL-PX-OK: chrome de overlay

/// Comprimento das marcas radiais nas pontas do arco de limite (as "paredes").
const LIMIT_TICK_PX: f64 = 6.0; // LITERAL-PX-OK: chrome de overlay

/// Amplitude do zigue-zague da MOLA, px de tela.
const SPRING_AMP_PX: f64 = 5.0; // LITERAL-PX-OK: chrome de overlay

/// Quantas meias-ondas o zigue-zague tem. Ímpar não fecha simétrico; par sim.
const SPRING_HUMPS: u32 = 8;

/// A projeção mundo→tela deste módulo, num lugar só.
pub fn screen_of(camera: &Camera2d, window: WindowSize, w: [f32; 2]) -> Point {
    let (sx, sy) = camera.world_to_screen(w, window);
    Point::new(f64::from(sx), f64::from(sy))
}

/// A base de tela da câmera `(û, ŵ)` (+x e +y de mundo): ângulo `a` cai em
/// `centro + R·(cos a·û + sen a·ŵ)`, seja qual for o flip.
fn screen_basis(
    camera: &Camera2d,
    window: WindowSize,
    centre: [f32; 2],
) -> (Point, (f64, f64), (f64, f64)) {
    let c = screen_of(camera, window, centre);
    let px = screen_of(camera, window, [centre[0] + 1.0, centre[1]]);
    let py = screen_of(camera, window, [centre[0], centre[1] + 1.0]);
    let norm = |vx: f64, vy: f64| {
        let l = vx.hypot(vy);
        if l < 1e-9 {
            (0.0, 0.0)
        } else {
            (vx / l, vy / l)
        }
    };
    (
        c,
        norm(px.x - c.x, px.y - c.y),
        norm(py.x - c.x, py.y - c.y),
    )
}

/// Um ponto sobre um arco, na base de tela já resolvida.
fn arc_point_in(c: Point, u: (f64, f64), v: (f64, f64), ang: f64, r: f64) -> Point {
    Point::new(
        c.x + r * (ang.cos() * u.0 + ang.sin() * v.0),
        c.y + r * (ang.cos() * u.1 + ang.sin() * v.1),
    )
}

/// **Onde uma parede do arco de limite está, em px de tela** (W-J3) — a mesma
/// conta de [`limit_arc`]: grip e parede são um lugar só.
pub fn limit_end_screen(
    camera: &Camera2d,
    window: WindowSize,
    centre_w: [f32; 2],
    angle_a: f32,
    limit: f32,
) -> Point {
    let (c, u, v) = screen_basis(camera, window, centre_w);
    arc_point_in(c, u, v, f64::from(angle_a) + f64::from(limit), LIMIT_ARC_PX)
}

/// Meia-extensão do trilho de um Slider SEM limites, px de tela (só DIREÇÃO).
const RAIL_PX: f64 = 26.0; // LITERAL-PX-OK: chrome de overlay

/// Meio-comprimento dos tracinhos de fim de curso, px de tela (irmão de `LIMIT_TICK_PX`).
const RAIL_TICK_PX: f64 = 7.0; // LITERAL-PX-OK: chrome de overlay

/// **O TRILHO de um Slider** (W-J5): reta pelo eixo, tracinhos nos fins de curso.
///
/// ⚠️ Fins de curso são MUNDO (metros); espessura, tracinhos e trilho ILIMITADO
/// são chrome. `axis_w` vem da ponte (`JointView::axis`), nunca re-derivado.
pub fn slider_rail(
    camera: &Camera2d,
    window: WindowSize,
    anchor_w: [f32; 2],
    axis_w: [f32; 2],
    limits: Option<[f32; 2]>,
) -> BezPath {
    let mut p = BezPath::new();
    let along = |t: f32| {
        screen_of(
            camera,
            window,
            [anchor_w[0] + axis_w[0] * t, anchor_w[1] + axis_w[1] * t],
        )
    };
    // Eixo JÁ PROJETADO (dois pontos dele): honra qualquer flip.
    let c = along(0.0);
    let ahead = along(1.0);
    let (dx, dy) = (ahead.x - c.x, ahead.y - c.y);
    let len = dx.hypot(dy);
    if len < 1e-9 {
        return p; // câmera degenerada: nada honesto a desenhar
    }
    let (ux, uy) = (dx / len, dy / len);
    let (perp_x, perp_y) = (-uy, ux);
    let (from, to) = match limits {
        // Com curso: a reta vai de ponta a ponta do alcance REAL.
        Some([min, max]) => (along(min), along(max)),
        // Sem curso: só a direção, num comprimento de chrome.
        None => (
            Point::new(c.x - ux * RAIL_PX, c.y - uy * RAIL_PX),
            Point::new(c.x + ux * RAIL_PX, c.y + uy * RAIL_PX),
        ),
    };
    p.move_to(from);
    p.line_to(to);
    // Tracinhos só quando há fim de curso.
    if limits.is_some() {
        end_ticks([from, to], (perp_x, perp_y), &mut p);
    }
    p
}

/// **Os tracinhos de FIM DE CURSO** em px de tela — porta única de
/// [`slider_rail`] e [`wheel_strut`].
fn end_ticks(ends: [Point; 2], perp: (f64, f64), p: &mut BezPath) {
    for end in ends {
        p.move_to(Point::new(
            end.x - perp.0 * RAIL_TICK_PX,
            end.y - perp.1 * RAIL_TICK_PX,
        ));
        p.line_to(Point::new(
            end.x + perp.0 * RAIL_TICK_PX,
            end.y + perp.1 * RAIL_TICK_PX,
        ));
    }
}

/// **O glifo da RODA** — o cubo (um anel) mais a MOLA da suspensão ao longo do
/// eixo, e os batentes de curso quando eles existem.
///
/// ⚠️ COMPOSTO de propósito: anel do [`pin_glyph`] + zigue-zague do
/// [`spring_zigzag`] + tracinhos do [`slider_rail`]; distinto dos vizinhos por
/// GEOMETRIA. Eixo da ponte; fins de curso em MUNDO.
pub fn wheel_strut(
    camera: &Camera2d,
    window: WindowSize,
    anchor_w: [f32; 2],
    axis_w: [f32; 2],
    limits: Option<[f32; 2]>,
) -> BezPath {
    let mut p = BezPath::new();
    let along = |t: f32| {
        screen_of(
            camera,
            window,
            [anchor_w[0] + axis_w[0] * t, anchor_w[1] + axis_w[1] * t],
        )
    };
    let c = along(0.0);
    let ahead = along(1.0);
    let (dx, dy) = (ahead.x - c.x, ahead.y - c.y);
    let len = dx.hypot(dy);
    if len < 1e-9 {
        return p; // câmera degenerada: nada honesto a desenhar
    }
    let (ux, uy) = (dx / len, dy / len);
    let (perp_x, perp_y) = (-uy, ux);
    // O CUBO: o mesmo anel do pino, porque é a mesma afirmação.
    ring_px(c, PIN_RING_PX, &mut p);
    // A MOLA: da borda do anel, ao longo do eixo, num comprimento de chrome.
    let base = Point::new(c.x + ux * PIN_RING_PX, c.y + uy * PIN_RING_PX);
    let tip = Point::new(c.x + ux * RAIL_PX, c.y + uy * RAIL_PX);
    p.extend(spring_zigzag(base, tip).iter());
    // Batentes em metros. ⚠️ Comprimir é POSITIVO: `max` = compressão, `min` = extensão (medido).
    if let Some([min, max]) = limits {
        end_ticks([along(min), along(max)], (perp_x, perp_y), &mut p);
    }
    p
}

/// **Onde o grip do anel de comprimento está, em MUNDO** (W-J3): sobre o anel,
/// na direção de `toward` (B). Degenerado (B sobre A): onde [`ring_px`] começa.
pub fn length_handle_world(centre_w: [f32; 2], toward: [f32; 2], length: f32) -> [f32; 2] {
    let (dx, dy) = (toward[0] - centre_w[0], toward[1] - centre_w[1]);
    let d = dx.hypot(dy);
    let (ux, uy) = if d < 1e-6 {
        (1.0, 0.0)
    } else {
        (dx / d, dy / d)
    };
    [centre_w[0] + ux * length, centre_w[1] + uy * length]
}

/// Um anel de raio em PIXELS DE TELA, centrado num ponto de tela.
pub fn ring_px(centre: Point, radius_px: f64, path: &mut BezPath) {
    path.move_to(Point::new(centre.x + radius_px, centre.y));
    for i in 1..=ARC_SEGS {
        let th = std::f64::consts::TAU * f64::from(i) / f64::from(ARC_SEGS);
        path.line_to(Point::new(
            centre.x + radius_px * th.cos(),
            centre.y + radius_px * th.sin(),
        ));
    }
}

/// **O glifo do PINO** — um anel: a dobradiça.
pub fn pin_glyph(centre: Point) -> BezPath {
    let mut p = BezPath::new();
    ring_px(centre, PIN_RING_PX, &mut p);
    p
}

/// **O glifo do WELD** — um quadrado, girado com o corpo A (um weld não gira).
/// ⚠️ Pin×Weld distinguem-se por GEOMETRIA, nunca por identificador.
pub fn weld_glyph(
    camera: &Camera2d,
    window: WindowSize,
    centre_w: [f32; 2],
    angle_a: f32,
) -> BezPath {
    let (c, u, v) = screen_basis(camera, window, centre_w);
    let a = f64::from(angle_a);
    let at = |ang: f64| arc_point_in(c, u, v, ang, WELD_HALF_PX);
    let mut p = BezPath::new();
    let corners = [0.25, 0.75, 1.25, 1.75].map(|k: f64| at(a + k * std::f64::consts::PI));
    p.move_to(corners[3]);
    for c in corners {
        p.line_to(c);
    }
    p
}

/// **O glifo do CUSTOM** — um HEXÁGONO, e um anel dentro dele quando o eixo de
/// rotação está livre (o anel interno é o vocabulário do PINO).
///
/// ⚠️ NÃO desenha os eixos lineares: vivem no frame do joint, que a
/// [`JointView`] não carrega — desenhá-los pelo corpo A MENTIRIA.
///
/// [`JointView`]: ph2d_physics_ecs::JointView
pub fn custom_glyph(centre: Point, rotation_free: bool) -> BezPath {
    let mut p = BezPath::new();
    let r = CUSTOM_HEX_PX;
    for i in 0..=6 {
        let th = std::f64::consts::TAU * f64::from(i) / 6.0;
        let pt = Point::new(centre.x + r * th.cos(), centre.y + r * th.sin());
        if i == 0 {
            p.move_to(pt);
        } else {
            p.line_to(pt);
        }
    }
    if rotation_free {
        ring_px(centre, CUSTOM_HEX_PX * 0.45, &mut p);
    }
    p
}

/// **O glifo da MOLA** — o zigue-zague entre as âncoras (inverte a cada
/// meia-onda; a corda nunca — um gate mede isso).
pub fn spring_zigzag(a: Point, b: Point) -> BezPath {
    let (dx, dy) = (b.x - a.x, b.y - a.y);
    let len = dx.hypot(dy);
    let mut p = BezPath::new();
    p.move_to(a);
    if len < 1e-6 {
        // Span zero: as pontas do chamador mantêm o joint visível.
        return p;
    }
    let (ux, uy) = (dx / len, dy / len);
    let (nx, ny) = (-uy, ux);
    for i in 1..SPRING_HUMPS {
        let t = f64::from(i) / f64::from(SPRING_HUMPS);
        let s = if i % 2 == 0 { -1.0 } else { 1.0 };
        p.line_to(Point::new(
            a.x + dx * t + nx * SPRING_AMP_PX * s,
            a.y + dy * t + ny * SPRING_AMP_PX * s,
        ));
    }
    p.line_to(b);
    p
}

/// **O glifo da CORDA** — reta quando TESA, curva quando FROUXA.
///
/// Flecha parabólica `s = d·√(3/8·(L/d − 1))` — um READOUT da folga, não simulação.
/// ⚠️ A barriga segue a GRAVIDADE; sem gravidade, reta.
pub fn rope_span(a: Point, b: Point, slack_ratio: f64, gravity_screen: (f64, f64)) -> BezPath {
    let (dx, dy) = (b.x - a.x, b.y - a.y);
    let d = dx.hypot(dy);
    let mut p = BezPath::new();
    p.move_to(a);
    // Tesa (ou sem gravidade): reta.
    if slack_ratio <= 0.0 || d < 1e-6 || gravity_screen.0.hypot(gravity_screen.1) < 1e-9 {
        p.line_to(b);
        return p;
    }
    // `slack_ratio` = L/d − 1 (do chamador); o cap limita a barriga.
    let sag = (d * (0.375 * slack_ratio).sqrt()).min(d * 0.6);
    let mid = Point::new(
        (a.x + b.x) * 0.5 + gravity_screen.0 * sag,
        (a.y + b.y) * 0.5 + gravity_screen.1 * sag,
    );
    p.quad_to(mid, b);
    p
}

/// **O glifo da BARRA** — duas PARALELAS que nunca tocam o segmento âncora→âncora
/// (a corda tesa passa sobre ele; um gate mede). Os anéis são do chamador.
pub fn rod_bar(a: Point, b: Point) -> BezPath {
    let (dx, dy) = (b.x - a.x, b.y - a.y);
    let len = dx.hypot(dy);
    let mut p = BezPath::new();
    if len < 1e-6 {
        // Span zero: os anéis do chamador mantêm o joint visível.
        return p;
    }
    // Perpendicular em px de TELA: a espessura é chrome, não mundo.
    let (nx, ny) = (-dy / len * ROD_HALF_PX, dx / len * ROD_HALF_PX);
    for s in [1.0, -1.0] {
        p.move_to(Point::new(a.x + nx * s, a.y + ny * s));
        p.line_to(Point::new(b.x + nx * s, b.y + ny * s));
    }
    p
}

/// **O arco de LIMITE de um pino** — as duas paredes e a agulha.
///
/// Limite RELATIVO (`θb − θa`): arco no frame do corpo A, agulha em `angle_b`.
pub fn limit_arc(
    camera: &Camera2d,
    window: WindowSize,
    centre_w: [f32; 2],
    angle_a: f32,
    limits: [f32; 2],
    angle_b: f32,
) -> BezPath {
    let (c, u, v) = screen_basis(camera, window, centre_w);
    let at = |ang: f64, r: f64| arc_point_in(c, u, v, ang, r);
    let a0 = f64::from(angle_a) + f64::from(limits[0]);
    let a1 = f64::from(angle_a) + f64::from(limits[1]);
    let mut p = BezPath::new();
    p.move_to(at(a0, LIMIT_ARC_PX));
    for i in 1..=ARC_SEGS {
        let t = f64::from(i) / f64::from(ARC_SEGS);
        p.line_to(at(a0 + (a1 - a0) * t, LIMIT_ARC_PX));
    }
    // As paredes: uma marca radial em cada extremo.
    for a in [a0, a1] {
        p.move_to(at(a, LIMIT_ARC_PX - LIMIT_TICK_PX));
        p.line_to(at(a, LIMIT_ARC_PX + LIMIT_TICK_PX));
    }
    // A agulha: onde o corpo B está AGORA, do centro até o arco.
    p.move_to(c);
    p.line_to(at(f64::from(angle_b), LIMIT_ARC_PX));
    p
}

/// **O anel de COMPRIMENTO** — repouso da mola, máximo da corda; em MUNDO projetado.
pub fn length_ring(
    camera: &Camera2d,
    window: WindowSize,
    centre_w: [f32; 2],
    length: f32,
) -> BezPath {
    let c = screen_of(camera, window, centre_w);
    let edge = screen_of(camera, window, [centre_w[0] + length, centre_w[1]]);
    let r = (edge.x - c.x).hypot(edge.y - c.y);
    let mut p = BezPath::new();
    if r > 0.5 {
        ring_px(c, r, &mut p);
    }
    p
}

/// Metade da largura da barra de chão do pino de MUNDO, em px de tela.
const GROUND_HALF_PX: f64 = 9.0; // LITERAL-PX-OK: chrome de overlay
/// Quanto a barra descansa ABAIXO da âncora (fora do [`PIN_RING_PX`]).
const GROUND_GAP_PX: f64 = 7.5; // LITERAL-PX-OK: chrome de overlay
/// O comprimento de cada risco da hachura.
const GROUND_TICK_PX: f64 = 5.0; // LITERAL-PX-OK: chrome de overlay
/// Quantos riscos. Quatro é o que cabe legível na barra acima sem virar mancha.
const GROUND_TICKS: usize = 4;

/// **A hachura de CHÃO** — a ponta deste joint é o CENÁRIO, não um corpo
/// (W-WorldPinGlyph): a notação de apoio fixo. Sem ela o pino de mundo saía
/// byte-idêntico ao de dois corpos.
/// ⚠️ Desce pela GRAVIDADE (`g_screen`), nunca pelo −Y da tela; sem gravidade, nada.
pub fn ground_hatch(at: Point, g_screen: (f64, f64)) -> BezPath {
    let mut p = BezPath::new();
    let (dx, dy) = g_screen;
    if dx.hypot(dy) < 1e-9 {
        return p;
    }
    // A barra deita ATRAVESSADA na gravidade.
    let (rx, ry) = (-dy, dx);
    let base = Point::new(at.x + dx * GROUND_GAP_PX, at.y + dy * GROUND_GAP_PX);
    let end = |t: f64, l: f64| Point::new(base.x + rx * t + dx * l, base.y + ry * t + dy * l);
    p.move_to(end(-GROUND_HALF_PX, 0.0));
    p.line_to(end(GROUND_HALF_PX, 0.0));
    // Os riscos, inclinados a 45° contra a barra — a hachura, não uma cerca.
    let step = (2.0 * GROUND_HALF_PX) / GROUND_TICKS as f64;
    for i in 0..GROUND_TICKS {
        let t = -GROUND_HALF_PX + step * (i as f64 + 0.5);
        p.move_to(end(t, 0.0));
        p.line_to(end(t - GROUND_TICK_PX, GROUND_TICK_PX));
    }
    p
}
