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
            id: 0,
            break_force: f32::INFINITY,
            ..RopeWheel::default()
        },
        RopeWheel {
            centre: [1.5, 2.0],
            radius: 0.5,
            side: 1,
            id: 0,
            break_force: f32::INFINITY,
            ..RopeWheel::default()
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

/// **Um TAMBOR DIFERENCIAL desenha DOIS anéis** (W4) — e é deles que o artista lê
/// a vantagem mecânica.
///
/// ⚠️ Sem o segundo anel o tambor seria indistinguível de uma roldana comum na
/// tela: o número viveria só na §13, e a wave inteira existe justamente porque um
/// número sem peça na cena (o `ratio` do W-Pulley) não é a resposta.
///
/// O oráculo conta CIRCUNFERÊNCIAS distintas em torno do centro — pontos do
/// glifo agrupados por distância ao eixo —, não elementos de path: um anel é
/// desenhado com muitos segmentos, e contá-los mediria a tesselação.
#[test]
fn a_differential_drum_draws_both_of_its_circles() {
    let radii = |r_out: Option<f32>| {
        let (cam, win) = (camera(), window());
        let mut v = view(JointKind::Pulley);
        v.anchor_a = [-1.5, -1.0];
        v.anchor_b = [1.5, -1.0];
        // ⚠️ A `view()` compartilhada nasce com DUAS roldanas (é o elevador), e
        // uma faixa que não cabe na arena devolve VAZIO — o desenho sairia sem
        // roldana nenhuma e o gate leria zero circunferências sobre um produto
        // correto. A fixture declara a própria contagem.
        v.wheel_count = 1;
        let mut wheels = vec![RopeWheel {
            centre: [0.0, 2.0],
            radius: 0.5,
            radius_out: r_out,
            side: 1,
            id: 0,
            break_force: f32::INFINITY,
            ..RopeWheel::default()
        }];
        rope_route::resolve_sides(v.anchor_a, v.anchor_b, &mut wheels, &mut Vec::new());
        let (a, b) = (
            screen_of(&cam, win, v.anchor_a),
            screen_of(&cam, win, v.anchor_b),
        );
        let mut span = BezPath::new();
        let mut glyph = BezPath::new();
        pulley_marks(&v, &wheels, &[0.0], a, b, &cam, win, &mut span, &mut glyph);
        let centre = screen_of(&cam, win, [0.0, 2.0]);
        // As distâncias distintas ao eixo, agrupadas com folga de 1 px.
        let mut ds: Vec<f64> = glyph
            .elements()
            .iter()
            .filter_map(|el| match el {
                PathEl::MoveTo(p) | PathEl::LineTo(p) => Some(*p),
                _ => None,
            })
            .map(|p| (p.x - centre.x).hypot(p.y - centre.y))
            // O raio-guia vai do CENTRO ao aro, então ele deposita pontos em toda
            // distância intermediária: só as pontas descrevem circunferências.
            .filter(|d| *d > 1.0)
            .collect();
        ds.sort_by(|x, y| x.partial_cmp(y).expect("distâncias finitas"));
        ds.dedup_by(|x, y| (*x - *y).abs() < 1.0);
        ds
    };
    let plain = radii(None);
    assert_eq!(
        plain.len(),
        1,
        "uma roldana COMUM tem UMA circunferência; saíram {plain:?}"
    );
    let drum = radii(Some(0.125));
    assert_eq!(
        drum.len(),
        2,
        "um tambor diferencial tem DUAS; saíram {drum:?}"
    );
    // E a menor é a de saída: um quarto da de entrada, como os raios dizem.
    assert!(
        (drum[0] / drum[1] - 0.25).abs() < 0.05,
        "os dois anéis estão em {drum:?}; os raios 0,125 e 0,50 pedem razão 0,25"
    );
}

/// **Uma corda cuja ROTA não resolve veste a cor do NÃO-SEGURA** (2026-07-29).
///
/// Medido: degenerar a geometria (a roldana engolindo a âncora) faz o passe de
/// impulso PULAR a corda — a carga cai a −1,237 m contra −0,460 do controle — e o
/// desenho era uma reta ÂMBAR, que é exactamente a figura de uma corda que
/// funciona. O único sinal na tela era a carga caindo.
///
/// ⚠️ **Duas metades, e a segunda é o CONECTOR:** a função que computa a rota
/// devolve o fato (`pulley_marks -> bool`), e o laço de pintura o usa para escolher
/// a cor. Gatear só a primeira deixaria o fato correto e a tela igual — a forma
/// exata do defeito que esta linha já pagou duas vezes.
///
/// Mutação: `v.broken || !acting` de volta a `v.broken` ⇒ a corda degenerada volta
/// ao âmbar e a 2ª metade fica VERMELHA (0 elementos na cor de ruptura).
#[test]
fn a_rope_that_cannot_route_wears_the_not_holding_colour() {
    use super::super::physics_overlay_joints::{JOINT_BROKEN_RGBA, JOINT_RGBA};

    // A MESMA montagem do elevador, com a 1a roldana ENGOLINDO a âncora A: a
    // tangente de um ponto DENTRO do círculo não existe, e a rota inteira cai.
    let (v, mut wheels) = elevator();
    wheels[0].centre = v.anchor_a;
    wheels[0].radius = 1.0;
    let mut segs = Vec::new();
    assert!(
        rope_route::route(v.anchor_a, v.anchor_b, &wheels, &mut segs).is_none(),
        "a fixture não degenerou a rota — o gate mediria uma corda sadia"
    );

    // (1) A função que COMPUTA a rota devolve o fato.
    let (cam, win) = (camera(), window());
    let (mut span, mut glyph) = (BezPath::new(), BezPath::new());
    let routed = pulley_marks(
        &v,
        &wheels,
        &[0.0; 2],
        Point::new(0.0, 0.0),
        Point::new(10.0, 0.0),
        &cam,
        win,
        &mut span,
        &mut glyph,
    );
    assert!(!routed, "pulley_marks disse que roteou uma rota degenerada");

    // (2) E o laço de pintura A HONRA — o conector.
    //
    // ⚠️ **Pelo `joint_marks` com a arena DEGENERADA**, não pelo helper `marks`:
    // ele crava a arena SADIA compartilhada, e a 1a versão deste gate nasceu
    // vermelha exactamente por isso — a metade que mede a cor estava pintando uma
    // corda que roteia. Fixture que não contém o fenômeno.
    let painted = super::super::physics_overlay_joints::joint_marks(
        true,
        std::slice::from_ref(&v),
        &wheels,
        &[0.0; 2],
        super::joint_tests::G,
        &cam,
        win,
    );
    let broken_els: usize = painted
        .iter()
        .filter(|(_, c)| *c == JOINT_BROKEN_RGBA)
        .map(|(p, _)| p.elements().len())
        .sum();
    let amber_els: usize = painted
        .iter()
        .filter(|(_, c)| *c == JOINT_RGBA)
        .map(|(p, _)| p.elements().len())
        .sum();
    assert!(
        broken_els > 0,
        "a corda degenerada não pintou nada na cor do não-segura"
    );
    assert!(
        amber_els == 0,
        "a corda degenerada ainda pinta {amber_els} elementos no âmbar de *isto está \
         segurando*"
    );
}

/// **E a corda SADIA continua âmbar** — o irmão de presença.
///
/// Sem ele, colorir TODA polia de vermelho passaria no gate acima.
#[test]
fn a_rope_that_routes_stays_in_the_holding_colour() {
    use super::super::physics_overlay_joints::{JOINT_BROKEN_RGBA, JOINT_RGBA};

    let (v, wheels) = elevator();
    let mut segs = Vec::new();
    assert!(
        rope_route::route(v.anchor_a, v.anchor_b, &wheels, &mut segs).is_some(),
        "a fixture do controle não roteia"
    );
    // Pela MESMA porta e com a MESMA arena do irmão acima — só a geometria muda.
    let painted = super::super::physics_overlay_joints::joint_marks(
        true,
        std::slice::from_ref(&v),
        &wheels,
        &[0.0; 2],
        super::joint_tests::G,
        &camera(),
        window(),
    );
    let amber: usize = painted
        .iter()
        .filter(|(_, c)| *c == JOINT_RGBA)
        .map(|(p, _)| p.elements().len())
        .sum();
    let broken: usize = painted
        .iter()
        .filter(|(_, c)| *c == JOINT_BROKEN_RGBA)
        .map(|(p, _)| p.elements().len())
        .sum();
    assert!(amber > 0, "a corda sadia não pintou nada no âmbar");
    assert!(
        broken == 0,
        "a corda sadia pintou {broken} elementos na cor do não-segura"
    );
}
