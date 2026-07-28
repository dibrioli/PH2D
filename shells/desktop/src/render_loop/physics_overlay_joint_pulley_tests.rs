//! **A corda ABRAÇA a roldana** (W-Pulley W1) — irmão de
//! `physics_overlay_joints_tests` pelo cap de 600 LOC da shell, e o corte é por
//! assunto: aqui só a figura que uma POLIA desenha, a única cujo caminho tem
//! quantos nós o artista quiser e cada nó é um círculo com raio próprio.
//!
//! Os dois gates deste arquivo são as duas metades da MESMA frase — *a corda
//! passa na superfície* —, e elas falham por bugs diferentes:
//!
//! - **ela TOCA o aro**: o modelo de PONTO da v1 levava a corda ao CENTRO, e o
//!   pedido (5) do artista foi exatamente esse;
//! - **ela NÃO ENTRA**: ligar dois pontos de tangência por uma reta desenha uma
//!   CORDA do círculo, que atravessa o disco. Foi o defeito que o smoke pegou,
//!   e ele passava por baixo de qualquer oráculo que olhasse só PONTOS — os
//!   dois extremos da corda estão em cima do aro, e é o meio dela que entra.
//!   Por isso a medida é a de SEGMENTO, não a de ponto.

use super::super::physics_overlay_joint_glyphs::screen_of;
use super::super::physics_overlay_pulley::pulley_marks;
use super::joint_tests::{camera, view, window};
use ph2d_physics_ecs::JointKind;
use ph2d_physics_ecs::rope_route::{self, RopeWheel};
use ph2d_vector::{BezPath, PathEl, Point};

/// Quanto um ponto desenhado pode se afastar do aro e ainda contar como
/// *tocando*, px. Os pontos de tangência e os do arco saem exatos; a folga é
/// para o arredondamento da projeção.
const TOUCH_TOL_PX: f64 = 0.5;

/// Quanto a corda pode invadir o disco, como fração do raio.
///
/// O arco é desenhado como polilinha na MESMA resolução do aro (24 cordas por
/// volta), então cada corda de 15° afunda no máximo `1 − cos(7,5°)` = 0,86 % do
/// raio — e o aro desenhado afunda o mesmo tanto, que é o que faz os dois
/// coincidirem na tela. **MEDIDO nesta fixture: 0,76 %** (49,62 px de um aro a
/// 50,00).
///
/// **2 %** deixa 2,6× de folga sobre o que o produto de fato faz e ainda fica
/// 17× abaixo do defeito que este gate existe para pegar: a corda reta MEDIDA
/// entra **35 %** do raio.
const ENTER_TOL_FRAC: f64 = 0.02;

/// O ELEVADOR: dois corpos embaixo, duas roldanas grandes em cima. O enlace em
/// cada roda passa de 90°, que é onde uma corda reta é escandalosa — uma fixture
/// de enlace raso mediria o fenômeno perto de zero e ficaria verde.
fn elevator() -> (ph2d_physics_ecs::JointView, Vec<RopeWheel>) {
    let mut v = view(JointKind::Pulley);
    v.anchor_a = [-1.5, -1.0];
    v.anchor_b = [1.5, -1.0];
    let mut wheels = vec![
        RopeWheel {
            centre: [-1.5, 2.0],
            radius: 0.5,
            side: 1,
        },
        RopeWheel {
            centre: [1.5, 2.0],
            radius: 0.5,
            side: 1,
        },
    ];
    // ⚠️ Os lados vêm da MESMA porta que a ponte usa. Cravá-los à mão faria a
    // fixture descrever uma corda que o produto não monta.
    rope_route::resolve_sides(v.anchor_a, v.anchor_b, &mut wheels, &mut Vec::new());
    (v, wheels)
}

/// A corda desenhada, em pontos de tela.
fn rope(v: &ph2d_physics_ecs::JointView, wheels: &[RopeWheel]) -> Vec<Point> {
    let (cam, win) = (camera(), window());
    let (a, b) = (
        screen_of(&cam, win, v.anchor_a),
        screen_of(&cam, win, v.anchor_b),
    );
    let mut span = BezPath::new();
    let mut glyph = BezPath::new();
    pulley_marks(v, wheels, &[0.0; 2], a, b, &cam, win, &mut span, &mut glyph);
    span.elements()
        .iter()
        .filter_map(|el| match el {
            PathEl::MoveTo(p) | PathEl::LineTo(p) => Some(*p),
            _ => None,
        })
        .collect()
}

