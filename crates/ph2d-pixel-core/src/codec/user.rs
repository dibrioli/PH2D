//! Os registos do user data e das preferências (MINOR 6).
//!
//! ```text
//! UserData : 1 cor (4 bytes; sem = transparente) · 2 texto · 3 Mapa*
//! Mapa     : 1 extensão (utf8; "" = do artista) · 2 Prop*
//! Prop     : 1 nome · 2 Valor
//! Valor    : UM campo, etiqueta = código do tipo `.ase` (1..=19):
//!            inteiros/bool em varint (com sinal: zigzag 64) · 11 f32 / 12 f64 em bits LE
//!            · 13 utf8 · 14/15 {1 a · 2 b} · 16 {1 x · 2 y · 3 w · 4 h} (zigzag 64)
//!            · 17 Valor* (campo 1) · 18 Prop* (campo 1) · 19 16 bytes
//! Prefs    : 1 grade {1 x · 2 y · 3 w · 4 h · 5 cor · 6 opacidade · 7 bandeiras (bit0 auto, bit1 visível)}
//!            · 2 grade de píxel {1 cor · 2 opacidade · 3 bandeiras}
//!            · 3 xadrez {1 w · 2 h · 3 cor a · 4 cor b · 5 apply zoom}
//!            · 4 onion {1 antes · 2 depois · 3 base · 4 passo · 5 bandeiras
//!              (bit0 tinta, bit1 à frente, bit2 laço na tag, bit3 só a camada)}
//! ```
//! Um campo que falta fica no valor de fábrica.

use super::{CodecError, Fields, put_field, put_uint, uint};
use crate::prefs::PixelPrefs;
use crate::user::{PropMap, PropValue, UserData};

fn zz(v: i64) -> u64 {
    ((v << 1) ^ (v >> 63)) as u64
}

fn unzz(v: u64) -> i64 {
    ((v >> 1) as i64) ^ -((v & 1) as i64)
}

fn int<T: TryFrom<i64>>(b: &[u8]) -> Result<T, CodecError> {
    T::try_from(unzz(uint(b)?)).map_err(|_| CodecError::Invalid("user.int"))
}

fn uns<T: TryFrom<u64>>(b: &[u8]) -> Result<T, CodecError> {
    T::try_from(uint(b)?).map_err(|_| CodecError::Invalid("user.uint"))
}

fn text(b: &[u8]) -> Result<String, CodecError> {
    String::from_utf8(b.to_vec()).map_err(|_| CodecError::Invalid("user.text"))
}

fn ints(out: &mut Vec<u8>, v: &[i32]) {
    for (i, n) in v.iter().enumerate() {
        put_uint(out, i as u64 + 1, zz(i64::from(*n)));
    }
    out.push(0);
}

fn read_ints<const N: usize>(b: &[u8]) -> Result<[i32; N], CodecError> {
    let mut r = [0; N];
    for (t, v) in Fields::new(b)? {
        if let Some(slot) = usize::try_from(t).ok().and_then(|t| r.get_mut(t - 1)) {
            *slot = int(v)?;
        }
    }
    Ok(r)
}

fn value_record(v: &PropValue) -> Vec<u8> {
    let mut body = Vec::new();
    let varint = |n: u64| {
        let mut b = Vec::new();
        super::put_varint(&mut b, n);
        b
    };
    let b = match v {
        PropValue::Bool(x) => varint(u64::from(*x)),
        PropValue::I8(x) => varint(zz(i64::from(*x))),
        PropValue::U8(x) => varint(u64::from(*x)),
        PropValue::I16(x) => varint(zz(i64::from(*x))),
        PropValue::U16(x) => varint(u64::from(*x)),
        PropValue::I32(x) | PropValue::Fixed(x) => varint(zz(i64::from(*x))),
        PropValue::U32(x) => varint(u64::from(*x)),
        PropValue::I64(x) => varint(zz(*x)),
        PropValue::U64(x) => varint(*x),
        PropValue::F32(x) => x.to_bits().to_le_bytes().to_vec(),
        PropValue::F64(x) => x.to_bits().to_le_bytes().to_vec(),
        PropValue::Str(s) => s.as_bytes().to_vec(),
        PropValue::Point(a, b) | PropValue::Size(a, b) => {
            let mut r = Vec::new();
            ints(&mut r, &[*a, *b]);
            r
        }
        PropValue::Rect(x, y, w, h) => {
            let mut r = Vec::new();
            ints(&mut r, &[*x, *y, *w, *h]);
            r
        }
        PropValue::Vector(items) => {
            let mut r = Vec::new();
            for i in items {
                put_field(&mut r, 1, &value_record(i));
            }
            r.push(0);
            r
        }
        PropValue::Map(m) => map_body(m),
        PropValue::Uuid(u) => u.to_vec(),
    };
    put_field(&mut body, u64::from(v.code()), &b);
    body.push(0);
    body
}

