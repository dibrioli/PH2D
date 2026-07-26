//! **O overlay de joints** — irmão do `physics_overlay`, separado dele quando os
//! dois juntos passaram do cap de 600 LOC do shell.
//!
//! Colliders e joints respondem duas perguntas diferentes com a mesma técnica:
//! *que forma é esta, fisicamente?* e *a que isto está preso?*. Manter os dois
//! num arquivo é o que o estourou; mantê-los VIZINHOS é o que guarda a regra de
//! espaço-de-tela (ver o cabeçalho do módulo pai) num lugar só para ambos.
//!
//! ## W-J1 — o joint DESENHA o que ele é
//!
//! Até esta wave havia uma figura só (segmento + dois anéis) para os quatro
//! tipos: o canvas dizia *"há um joint aqui"* e todo o resto — tipo, alcance de
//! limite, comprimento de repouso, folga, deformação, e **de quem é cada
//! ponta** — era número cego no §12 ou nada. Agora cada fato tem geometria:
//!
//! | fato | como se vê |
//! |---|---|
//! | qual tipo | o GLIFO (anel · quadrado · zigue-zague · fio) |
//! | de quem é cada ponta | a linha de posse: **A sólida, B tracejada** |
//! | alcance de um limite | o arco, com paredes, e a agulha no ângulo VIVO |
//! | para que lado o motor gira | o mesmo glifo de giro da zona de torque |
//! | repouso / máximo | o anel de comprimento, em MUNDO (dá zoom, cresce) |
//! | a restrição NÃO está sendo imposta | o vão entre as âncoras, em VERMELHO |
//!
//! ⚠️ **A distinção entre as pontas é GEOMÉTRICA, não de cor** — a paleta do
//! overlay já está cheia (verde estático · ciano dinâmico · branco contato ·
//! laranja força · violeta torque · amarelo lançamento · magenta sensor ·
//! ciano-claro linha d'água), e um azul-esverdeado novo leria como contorno de
//! collider. Tracejar é a diferença que sobra, e ela não colide com nada.
//!
//! ⚠️ **O desenho lê a ponte, nunca o componente ECS** ([`JointView`], plano 02
//! P2): um joint cujos corpos não resolvem não tem view, e desenhá-lo do
//! componente pintaria uma relação que o solver não está impondo.

use ph2d_ecs::SimWorld;
use ph2d_host::WindowSize;
use ph2d_physics_ecs::{JointKind, JointView};
use ph2d_render::Camera2d;
use ph2d_vector::{BezPath, Point};

use super::physics_overlay_annotations::torque_glyph;
use super::physics_overlay_joint_glyphs::{
    length_ring, limit_arc, pin_glyph, ring_px, rope_span, screen_of, slider_rail, spring_zigzag,
    weld_glyph,
};

/// Raio do anel desenhado em cada âncora, px de tela. Grande o bastante para
/// ver, pequeno o bastante para não esconder a arte sob uma corrente deles.
const JOINT_DOT_PX: f64 = 3.0; // LITERAL-PX-OK: chrome de overlay, raio de tela

/// Traço e vão da linha de posse do corpo **B**, px de tela.
const DASH_PX: f64 = 6.0; // LITERAL-PX-OK: chrome de overlay
const GAP_PX: f64 = 4.5; // LITERAL-PX-OK: chrome de overlay

/// A partir de quantos pixels de TELA um vão entre âncoras que deveriam
/// coincidir vira a marca vermelha de deformação.
///
/// Um pixel: abaixo disso não há o que desenhar, e o resíduo do solver em
/// repouso é **muito** menor que isso (medido no gate — 0,0000 px de separação
/// num pino assentado). Um limiar em METROS precisaria de um número mágico e
/// mentiria em zoom alto; a pergunta honesta é *"dá para ver?"*.
const STRAIN_MIN_PX: f64 = 1.0; // LITERAL-PX-OK: limiar de visibilidade

/// Joints — âmbar, para lerem como uma terceira coisa ao lado do cenário verde
/// e dos movedores ciano, em vez de como um deles.
pub(super) const JOINT_RGBA: [f32; 4] = [0.98, 0.75, 0.25, 0.9]; // LITERAL-COLOR-OK: overlay de joint

/// As linhas de posse e o arco de limite: o MESMO âmbar, apagado — não são
/// objetos novos, são o mesmo joint dizendo a quem se prende e até onde vai.
pub(super) const JOINT_DIM_RGBA: [f32; 4] = [0.98, 0.75, 0.25, 0.5]; // LITERAL-COLOR-OK: overlay de joint (posse/limite)

