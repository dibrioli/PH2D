//! **A linha do tempo**: o vocabulário do chrome ↔ o do núcleo, o instantâneo que o painel pinta e
//! os passos de cel/tag que não vêem o estado da família (saiu de `ph2d-app-pixel::quadros`, P14).

use ph2d_editor_core::pixel_panel::{
    PixelCel, PixelDirection, PixelFrameCol, PixelSpeed, PixelTagInfo,
};
use ph2d_pixel_core::frames;
use ph2d_pixel_core::play::PlaySpeed;
use ph2d_pixel_core::{CelKey, Direction, FrameId, LayerId, Op, Sprite};

/// A direcção do chrome no vocabulário do núcleo.
#[must_use]
pub fn dir_in(d: PixelDirection) -> Direction {
    match d {
        PixelDirection::Forward => Direction::Forward,
        PixelDirection::Reverse => Direction::Reverse,
        PixelDirection::PingPong => Direction::PingPong,
        PixelDirection::PingPongReverse => Direction::PingPongReverse,
    }
}

/// A direcção do núcleo no vocabulário do chrome.
#[must_use]
pub fn dir_out(d: Direction) -> PixelDirection {
    match d {
        Direction::Forward => PixelDirection::Forward,
        Direction::Reverse => PixelDirection::Reverse,
        Direction::PingPong => PixelDirection::PingPong,
        Direction::PingPongReverse => PixelDirection::PingPongReverse,
    }
}

/// O instantâneo da linha do tempo de um sprite: colunas, cels e tags.
#[must_use]
pub fn timeline(s: &Sprite) -> (Vec<PixelFrameCol>, Vec<PixelCel>, Vec<PixelTagInfo>) {
    let cols = s
        .frames()
        .iter()
        .map(|f| PixelFrameCol {
            id: f.id.0,
            duration_ms: f.duration_ms,
        })
        .collect();
    let cels = s
        .entries()
        .map(|(&k, e)| {
            let img = s.cel(k);
            PixelCel {
                layer: k.0.0,
                frame: k.1.0,
                owner: s.owner(k).1.0,
                opacity: img.map_or(255, |c| c.opacity),
                z: e.z,
                empty: img.is_none_or(|c| c.bbox.is_empty()),
            }
        })
        .collect();
    let tags = s
        .tags()
        .iter()
        .map(|t| PixelTagInfo {
            name: t.name.clone(),
            from: t.from,
            to: t.to,
            direction: dir_out(t.direction),
            repeat: t.repeat,
            color: t.color,
        })
        .collect();
    (cols, cels, tags)
}

/// Cel ▸ Link Cels sobre um intervalo: os cels de cada camada passam a um só ligado (um passo).
#[must_use]
pub fn link_range(s: &Sprite, keys: &[CelKey]) -> Option<Op> {
    let mut camadas: Vec<LayerId> = keys.iter().map(|k| k.0).collect();
    camadas.dedup();
    let ops: Vec<Op> = camadas
        .into_iter()
        .filter_map(|l| {
            let fs: Vec<FrameId> = keys.iter().filter(|k| k.0 == l).map(|k| k.1).collect();
            frames::link(s, l, &fs)
        })
        .collect();
    (!ops.is_empty()).then_some(Op::Batch(ops))
}

/// A velocidade do chrome no vocabulário do núcleo.
#[must_use]
pub fn velocidade(v: PixelSpeed) -> PlaySpeed {
    match v {
        PixelSpeed::Quarter => PlaySpeed::Quarter,
        PixelSpeed::Half => PlaySpeed::Half,
        PixelSpeed::One => PlaySpeed::One,
        PixelSpeed::OneAndHalf => PlaySpeed::OneAndHalf,
        PixelSpeed::Two => PlaySpeed::Two,
        PixelSpeed::Three => PlaySpeed::Three,
    }
}

/// O nome da tag do *Set Loop Section* (o do Aseprite: é um nome do documento, não da UI).
const LOOP_TAG: &str = "Loop";

/// A tag «Loop» passa a `a..=b` (Forward, para sempre); nasce se não existe (o Trial: chamar de
/// novo MOVE a mesma).
#[must_use]
pub fn loop_section(s: &Sprite, a: usize, b: usize) -> Option<Op> {
    let (a, b) = (u32::try_from(a).ok()?, u32::try_from(b).ok()?);
    match s.tags().iter().position(|t| t.name == LOOP_TAG) {
        Some(i) => frames::edit_tag(s, i, |t| {
            t.from = a;
            t.to = b;
        }),
        None => frames::new_tag(s, LOOP_TAG, a, b),
    }
}

/// **Cel ▸ Tween…** (P13 T): A e B = o primeiro e o último quadro do intervalo; sem intervalo, o
/// activo e o seguinte. O Ease do chrome vira a [`ph2d_pixel_tween::Facilidade`] (a curva entra
/// por aqui: a família não depende do `ph2d-ease`).
#[must_use]
pub fn tween(
    s: &mut Sprite,
    layer: LayerId,
    range: &[FrameId],
    active: FrameId,
    n: u32,
    ease: ph2d_editor_core::pixel_panel::PixelTweenEase,
    rotate: i32,
) -> Option<Op> {
    use ph2d_editor_core::pixel_panel::PixelTweenEase as E;
    use ph2d_pixel_tween::Facilidade as F;
    let idx = |f: &FrameId| s.frame_index(*f);
    let (a, b) = match (
        range.iter().min_by_key(|f| idx(f)),
        range.iter().max_by_key(|f| idx(f)),
    ) {
        (Some(&a), Some(&b)) if a != b => (a, b),
        _ => (active, s.frames().get(s.frame_index(active)? + 1)?.id),
    };
    let f = match ease {
        E::Linear => F::Linear,
        E::In => F::In,
        E::Out => F::Out,
        E::InOut => F::InOut,
    };
    ph2d_pixel_tween::tween(s, layer, a, b, n, &|t| f.em(t), f64::from(rotate))
}
