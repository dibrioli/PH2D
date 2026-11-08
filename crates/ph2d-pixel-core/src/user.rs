//! **User data** (P12, `docs/PixelArt/P12_plano.md` §9.5): cor + texto + a tabela `name = value`.
//!
//! O Aseprite guarda o mesmo no chunk `0x2020` de cada dono (O17); as propriedades tipadas, que lá só
//! se escrevem por script, aqui editam-se na tela. Os mapas são por extensão (`""` = o do artista),
//! como no ficheiro, para que um `.ase` vá e volte sem perder nada.

use std::collections::BTreeMap;

use crate::sprite::{CelKey, LayerId};
use crate::tiles::TilesetId;

/// Um mapa `nome → valor`, em ordem de nome.
pub type PropMap = BTreeMap<String, PropValue>;

/// Um valor, com um tipo por tipo do `.ase` (`0x0001..=0x0013`).
#[derive(Debug, Clone)]
pub enum PropValue {
    Bool(bool),
    I8(i8),
    U8(u8),
    I16(i16),
    U16(u16),
    I32(i32),
    U32(u32),
    I64(i64),
    U64(u64),
    /// Ponto fixo 16.16, cru.
    Fixed(i32),
    F32(f32),
    F64(f64),
    Str(String),
    Point(i32, i32),
    Size(i32, i32),
    Rect(i32, i32, i32, i32),
    Vector(Vec<PropValue>),
    Map(PropMap),
    Uuid([u8; 16]),
}

/// Igualdade BIT a bit nos reais: o inverso de um [`crate::Op::UserData`] tem de devolver o mesmo.
impl PartialEq for PropValue {
    fn eq(&self, o: &Self) -> bool {
        use PropValue as P;
        match (self, o) {
            (P::F32(a), P::F32(b)) => a.to_bits() == b.to_bits(),
            (P::F64(a), P::F64(b)) => a.to_bits() == b.to_bits(),
            (P::Bool(a), P::Bool(b)) => a == b,
            (P::I8(a), P::I8(b)) => a == b,
            (P::U8(a), P::U8(b)) => a == b,
            (P::I16(a), P::I16(b)) => a == b,
            (P::U16(a), P::U16(b)) => a == b,
            (P::I32(a), P::I32(b)) | (P::Fixed(a), P::Fixed(b)) => a == b,
            (P::U32(a), P::U32(b)) => a == b,
            (P::I64(a), P::I64(b)) => a == b,
            (P::U64(a), P::U64(b)) => a == b,
            (P::Str(a), P::Str(b)) => a == b,
            (P::Point(a, b), P::Point(c, d)) | (P::Size(a, b), P::Size(c, d)) => (a, b) == (c, d),
            (P::Rect(a, b, c, d), P::Rect(e, f, g, h)) => (a, b, c, d) == (e, f, g, h),
            (P::Vector(a), P::Vector(b)) => a == b,
            (P::Map(a), P::Map(b)) => a == b,
            (P::Uuid(a), P::Uuid(b)) => a == b,
            _ => false,
        }
    }
}

impl Eq for PropValue {}

impl PropValue {
    /// O código do tipo no `.ase` (e no nosso formato).
    #[must_use]
    pub fn code(&self) -> u8 {
        match self {
            Self::Bool(_) => 1,
            Self::I8(_) => 2,
            Self::U8(_) => 3,
            Self::I16(_) => 4,
            Self::U16(_) => 5,
            Self::I32(_) => 6,
            Self::U32(_) => 7,
            Self::I64(_) => 8,
            Self::U64(_) => 9,
            Self::Fixed(_) => 10,
            Self::F32(_) => 11,
            Self::F64(_) => 12,
            Self::Str(_) => 13,
            Self::Point(..) => 14,
            Self::Size(..) => 15,
            Self::Rect(..) => 16,
            Self::Vector(_) => 17,
            Self::Map(_) => 18,
            Self::Uuid(_) => 19,
        }
    }
}

/// Os mapas de propriedades por extensão; a chave `""` é o mapa do artista.
pub type Properties = BTreeMap<String, PropMap>;

/// O user data de um dono. Por omissão: cor transparente (O17, `#00000000`), sem texto, sem mapas.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct UserData {
    pub color: [u8; 4],
    pub text: String,
    pub props: Properties,
}

impl UserData {
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.color == [0; 4] && self.text.is_empty() && self.props.is_empty()
    }
}

/// O dono de um user data. A tag é pela POSIÇÃO na lista (as tags não têm id).
///
/// A cor de uma tag é `Tag::color` (no Aseprite 1.3 a tag tem UMA cor, a do user data: o Trial
/// mostra-a por cima da `rgb` do chunk); a `UserData::color` de uma tag não vai ao `.ase`.
///
/// A slice também passa por `Op::Slices` (nome, cor e texto no cartão da slice); as duas portas
/// escrevem o mesmo `Slice::user`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Owner {
    Sprite,
    Layer(LayerId),
    Cel(CelKey),
    Tag(usize),
    Tileset(TilesetId),
    Slice(crate::slice::SliceId),
}