/// Onde a roldana está e que tamanho ela tem, em px de tela.
fn wheel_on_screen(w: &RopeWheel) -> (Point, f64) {
    let (cam, win) = (camera(), window());
    let c = screen_of(&cam, win, w.centre);
    let rim = screen_of(&cam, win, [w.centre[0] + w.radius, w.centre[1]]);
    (c, (rim.x - c.x).hypot(rim.y - c.y))
}

/// A menor distância de `p` ao segmento `a→b`.
fn segment_distance(a: Point, b: Point, p: Point) -> f64 {
    let (vx, vy) = (b.x - a.x, b.y - a.y);
    let len2 = vx * vx + vy * vy;
    let t = if len2 <= f64::EPSILON {
        0.0
    } else {
        (((p.x - a.x) * vx + (p.y - a.y) * vy) / len2).clamp(0.0, 1.0)
    };
    (a.x + t * vx - p.x).hypot(a.y + t * vy - p.y)
}

/// **A corda toca a SUPERFÍCIE da roldana** — o pedido (5) do artista.
///
/// Antes do W1 uma roldana era um ponto e a corda ia até o CENTRO dela. O
/// oráculo é a distância do ponto mais próximo ao centro: ela tem de valer o
/// RAIO, e valia zero.
#[test]
fn the_rope_touches_the_rim_it_does_not_reach_the_hub() {
    let (v, wheels) = elevator();
    let pts = rope(&v, &wheels);
    assert!(pts.len() > 4, "a corda não desenhou nada");
    for (i, w) in wheels.iter().enumerate() {
        let (c, r) = wheel_on_screen(w);
        let closest = pts
            .iter()
            .map(|p| (p.x - c.x).hypot(p.y - c.y))
            .fold(f64::MAX, f64::min);
        assert!(
            (closest - r).abs() <= TOUCH_TOL_PX,
            "na roldana {i} o ponto mais próximo da corda ficou a {closest:.2} px \
             do centro, e o aro está a {r:.2} px: a corda tem de encostar na \
             SUPERFÍCIE — {} ",
            if closest < r {
                "isto é o modelo de PONTO, em que ela ia até o eixo"
            } else {
                "ela passou longe da roda"
            }
        );
    }
}

/// **E NÃO atravessa o disco** — o defeito que o smoke pegou.
///
/// ⚠️ O oráculo mede SEGMENTOS, não pontos, e a diferença é o bug inteiro: uma
/// corda reta de tangência a tangência tem as duas PONTAS exatamente sobre o
/// aro, então um oráculo de pontos fica verde sobre a figura errada. O que entra
/// no disco é o meio dela.
///
/// Ele também recusa um arco varrido para o lado ERRADO: o arco termina no
/// ponto oposto ao de partida do trecho seguinte, e a reta que fecha esse vão
/// passa perto do eixo.
#[test]
fn the_rope_wraps_the_wheel_it_does_not_cut_across_it() {
    let (v, wheels) = elevator();
    let pts = rope(&v, &wheels);
    for (i, w) in wheels.iter().enumerate() {
        let (c, r) = wheel_on_screen(w);
        let deepest = pts
            .windows(2)
            .map(|s| segment_distance(s[0], s[1], c))
            .fold(f64::MAX, f64::min);
        let allowed = r * (1.0 - ENTER_TOL_FRAC);
        assert!(
            deepest >= allowed,
            "na roldana {i} a corda desenhada chegou a {deepest:.2} px do centro \
             com o aro a {r:.2} px ({:.0} % para dentro): entre os dois pontos \
             de tangência ela tem de seguir a SUPERFÍCIE, e uma reta entre eles \
             é uma corda do círculo — atravessa a roda",
            (1.0 - deepest / r) * 100.0
        );
    }
}
