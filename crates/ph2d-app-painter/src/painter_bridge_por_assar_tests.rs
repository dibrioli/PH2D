//! `BUGS_painter_fechados_41_a_46.md` #53: o que um verbo da shell leva de uma sprite que o Painter
//! pinta sem a ter assado — a tinta ([`PorAssar`]) a quem só LÊ, o assar primeiro
//! ([`a_assar_antes`]) a quem ESCREVE. As costuras de cada verbo: `tests/it/a_sprite_verb_asks_the_painter_first.rs`.

use crate::painter_bridge_queries::{PorAssar, a_assar_antes, por_assar_do};
use ph2d_editor_core::tool::{CanvasPaintTool, CanvasPointer, PointerPhase, Tool};
use ph2d_render::{AlphaMode, SpriteImage};
use ph2d_tool_painter::{PaintMedia, PainterTool};

const L: u32 = 64;
const E: u64 = 7;

/// Tela branca, Digital, um traço amarelo; outro ainda NO AR. Nada assado.
fn pintado() -> PainterTool {
    let mut t = PainterTool::default();
    t.bind_document(E, vec![255; (L * L * 4) as usize], L, L);
    t.set_paint_media(PaintMedia::Digital);
    t.set_brush_color_srgb8([230, 190, 40]); // LITERAL-COLOR-OK: a cor do artista, não da UI
    for (y, levanta) in [(32.0, true), (16.0, false)] {
        for i in 0..12 {
            #[expect(clippy::cast_precision_loss, reason = "poucos passos")]
            let x = 8.0 + 48.0 * i as f32 / 11.0;
            let phase = match i {
                0 => PointerPhase::Down,
                11 if levanta => PointerPhase::Up,
                _ => PointerPhase::Move,
            };
            t.on_canvas_pointer(CanvasPointer {
                pos: [x, y],
                pressure: 1.0,
                tilt: [0.0, 0.0],
                phase,
            });
            t.on_tick(16.667);
        }
    }
    t
}

fn branca() -> Option<SpriteImage> {
    Some(SpriteImage::from_bytes(
        L,
        L,
        vec![255; (L * L * 4) as usize],
        AlphaMode::Straight,
    ))
}

/// ⭐ Quem só LÊ a sprite pintada leva a tinta (a mesma imagem que o Apply escreveria, com o traço no
/// ar), em alfa reto; outra sprite, ou nada por assar, lê a sprite.
#[test]
fn quem_le_a_sprite_pintada_leva_a_tinta_por_assar() {
    let mut t = pintado();
    let (aplicada, w, h) = t.imagem_aplicada().expect("documento");
    let por = por_assar_do(&t);
    let img = por.ou(E, branca).expect("pixels");
    assert_eq!(
        (img.width, img.height, img.alpha),
        (w, h, AlphaMode::Straight)
    );
    assert_eq!(img.pixels, aplicada, "a tinta, ao byte");
    assert_ne!(
        img.pixels,
        branca().unwrap().pixels,
        "controlo: a tela foi pintada"
    );
    let outra = por.ou(E + 1, branca).expect("pixels");
    assert_eq!(
        outra.pixels,
        branca().unwrap().pixels,
        "outra sprite lê-se a si"
    );
    assert!(
        PorAssar::default().ou(E, || None).is_none(),
        "vazio = a sprite"
    );
    t.mark_baked();
    let assada = por_assar_do(&t).ou(E, branca).expect("pixels");
    assert_eq!(
        assada.pixels,
        branca().unwrap().pixels,
        "assado: a sprite já é a verdade"
    );
}

/// ⭐ Quem ESCREVE a sprite pintada assa-a antes — só ela, e só com tinta por assar.
#[test]
fn quem_escreve_a_sprite_pintada_assa_a_antes() {
    let mut t = pintado();
    assert_eq!(a_assar_antes(&t, |e| e == E), Some(E));
    assert_eq!(
        a_assar_antes(&t, |e| e == E + 1),
        None,
        "o verbo é sobre outra"
    );
    t.mark_baked();
    assert_eq!(a_assar_antes(&t, |_| true), None, "nada por assar");
    assert_eq!(
        a_assar_antes(&PainterTool::default(), |_| true),
        None,
        "sem documento"
    );
}
