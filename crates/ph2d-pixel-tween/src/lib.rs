//! **Os quadros do meio** (P13 §4, bloco T): Timeline ▸ cels ▸ *Tween…* gera `n` quadros REAIS
//! entre o cel A e o cel B de uma camada — cada um com um cel próprio que se retoca (§11.1: nada de
//! movimento vivo).
//!
//! Leis:
//! - o quadro `k` (`1..=n`) está em `t = k/n` da curva ⇒ o ÚLTIMO gerado chega a B (posição e
//!   rotação inteiras); o cel B fica intacto, logo depois deles;
//! - posição = canto ↖ do bbox, `A + round((B − A)·e(t))` — célula INTEIRA, arredondada;
//! - a imagem é a do cel A; com rotação ≠ 0, girada `rot·e(t)` em torno do centro do bbox pelo
//!   RotSprite do núcleo ([`transform_image`] com [`Resample::RotSprite`], a lei do Sprite Size);
//! - cada quadro novo dura o que dura A (como Frame ▸ New Frame); as outras camadas ficam LIGADAS
//!   ao cel delas em A (Duplicate Linked Cels), por isso pintar um quadro gerado só muda o dele;
//! - tudo num [`Op::Batch`] ⇒ um passo de desfazer.

use ph2d_ease::{CUBIC, Shape, ease};
use ph2d_pixel_core::Op;
use ph2d_pixel_core::affine::{Affine, Linear, Resample, transform_image};
use ph2d_pixel_core::frames::{self, NewFrame};
use ph2d_pixel_core::transform::Image;
use ph2d_pixel_core::{Cel, FrameId, LayerId, Rect, Sprite, UserData, apply};

/// O teto de quadros gerados num gesto: o diálogo prende-se aqui.
pub const MAX_QUADROS: u32 = 256;

/// O Ease do diálogo: as direcções do `ph2d-ease` sobre a família cúbica.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Facilidade {
    #[default]
    Linear,
    In,
    Out,
    InOut,
}

impl Facilidade {
    pub const ALL: [Facilidade; 4] = [Self::Linear, Self::In, Self::Out, Self::InOut];

    /// `e(t)` para `t ∈ [0, 1]` (`e(0) = 0`, `e(1) = 1`).
    #[must_use]
    pub fn em(self, t: f32) -> f32 {
        match self {
            Self::Linear => t,
            Self::In => ease(CUBIC, 0, t, &Shape::DEFAULT),
            Self::Out => ease(CUBIC, 1, t, &Shape::DEFAULT),
            Self::InOut => ease(CUBIC, 2, t, &Shape::DEFAULT),
        }
    }
}

/// `a + round((b − a)·e)` — a célula inteira mais próxima.
fn passo(a: i32, b: i32, e: f32) -> i32 {
    a + ((f64::from(b) - f64::from(a)) * f64::from(e)).round() as i32
}

/// O cel da camada no quadro `k`: o de A deslocado (e girado) para `t = k/n`.
fn cel_do_meio(s: &Sprite, a: &Cel, b: Rect, z: i16, e: f32, rot: f64) -> Option<Cel> {
    let at = Rect::new(
        passo(a.bbox.x, b.x, e),
        passo(a.bbox.y, b.y, e),
        a.bbox.w,
        a.bbox.h,
    );
    let src = Image {
        w: a.bbox.w,
        h: a.bbox.h,
        bpp: s.mode().bpp(),
        data: a.data.clone(),
    };
    let graus = rot * f64::from(e);
    let (bbox, data) = if graus == 0.0 || a.bbox.w == 0 {
        (at, src.data)
    } else {
        let lin = Linear::from_screen_ccw(graus, 1.0, 1.0, 0.0);
        let t = Affine::around_center(lin, at);
        let (r, img) = transform_image(&src, &t, Resample::RotSprite, s.transparent_value(), None)?;
        (r, img.data)
    };
    Some(Cel {
        bbox,
        data,
        opacity: a.opacity,
        z,
        link: None,
        tiles: None,
        user: UserData::default(),
    })
}

/// **Tween…**: `n` quadros entre `frame_a` e `frame_b` na `camada`, com a curva `curva` (a porta:
/// `e(t)`, ver [`Facilidade::em`]) e a rotação total `rotacao_graus` (anti-horário, como o campo
/// do Aseprite). `None` se A não está antes de B, se falta um dos cels, se a camada é tilemap/grupo
/// ou `n ∉ 1..=MAX_QUADROS`.
pub fn tween(
    s: &mut Sprite,
    camada: LayerId,
    frame_a: FrameId,
    frame_b: FrameId,
    n: u32,
    curva: &dyn Fn(f32) -> f32,
    rotacao_graus: f64,
) -> Option<Op> {
    let (ia, ib) = (s.frame_index(frame_a)?, s.frame_index(frame_b)?);
    let l = s.layer(camada)?;
    if ia >= ib || !(1..=MAX_QUADROS).contains(&n) || l.is_group() || l.tileset().is_some() {
        return None;
    }
    let cel_a = s.cel((camada, frame_a))?.clone();
    let alvo = s.cel((camada, frame_b))?.bbox;
    let z = s.entry((camada, frame_a)).map_or(0, |c| c.z);
    let ms = s.frames()[ia].duration_ms;
    // Cada quadro novo é o Frame ▸ insert do núcleo (tags, slices, ligações) sobre um rascunho que
    // avança; o cel da camada troca-se pelo do meio. Os ids reservam-se em `s` na mesma ordem.
    let mut rascunho = s.clone();
    let mut ops = Vec::with_capacity(n as usize);
    for k in 1..=n {
        let (op, id) = frames::insert(
            &mut rascunho,
            ia + k as usize,
            frame_a,
            NewFrame::Linked,
            ms,
        )?;
        let Op::Frames(mut set) = op else { return None };
        let e = curva(k as f32 / n as f32);
        let cel = cel_do_meio(&rascunho, &cel_a, alvo, z, e, rotacao_graus)?;
        set.cels.retain(|(key, _)| key.0 != camada);
        set.cels.push(((camada, id), Some(cel)));
        let op = Op::Frames(set);
        apply(&mut rascunho, op.clone()).ok()?;
        ops.push(op);
    }
    for _ in 0..n {
        s.reserve_id();
    }
    Some(Op::Batch(ops))
}