/// **A restrição NÃO está sendo imposta** — vermelho, a única cor do overlay
/// que diz *isto não está onde deveria*.
///
/// ⚠️ **O significado foi MEDIDO, e não é o que o nome "tensão" sugeria.** Os
/// impulse joints do rapier são RÍGIDOS: um pino segurando um corpo 500× mais
/// pesado, e outro levando um martelo de 400×, abriram **0,00000 m** entre as
/// âncoras nos 200 ticks — o vermelho por CARGA é inalcançável, e a linha
/// vermelha do RUBE descreve os joints *soft* do Box2D, que não portam.
///
/// O que ABRE o vão é a arquitetura: um joint não move um corpo **kinematic**
/// (massa infinita), então dois corpos curva-dirigidos que a animação afasta
/// ficam separados com o pino desenhado por cima — medido, **1,50 m = 150 px**.
/// É exatamente o estado em que o W-BakeJoint deixa um rig assado, e sem esta
/// marca ele desenharia um pino perfeitamente normal sobre dois objetos que já
/// não estão presos um ao outro.
pub(super) const JOINT_STRAIN_RGBA: [f32; 4] = [0.96, 0.32, 0.28, 0.95]; // LITERAL-COLOR-OK: overlay de joint nao imposto

/// **O overlay de joints.** Um caminho por cor, como o contorno faz.
///
/// `gravity` decide para onde uma corda frouxa pendura — a mesma fonte que diz
/// onde fica a superfície de uma poça (W-Buoyancy). Sem gravidade a corda
/// desenha reta.
pub(super) fn joint_marks(
    show: bool,
    views: &[JointView],
    gravity: [f32; 2],
    camera: &Camera2d,
    window: WindowSize,
) -> Vec<(BezPath, [f32; 4])> {
    if !show {
        return Vec::new();
    }
    let mut out = Vec::new();
    // A gravidade em direção de TELA, uma vez por quadro: a barriga da corda
    // cai para onde as coisas caem, sob qualquer flip da câmera.
    let g_screen = gravity_on_screen(gravity, camera, window);
    for v in views {
        let a = screen_of(camera, window, v.anchor_a);
        let b = screen_of(camera, window, v.anchor_b);
        out.push((ownership_lines(v, a, b, camera, window), JOINT_DIM_RGBA));
        let (span, glyph) = kind_marks(v, a, b, g_screen, camera, window);
        out.push((span, JOINT_RGBA));
        out.push((glyph, JOINT_RGBA));
        if let Some(strain) = strain_mark(v, a, b) {
            out.push((strain, JOINT_STRAIN_RGBA));
        }
        // O ENVELOPE AUTORADO, na banda apagada: até onde a dobradiça pode ir,
        // e o comprimento que a mola/corda nomeia. Não são o joint — são o que
        // o artista permitiu a ele, e por isso lêem como fundo do glifo (a
        // mesma distinção que separa a seta de força do anel do falloff).
        // ⚠️ **The arc is the envelope of an ANGULAR range only.** A Slider is
        // limited too, and its range is a stroke in metres — drawn by
        // `slider_rail`'s end-of-travel ticks, not by a circle at 0.5 radians.
        // Without this question a rail painted BOTH, and the arc it painted
        // described a hinge that does not exist.
        if let Some(arc) = v
            .limits
            .filter(|_| !v.kind.limits_in_metres())
            .map(|l| limit_arc(camera, window, v.anchor_a, v.angle_a, l, v.angle_b))
        {
            out.push((arc, JOINT_DIM_RGBA));
        }
        if let Some(len) = v.length {
            out.push((length_ring(camera, window, v.anchor_a, len), JOINT_DIM_RGBA));
        }
        // O motor reusa o glifo de giro da zona de torque: é a MESMA pergunta
        // (*para que lado isto gira?*), então é a mesma figura, e a cor diz de
        // quem ela é. Uma segunda figura para a mesma pergunta seria um segundo
        // vocabulário a aprender.
        if let Some(path) = v
            .motor_speed
            .filter(|s| *s != 0.0)
            .and_then(|s| torque_glyph(v.anchor_a[0], v.anchor_a[1], s, camera, window))
        {
            out.push((path, JOINT_RGBA));
        }
    }
    out
}

/// **O FANTASMA do corpo B** — âmbar quase apagado, a silhueta de onde o limite
/// que está sendo arrastado deixaria o corpo parar.
///
/// É o *'L'* do RUBE sem modo: arrastar a parede JÁ posa. Sem ele o artista
/// arrasta um tracinho num arco e descobre o que autorou só depois de dar Play —
/// o arco tem uma agulha viva em `angle_b`, que diz onde o corpo ESTÁ, e nada
/// dizia onde ele PARARIA.
pub(super) const JOINT_GHOST_RGBA: [f32; 4] = [0.98, 0.75, 0.25, 0.28]; // LITERAL-COLOR-OK: overlay de joint (fantasma)

