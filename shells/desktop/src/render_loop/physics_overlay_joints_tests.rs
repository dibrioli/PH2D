//! **Os gates do overlay de joints (W-J1).**
//!
//! Um joint é uma relação, e uma relação não tem geometria — então TUDO que se
//! vê dele é escolha de desenho, e cada escolha aqui é uma afirmação sobre o
//! modelo que pode estar errada em silêncio. Estes gates atacam as quatro que
//! importam: **os quatro tipos desenham figuras DIFERENTES** (nunca o mesmo
//! desenho com nome diferente — a cicatriz `Layer`/`Layers` da timeline), **o
//! que é comprimento cresce com o mundo e o que é ângulo não**, **a deformação
//! só é acusada onde a restrição não está sendo imposta**, e **cada ponta diz
//! de quem é**.

use super::super::physics_overlay_joint_glyphs::LIMIT_ARC_PX;
use super::{JOINT_DIM_RGBA, JOINT_RGBA, JOINT_STRAIN_RGBA, joint_marks};
use ph2d_host::WindowSize;
use ph2d_physics_ecs::{JointKind, JointView};
use ph2d_render::Camera2d;
use ph2d_vector::{BezPath, PathEl};

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

/// Gravidade padrão (Y-down no mundo Y-up): o que faz uma corda frouxa pendurar.
const G: [f32; 2] = [0.0, -9.81];

fn view(kind: JointKind) -> JointView {
    JointView {
        entity: ph2d_ecs::Entity::from_bits(1),
        kind,
        anchor_a: [0.0, 0.0],
        anchor_b: [0.0, 0.0],
        centre_a: [-1.0, 0.0],
        centre_b: [1.0, 0.0],
        body_b: ph2d_ecs::Entity::from_bits(3),
        angle_a: 0.0,
        angle_b: 0.0,
        limits: None,
        motor_speed: None,
        length: None,
        // Um Slider precisa do eixo resolvido; todo outro tipo não tem um. A
        // fixture o dá para TODO kind de propósito — quem ignora, ignora, e o
        // gate do trilho quer o `Some` sem montar uma segunda fixture.
        axis: Some([1.0, 0.0]),
        // A fixture declara a premissa: TODO gate deste arquivo descreve um joint
        // que está SEGURANDO. Um rompido tem vocabulário próprio (vermelho, sem
        // envelope, com estouro) e gates próprios em `physics_overlay_joint_break_tests`.
        broken: false,
        active: true,
        // W-J7b: a fixture descreve um joint SEM teto e sem carga — o readout é
        // assunto do irmão `physics_overlay_joint_readout_tests`, e um teto aqui
        // faria estes gates contarem um rótulo que não é sobre eles.
        load: ph2d_physics_ecs::JointLoad::ZERO,
        peak: ph2d_physics_ecs::JointLoad::ZERO,
        break_force: f32::INFINITY,
        break_torque: f32::INFINITY,
    }
}

fn marks(v: &JointView) -> Vec<(BezPath, [f32; 4])> {
    joint_marks(true, std::slice::from_ref(v), G, &camera(), window())
}

/// Todos os pontos desenhados numa cor.
///
/// ⚠️ O ponto de CONTROLE de uma quadrática entra na lista. A 1ª versão deste
/// helper guardava só o ponto final (`QuadTo(_, p) => p`) — e a barriga de uma
/// corda pendurada mora INTEIRA no controle, então o gate do sag media a corda
/// frouxa e a tesa com a mesma altura (6,0 px, as duas pontas) sobre um desenho
/// correto. O polígono de controle limita a curva e fica do mesmo lado dela
/// (propriedade do fecho convexo), que é o que um oráculo de FORMA precisa.
fn points_of(marks: &[(BezPath, [f32; 4])], rgba: [f32; 4]) -> Vec<(f64, f64)> {
    marks
        .iter()
        .filter(|(_, c)| *c == rgba)
        .flat_map(|(p, _)| p.elements())
        .flat_map(|el| match el {
            PathEl::MoveTo(p) | PathEl::LineTo(p) => vec![(p.x, p.y)],
            PathEl::QuadTo(c, p) => vec![(c.x, c.y), (p.x, p.y)],
            _ => Vec::new(),
        })
        .collect()
}

