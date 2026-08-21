//! ⭐ **O que um arrasto PEDE** — a metade do gizmo que responde ao movimento.
//!
//! ⚠️ É um **módulo-filho** de [`super`], e não uma crate nem um irmão de topo: ele partilha as
//! constantes de desenho e os utilitários de geometria do pai (`use super::*`), e todos os caminhos
//! que já existiam (`field3d_gizmo::drag`, `field3d_gizmo::Motion`) continuam a valer pelo
//! re-export. *Cortar um arquivo não pode custar uma reescrita a cada sítio que o chamava.*
//!
//! # Por que a linha do corte cai AQUI
//!
//! O irmão responde *"onde estão as alças, e qual delas o cursor apanhou"* — projeção e
//! apontamento. Este responde *"o que o nó faz enquanto o ponteiro se move"* — a lei do gesto. As
//! duas metades tocam-se num sítio só ([`super::Handle`]), e é o corte que o próprio arquivo já
//! tinha nos comentários antes de o ter nos arquivos.

use super::*;

/// ⭐ **O que um arrasto pede** — e por que não é sempre um vetor.
///
/// ⚠️ Os três verbos compõem de formas diferentes: translação **soma**, rotação **compõe** e escala
/// **multiplica**. Um `[f32; 3]` para os três obrigaria quem recebe a adivinhar qual é qual pelo
/// modo em que o gizmo estava — e num quadro em que o modo mudou a meio, a adivinha erra.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum Motion {
    Translate([f32; 3]),
    /// Em torno de `axis` (unitário, no mundo), pelo **pivô da âncora**.
    Rotate {
        axis: [f32; 3],
        angle: f32,
    },
    /// Fator **uniforme**. Ver o doc do módulo.
    Scale(f32),
}

impl Motion {
    /// ⭐ **Acumular dois pedidos do MESMO arrasto.**
    ///
    /// ⚠️ Entre dois quadros chegam vários eventos de ponteiro, e cada verbo tem a sua lei de
    /// composição. Somar fatores de escala, por exemplo, faria dois passos de ×1,1 valerem ×2,2 em
    /// vez de ×1,21 — e o defeito só apareceria com o rato depressa, que é o mais difícil de
    /// acreditar quando alguém o reporta.
    ///
    /// Variantes diferentes não se compõem: o segundo ganha. Não pode acontecer num arrasto (a alça
    /// fixa o verbo), e inventar uma soma entre um giro e uma escala seria pior do que ceder.
    pub(crate) fn merge(self, next: Motion) -> Motion {
        match (self, next) {
            (Motion::Translate(a), Motion::Translate(b)) => {
                Motion::Translate([a[0] + b[0], a[1] + b[1], a[2] + b[2]])
            }
            (Motion::Rotate { axis, angle: a }, Motion::Rotate { angle: b, .. }) => {
                Motion::Rotate { axis, angle: a + b }
            }
            (Motion::Scale(a), Motion::Scale(b)) => Motion::Scale(a * b),
            (_, other) => other,
        }
    }

    /// ⭐ **O que falta aplicar**, dado o que já foi: `self` é o TOTAL desde a pegada e `applied` o
    /// que o mundo já recebeu.
    ///
    /// ⚠️ É a inversa exacta de [`Motion::merge`], e existe pelo mesmo motivo que ela: cada verbo
    /// compõe à maneira dele. `total.since(applied).merge(applied) == total` — que é o gate.
    pub(crate) fn since(self, applied: Motion) -> Motion {
        match (self, applied) {
            (Motion::Translate(t), Motion::Translate(a)) => {
                Motion::Translate([t[0] - a[0], t[1] - a[1], t[2] - a[2]])
            }
            (Motion::Rotate { axis, angle: t }, Motion::Rotate { angle: a, .. }) => {
                Motion::Rotate { axis, angle: t - a }
            }
            (Motion::Scale(t), Motion::Scale(a)) if a.abs() > f32::MIN_POSITIVE => {
                Motion::Scale(t / a)
            }
            (total, _) => total,
        }
    }

    /// O pedido **neutro** deste verbo — o ponto de partida de um arrasto.
    pub(crate) fn neutral(self) -> Motion {
        match self {
            Motion::Translate(_) => Motion::Translate([0.0; 3]),
            Motion::Rotate { axis, .. } => Motion::Rotate { axis, angle: 0.0 },
            Motion::Scale(_) => Motion::Scale(1.0),
        }
    }

