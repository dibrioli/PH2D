//! O chunk User Data (`0x2020`) e o External Files (`0x2008`), pela spec pública do formato.
//!
//! ```text
//! UserData : DWORD flags (1 texto · 2 cor · 4 propriedades)
//!            [STRING texto] [BYTE r g b a]
//!            [DWORD tamanho (conta este campo e o seguinte) · DWORD n.º de mapas
//!             · { DWORD chave (0 = do utilizador; senão id de uma extensão no 0x2008)
//!                 · DWORD n · { STRING nome · WORD tipo · valor }* }* ]
//! ```

use crate::read::Reader;

/// O tipo `0x0011` (vector) com elementos de tipos diferentes grava `0` como tipo comum.
const VECTOR_MIXED: u16 = 0;

/// Um valor de propriedade, com o tipo do ficheiro (um por código da spec, `0x0001..=0x0013`).
#[derive(Clone, Debug, PartialEq)]
pub enum AseProp {
    Bool(bool),
    I8(i8),
    U8(u8),
    I16(i16),
    U16(u16),
    I32(i32),
    U32(u32),
    I64(i64),
    U64(u64),
    /// 16.16.
    Fixed(i32),
    F32(f32),
    F64(f64),
    Str(String),
    Point(i32, i32),
    Size(i32, i32),
    Rect(i32, i32, i32, i32),
    Vector(Vec<AseProp>),
    Map(Vec<(String, AseProp)>),
    Uuid([u8; 16]),
}

impl Eq for AseProp {}

