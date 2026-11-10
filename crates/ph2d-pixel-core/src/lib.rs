//! **Pixel Art — o documento e as leis** (ADR-0196, `docs/PixelArt/02_plano_modulo.md`).
//!
//! Um [`Sprite`] é a grelha de células de um objecto Pixel Sprite: modo de cor (RGBA ou Indexed),
//! paleta, camadas, frames e cels esparsos. Toda mutação passa por [`apply`], que devolve o inverso
//! exato — é daí que nasce o desfazer ([`History`]). Os kernels são inteiros: não existe meio pixel.
//!
//! ⛔ Nada aqui lê relógio, aleatoriedade ou o SO; a iteração é em ordem definida (`BTreeMap`) e o
//! formato gravado ([`codec`]) é byte-determinístico.

pub mod affine;
pub mod audio;
pub mod blend;
pub mod brush;
pub mod canvas;
pub mod codec;
pub mod color;
mod compose;
pub mod convert;
pub mod cycle;
pub mod dither;
pub mod dynamics;
pub mod edit;
pub mod efeito;
pub mod fill;
pub mod filter;
pub mod forma;
pub mod frames;
pub mod gradient;
pub mod guia;
mod history;
pub mod ink;
pub mod layers;
mod op;
pub mod palette;
pub mod pattern;
pub mod play;
pub mod prefs;
pub mod ramp;
pub mod raster;
mod rect;
pub mod selection;
pub mod slice;
mod sprite;
mod stroke;
pub mod symmetry;
pub mod tiles;
pub mod transform;
pub mod user;

pub use audio::FaixaDeAudio;
pub use codec::{CodecError, MAJOR, MINOR};
pub use compose::{
    Ghost, NORMAL_FLAT, Onion, Purpose, Sample, compose_normal, compose_picked, compose_region,
    compose_rgba, compose_window, compose_window_em, compose_with_onion, compose_with_onion_em,
    compose_with_onion_of, ghosts, onion_tint, pick,
};
pub use cycle::{Cycle, CycleDirection, ciclo_em, instante_do_quadro};
pub use efeito::{EfeitoDeCamada, OutlineParams, aplicar_efeitos, imagem_com_efeitos};
pub use forma::Forma;
pub use guia::{Guias, Iso2x1};
pub use history::{DEFAULT_CAP_BYTES, History, Label, PrettySize, StepState, Touched};
pub use op::{CanvasSet, DocOp, FrameSet, LayerSet, MaskSet, Op, OpError, Patch, Whole, apply};
pub use prefs::{Checker, Grid, PixelGrid, PixelPrefs};
pub use ramp::{PaletteRamps, Ramp, RampId, rampa_de};
pub use slice::{DEFAULT_SLICE_COLOR, Slice, SliceId, SliceKey, SliceShape};
pub use sprite::{
    Background, Cel, CelKey, ColorMode, DEFAULT_FRAME_MS, Direction, ExportPreset, Frame, FrameId,
    Layer, LayerId, LayerKind, MAX_SIDE, NewSprite, Rect, Sprite, SpriteError, Tag,
};
pub use stroke::{Paint, Stroke};
pub use tiles::{
    TERRAIN_TILES, Terrain, TileGrid, TileRef, TilesSet, Tileset, TilesetId, derived_is_consistent,
};
pub use user::{Owner, PropMap, PropValue, Properties, UserData};
