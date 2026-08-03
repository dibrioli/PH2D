//! **O FANTASMA do limite** — a silhueta de onde o corpo B PARARIA se o limite
//! que a mão está arrastando fosse solto agora.
//!
//! Irmão do [`super::physics_overlay_joints`], separado dele em W-RopeSays pelo
//! cap de 600 LOC da shell. ⚠️ **O corte é por ASSUNTO:** o pai desenha *o que um
//! joint É* — a figura dele, a cor que ele veste, o envelope que ele impõe —, e
//! isto responde outra pergunta, *onde este arrasto vai parar o corpo*. É a única
//! coisa neste overlay que descreve um FUTURO, e por isso ela é a única que
//! precisa do `SimWorld` (para ler a silhueta do collider que vai ser posado).

use ph2d_ecs::SimWorld;
use ph2d_host::WindowSize;
use ph2d_physics_ecs::JointView;
use ph2d_render::Camera2d;
use ph2d_vector::BezPath;

/// **O FANTASMA do corpo B** — âmbar quase apagado, a silhueta de onde o limite
/// que está sendo arrastado deixaria o corpo parar.
///
/// É o *'L'* do RUBE sem modo: arrastar a parede JÁ posa. Sem ele o artista
/// arrasta um tracinho num arco e descobre o que autorou só depois de dar Play —
/// o arco tem uma agulha viva em `angle_b`, que diz onde o corpo ESTÁ, e nada
/// dizia onde ele PARARIA.
pub(super) const JOINT_GHOST_RGBA: [f32; 4] = [0.98, 0.75, 0.25, 0.28]; // LITERAL-COLOR-OK: overlay de joint (fantasma)

/// A silhueta de B na pose que `limit` permite, ou `None` quando não há arrasto
/// de limite em voo / o corpo B não tem collider.
///
/// ⚠️ **Desenha e nada mais.** O fantasma nunca escreve pose: ele é uma função
/// pura da view e do número que o arrasto está autorando. O corpo real só se move
/// quando o solver o move — e é justamente essa separação que torna possível
/// posar um limite com a simulação parada.
///
/// ⚠️ **O MOVIMENTO é o do grau de liberdade livre, e esse foi o bug.** Numa
/// dobradiça o corpo GIRA em torno da âncora por `Δ = (angle_a + limit) −
/// angle_b`; num trilho ele **DESLIZA** pelo eixo, porque um curso é uma
/// distância. Até 2026-07-26 este era o **quarto** leitor de `JointView::limits`
/// que o W-J5 não avisou (os outros três: o arco, as alças e a escrita do
/// arrasto) — ele girava o corpo por *0,9 radiano* para um curso de *0,9 metro*,
/// e o resultado era a silhueta solta que o Enio fotografou: *"aparece um gizmo
/// fantasma rodando que parece não estar relacionado corretamente ao joint"*.
///
/// Deslizando, ele passa a ser a coisa mais útil que este overlay desenha num
/// Slider: **o carrinho onde ele vai PARAR**, enquanto a alça ainda está na mão.
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
    // ⚠️ Um joint preso ao MUNDO não tem silhueta de B para o fantasma mostrar
    // (W-JointWorld) — e o `?` já é a resposta certa, porque este passe inteiro
    // é *"desenhe onde B ESTARIA"*. Sem corpo B não há pergunta.
    let body_b = v.body_b?;
    let col = world.get::<ph2d_physics_ecs::Collider>(body_b)?;
    let mut chain = Vec::new();
    let t = ph2d_ecs::world_transform_into(world, body_b, &mut chain)?;
    let live = [t.translation.x, t.translation.y];

    // Onde o corpo estaria, e virado como — uma resposta por tipo de movimento.
    let (centre, d) = if v.kind.limits_in_metres() {
        // TRILHO: desliza pelo eixo. O deslocamento vivo é a separação das duas
        // âncoras ao longo dele (é isso que o rapier chama de posição do
        // prismatic), então o fantasma anda o que falta até o fim de curso.
        let axis = v.axis?;
        let along =
            |p: [f32; 2]| (p[0] - v.anchor_a[0]) * axis[0] + (p[1] - v.anchor_a[1]) * axis[1];
        let step = limit - along(v.anchor_b);
        ([live[0] + axis[0] * step, live[1] + axis[1] * step], 0.0)
    } else {
        // DOBRADIÇA: gira rígido em torno da âncora A. `Δ` leva a pose VIVA de B
        // até a que o limite nomeia, então o fantasma é o corpo tal como ele
        // encostaria na parede — não uma figura nova.
        let d = (v.angle_a + limit) - v.angle_b;
        let (sin_d, cos_d) = libm::sincosf(d);
        let (dx, dy) = (live[0] - v.anchor_a[0], live[1] - v.anchor_a[1]);
        (
            [
                v.anchor_a[0] + dx * cos_d - dy * sin_d,
                v.anchor_a[1] + dx * sin_d + dy * cos_d,
            ],
            d,
        )
    };
    // O collider onde o SOLVER o põe: offset com a escala assinada dobrada,
    // girado com o corpo — a mesma leitura do `outlines`, porque um fantasma que
    // não casa com o contorno descreveria outro corpo.
    let (ox, oy) = (col.offset[0] * t.scale.x, col.offset[1] * t.scale.y);
    let (sin_r, cos_r) = (t.rotation + d).sin_cos();
    let (wox, woy) = (ox * cos_r - oy * sin_r, ox * sin_r + oy * cos_r);
    Some(super::physics_overlay::collider_outline(
        ph2d_physics_ecs::scaled_shape(col.shape, t.scale),
        centre[0] + wox,
        centre[1] + woy,
        t.rotation + d,
        camera,
        window,
    ))
}