impl AseProp {
    #[must_use]
    pub fn code(&self) -> u16 {
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

/// Um mapa de propriedades: `key` 0 = do utilizador; senão o id de uma entrada do `0x2008`.
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct AsePropMap {
    pub key: u32,
    pub props: Vec<(String, AseProp)>,
}

/// O user data de um dono (sprite, camada, cel, tag, tileset, slice).
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct AseUserData {
    pub text: Option<String>,
    pub color: Option<[u8; 4]>,
    pub maps: Vec<AsePropMap>,
}

impl AseUserData {
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.text.is_none() && self.color.is_none() && self.maps.is_empty()
    }
}

/// Uma entrada do External Files (`0x2008`); `kind` 3 = extensão (as chaves dos mapas).
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct AseExternalFile {
    pub id: u32,
    pub kind: u8,
    pub name: String,
}

fn u64_of(r: &mut Reader<'_>) -> Option<u64> {
    let lo = u64::from(r.u32()?);
    let hi = u64::from(r.u32()?);
    Some(lo | hi << 32)
}

fn i32_of(r: &mut Reader<'_>) -> Option<i32> {
    Some(r.u32()? as i32)
}

fn value(r: &mut Reader<'_>, code: u16, depth: u8) -> Option<AseProp> {
    // Um ficheiro hostil aninha mapas sem fim; 64 níveis cabem em qualquer uso real.
    if depth > 64 {
        return None;
    }
    Some(match code {
        1 => AseProp::Bool(r.u8()? != 0),
        2 => AseProp::I8(r.u8()? as i8),
        3 => AseProp::U8(r.u8()?),
        4 => AseProp::I16(r.i16()?),
        5 => AseProp::U16(r.u16()?),
        6 => AseProp::I32(i32_of(r)?),
        7 => AseProp::U32(r.u32()?),
        8 => AseProp::I64(u64_of(r)? as i64),
        9 => AseProp::U64(u64_of(r)?),
        10 => AseProp::Fixed(i32_of(r)?),
        11 => AseProp::F32(f32::from_bits(r.u32()?)),
        12 => AseProp::F64(f64::from_bits(u64_of(r)?)),
        13 => AseProp::Str(r.string()?),
        14 => AseProp::Point(i32_of(r)?, i32_of(r)?),
        15 => AseProp::Size(i32_of(r)?, i32_of(r)?),
        16 => AseProp::Rect(i32_of(r)?, i32_of(r)?, i32_of(r)?, i32_of(r)?),
        17 => {
            let n = r.u32()?;
            let common = r.u16()?;
            let mut v = Vec::new();
            for _ in 0..n {
                let c = if common == VECTOR_MIXED {
                    r.u16()?
                } else {
                    common
                };
                v.push(value(r, c, depth + 1)?);
            }
            AseProp::Vector(v)
        }
        18 => AseProp::Map(props(r, depth + 1)?),
        19 => {
            let mut u = [0; 16];
            for b in &mut u {
                *b = r.u8()?;
            }
            AseProp::Uuid(u)
        }
        _ => return None,
    })
}

fn props(r: &mut Reader<'_>, depth: u8) -> Option<Vec<(String, AseProp)>> {
    let n = r.u32()?;
    let mut out = Vec::new();
    for _ in 0..n {
        let name = r.string()?;
        let code = r.u16()?;
        out.push((name, value(r, code, depth)?));
    }
    Some(out)
}

/// Lê o corpo de um `0x2020`. O que vier partido no fim perde-se; o que veio antes fica.
pub(crate) fn read(r: &mut Reader<'_>) -> AseUserData {
    let mut u = AseUserData::default();
    let Some(flags) = r.u32() else { return u };
    if flags & 1 != 0 {
        u.text = Some(r.string().unwrap_or_default());
    }
    if flags & 2 != 0 {
        let (Some(cr), Some(g), Some(b), Some(a)) = (r.u8(), r.u8(), r.u8(), r.u8()) else {
            return u;
        };
        u.color = Some([cr, g, b, a]);
    }
    if flags & 4 != 0 {
        let (Some(_size), Some(n)) = (r.u32(), r.u32()) else {
            return u;
        };
        for _ in 0..n {
            let Some(key) = r.u32() else { break };
            let Some(props) = props(r, 0) else { break };
            u.maps.push(AsePropMap { key, props });
        }
    }
    u
}

fn put_string(out: &mut Vec<u8>, t: &str) {
    let b = t.as_bytes();
    let n = b.len().min(usize::from(u16::MAX));
    out.extend_from_slice(&(n as u16).to_le_bytes());
    out.extend_from_slice(&b[..n]);
}

fn put_value(b: &mut Vec<u8>, v: &AseProp) {
    match v {
        AseProp::Bool(x) => b.push(u8::from(*x)),
        AseProp::I8(x) => b.push(*x as u8),
        AseProp::U8(x) => b.push(*x),
        AseProp::I16(x) => b.extend_from_slice(&x.to_le_bytes()),
        AseProp::U16(x) => b.extend_from_slice(&x.to_le_bytes()),
        AseProp::I32(x) | AseProp::Fixed(x) => b.extend_from_slice(&x.to_le_bytes()),
        AseProp::U32(x) => b.extend_from_slice(&x.to_le_bytes()),
        AseProp::I64(x) => b.extend_from_slice(&x.to_le_bytes()),
        AseProp::U64(x) => b.extend_from_slice(&x.to_le_bytes()),
        AseProp::F32(x) => b.extend_from_slice(&x.to_bits().to_le_bytes()),
        AseProp::F64(x) => b.extend_from_slice(&x.to_bits().to_le_bytes()),
        AseProp::Str(s) => put_string(b, s),
        AseProp::Point(x, y) | AseProp::Size(x, y) => {
            b.extend_from_slice(&x.to_le_bytes());
            b.extend_from_slice(&y.to_le_bytes());
        }
        AseProp::Rect(x, y, w, h) => {
            for n in [x, y, w, h] {
                b.extend_from_slice(&n.to_le_bytes());
            }
        }
        AseProp::Vector(items) => {
            b.extend_from_slice(&(items.len() as u32).to_le_bytes());
            let common = items.first().map_or(VECTOR_MIXED, AseProp::code);
            let same = items.iter().all(|i| i.code() == common);
            let common = if same { common } else { VECTOR_MIXED };
            b.extend_from_slice(&common.to_le_bytes());
            for i in items {
                if common == VECTOR_MIXED {
                    b.extend_from_slice(&i.code().to_le_bytes());
                }
                put_value(b, i);
            }
        }
        AseProp::Map(m) => put_props(b, m),
        AseProp::Uuid(u) => b.extend_from_slice(u),
    }
}

fn put_props(b: &mut Vec<u8>, m: &[(String, AseProp)]) {
    b.extend_from_slice(&(m.len() as u32).to_le_bytes());
    for (name, v) in m {
        put_string(b, name);
        b.extend_from_slice(&v.code().to_le_bytes());
        put_value(b, v);
    }
}

/// O corpo de um `0x2020`.
#[must_use]
pub(crate) fn write(u: &AseUserData) -> Vec<u8> {
    let flags = u32::from(u.text.is_some())
        | u32::from(u.color.is_some()) << 1
        | u32::from(!u.maps.is_empty()) << 2;
    let mut b = flags.to_le_bytes().to_vec();
    if let Some(t) = &u.text {
        put_string(&mut b, t);
    }
    if let Some(c) = &u.color {
        b.extend_from_slice(c);
    }
    if !u.maps.is_empty() {
        let mut maps = Vec::new();
        for m in &u.maps {
            maps.extend_from_slice(&m.key.to_le_bytes());
            put_props(&mut maps, &m.props);
        }
        b.extend_from_slice(&((maps.len() + 8) as u32).to_le_bytes());
        b.extend_from_slice(&(u.maps.len() as u32).to_le_bytes());
        b.extend_from_slice(&maps);
    }
    b
}

/// Lê o corpo de um `0x2008`.
pub(crate) fn read_external(r: &mut Reader<'_>) -> Vec<AseExternalFile> {
    let mut out = Vec::new();
    let Some(n) = r.u32() else { return out };
    if r.skip(8).is_none() {
        return out;
    }
    for _ in 0..n {
        let (Some(id), Some(kind)) = (r.u32(), r.u8()) else {
            break;
        };
        if r.skip(7).is_none() {
            break;
        }
        let Some(name) = r.string() else { break };
        out.push(AseExternalFile { id, kind, name });
    }
    out
}

/// O corpo de um `0x2008`.
#[must_use]
pub(crate) fn write_external(files: &[AseExternalFile]) -> Vec<u8> {
    let mut b = (files.len() as u32).to_le_bytes().to_vec();
    b.extend_from_slice(&[0; 8]);
    for f in files {
        b.extend_from_slice(&f.id.to_le_bytes());
        b.push(f.kind);
        b.extend_from_slice(&[0; 7]);
        put_string(&mut b, &f.name);
    }
    b
}