/// **A BANDA ELÁSTICA do gesto de criar** (W-J4) — âmbar, tracejada, do ponto do
/// press até o cursor.
///
/// ⚠️ **Desenhada mesmo com o overlay DESLIGADO** (tecla `B`), ao contrário de
/// todo o resto deste módulo: os outros traços são ANOTAÇÃO de coisas que
/// existem (*onde está o collider, até onde vai o limite*) e o artista escolhe
/// vê-los; esta é o **feedback de um gesto em andamento**, e um gesto que não se
/// vê é um gesto que parece não ter começado.
///
/// Tracejada porque o joint ainda **não existe** — a linha de posse sólida é a de
/// um vínculo real (W-J1), e usar o mesmo traço aqui prometeria que já há um.
pub(super) fn draw_band(
    band: Option<([f32; 2], [f32; 2])>,
    camera: &Camera2d,
    window: WindowSize,
) -> Option<BezPath> {
    let (from, to) = band?;
    let a = screen_of(camera, window, from);
    let b = screen_of(camera, window, to);
    let mut p = BezPath::new();
    // Um anel no ponto de origem: ele é a ÂNCORA que vai nascer ali, e sem ele o
    // press não deixa marca nenhuma até o cursor andar.
    ring_px(a, JOINT_DOT_PX * 2.0, &mut p);
    dashed(a, b, &mut p);
    Some(p)
}

/// A silhueta de B na pose que `limit` permite, ou `None` quando não há arrasto
/// de limite em voo / o corpo B não tem collider.
///
/// ⚠️ **Desenha e nada mais.** O fantasma nunca escreve pose: ele é o collider de
/// B **girado em torno da âncora A** por `Δ = (angle_a + limit) − angle_b`, uma
/// função pura da view e do número que o arrasto está autorando. O corpo real
/// só se move quando o solver o move — e é justamente essa separação que torna
/// possível posar um limite com a simulação parada.
pub(super) fn limit_ghost(
    sim: &SimWorld,
    views: &[JointView],
    posed: Option<(ph2d_ecs::Entity, f32)>,
    camera: &Camera2d,
    window: WindowSize,
) -> Option<BezPath> {
    let (joint, limit) = posed?;
    let v = views.iter().find(|v| v.entity == joint)?;
    let world = sim.world();
    let col = world.get::<ph2d_physics_ecs::Collider>(v.body_b)?;
    let mut chain = Vec::new();
    let t = ph2d_ecs::world_transform_into(world, v.body_b, &mut chain)?;

    // O giro rígido em torno da âncora A. `Δ` leva a pose VIVA de B até a que o
    // limite nomeia, então o fantasma é o corpo tal como ele encostaria na
    // parede — não uma figura nova.
    let d = (v.angle_a + limit) - v.angle_b;
    let (sin_d, cos_d) = libm::sincosf(d);
    let rot_about = |p: [f32; 2]| {
        let (dx, dy) = (p[0] - v.anchor_a[0], p[1] - v.anchor_a[1]);
        [
            v.anchor_a[0] + dx * cos_d - dy * sin_d,
            v.anchor_a[1] + dx * sin_d + dy * cos_d,
        ]
    };
    // O collider onde o SOLVER o põe: offset com a escala assinada dobrada,
    // girado com o corpo — a mesma leitura do `outlines`, porque um fantasma que
    // não casa com o contorno descreveria outro corpo.
    let (ox, oy) = (col.offset[0] * t.scale.x, col.offset[1] * t.scale.y);
    let (sin_r, cos_r) = (t.rotation + d).sin_cos();
    let (wox, woy) = (ox * cos_r - oy * sin_r, ox * sin_r + oy * cos_r);
    let c = rot_about([t.translation.x, t.translation.y]);
    Some(super::physics_overlay::collider_outline(
        ph2d_physics_ecs::scaled_shape(col.shape, t.scale),
        c[0] + wox,
        c[1] + woy,
        t.rotation + d,
        camera,
        window,
    ))
}