/// Quantas vezes a poligonal desenhada numa cor INVERTE de lado — a assinatura
/// de um zigue-zague, e o que uma linha (mesmo curva) não faz.
///
/// ⚠️ A 1ª versão comparava o produto vetorial de cada terno com o do MESMO
/// terno com os dois vetores negados — que é o próprio produto de volta, então
/// o teste era `cross² < 0` e contava **zero** inversões em qualquer figura.
/// Aritmética morta lê como oráculo. O que conta é a mudança de SINAL entre
/// ternos consecutivos.
fn direction_flips(marks: &[(BezPath, [f32; 4])], rgba: [f32; 4]) -> usize {
    let pts = points_of(marks, rgba);
    let mut flips = 0;
    let mut prev_sign = 0.0f64;
    for w in pts.windows(3) {
        let (a, b, c) = (w[0], w[1], w[2]);
        let cross = (b.0 - a.0) * (c.1 - b.1) - (b.1 - a.1) * (c.0 - b.0);
        if cross.abs() < 1e-9 {
            continue;
        }
        if prev_sign != 0.0 && cross * prev_sign < 0.0 {
            flips += 1;
        }
        prev_sign = cross;
    }
    flips
}

/// As distâncias, em px de tela, de cada ponto de uma cor até um ponto de
/// mundo — como se mede um ANEL (que tem raio) em vez de uma caixa.
fn radii_from(
    marks: &[(BezPath, [f32; 4])],
    rgba: [f32; 4],
    centre_w: [f32; 2],
    cam: &Camera2d,
) -> Vec<f64> {
    let (cx, cy) = cam.world_to_screen(centre_w, window());
    let (cx, cy) = (f64::from(cx), f64::from(cy));
    points_of(marks, rgba)
        .iter()
        .map(|(x, y)| (x - cx).hypot(y - cy))
        .collect()
}

/// O quanto de ÂNGULO os pontos de uma cor cobrem numa faixa de raio — como se
/// mede um ARCO. Uma linha reta que atravessa a faixa contribui uma direção só;
/// um arco cobre a faixa inteira do limite.
fn angular_spread(
    marks: &[(BezPath, [f32; 4])],
    rgba: [f32; 4],
    centre_w: [f32; 2],
    band: (f64, f64),
) -> f64 {
    let cam = camera();
    let (cx, cy) = cam.world_to_screen(centre_w, window());
    let (cx, cy) = (f64::from(cx), f64::from(cy));
    let angles: Vec<f64> = points_of(marks, rgba)
        .iter()
        .filter_map(|(x, y)| {
            let (dx, dy) = (x - cx, y - cy);
            let r = dx.hypot(dy);
            (r >= band.0 && r <= band.1).then(|| dy.atan2(dx))
        })
        .collect();
    match (
        angles.iter().cloned().fold(f64::MAX, f64::min),
        angles.iter().cloned().fold(f64::MIN, f64::max),
    ) {
        (lo, hi) if lo <= hi => hi - lo,
        _ => 0.0,
    }
}

/// A extensão (largura + altura) do que foi desenhado numa cor.
fn extent(marks: &[(BezPath, [f32; 4])], rgba: [f32; 4]) -> (f64, f64) {
    let pts = points_of(marks, rgba);
    if pts.is_empty() {
        return (0.0, 0.0);
    }
    let (mut x0, mut x1, mut y0, mut y1) = (f64::MAX, f64::MIN, f64::MAX, f64::MIN);
    for (x, y) in pts {
        x0 = x0.min(x);
        x1 = x1.max(x);
        y0 = y0.min(y);
        y1 = y1.max(y);
    }
    (x1 - x0, y1 - y0)
}