fn map_body(m: &PropMap) -> Vec<u8> {
    let mut r = Vec::new();
    for (name, v) in m {
        let mut p = Vec::new();
        put_field(&mut p, 1, name.as_bytes());
        put_field(&mut p, 2, &value_record(v));
        p.push(0);
        put_field(&mut r, 1, &p);
    }
    r.push(0);
    r
}

fn read_map(b: &[u8], depth: u8) -> Result<PropMap, CodecError> {
    let mut m = PropMap::new();
    for (t, p) in Fields::new(b)? {
        if t != 1 {
            continue;
        }
        let (mut name, mut value) = (String::new(), None);
        for (t, v) in Fields::new(p)? {
            match t {
                1 => name = text(v)?,
                2 => value = read_value(v, depth + 1)?,
                _ => {}
            }
        }
        if let Some(v) = value {
            m.insert(name, v);
        }
    }
    Ok(m)
}

/// `None` = tipo de um MINOR futuro: salta-se, como uma etiqueta desconhecida.
fn read_value(b: &[u8], depth: u8) -> Result<Option<PropValue>, CodecError> {
    if depth > 64 {
        return Err(CodecError::Invalid("user.depth"));
    }
    let Some((code, v)) = Fields::new(b)?.next() else {
        return Ok(None);
    };
    let bits = |n: usize| -> Result<u64, CodecError> {
        if v.len() != n {
            return Err(CodecError::Invalid("user.float"));
        }
        let mut a = [0u8; 8];
        a[..n].copy_from_slice(v);
        Ok(u64::from_le_bytes(a))
    };
    Ok(Some(match code {
        1 => PropValue::Bool(uint(v)? != 0),
        2 => PropValue::I8(int(v)?),
        3 => PropValue::U8(uns(v)?),
        4 => PropValue::I16(int(v)?),
        5 => PropValue::U16(uns(v)?),
        6 => PropValue::I32(int(v)?),
        7 => PropValue::U32(uns(v)?),
        8 => PropValue::I64(unzz(uint(v)?)),
        9 => PropValue::U64(uint(v)?),
        10 => PropValue::Fixed(int(v)?),
        11 => PropValue::F32(f32::from_bits(bits(4)? as u32)),
        12 => PropValue::F64(f64::from_bits(bits(8)?)),
        13 => PropValue::Str(text(v)?),
        14 => {
            let [a, b] = read_ints(v)?;
            PropValue::Point(a, b)
        }
        15 => {
            let [a, b] = read_ints(v)?;
            PropValue::Size(a, b)
        }
        16 => {
            let [x, y, w, h] = read_ints(v)?;
            PropValue::Rect(x, y, w, h)
        }
        17 => {
            let mut items = Vec::new();
            for (t, i) in Fields::new(v)? {
                if t == 1
                    && let Some(i) = read_value(i, depth + 1)?
                {
                    items.push(i);
                }
            }
            PropValue::Vector(items)
        }
        18 => PropValue::Map(read_map(v, depth)?),
        19 => {
            PropValue::Uuid(<[u8; 16]>::try_from(v).map_err(|_| CodecError::Invalid("user.uuid"))?)
        }
        _ => return Ok(None),
    }))
}

/// `None` quando o user data é o de omissão (nada se grava: o ficheiro fica como no MINOR 5).
pub(super) fn user_record(u: &UserData) -> Option<Vec<u8>> {
    if u.is_empty() {
        return None;
    }
    let mut b = Vec::new();
    if u.color != [0; 4] {
        put_field(&mut b, 1, &u.color);
    }
    if !u.text.is_empty() {
        put_field(&mut b, 2, u.text.as_bytes());
    }
    for (ext, m) in &u.props {
        let mut r = Vec::new();
        put_field(&mut r, 1, ext.as_bytes());
        put_field(&mut r, 2, &map_body(m));
        r.push(0);
        put_field(&mut b, 3, &r);
    }
    b.push(0);
    Some(b)
}

pub(super) fn read_user(b: &[u8]) -> Result<UserData, CodecError> {
    let mut u = UserData::default();
    for (t, v) in Fields::new(b)? {
        match t {
            1 => u.color = <[u8; 4]>::try_from(v).map_err(|_| CodecError::Invalid("user.color"))?,
            2 => u.text = text(v)?,
            3 => {
                let (mut ext, mut m) = (String::new(), PropMap::new());
                for (t, v) in Fields::new(v)? {
                    match t {
                        1 => ext = text(v)?,
                        2 => m = read_map(v, 0)?,
                        _ => {}
                    }
                }
                u.props.insert(ext, m);
            }
            _ => {}
        }
    }
    Ok(u)
}

/// Grava `field` com o user data de `u`, se não for o de omissão.
pub(super) fn put_user(out: &mut Vec<u8>, field: u64, u: &UserData) {
    if let Some(b) = user_record(u) {
        put_field(out, field, &b);
    }
}

