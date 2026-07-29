//! **O feedback de um GESTO EM ANDAMENTO** — a banda elástica de criar um joint
//! (W-J4) e a mola da mão (W-Grab).
//!
//! Irmão do [`super::physics_overlay_joints`], e o corte é a distinção que aquele
//! módulo já articula: tudo lá é **ANOTAÇÃO** de coisas que existem (*onde está o
//! collider, até onde vai o limite*) e o artista escolhe vê-las com a tecla `B`;
//! estas duas são **feedback de um gesto em voo**, desenhadas mesmo com o overlay
//! DESLIGADO, porque um gesto que não se vê é um gesto que parece não ter
//! começado.
//!
//! Nasceu do cap de 600 LOC quando a cor do não-segura chegou ao laço de pintura.

use ph2d_host::WindowSize;
use ph2d_render::Camera2d;
use ph2d_vector::BezPath;

use super::physics_overlay_joint_glyphs::{ring_px, screen_of, spring_zigzag};
use super::physics_overlay_joints::{JOINT_DOT_PX, dashed};

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

/// **A MÃO** (W-Grab) — verde-limão, a única cor livre na paleta deste overlay
/// (verde=estático · ciano=dinâmico · violeta=kinematic/torque · magenta=sensor ·
/// âmbar=joint · vermelho=ruptura · branco=contato · laranja=força).
pub(super) const GRAB_RGBA: [f32; 4] = [0.55, 1.0, 0.30, 0.95]; // LITERAL-COLOR-OK: overlay da mão

/// **A mola da mão**, do cursor até o ponto de pega — desenhada como o **ZIGZAG**
/// de mola, com um anel no ponto pego.
///
/// ⚠️ **A FORMA diz o mecanismo e a COR diz de quem é:** o artista já aprendeu no
/// W-J1 que zigzag é mola, e a mão **é** uma mola (uma `SpringJoint` de verdade
/// para uma âncora invisível no cursor). Um traço reto diria *"isto é rígido"*, o
/// que é exactamente a coisa errada a prometer — ela cede contra parede, e é isso
/// que a distingue de um teleporte.
///
/// ⚠️ **Desenhada mesmo com o overlay DESLIGADO**, pela mesma razão da
/// [`draw_band`]: é feedback de um gesto em andamento, não anotação.
pub(super) fn draw_grab(
    grab: Option<([f32; 2], [f32; 2])>,
    camera: &Camera2d,
    window: WindowSize,
) -> Option<BezPath> {
    let (cursor, hold) = grab?;
    let a = screen_of(camera, window, cursor);
    let b = screen_of(camera, window, hold);
    let mut p = spring_zigzag(a, b);
    // O anel marca ONDE no corpo a mão pegou — o ponto que a mola persegue. Sem
    // ele, um clique sem arrasto (que não move nada, de propósito) não deixaria
    // marca nenhuma, e o gesto pareceria não ter começado.
    ring_px(b, JOINT_DOT_PX * 2.0, &mut p);
    Some(p)
}