/// **Os quatro tipos desenham figuras DIFERENTES.**
///
/// ⚠️ O oráculo compara GEOMETRIA, nunca o identificador do tipo: o par
/// `Layer`/`Layers` da timeline foi reprovado num smoke por serem duas figuras
/// idênticas com nomes distintos, e o gate que as separava lia o nome. Aqui,
/// dois tipos que passassem a desenhar o mesmo conjunto de pontos falham.
#[test]
fn each_joint_kind_draws_a_different_figure() {
    let mut spring = view(JointKind::Spring);
    spring.anchor_b = [2.0, 0.0];
    spring.length = Some(2.0);
    let mut rope = view(JointKind::Rope);
    rope.anchor_b = [2.0, 0.0];
    rope.length = Some(2.0);

    let figures: Vec<(&str, Vec<(f64, f64)>)> = vec![
        ("Pin", points_of(&marks(&view(JointKind::Pin)), JOINT_RGBA)),
        (
            "Weld",
            points_of(&marks(&view(JointKind::Weld)), JOINT_RGBA),
        ),
        ("Spring", points_of(&marks(&spring), JOINT_RGBA)),
        ("Rope", points_of(&marks(&rope), JOINT_RGBA)),
    ];
    for (i, (na, a)) in figures.iter().enumerate() {
        assert!(
            !a.is_empty(),
            "{na} desenhou NADA — um joint invisível é indistinguível de um joint ausente"
        );
        for (nb, b) in figures.iter().skip(i + 1) {
            assert_ne!(
                a, b,
                "{na} e {nb} desenham a MESMA figura: o tipo do joint não é \
                 legível no canvas, só no painel (a cicatriz Layer/Layers)"
            );
        }
    }
}

/// **A mola serpenteia; a corda não.**
///
/// Distinção estrutural entre os dois glifos que ATRAVESSAM o vão: o
/// zigue-zague inverte de lado a cada meia-onda, uma corda (reta ou pendurada)
/// nunca inverte. Um gate que só comparasse conjuntos de pontos passaria com
/// uma mola desenhada como linha levemente diferente.
#[test]
fn the_spring_zigzags_and_the_rope_does_not() {
    let mut spring = view(JointKind::Spring);
    spring.anchor_b = [2.0, 0.0];
    spring.length = Some(2.0);
    let mut rope = view(JointKind::Rope);
    rope.anchor_b = [2.0, 0.0];
    rope.length = Some(3.0); // frouxa: a corda pendura, e ainda assim não serpenteia

    let s = direction_flips(&marks(&spring), JOINT_RGBA);
    let r = direction_flips(&marks(&rope), JOINT_RGBA);
    assert!(
        s >= 4,
        "a mola inverteu de lado {s}× — sem as meias-ondas ela é uma linha, \
         e uma linha já é o desenho da corda"
    );
    assert!(
        r <= s / 2,
        "a corda inverteu de lado {r}× contra {s}× da mola: as duas leem como \
         a mesma figura"
    );
}

