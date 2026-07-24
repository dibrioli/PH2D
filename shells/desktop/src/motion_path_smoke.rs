//! `PH2D_PATH_SMOKE=1|2` — as cenas PRONTAS PARA VER do **motion path** ([ADR-0141]).
//!
//! # `=1` — a trajetória e a leitura de tempo
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
//! 6. **Arraste um dos QUADRADOS** (as âncoras): a curva responde, os losangos se
//!    reacomodam, e o objeto passa a fazer o caminho novo — no MESMO compasso, porque
//!    puxar a curva é uma edição espacial. Um Ctrl+Z desfaz o arrasto inteiro.
//! 7. Com o objeto selecionado, **K** acrescenta uma âncora onde ele está.
//!
//! # `=2` — o auto-orient, e a recusa
//!
//! 1. A seta LARANJA tem auto-orient ligado: ela **encara para onde vai**, e vira ao
//!    longo da curva sem nenhuma key de rotação.
//! 2. A seta AZUL tem a mesma trajetória, o mesmo pedido de auto-orient — e uma track
//!    de **Rotation**. Ela NÃO gira com o caminho: o auto-orient está **recusado**,
//!    porque dois autores do mesmo ângulo é o que ninguém quer descobrir por acidente.
//!    ⚠️ Apague a track de Rotation dela (botão direito na label da row → *Delete
//!    Track*) e ela passa a girar: o pedido sobreviveu à recusa.
//! 3. O botão direito na label de uma track de **trajetória** tem uma linha que as
//!    outras não têm — *Auto-Orient*. Desligue-a na laranja e ela para de virar.
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
        if std::env::var_os("PH2D_PATH_SMOKE").is_some_and(|v| v == "2") {
            self.path_scene_orient();
            return;
        }

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

    /// Cena `=2`: **o auto-orient, e a recusa ao lado dele.**
    ///
    /// Duas setas na MESMA trajetória, com o MESMO pedido — e uma delas tem uma track
    /// de Rotation. Sem o par, "recusado" seria uma palavra num doc; com ele, é a
    /// diferença entre duas coisas na tela.
    fn path_scene_orient(&mut self) {
        let path = demo_path();
        let mut spawn = |y: f32, tint: [f32; 4], name: &str| -> u64 {
            let gfx = self.gfx.as_mut().expect("gfx");
            gfx.sim
                .world_mut()
                .spawn((
                    Transform::from_translation(Vec2::new(-6.0, -2.0 + y)),
                    // Fina e comprida: um quadrado girando é indistinguível de um
                    // quadrado parado, e o que esta cena mostra é EXATAMENTE o giro.
                    Sprite::atlas(0, [1.4, 0.35], tint),
                    Name::new(name),
                ))
                .id()
                .to_bits()
        };
        let follower = spawn(0.0, [1.0, 0.55, 0.15, 1.0], "Follower");
        let blocked = spawn(5.0, [0.35, 0.55, 1.0, 1.0], "Blocked");

        let doc = &mut self.timeline.doc;
        for bits in [follower, blocked] {
            author(doc, bits, &path);
            doc.set_auto_orient(bits, true);
        }
        // A track que RECUSA — um único key, que já basta: o conflito é sobre quem
        // possui o campo, não sobre quanto ele se move.
        doc.insert_key(
            blocked,
            PropKind::Rotation,
            RationalTime::from_seconds(0.0),
            AnimValue::Float(0.0),
            Interp::Hold,
        );

        let (a, b) = (doc.auto_orient(follower), doc.auto_orient(blocked));
        if let Some(hero) = self.gfx.as_mut().and_then(|g| g.hero_screen.as_mut()) {
            hero.gizmo.replace_selection(Some(follower));
        }
        eprintln!("[path-smoke] laranja {a:?} | azul {b:?}");
        eprintln!(
            "[path-smoke] Play: a LARANJA encara para onde vai; a AZUL nao gira \
             (tem track de Rotation, e o auto-orient dela esta RECUSADO)."
        );
    }
}

#[cfg(test)]
#[path = "motion_path_smoke_tests.rs"]
mod tests;
