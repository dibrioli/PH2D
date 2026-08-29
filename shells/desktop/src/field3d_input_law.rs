//! **As três leis da câmera, puras** — sem `App`, sem ponteiro, sem estado do smoke.
//!
//! ⭐ A separação não é estética: é o que torna as leis **testáveis pela porta do produto**. Um gate
//! que precisasse de um `App` inteiro para perguntar *"arrastar para a direita vira o modelo para a
//! direita?"* não seria escrito — e foi exatamente essa pergunta que a `line/sculpt3d` respondeu
//! errado nos dois sinais até um smoke a pegar. Aqui o gate traça a peça e **mede-a na tela**.

use ph2d_field_render::{Lens, Orbit};

use super::{
    FRAME_MARGIN, HOME_YAW_PITCH, MAX_HALF_EXTENT, MIN_HALF_EXTENT, ORBIT_RAD_PER_PX, ZOOM_PER_STEP,
};

/// ⭐ **Rotação LIVRE** por um arrasto de `(dx, dy)` pixels.
///
/// O arrasto nomeia um eixo **na tela**, e a peça gira em torno dele: o eixo é perpendicular ao
/// movimento, no plano da imagem. Um arrasto horizontal cai no eixo vertical da câmera, um
/// vertical cai no horizontal — e qualquer diagonal cai onde tem de cair, que é a metade que
/// uma câmera de dois ângulos não consegue exprimir.
///
/// ⚠️ **Nenhum eixo do MUNDO entra nesta conta**, e é daí que vem a ausência de polo. A câmera
/// antiga girava `yaw` em torno do Y do mundo, e era esse Y que criava a parede a ±90°.
///
/// O sinal é o da manipulação direta — *o modelo segue a mão* — e quem o prende é um gate que
/// mede a peça **na tela**.
pub(crate) fn orbit(cam: &mut Orbit, dx: f32, dy: f32) {
    let angle = dx.hypot(dy) * ORBIT_RAD_PER_PX;
    if angle <= 0.0 {
        return;
    }
    cam.turn_local([-dy, -dx, 0.0], angle);
}

/// Repõe a orientação e o enquadramento, mantendo o alvo onde está.
pub(crate) fn home(cam: &mut Orbit) {
    let fresh = Orbit::from_yaw_pitch(HOME_YAW_PITCH.0, HOME_YAW_PITCH.1);
    cam.rotation = fresh.rotation;
    cam.half_extent = fresh.half_extent;
    cam.target = [0.0; 3];
}

/// ⭐⭐ **ENQUADRA A PEÇA** (W46) — o alvo é o centro dela, e o meio-alcance é o raio com folga.
///
/// ⚠️ **O bordo NÃO é calculado aqui**: é o [`ph2d_field_eval::bounds::bounding_ball`], o mesmo
/// que o exportador usa desde a W33. *Duas réguas para a mesma grandeza é a doença que este
/// módulo já nomeou três vezes* — e a esfera é a moeda certa porque a composição não a estraga
/// (a nota do §34.2).
///
/// ⚠️ A orientação **não se toca**: enquadrar responde *"onde e quão longe"*, não *"de que
/// lado"*. Quem repõe o ângulo é o [`home`], e é ele que chama os dois.
pub(crate) fn frame(cam: &mut Orbit, ball: ph2d_field_eval::bounds::Ball) {
    cam.target = ball.center;
    cam.half_extent = (ball.radius * FRAME_MARGIN).clamp(MIN_HALF_EXTENT, MAX_HALF_EXTENT);
}

/// Pan por um arrasto de `(dx, dy)` pixels, num quadro cujo lado menor mede `half_px` de meia
/// altura.
pub(crate) fn pan(cam: &mut Orbit, dx: f32, dy: f32, half_px: f32) {
    let k = cam.half_extent / half_px.max(1.0);
    let (right, up, _) = cam.basis();
    for i in 0..3 {
        cam.target[i] += -right[i] * dx * k + up[i] * dy * k;
    }
}

/// **A outra lente** — a troca, como lei pura.
///
/// ⚠️ Ela mora aqui e não na porta da tecla porque é a lei, e uma lei tem de ser gateável sem
/// janela: a `half_fov` que a convergente recebe ao voltar é a da referência
/// ([`ph2d_field_render::DEFAULT_HALF_FOV`]), e não a última que estava — guardá-la seria um
/// estado a mais para responder a uma pergunta que a referência já responde.
pub(crate) fn other_lens(lens: Lens) -> Lens {
    match lens {
        Lens::Perspective { .. } => Lens::Ortho,
        Lens::Ortho => Lens::Perspective {
            half_fov: ph2d_field_render::DEFAULT_HALF_FOV,
        },
    }
}

/// Zoom por `steps` linhas de roda.
pub(crate) fn zoom(cam: &mut Orbit, steps: f32) {
    cam.half_extent =
        (cam.half_extent / ZOOM_PER_STEP.powf(steps)).clamp(MIN_HALF_EXTENT, MAX_HALF_EXTENT);
}