    /// ⭐ **O mesmo pedido, preso à grelha** — o gesto de precisão (`Ctrl`).
    ///
    /// `step` é o passo da translação, em unidades de mundo, e vem **derivado do enquadramento**
    /// ([`snap_step`]). O ângulo e o fator têm passos próprios, e cada um diz por que é aquele.
    pub(crate) fn snapped(self, step: f32) -> Motion {
        let round_to = |v: f32, q: f32| -> f32 { if q > 0.0 { (v / q).round() * q } else { v } };
        match self {
            Motion::Translate(d) => Motion::Translate([
                round_to(d[0], step),
                round_to(d[1], step),
                round_to(d[2], step),
            ]),
            // ⚠️ **15°, e a razão é o que se pede pelo NOME**: é o maior passo que ainda contém 30,
            // 45, 60 e 90 — os ângulos que um artista diz em voz alta. Um passo mais fino não os
            // perde, mas obriga a mira; um mais grosso perde o 45.
            Motion::Rotate { axis, angle } => Motion::Rotate {
                axis,
                angle: round_to(angle, SNAP_ANGLE),
            },
            // ⚠️ **O passo do fator é o que a LEITURA consegue exprimir.** O número aparece com uma
            // casa decimal, então prender a 0,1 faz um valor preso ser exatamente o que se lê. Um
            // passo mais fino mostraria "×1,5" para dois tamanhos diferentes.
            Motion::Scale(f) => Motion::Scale(round_to(f, SNAP_FACTOR).max(SNAP_FACTOR)),
        }
    }

    /// Um pedido que não pede nada — o que uma alça degenerada devolve.
    pub(crate) fn is_idle(self) -> bool {
        match self {
            Motion::Translate(d) => d.iter().all(|v| v.abs() < f32::EPSILON),
            Motion::Rotate { angle, .. } => angle.abs() < f32::EPSILON,
            Motion::Scale(f) => (f - 1.0).abs() < f32::EPSILON,
        }
    }
}

/// O passo de ângulo do gesto preso. Ver [`Motion::snapped`].
pub(crate) const SNAP_ANGLE: f32 = std::f32::consts::PI / 12.0;

/// O passo do fator de tamanho. Ver [`Motion::snapped`].
pub(crate) const SNAP_FACTOR: f32 = 0.1;

/// ⭐ **O passo da translação presa, DERIVADO do enquadramento** — o menor número redondo (1-2-5)
/// cujo comprimento na tela ainda se consegue mirar.
///
/// ⚠️ Um passo fixo em unidades de mundo é inútil nos dois extremos: aproximado, dois pontos da
/// grelha ficam a meia tela um do outro; afastado, ficam dentro do mesmo pixel. A grelha do Blender
/// subdivide com o zoom pela mesma razão.
///
/// **A condição que fixa o número:** dois pontos vizinhos da grelha têm de estar mais afastados do
/// que a tolerância do próprio ponteiro ([`GRAB_PX`]) — abaixo disso o gesto deixa de conseguir
/// escolher entre eles, e prender à grelha passa a ser sorteio. Sobe-se então a escada 1-2-5 até o
/// primeiro degrau que passa.
#[must_use]
pub(crate) fn snap_step(screen: Screen) -> f32 {
    let min_world = GRAB_PX / screen.px_per_world().max(f32::MIN_POSITIVE);
    if !min_world.is_finite() || min_world <= 0.0 {
        return SNAP_FACTOR;
    }
    let decade = 10f32.powf(min_world.log10().floor());
    for m in [1.0, 2.0, 5.0] {
        let step = m * decade;
        if step >= min_world {
            return step;
        }
    }
    decade * 10.0
}

/// ⭐ **O arrasto**: o que o nó faz quando o ponteiro vai de `from_px` a `to_px`.
///
/// Devolve um pedido **inerte** ([`Motion::is_idle`]) quando a alça não é utilizável neste
/// enquadramento — a mesma condição que [`project`] usa para a esconder, porque uma alça invisível
/// não pode arrastar.
pub(crate) fn drag(
    handle: Handle,
    anchor: Anchor,
    cam: &Orbit,
    screen: Screen,
    from_px: [f32; 2],
    to_px: [f32; 2],
) -> Motion {
    // A MESMA escala que a projeção usa — ver `project`. Duas contas para o comprimento do braço
    // dariam um arrasto que anda diferente do que a seta desenhada promete.
    let px_per_world = cam
        .px_per_world_at(anchor.origin, screen)
        .max(f32::MIN_POSITIVE);
    let arm = ARM_PX / px_per_world;
    match handle {
        // A conta é uma projeção escalar: `d` é quanto o braço inteiro mede na tela, e a fração do
        // movimento do rato ao longo dele é a fração do braço que a peça anda.
        //
        // ⚠️ **Sem divisão por zero possível**: `dot(d,d)` só é nulo quando o eixo aponta ao
        // observador, e aí a alça já não está viva.
        Handle::Axis(n) => {
            // Sem projeção não há para onde arrastar — a mesma condição que esconde a alça.
            let (Some((o2, _)), Some((tip, _))) = (
                cam.project(anchor.origin, screen),
                cam.project(offset(anchor.origin, anchor.axes[n], arm), screen),
            ) else {
                return Motion::Translate([0.0; 3]);
            };
            let d = [tip[0] - o2[0], tip[1] - o2[1]];
            let dd = d[0].mul_add(d[0], d[1] * d[1]);
            if dd < MIN_ARM_PX * MIN_ARM_PX {
                return Motion::Translate([0.0; 3]);
            }
            let m = [to_px[0] - from_px[0], to_px[1] - from_px[1]];
            let t = m[0].mul_add(d[0], m[1] * d[1]) / dd * arm;
            Motion::Translate([
                anchor.axes[n][0] * t,
                anchor.axes[n][1] * t,
                anchor.axes[n][2] * t,
            ])
        }
        // Num plano, o deslocamento é a diferença entre dois pontos do plano — cada um o encontro do
        // raio do cursor com ele. É a mesma conta do gizmo 2D, com o plano a vir do mundo.
        Handle::Plane(n) => Motion::Translate(plane_delta(
            anchor.axes[n],
            anchor.origin,
            cam,
            screen,
            from_px,
            to_px,
        )),
        // O plano da tela: a normal é a direção da vista, e o denominador vale 1 — nunca degenera.
        Handle::View => {
            let (_, _, fwd) = cam.basis();
            Motion::Translate(plane_delta(fwd, anchor.origin, cam, screen, from_px, to_px))
        }
        Handle::Ring(n) => spin(anchor.axes[n], anchor.origin, cam, screen, from_px, to_px),
        Handle::ViewRing => {
            let (_, _, fwd) = cam.basis();
            spin(fwd, anchor.origin, cam, screen, from_px, to_px)
        }
        // ⭐ Tamanho é **razão de raios**, e não diferença: é o que faz duas metades de um arrasto
        // valerem o produto e não a soma — a mesma lei que um zoom de roda usa.
        Handle::Grip => {
            let Some((o2, _)) = cam.project(anchor.origin, screen) else {
                return Motion::Scale(1.0);
            };
            let r0 = dist(o2, from_px);
            let r1 = dist(o2, to_px);
            // ⚠️ O piso é do RAIO INICIAL, não do fator: agarrar em cima do centro daria uma razão
            // infinita e a peça saltaria num pixel. O punho vive a `ARM_PX` do centro, então este
            // piso só morde se alguém arrastar **para dentro** do centro.
            if r0 < GRAB_PX || !r1.is_finite() {
                return Motion::Scale(1.0);
            }
            Motion::Scale((r1 / r0).max(f32::MIN_POSITIVE))
        }
    }
}

