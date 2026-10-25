//! **Pixel Art — o documento e as leis** (ADR-0192, `docs/PixelArt/02_plano_modulo.md`).
//!
//! Um [`Sprite`] é a grelha de células de um objecto Pixel Sprite: modo de cor (RGBA ou Indexed),
//! paleta, camadas, frames e cels esparsos. Toda mutação passa por [`apply`], que devolve o inverso
//! exato — é daí que nasce o desfazer ([`History`]). Os kernels são inteiros: não existe meio pixel.
//!
//! ⛔ Nada aqui lê relógio, aleatoriedade ou o SO; a iteração é em ordem definida (`BTreeMap`) e o
//! formato gravado ([`codec`]) é byte-determinístico.

pub mod brush;
pub mod codec;
pub mod color;
mod compose;
pub mod convert;
pub mod fill;
mod history;
pub mod ink;
mod op;
pub mod palette;
pub mod raster;
mod sprite;
mod stroke;
pub mod symmetry;

pub use codec::{CodecError, MAJOR, MINOR};
pub use compose::{Sample, compose_rgba, pick};
pub use history::{DEFAULT_CAP_BYTES, History};
pub use op::{Op, OpError, Patch, Whole, apply};
pub use sprite::{
    Background, Cel, CelKey, ColorMode, Frame, FrameId, Layer, LayerId, MAX_SIDE, NewSprite, Rect,
    Sprite, SpriteError,
};
pub use stroke::{Paint, Stroke};