/// O span + o glifo de um tipo. Devolve os dois separados porque o span pode
/// ser vazio (Pin/Weld não vão a lugar nenhum) sem apagar o glifo.
fn kind_marks(
    v: &JointView,
    a: Point,
    b: Point,
    g_screen: (f64, f64),
    camera: &Camera2d,
    window: WindowSize,
) -> (BezPath, BezPath) {
    let mut span = BezPath::new();
    let mut glyph = BezPath::new();
    match v.kind {
        // Pin e Weld COMPARTILHAM um ponto: o glifo mora na âncora, e o span
        // existe só quando as duas discordam — que é a deformação, pintada em
        // vermelho logo abaixo. Aqui ele fica vazio de propósito.
        JointKind::Pin => {
            glyph = pin_glyph(a);
            ring_px(b, JOINT_DOT_PX, &mut glyph);
        }
        JointKind::Weld => {
            glyph = weld_glyph(camera, window, v.anchor_a, v.angle_a);
            ring_px(b, JOINT_DOT_PX, &mut glyph);
        }
        JointKind::Spring => {
            span = spring_zigzag(a, b);
            ring_px(a, JOINT_DOT_PX, &mut glyph);
            ring_px(b, JOINT_DOT_PX, &mut glyph);
        }
        JointKind::Rope => {
            // A folga é medida em METROS (é aqui que eles existem); o glifo só
            // recebe a razão adimensional.
            let d = (v.anchor_b[0] - v.anchor_a[0]).hypot(v.anchor_b[1] - v.anchor_a[1]);
            let slack = match v.length {
                Some(l) if d > 1e-4 => f64::from(l / d) - 1.0,
                _ => 0.0,
            };
            span = rope_span(a, b, slack, g_screen);
            ring_px(a, JOINT_DOT_PX, &mut glyph);
            ring_px(b, JOINT_DOT_PX, &mut glyph);
        }
        // O Slider compartilha um ponto como o Pin — o glifo é o TRILHO, e o span
        // fica vazio pela mesma razão (quando as duas âncoras discordam, isso é a
        // deformação, pintada em vermelho logo abaixo).
        JointKind::Slider => {
            if let Some(axis) = v.axis {
                glyph = slider_rail(camera, window, v.anchor_a, axis, v.limits);
            }
            ring_px(b, JOINT_DOT_PX, &mut glyph);
        }
    }
    (span, glyph)
}

/// As duas linhas de posse: âncora→centro de cada corpo. **A sólida, B
/// tracejada** — ver o cabeçalho do módulo para o porquê de não ser cor.
fn ownership_lines(
    v: &JointView,
    a: Point,
    b: Point,
    camera: &Camera2d,
    window: WindowSize,
) -> BezPath {
    let mut p = BezPath::new();
    let ca = screen_of(camera, window, v.centre_a);
    let cb = screen_of(camera, window, v.centre_b);
    p.move_to(a);
    p.line_to(ca);
    dashed(b, cb, &mut p);
    p
}

/// Um segmento tracejado, em pixels de tela.
fn dashed(from: Point, to: Point, path: &mut BezPath) {
    let (dx, dy) = (to.x - from.x, to.y - from.y);
    let len = dx.hypot(dy);
    if len < 1e-6 {
        return;
    }
    let (ux, uy) = (dx / len, dy / len);
    let mut t = 0.0;
    while t < len {
        let e = (t + DASH_PX).min(len);
        path.move_to(Point::new(from.x + ux * t, from.y + uy * t));
        path.line_to(Point::new(from.x + ux * e, from.y + uy * e));
        t = e + GAP_PX;
    }
}

/// A marca de **restrição não-imposta**: o vão entre duas âncoras que deveriam
/// coincidir (ver [`JOINT_STRAIN_RGBA`] para o que de fato o abre — medido).
///
/// Só existe para os tipos que compartilham um ponto — as âncoras de uma mola
/// ou de uma corda são as DUAS PONTAS e estão separadas por construção, então
/// pintá-las de vermelho chamaria de erro o funcionamento normal.
fn strain_mark(v: &JointView, a: Point, b: Point) -> Option<BezPath> {
    if !v.kind.shares_a_point() || (b.x - a.x).hypot(b.y - a.y) < STRAIN_MIN_PX {
        return None;
    }
    let mut p = BezPath::new();
    p.move_to(a);
    p.line_to(b);
    Some(p)
}

/// A gravidade como direção unitária de TELA (para onde as coisas caem).
fn gravity_on_screen(gravity: [f32; 2], camera: &Camera2d, window: WindowSize) -> (f64, f64) {
    let m = gravity[0].hypot(gravity[1]);
    if m < 1e-6 {
        return (0.0, 0.0);
    }
    let o = screen_of(camera, window, [0.0, 0.0]);
    let g = screen_of(camera, window, [gravity[0] / m, gravity[1] / m]);
    let (dx, dy) = (g.x - o.x, g.y - o.y);
    let l = dx.hypot(dy);
    if l < 1e-9 {
        (0.0, 0.0)
    } else {
        (dx / l, dy / l)
    }
}

#[cfg(test)]
#[path = "physics_overlay_joints_tests.rs"]
mod joint_tests;
