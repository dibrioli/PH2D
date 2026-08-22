//! **Os marcadores das âncoras no canvas** (spec
//! [`07_named_anchors.md`](../../../../docs/Sprite_projeto/07_named_anchors.md) §7.6).
//!
//! # Por que isto não é opcional
//!
//! ⚠️ Sem marcador, a §12 é um formulário: o artista escreve `pos = (28, -4)` e **não acontece
//! nada na tela**. Uma posição que não se vê não é uma posição — é a mesma lição que o realce de
//! seleção do Flip pagou («uma seleção que não se VÊ não existe»). É o marcador que fecha o
//! ciclo autoria → efeito, e sem ele a seção seria entregue morta.
//!
//! # A linguagem, derivada da FORMA
//!
//! - **Socket** (sem área) → uma **cruz**.
//! - **Slice** (com área) → a cruz mais o **retângulo**.
//! - **Região 9-slice** (área + miolo) → mais o **retângulo interno**.
//!
//! A cor vem do **hash do nome**, por isso é estável entre sessões e distinta entre âncoras: duas
//! âncoras coincidentes continuam a distinguir-se.
//!
//! ⚠️ **A espessura sai daqui em px de TELA, sob `Affine::IDENTITY`.** No Vello o transform do
//! `stroke` **multiplica** a espessura: entregar o afim mundo→tela como transform transforma
//! 2 px em `2 × px_por_unidade_de_mundo`. É o defeito que o realce do Flip apanhou num smoke em
//! 2026-07-13, e a razão de os pontos serem transformados e o traço não.
//!
//! # A decisão que a spec não fixa
//!
//! ⚠️ A spec diz que `bounds` é `[x, y, w, h]` e não diz **em relação a quê**. Aqui é **relativo
//! à própria âncora**, em pixels da fonte, com **+Y para cima** (a convenção do mundo, a mesma
//! do `QUAD_STRIP`). Motivo: uma âncora com área é «um socket que também é uma caixa» — a
//! hitbox da mão anda com a mão. Absoluto na imagem (a leitura do Aseprite) faria mover a âncora
//! deixar a caixa para trás.

use ph2d_ecs::{Entity, NamedAnchorList, World};
use ph2d_host::WindowSize;
use ph2d_render::Camera2d;
use ph2d_vector::{Affine, BezPath, Brush, Color, Point, Stroke, VectorScene};

/// Espessura do traço do marcador, em px de tela.
const MARK_PX: f64 = 1.5; // LITERAL-PX-OK: chrome de overlay, espessura de tela
/// Meio-braço da cruz de um socket, em px de tela.
const CROSS_PX: f64 = 7.0; // LITERAL-PX-OK: chrome de overlay, tamanho de tela

/// Cor estável a partir do nome — FNV-1a sobre os bytes, depois um passeio pelo círculo de
/// matiz. ⚠️ Determinística: a mesma âncora tem a mesma cor em todas as sessões e máquinas.
fn color_of(name: &str) -> [f32; 4] {
    let mut h: u32 = 0x811c_9dc5;
    for b in name.as_bytes() {
        h ^= u32::from(*b);
        h = h.wrapping_mul(0x0100_0193);
    }
    // Matiz em 6 setores, saturação e valor fixos e altos: chrome tem de ler sobre arte clara e
    // escura.
    let hue = (h % 360) as f32 / 60.0;
    let i = hue as u32 % 6;
    let f = hue - hue.floor();
    let (q, t) = (1.0 - f, f);
    let (r, g, b) = match i {
        0 => (1.0, t, 0.0),
        1 => (q, 1.0, 0.0),
        2 => (0.0, 1.0, t),
        3 => (0.0, q, 1.0),
        4 => (t, 0.0, 1.0),
        _ => (1.0, 0.0, q),
    };
    [r, g, b, 0.95] // LITERAL-COLOR-OK: chrome de overlay, opacidade de marcador
}

