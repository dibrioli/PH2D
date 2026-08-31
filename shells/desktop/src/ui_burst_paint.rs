//! **DESENHAR a poeira de impacto** — a metade do chrome do [`ph2d_editor::motion_burst`].
//!
//! ⚠️ **Separada da lei de propósito**: a lei é aritmética pura e testa-se sem arnês nenhum; isto
//! toca numa `VectorScene` e não é alcançável de um teste. *Misturá-las poria a lei fora do alcance
//! dos gates*, que é o corte que o `ui_sound` e o `motion` desta casa já fazem.
//!
//! ⛔ **Zero trabalho quando não há faísca** — o caminho comum sai na primeira linha, e é por isso
//! que este módulo pode viver no fim de todo quadro.

use ph2d_editor::motion_burst::BurstField;
use ph2d_vector::{Affine, Brush, Circle, Color as VelloColor, Point, Shape as _, VectorScene};

/// O raio de uma partícula, em pixels de ecrã.
///
/// ⚠️ **Pequeno e FIXO**: a poeira confirma um ponto, não desenha uma forma. Um raio que crescesse
/// com a idade faria a faísca competir com o desenho do artista, que é o que o chrome existe para
/// não fazer.
const RAIO: f64 = 1.6;

/// Pinta as faíscas vivas. No-op quando não há nenhuma.
pub(crate) fn paint(campo: &BurstField, scene: &mut VectorScene) {
    if campo.is_empty() {
        return;
    }
    for b in campo.live() {
        for i in 0..ph2d_editor::motion_burst::SPARKS {
            let Some((p, alfa)) = b.spark(i) else {
                continue;
            };
            // ⚠️ **A cor é branca com a opacidade da lei**, e não um token de tema: uma faísca é
            // luz sobre o que estiver por baixo, e um token de superfície desapareceria sobre um
            // canvas claro. É a mesma razão pela qual o realce de proveniência não usa `Surface`.
            #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
            let a = (alfa * 255.0).clamp(0.0, 255.0) as u8;
            let brush = Brush::Solid(VelloColor::from_rgba8(255, 255, 255, a));
            let c = Circle::new(Point::new(f64::from(p[0]), f64::from(p[1])), RAIO);
            scene.fill_path(&c.to_path(0.1), &brush, Affine::IDENTITY);
        }
    }
}