/// **O que é COMPRIMENTO cresce com o zoom; o que é ÂNGULO não.**
///
/// O anel de repouso descreve uma distância do mundo, então dar zoom tem de
/// dobrá-lo — como dobra a bola que ele mede. O arco de limite descreve um
/// ângulo, que não tem tamanho, então ele é ornamento de tela e fica onde está.
/// É a mesma lei que separa a seta de força (mundo) do arrowhead dela (tela).
#[test]
fn a_length_scales_with_the_world_and_an_angle_does_not() {
    let mut spring = view(JointKind::Spring);
    spring.anchor_b = [2.0, 0.0];
    spring.length = Some(2.0);
    let mut pin = view(JointKind::Pin);
    pin.limits = Some([-0.5, 0.5]);

    let zoomed = Camera2d {
        height_world: 5.0, // metade da altura = 2× o zoom
        ..camera()
    };
    let zoom_marks =
        |v: &JointView| joint_marks(true, std::slice::from_ref(v), G, &zoomed, window());

    // ⚠️ O anel se mede pela ASSINATURA — MUITOS pontos a UM raio — e não pelo
    // raio máximo da banda. A 1ª versão pegava o máximo, e no fixture a linha
    // de posse até o corpo B alcança exatamente o mesmo raio do anel (200 px a
    // 1×, 400 a 2×): o gate passava medindo a POSSE, e sobreviveu à mutação que
    // cravava o raio em pixels de tela. Nenhuma linha põe 24 pontos num raio só.
    // 1000 px / 10 unidades de mundo = 100 px por metro; a mola repousa em 2 m.
    let ring_pts = |m: &[(BezPath, [f32; 4])], cam: &Camera2d, expect: f64| {
        radii_from(m, JOINT_DIM_RGBA, spring.anchor_a, cam)
            .into_iter()
            .filter(|r| (*r - expect).abs() < 2.0)
            .count()
    };
    let n1 = ring_pts(&marks(&spring), &camera(), 200.0);
    let n2 = ring_pts(&zoom_marks(&spring), &zoomed, 400.0);
    assert!(
        n1 >= 20 && n2 >= 20,
        "o anel de repouso pôs {n1} pontos a 200 px (1×) e {n2} a 400 px (2×): \
         2 m valem 200 px nesta câmera e 400 no dobro do zoom — ele mede \
         METROS, então tem de crescer exatamente como o mundo"
    );

    // O arco é ornamento de TELA: o raio dele é o mesmo número de pixels nos
    // dois zooms, e é isso que o distingue de tudo que mede o mundo.
    let arc_r = |m: &[(BezPath, [f32; 4])]| {
        radii_from(m, JOINT_DIM_RGBA, pin.anchor_a, &camera())
            .into_iter()
            .filter(|r| (*r - LIMIT_ARC_PX).abs() < 3.0)
            .count()
    };
    assert!(
        arc_r(&marks(&pin)) > 10 && arc_r(&zoom_marks(&pin)) > 10,
        "o arco de limite não está a {LIMIT_ARC_PX} px da âncora nos dois zooms \
         — um ângulo não tem tamanho de mundo para escalar"
    );
}

/// **A deformação é acusada num pino, e NUNCA numa mola.**
///
/// As âncoras de um pino coincidem — é o que um pino É — então um vão entre
/// elas é a restrição sendo esticada, o único fato do modelo que merece a única
/// cor de alarme do overlay. As âncoras de uma mola são as DUAS PONTAS e estão
/// separadas por construção: pintá-las de vermelho chamaria de erro o
/// funcionamento normal.
#[test]
fn strain_is_flagged_on_a_pin_and_never_on_a_spring() {
    let mut strained = view(JointKind::Pin);
    strained.anchor_b = [0.5, 0.0]; // meio metro de vão: bem visível
    assert!(
        !points_of(&marks(&strained), JOINT_STRAIN_RGBA).is_empty(),
        "um pino com as âncoras a meio metro NÃO acusou tensão — a única \
         situação em que o vermelho existe passou despercebida"
    );

    let resting = view(JointKind::Pin); // âncoras coincidentes
    assert!(
        points_of(&marks(&resting), JOINT_STRAIN_RGBA).is_empty(),
        "um pino EM REPOUSO foi pintado como deformado: o alarme dispara \
         sempre, e um alarme que sempre toca não é lido"
    );

    let mut spring = view(JointKind::Spring);
    spring.anchor_b = [2.0, 0.0];
    spring.length = Some(2.0);
    assert!(
        points_of(&marks(&spring), JOINT_STRAIN_RGBA).is_empty(),
        "a mola foi pintada como deformada por ter as pontas separadas — que \
         é exatamente o que uma mola tem"
    );
}