/// Desenha os marcadores das âncoras da entidade selecionada.
///
/// `expanded` é a seção §12 estar aberta — a spec §7.6 pede exatamente isso: os handles aparecem
/// **quando a seção está expandida**, senão todo sprite com âncoras ficaria coberto de cruzes.
#[allow(clippy::too_many_arguments)]
pub(super) fn draw_anchor_marks(
    expanded: bool,
    sim: &World,
    selected: Option<u64>,
    pixels_per_meter: f32,
    camera: &Camera2d,
    window: WindowSize,
    vector_scene: &mut VectorScene,
) {
    if !expanded {
        return;
    }
    let Some(bits) = selected else {
        return;
    };
    let entity = Entity::from_bits(bits);
    let Some(list) = sim.get::<NamedAnchorList>(entity) else {
        return;
    };
    if list.is_empty() {
        return;
    }
    let origin = sim
        .get::<ph2d_ecs::GlobalTransform>(entity)
        .map_or(ph2d_core::Vec2::ZERO, |g| g.translation());
    let ppm = pixels_per_meter.max(crate::EPS_PIXELS_PER_METER);
    let to_screen = camera.world_to_screen_affine(window);

    for a in list.iter() {
        let rgba = color_of(&a.name);
        let brush = Brush::Solid(Color::new(rgba));
        // O centro da âncora, em MUNDO, depois em tela.
        let world = origin + a.transform.translation;
        let c = to_screen * Point::new(f64::from(world.x), f64::from(world.y));

        // A cruz: sempre, para toda âncora. É o «onde» — e o retângulo, quando existe, é o
        // «quanto».
        let mut cross = BezPath::new();
        cross.move_to(Point::new(c.x - CROSS_PX, c.y));
        cross.line_to(Point::new(c.x + CROSS_PX, c.y));
        cross.move_to(Point::new(c.x, c.y - CROSS_PX));
        cross.line_to(Point::new(c.x, c.y + CROSS_PX));
        vector_scene.inner_mut().stroke(
            &Stroke::new(MARK_PX),
            Affine::IDENTITY,
            &brush,
            None,
            &cross,
        );

        // A área e o miolo, em px da fonte relativos à âncora. ⚠️ **+Y para cima**: o `h` de um
        // rect cresce para cima, como no mundo.
        for (rect, dash) in [(a.bounds, false), (a.center, true)] {
            let Some([rx, ry, rw, rh]) = rect else {
                continue;
            };
            if rw <= 0.0 || rh <= 0.0 {
                continue;
            }
            let p = |px: f32, py: f32| {
                let w = world + ph2d_core::Vec2::new(px / ppm, py / ppm);
                to_screen * Point::new(f64::from(w.x), f64::from(w.y))
            };
            let (a0, a1, a2, a3) = (
                p(rx, ry),
                p(rx + rw, ry),
                p(rx + rw, ry + rh),
                p(rx, ry + rh),
            );
            let mut path = BezPath::new();
            path.move_to(a0);
            path.line_to(a1);
            path.line_to(a2);
            path.line_to(a3);
            path.close_path();
            // O miolo desenha-se mais fino: ele é uma subdivisão da área, não outra área.
            let width = if dash { MARK_PX * 0.6 } else { MARK_PX };
            vector_scene.inner_mut().stroke(
                &Stroke::new(width),
                Affine::IDENTITY,
                &brush,
                None,
                &path,
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A cor é **estável** e **distinta** — é o que permite distinguir duas âncoras
    /// sobrepostas, e o que faz o mesmo socket ter a mesma cor amanhã.
    #[test]
    fn the_colour_is_stable_and_distinguishes_names() {
        assert_eq!(
            color_of("muzzle"),
            color_of("muzzle"),
            "a cor mudou sozinha"
        );
        assert_ne!(
            color_of("muzzle"),
            color_of("face_box"),
            "dois nomes com a mesma cor: duas ancoras sobrepostas ficam indistinguiveis"
        );
        // Opaca o suficiente para ler sobre arte clara e escura.
        for name in ["a", "b", "left_hand", "anchor_63"] {
            let c = color_of(name);
            assert!(c[3] > 0.9, "'{name}' saiu translucido demais para chrome");
            assert!(
                c[0] + c[1] + c[2] > 0.5,
                "'{name}' saiu quase preto — invisivel sobre arte escura"
            );
        }
    }
}