/// ⭐ **O ângulo que o cursor varreu em torno de um eixo** — medido **no plano de rotação**, e não
/// na tela.
///
/// ⚠️ A alternativa (medir o ângulo em pixels em torno do centro projetado) é a que muitos editores
/// usam, e ela **mente fora do eixo da vista**: a projeção de um círculo é uma elipse, e o ângulo na
/// elipse não é o ângulo no círculo. O gesto ficaria rápido de um lado e lento do outro, e uma volta
/// inteira não fecharia. Aqui os dois pontos do cursor são levados ao plano real e o ângulo sai do
/// produto vetorial — exato, e com o sinal já certo.
fn spin(
    axis: [f32; 3],
    origin: [f32; 3],
    cam: &Orbit,
    screen: Screen,
    from_px: [f32; 2],
    to_px: [f32; 2],
) -> Motion {
    let axis = normalize(axis);
    let idle = Motion::Rotate { axis, angle: 0.0 };
    let (Some(a), Some(b)) = (
        ray_plane(cam, screen, from_px, origin, axis),
        ray_plane(cam, screen, to_px, origin, axis),
    ) else {
        return idle;
    };
    let d0 = [a[0] - origin[0], a[1] - origin[1], a[2] - origin[2]];
    let d1 = [b[0] - origin[0], b[1] - origin[1], b[2] - origin[2]];
    // Em cima do eixo o ângulo é ruído: um pixel de rato varreria meia volta.
    if len3(d0) < f32::EPSILON || len3(d1) < f32::EPSILON {
        return idle;
    }
    let angle = dot(cross(d0, d1), axis).atan2(dot(d0, d1));
    Motion::Rotate { axis, angle }
}

fn plane_delta(
    normal: [f32; 3],
    origin: [f32; 3],
    cam: &Orbit,
    screen: Screen,
    from_px: [f32; 2],
    to_px: [f32; 2],
) -> [f32; 3] {
    let a = ray_plane(cam, screen, from_px, origin, normal);
    let b = ray_plane(cam, screen, to_px, origin, normal);
    match (a, b) {
        (Some(a), Some(b)) => [b[0] - a[0], b[1] - a[1], b[2] - a[2]],
        _ => [0.0; 3],
    }
}

/// Onde o raio de um pixel encontra o plano que passa por `p0` com normal `n`. `None` de perfil.
fn ray_plane(
    cam: &Orbit,
    screen: Screen,
    px: [f32; 2],
    p0: [f32; 3],
    n: [f32; 3],
) -> Option<[f32; 3]> {
    let (o, dir) = cam.ray(px[0], px[1], screen);
    let denom = dot(dir, n);
    // ⚠️ O limiar não é folclore: abaixo dele um pixel de rato vale um salto arbitrário no plano, e
    // o gesto deixa de ser manipulação para ser sorteio. É a mesma razão de `MIN_ARM_PX`.
    if denom.abs() < 1.0e-3 {
        return None;
    }
    let t = dot([p0[0] - o[0], p0[1] - o[1], p0[2] - o[2]], n) / denom;
    Some([o[0] + dir[0] * t, o[1] + dir[1] * t, o[2] + dir[2] * t])
}