/// **Cada ponta diz de quem é** — e a diferença é de GEOMETRIA, não de cor.
///
/// A linha do corpo A é UM segmento; a do corpo B é tracejada. Um artista que
/// vai re-apontar uma ponta (o eyedropper do §12) precisa saber qual é qual
/// olhando a cena; e a paleta do overlay já está cheia, então uma cor nova
/// leria como outro sistema (ciano = collider dinâmico).
#[test]
fn each_end_says_which_body_it_belongs_to() {
    let v = view(JointKind::Pin);
    let m = marks(&v);
    let own = m
        .iter()
        .filter(|(_, c)| *c == JOINT_DIM_RGBA)
        .flat_map(|(p, _)| p.elements())
        .collect::<Vec<_>>();
    let moves = own
        .iter()
        .filter(|el| matches!(el, PathEl::MoveTo(_)))
        .count();
    assert!(
        moves >= 3,
        "as linhas de posse têm {moves} sub-caminhos: com A e B desenhadas \
         iguais não há como saber qual ponta é qual sem abrir o painel"
    );
    // A de A é inteira; a de B é a soma dos traços, mais curta que o vão.
    let (w, _) = extent(&m, JOINT_DIM_RGBA);
    assert!(
        w > 1.0,
        "as linhas de posse não alcançam os corpos (largura {w:.1} px): elas \
         existem justamente para ligar a âncora ao objeto"
    );
}

/// **Um limite autorado é um limite DESENHADO** — e um pino livre não inventa
/// paredes.
#[test]
fn a_limited_hinge_draws_its_arc_and_a_free_one_does_not() {
    let mut limited = view(JointKind::Pin);
    limited.limits = Some([-0.7, 0.7]);
    let free = view(JointKind::Pin);

    // ⚠️ O oráculo é ANGULAR, não de extensão: a caixa da banda apagada é
    // dominada pelas linhas de posse (2 m = 200 px), e o arco de 21 px cabe
    // inteiro dentro dela — a 1ª versão deste gate mediu 200,0 px nos dois
    // casos e não podia falhar. Um arco é a única coisa que COBRE ângulo à
    // distância fixa da âncora.
    let band = (LIMIT_ARC_PX - 3.0, LIMIT_ARC_PX + 3.0);
    let with = angular_spread(&marks(&limited), JOINT_DIM_RGBA, limited.anchor_a, band);
    let without = angular_spread(&marks(&free), JOINT_DIM_RGBA, free.anchor_a, band);
    assert!(
        with > 1.0,
        "o alcance autorado cobriu {with:.2} rad a {LIMIT_ARC_PX} px da âncora \
         — o limite segue sendo número cego no §12"
    );
    assert!(
        without < 0.2,
        "um pino LIVRE desenhou {without:.2} rad de arco: paredes que ninguém \
         autorou, que é pior que nenhuma parede"
    );
}

/// **Um motor autorado gira na tela** — e o glifo é o MESMO da zona de torque,
/// porque a pergunta é a mesma (*para que lado?*); a cor é que diz de quem.
#[test]
fn a_motor_is_visible_and_its_direction_is_the_sign() {
    let mut cw = view(JointKind::Pin);
    cw.motor_speed = Some(-2.0);
    let mut ccw = view(JointKind::Pin);
    ccw.motor_speed = Some(2.0);
    let passive = view(JointKind::Pin);

    let n = |v: &JointView| points_of(&marks(v), JOINT_RGBA).len();
    assert!(
        n(&cw) > n(&passive) && n(&ccw) > n(&passive),
        "um pino motorizado desenhou o mesmo que um passivo: o motor é \
         invisível na cena"
    );
    assert_ne!(
        points_of(&marks(&cw), JOINT_RGBA),
        points_of(&marks(&ccw), JOINT_RGBA),
        "motor horário e anti-horário desenham igual — o SINAL é a informação \
         inteira do glifo"
    );
}