fn color(b: &[u8]) -> Result<[u8; 4], CodecError> {
    <[u8; 4]>::try_from(b).map_err(|_| CodecError::Invalid("prefs.color"))
}

/// `None` = as de fábrica (nada se grava).
pub(super) fn prefs_record(p: &PixelPrefs) -> Option<Vec<u8>> {
    if *p == PixelPrefs::default() {
        return None;
    }
    let mut r = Vec::new();
    let g = &p.grid;
    let mut b = Vec::new();
    put_uint(&mut b, 1, zz(i64::from(g.x)));
    put_uint(&mut b, 2, zz(i64::from(g.y)));
    put_uint(&mut b, 3, u64::from(g.w));
    put_uint(&mut b, 4, u64::from(g.h));
    put_field(&mut b, 5, &g.color);
    put_uint(&mut b, 6, u64::from(g.opacity));
    put_uint(
        &mut b,
        7,
        u64::from(g.auto_opacity) | u64::from(g.visible) << 1,
    );
    b.push(0);
    put_field(&mut r, 1, &b);
    let q = &p.pixel_grid;
    let mut b = Vec::new();
    put_field(&mut b, 1, &q.color);
    put_uint(&mut b, 2, u64::from(q.opacity));
    put_uint(
        &mut b,
        3,
        u64::from(q.auto_opacity) | u64::from(q.visible) << 1,
    );
    b.push(0);
    put_field(&mut r, 2, &b);
    let c = &p.checker;
    let mut b = Vec::new();
    put_uint(&mut b, 1, u64::from(c.w));
    put_uint(&mut b, 2, u64::from(c.h));
    put_field(&mut b, 3, &c.color_a);
    put_field(&mut b, 4, &c.color_b);
    put_uint(&mut b, 5, u64::from(c.apply_zoom));
    b.push(0);
    put_field(&mut r, 3, &b);
    let o = &p.onion;
    let mut b = Vec::new();
    put_uint(&mut b, 1, u64::from(o.prev));
    put_uint(&mut b, 2, u64::from(o.next));
    put_uint(&mut b, 3, u64::from(o.base));
    put_uint(&mut b, 4, u64::from(o.step));
    put_uint(
        &mut b,
        5,
        u64::from(o.tint)
            | u64::from(o.front) << 1
            | u64::from(o.loop_tag) << 2
            | u64::from(o.current_layer_only) << 3,
    );
    b.push(0);
    put_field(&mut r, 4, &b);
    r.push(0);
    Some(r)
}

pub(super) fn read_prefs(bytes: &[u8]) -> Result<PixelPrefs, CodecError> {
    let mut p = PixelPrefs::default();
    for (t, b) in Fields::new(bytes)? {
        match t {
            1 => {
                let g = &mut p.grid;
                for (t, v) in Fields::new(b)? {
                    match t {
                        1 => g.x = int(v)?,
                        2 => g.y = int(v)?,
                        3 => g.w = uns(v)?,
                        4 => g.h = uns(v)?,
                        5 => g.color = color(v)?,
                        6 => g.opacity = uns(v)?,
                        7 => {
                            let f = uint(v)?;
                            g.auto_opacity = f & 1 != 0;
                            g.visible = f & 2 != 0;
                        }
                        _ => {}
                    }
                }
            }
            2 => {
                let q = &mut p.pixel_grid;
                for (t, v) in Fields::new(b)? {
                    match t {
                        1 => q.color = color(v)?,
                        2 => q.opacity = uns(v)?,
                        3 => {
                            let f = uint(v)?;
                            q.auto_opacity = f & 1 != 0;
                            q.visible = f & 2 != 0;
                        }
                        _ => {}
                    }
                }
            }
            3 => {
                let c = &mut p.checker;
                for (t, v) in Fields::new(b)? {
                    match t {
                        1 => c.w = uns(v)?,
                        2 => c.h = uns(v)?,
                        3 => c.color_a = color(v)?,
                        4 => c.color_b = color(v)?,
                        5 => c.apply_zoom = uint(v)? != 0,
                        _ => {}
                    }
                }
            }
            4 => {
                let o = &mut p.onion;
                for (t, v) in Fields::new(b)? {
                    match t {
                        1 => o.prev = uns(v)?,
                        2 => o.next = uns(v)?,
                        3 => o.base = uns(v)?,
                        4 => o.step = uns(v)?,
                        5 => {
                            let f = uint(v)?;
                            o.tint = f & 1 != 0;
                            o.front = f & 2 != 0;
                            o.loop_tag = f & 4 != 0;
                            o.current_layer_only = f & 8 != 0;
                        }
                        _ => {}
                    }
                }
            }
            _ => {}
        }
    }
    if p.grid.w == 0 || p.grid.h == 0 || p.checker.w == 0 || p.checker.h == 0 {
        return Err(CodecError::Invalid("prefs.size"));
    }
    Ok(p)
}
