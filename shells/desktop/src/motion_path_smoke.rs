//! `PH2D_PATH_SMOKE=1` — a cena PRONTA PARA VER do **motion path** ([ADR-0141]).
//!
//! O que se olha, e a ordem:
//!
//! 1. Abra a timeline (`L`). O objeto está **selecionado**, então a trajetória
//!    aparece no canvas: um fio âmbar fraco (a FORMA) coberto de losangos (o TEMPO).
//! 2. ⚠️ **Olhe o ESPAÇAMENTO dos losangos.** Ele *é* a velocidade. Nas pontas do
//!    percurso eles se aglomeram (o ease) e no meio se esparramam. É a leitura do AE,
//!    e é a coisa inteira que a figura existe para dizer.
//! 3. Dê **Play**. O objeto segue o fio, e passa por cima de cada losango exatamente
//!    no quadro que aquele losango marca.
//! 4. O gráfico da timeline mostra **uma track só** — o valor dela é *distância
//!    percorrida*, não X nem Y. A inclinação que você vê ali É a velocidade na tela.
//! 5. Clique noutro objeto: a trajetória **some**. Ela é do que está na mão.
//!
//! **Os números que esta cena afirma são MEDIDOS**, não escolhidos — a sonda headless
//! roda em `motion_path_smoke_tests.rs` e imprime os mesmos que o prólogo anuncia.
//!
//! [ADR-0141]: ../../docs/architecture/decisions/0141-timeline-position-is-one-2d-channel-and-separate-axes-are-a-mode.md

use ph2d_anim::{AnimValue, Interp, RationalTime};
use ph2d_core::Vec2;
use ph2d_ecs::{Name, Transform};
use ph2d_render::Sprite;
use ph2d_timeline::{MotionPath, PropKind, TimelineDoc};

/// A trajetória da cena: um **S** deitado, feito de quatro âncoras suaves. Uma curva
/// com duas inflexões, e não um arco: numa curva de curvatura constante o fio e os
/// pontos contam a mesma história, e a diferença entre *forma* e *tempo* — que é o que
/// se está demonstrando — não aparece.
pub(crate) fn demo_path() -> MotionPath {
    let pts = [[-6.0_f32, -2.0], [-2.0, 2.0], [2.0, -2.0], [6.0, 2.0]];
    MotionPath::new(
        (0..pts.len())
            .map(|i| {
                MotionPath::auto_smooth(
                    (i > 0).then(|| pts[i - 1]),
                    pts[i],
                    (i + 1 < pts.len()).then(|| pts[i + 1]),
                )
            })
            .collect(),
    )
}

/// Autora a track: parte em repouso, acelera, e freia no fim — o ease que faz o
/// espaçamento dos pontos DIZER alguma coisa. Uma track linear desenharia pontos
/// igualmente espaçados, que é correto e não demonstra nada.
pub(crate) fn author(doc: &mut TimelineDoc, bits: u64, path: &MotionPath) {
    doc.bind(bits, PropKind::Position);
    let total = path.length() as f32;
    // ⚠️ A key de saída carrega o ease; a de chegada só fecha o segmento.
    doc.insert_key(
        bits,
        PropKind::Position,
        RationalTime::from_seconds(0.0),
        AnimValue::Float(0.0),
        Interp::Bezier {
            x1: 0.85,
            y1: 0.0,
            x2: 0.15,
            y2: 1.0,
        },
    );
    doc.insert_key(
        bits,
        PropKind::Position,
        RationalTime::from_seconds(3.0),
        AnimValue::Float(total),
        Interp::Linear,
    );
    let i = doc
        .bindings()
        .iter()
        .position(|b| b.entity == bits && b.prop == PropKind::Position)
        .expect("o bind acima");
    doc.bindings_mut()[i].path = Some(path.clone());
}

impl crate::App {
    /// No prólogo do frame, uma vez. No-op sem a env.
    pub(crate) fn motion_path_smoke(&mut self) {
        if self.motion_path_smoke_done {
            return;
        }
        if std::env::var_os("PH2D_PATH_SMOKE").is_none() {
            return;
        }
        if self.gfx.is_none() {
            return; // ainda sem mundo; tenta no próximo frame
        }
        self.motion_path_smoke_done = true;

        let bits = {
            let gfx = self.gfx.as_mut().expect("gfx");
            gfx.sim
                .world_mut()
                .spawn((
                    Transform::from_translation(Vec2::new(-6.0, -2.0)),
                    Sprite::atlas(0, [0.8, 0.8], [1.0, 0.55, 0.15, 1.0]),
                    Name::new("Traveller"),
                ))
                .id()
                .to_bits()
        };
        // Um SEGUNDO objeto, parado: é ele que prova o item 5 (clicar nele apaga a
        // trajetória). Sem um vizinho, "só o selecionado" não é demonstrável.
        {
            let gfx = self.gfx.as_mut().expect("gfx");
            gfx.sim.world_mut().spawn((
                Transform::from_translation(Vec2::new(0.0, 4.0)),
                Sprite::atlas(0, [0.8, 0.8], [0.35, 0.45, 0.6, 1.0]),
                Name::new("Bystander"),
            ));
        }

        let path = demo_path();
        author(&mut self.timeline.doc, bits, &path);

        // A trajetória só é desenhada para o SELECIONADO — então a cena o seleciona,
        // senão o smoke abre sem mostrar a própria feature.
        if let Some(hero) = self.gfx.as_mut().and_then(|g| g.hero_screen.as_mut()) {
            hero.gizmo.replace_selection(Some(bits));
        }

        let dots = (3.0 * self.timeline.doc.fps_display).round() as usize;
        eprintln!(
            "[path-smoke] trajetoria em S: {} ancoras, {:.2} unidades de percurso, \
             {dots} pontos de tempo em 3 s.",
            path.len(),
            path.length()
        );
        eprintln!(
            "[path-smoke] abra a timeline (L) e olhe o ESPACAMENTO dos losangos: \
             juntos nas pontas (ease), esparramados no meio. Play para conferir."
        );
    }
}

#[cfg(test)]
#[path = "motion_path_smoke_tests.rs"]
mod tests;