/// **A corda frouxa pendura; a tesa é uma reta** — e a barriga cai para onde a
/// GRAVIDADE aponta, não para um "para baixo" de tela escrito à mão.
#[test]
fn a_slack_rope_sags_along_gravity_and_a_taut_one_is_straight() {
    let mut taut = view(JointKind::Rope);
    taut.anchor_b = [2.0, 0.0];
    taut.length = Some(2.0); // exatamente o vão: sob carga
    let mut slack = view(JointKind::Rope);
    slack.anchor_b = [2.0, 0.0];
    slack.length = Some(3.0);

    let (_, taut_h) = extent(&marks(&taut), JOINT_RGBA);
    let (_, slack_h) = extent(&marks(&slack), JOINT_RGBA);
    assert!(
        slack_h > taut_h + 5.0,
        "a corda frouxa mediu {slack_h:.1} px de altura contra {taut_h:.1} da \
         tesa: as duas leem igual, e a folga — o fato de a restrição NÃO estar \
         agindo — fica invisível"
    );

    // Y de tela cresce para BAIXO e a gravidade aponta para −y de mundo, então
    // a barriga tem de cair no maior y de tela.
    let pts = points_of(&marks(&slack), JOINT_RGBA);
    let lowest = pts.iter().fold(f64::MIN, |m, (_, y)| m.max(*y));
    let anchors_y = pts.first().map(|(_, y)| *y).unwrap_or_default();
    assert!(
        lowest > anchors_y,
        "a corda pendurou para CIMA (menor y de tela): a direção da barriga \
         não veio da gravidade"
    );

    // Sem gravidade não há para onde pendurar — o degenerado que a física
    // resolve sozinha, como o empuxo sem gravidade (W-Buoyancy).
    let zero_g = joint_marks(
        true,
        std::slice::from_ref(&slack),
        [0.0, 0.0],
        &camera(),
        window(),
    );
    assert!(
        extent(&zero_g, JOINT_RGBA).1 < taut_h + 1.0,
        "sem gravidade a corda ainda pendurou — a barriga estava vindo de um \
         'para baixo' inventado, não do mundo"
    );
}

/// **Desligado desenha NADA** — o mesmo toggle que os colliders obedecem.
#[test]
fn the_toggle_silences_the_joint_marks_too() {
    assert!(joint_marks(false, &[view(JointKind::Pin)], G, &camera(), window()).is_empty());
}

/// **Uma cena sem joints não custa nada.**
#[test]
fn no_joints_no_paths() {
    assert!(joint_marks(true, &[], G, &camera(), window()).is_empty());
}

/// **Um pino em repouso ainda desenha alguma coisa.**
///
/// As duas âncoras dele são o MESMO ponto de mundo — é o que um pino é — então
/// o segmento entre elas tem comprimento zero e não pinta nada. O glifo é o que
/// impede o joint mais comum do editor de ser invisível.
#[test]
fn a_joint_whose_anchors_coincide_is_still_visible() {
    let mut v = view(JointKind::Pin);
    v.anchor_a = [2.0, 1.0];
    v.anchor_b = [2.0, 1.0];
    let m = marks(&v);
    let (cx, cy) = camera().world_to_screen([2.0, 1.0], window());
    let (cx, cy) = (f64::from(cx), f64::from(cy));
    let spread = points_of(&m, JOINT_RGBA)
        .iter()
        .map(|(x, y)| ((x - cx).powi(2) + (y - cy).powi(2)).sqrt())
        .fold(0.0f64, f64::max);
    assert!(
        spread > 1.0,
        "um pino em repouso pintou tudo a {spread:.3} px da âncora — um \
         segmento de comprimento zero não desenha nada, e o joint mais comum \
         do editor ficaria invisível"
    );
}

#[path = "physics_overlay_joint_pose_tests.rs"]
mod pose_tests;

/// Os gates do TRILHO (Slider), irmão próprio pelo cap de 600 LOC da shell — e o
/// corte é por assunto: tudo aqui é sobre a figura que um `JointKind::Slider`
/// desenha, que é a única cujo alcance é uma DISTÂNCIA e não um ângulo.
#[path = "physics_overlay_joint_rail_tests.rs"]
mod rail_tests;

/// Os gates do joint ROMPIDO (W-J7), irmão próprio pelo mesmo cap e pelo mesmo
/// corte: tudo aqui descreve um joint que **não está segurando**.
#[path = "physics_overlay_joint_break_tests.rs"]
mod break_tests;

/// Os gates do joint DESLIGADO (W-J8) — o vizinho do anterior, e a distinção
/// entre os dois é o assunto: um deixou de segurar, o outro foi desarmado.
#[path = "physics_overlay_joint_active_tests.rs"]
mod active_tests;
