//! **O documento do Quadro** (MiroClone) — os quadros de um projecto e o que há dentro deles.
//!
//! Um quadro é um DOCUMENTO numa aba da barra superior, nunca um objecto da cena (ordem do dono,
//! 2026-10-05; plano em `docs/MiroClone/02_plano.md` §1). Por isso este crate não conhece ECS,
//! render nem UI: o editor muda-o só por [`BoardOp`] e grava-o só por [`BoardSet::to_bytes`].
//!
//! Cada [`Element`] já nasce com o que a colaboração ao vivo (Etapa 2) precisa — `version`,
//! `nonce`, lápide `deleted` e ordem por [`FracKey`] —, para essa etapa não pedir migração.

mod board;
mod camera;
mod connector;
mod element;
mod frac;
mod history;
mod ink;
mod legacy;
mod ops;
mod rich;

pub use board::{Board, BoardId, BoardSet, Camera, FORMAT_VERSION};
pub use camera::{Area, ZOOM_RANGE};
pub use connector::{Anchor, Connector, End, Head, Route};
pub use element::{
    BoardDoc, Dash, Element, ElementId, ElementKind, Rgba, Shape, ShapeType, Style, rotate_about,
};
pub use frac::FracKey;
pub use history::{History, MAX_STEPS};
pub use ink::{Ink, Pen};
pub use ops::{BoardOp, apply_batch};
pub use rich::{Mark, Marks, RichText, Span};

/// Espessura do contorno de uma forma nova, em unidades do mundo: a FINA do Excalidraw
/// (`STROKE_WIDTH.thin = 1`) — ordem do dono (07/10, captura `capturas_excalidraw/
/// formas_finas_do_dono.png`: *«a espessura padrão deve ser a mais fina»*). A de nascença do
/// Excalidraw é a `bold` (2), medida no oráculo na W1.
pub const DEFAULT_STROKE_WIDTH: f64 = 1.0;
/// Tamanho do texto de uma forma nova, em unidades do mundo — o «M» do Excalidraw (oráculo:
/// `fontSize: 20`).
pub const DEFAULT_FONT_SIZE: f64 = 20.0;
/// Tinta do texto quando o documento não diz outra (dado do documento, não da UI).
pub const DEFAULT_INK: [u8; 4] = [30, 30, 30, 255];
/// A tinta clara (texto sobre preenchimentos escuros).
pub const DEFAULT_PAPER: [u8; 4] = [250, 250, 250, 255];

/// ⭐ **As 16 cores das notas** — as do MIRO, pela ordem da enumeração `fillColor` da documentação
/// dele (`docs/MiroClone/ferramentas/miro_api_notas.txt`): gray, light_yellow, yellow, orange,
/// light_green, green, dark_green, cyan, light_pink, pink, violet, red, light_blue, blue, dark_blue,
/// black. São DADO do documento (a cor de cada nota grava-se), não da interface.
pub const STICKY_COLORS: [[u8; 4]; 16] = [
    // LITERAL-COLOR-OK: paleta de documento medida na documentação do Miro (ver acima).
    [0xf5, 0xf6, 0xf8, 0xff],
    [0xff, 0xf9, 0xb1, 0xff],
    [0xf5, 0xd1, 0x28, 0xff],
    [0xff, 0x9d, 0x48, 0xff],
    [0xd5, 0xf6, 0x92, 0xff],
    [0xc9, 0xdf, 0x56, 0xff],
    [0x93, 0xd2, 0x75, 0xff],
    [0x67, 0xc6, 0xc0, 0xff],
    [0xff, 0xce, 0xe0, 0xff],
    [0xea, 0x94, 0xbb, 0xff],
    [0xc6, 0xa2, 0xd2, 0xff],
    [0xf0, 0x93, 0x9d, 0xff],
    [0xa6, 0xcc, 0xf5, 0xff],
    [0x6c, 0xd8, 0xfa, 0xff],
    [0x9e, 0xa9, 0xff, 0xff],
    [0x00, 0x00, 0x00, 0xff],
];
/// A cor de nascença de uma nota: `light_yellow` (a «Default» da documentação do Miro).
pub const STICKY_DEFAULT_COLOR: usize = 1;
/// O lado da nota QUADRADA de nascença, em unidades do mundo: os `199 dp` de largura do Miro (a
/// altura que ele dá, `228`, inclui a sombra).
pub const STICKY_SIDE: f64 = 199.0;
/// A largura da nota LARGA (`rectangle`) do Miro, mesma altura que a quadrada.
pub const STICKY_WIDE: f64 = 350.0;
