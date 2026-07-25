//! **O overlay dos helpers do Gap Closure** (doc `06 §8`): em modo Fill, cada vão que o
//! alcance atual fecha ganha um segmento no canvas — o artista VÊ o que o clique vai
//! selar, e ajusta o alcance com Ctrl+roda sem largar o mouse.
//!
//! **É chrome, não arte** (a regra de todo overlay do Flip — ver `flip_tween_overlay`):
//! a geometria sobe para px de TELA e o `stroke` desenha sob `Affine::IDENTITY`, porque
//! no Vello o transform do `stroke` MULTIPLICA a espessura. A cadeia arte→tela é a MESMA
//! que o render dobra (`câmera ∘ objeto ∘ pose_da_chave` — o `screen_affine` do tween).
//!
//! Quem computa os segmentos é o worker (`flip_gap_live.rs` — o custo foi MEDIDO em
//! 5-339 ms, ver `measure_closures.rs`); aqui só se projeta o que está instalado. É por
//! estarem em coords de ARTE que os helpers ficam colados no desenho durante zoom/pan
//! enquanto o resultado novo não chega.

use ph2d_flip::Pose;
use ph2d_flip_fill::Closure;
use ph2d_host::WindowSize;
use ph2d_render::Camera2d;
use ph2d_vec_scene::Xform;
use ph2d_vector::{Affine, BezPath, Brush, Circle, Color, Fill, Point, Stroke, VectorScene};

use crate::flip_tween_correct::screen_affine;

/// Espessura do segmento de fechamento, em px de tela.
const SEG_PX: f64 = 2.0; // LITERAL-PX-OK: chrome de overlay, espessura de tela
/// Raio do ponto em cada ponta do vão, em px de tela — as pontas são o FATO (pontos
/// reais do desenho); o segmento é a promessa entre elas.
const END_DOT_PX: f64 = 3.0; // LITERAL-PX-OK: chrome de overlay, raio de tela
/// Verde "vai fechar" — o matiz de ação positiva; nenhum outro overlay do Flip o usa
/// para geometria de canvas (o verde do tween é a COR DE LINHA da confiança de par, em
/// outra sessão).
const SEAL_RGBA: [f32; 4] = [0.35, 0.9, 0.45, 0.9]; // LITERAL-COLOR-OK: overlay de helper

/// Desenha os helpers instalados. `active` = a pergunta do modo, respondida pela MESMA
/// porta do tick ([`crate::flip_gap_live::wants_gap_helpers`]) — o caller a passa
/// resolvida para este módulo não re-derivar política.
pub(super) fn draw(
    active: bool,
    segments: &[Closure],
    l2w: &Xform,
    pose: Pose,
    camera: &Camera2d,
    window: WindowSize,
    vector_scene: &mut VectorScene,
) {
    if !active || segments.is_empty() {
        return;
    }
    let aff = screen_affine(l2w, pose, camera.world_to_screen_affine(window));
    let scene = vector_scene.inner_mut();
    for c in segments {
        let a = aff * Point::new(f64::from(c.a.x), f64::from(c.a.y));
        let b = aff * Point::new(f64::from(c.b.x), f64::from(c.b.y));
        let mut path = BezPath::new();
        path.move_to(a);
        path.line_to(b);
        scene.stroke(
            &Stroke::new(SEG_PX),
            Affine::IDENTITY,
            &Brush::Solid(Color::new(SEAL_RGBA)),
            None,
            &path,
        );
        for p in [a, b] {
            scene.fill(
                Fill::NonZero,
                Affine::IDENTITY,
                &Brush::Solid(Color::new(SEAL_RGBA)),
                None,
                &Circle::new(p, END_DOT_PX),
            );
        }
    }
}
