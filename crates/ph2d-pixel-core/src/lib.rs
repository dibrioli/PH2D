//! **Pixel Art — o documento e as leis** (ADR-0192, `docs/PixelArt/02_plano_modulo.md`).
//!
//! Um [`Sprite`] é a grelha de células de um objecto Pixel Sprite: modo de cor (RGBA ou Indexed),
//! paleta, camadas, frames e cels esparsos. Toda mutação passa por [`apply`], que devolve o inverso
//! exato — é daí que nasce o desfazer ([`History`]). Os kernels são inteiros: não existe meio pixel.
//!
//! ⛔ Nada aqui lê relógio, aleatoriedade ou o SO; a iteração é em ordem definida (`BTreeMap`) e o
//! formato gravado ([`codec`]) é byte-determinístico.

pub mod affine;
pub mod blend;
pub mod brush;
pub mod canvas;
pub mod codec;
pub mod color;
mod compose;
pub mod convert;
pub mod edit;
pub mod fill;
pub mod frames;
mod history;
pub mod ink;
pub mod layers;
mod op;
pub mod palette;
pub mod play;
pub mod raster;
pub mod selection;
mod sprite;
mod stroke;
pub mod symmetry;
pub mod transform;

pub use codec::{CodecError, MAJOR, MINOR};
pub use compose::{
    Ghost, Onion, Purpose, Sample, compose_region, compose_rgba, compose_window,
    compose_with_onion, ghosts, onion_tint, pick,
};
pub use history::{DEFAULT_CAP_BYTES, History};
pub use op::{CanvasSet, FrameSet, LayerSet, MaskSet, Op, OpError, Patch, Whole, apply};
pub use sprite::{
    Background, Cel, CelKey, ColorMode, Direction, Frame, FrameId, Layer, LayerId, LayerKind,
    MAX_SIDE, NewSprite, Rect, Sprite, SpriteError, Tag,
};
pub use stroke::{Paint, Stroke};
