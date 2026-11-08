//! O registo de uma slice (MINOR 5).
//!
//! ```text
//! Slice    : 1 id · 2 nome · 3 cor (4 bytes; sem = não tem) · 4 texto · 5 Key*
//!            · 6 propriedades (o registo de user data, MINOR 6, só com os mapas)
//! Key      : 1 quadro · 2 x (zigzag) · 3 y (zigzag) · 4 w · 5 h
//!            · 6 centro (x, y em zigzag; w, h) · 7 pivot (x, y em zigzag)
//! ```

use super::{CodecError, Fields, put_field, put_uint, u32_of, uint, unzigzag, zigzag};
use crate::slice::{Slice, SliceId, SliceKey, SliceShape};
use crate::sprite::Rect;
use crate::user::UserData;

fn key_record(k: &SliceKey) -> Vec<u8> {
    let mut b = Vec::new();
    put_uint(&mut b, 1, u64::from(k.frame));
    put_uint(&mut b, 2, zigzag(k.shape.bounds.x));
    put_uint(&mut b, 3, zigzag(k.shape.bounds.y));
    put_uint(&mut b, 4, u64::from(k.shape.bounds.w));
    put_uint(&mut b, 5, u64::from(k.shape.bounds.h));
    if let Some(c) = k.shape.center {
        let mut r = Vec::new();
        put_uint(&mut r, 1, zigzag(c.x));
        put_uint(&mut r, 2, zigzag(c.y));
        put_uint(&mut r, 3, u64::from(c.w));
        put_uint(&mut r, 4, u64::from(c.h));
        r.push(0);
        put_field(&mut b, 6, &r);
    }
    if let Some((x, y)) = k.shape.pivot {
        let mut r = Vec::new();
        put_uint(&mut r, 1, zigzag(x));
        put_uint(&mut r, 2, zigzag(y));
        r.push(0);
        put_field(&mut b, 7, &r);
    }
    b.push(0);
    b
}

pub(super) fn slice_record(c: &Slice) -> Vec<u8> {
    let mut b = Vec::new();
    put_uint(&mut b, 1, u64::from(c.id.0));
    put_field(&mut b, 2, c.name.as_bytes());
    if c.user.color != [0; 4] {
        put_field(&mut b, 3, &c.user.color);
    }
    if !c.user.text.is_empty() {
        put_field(&mut b, 4, c.user.text.as_bytes());
    }
    for k in &c.keys {
        put_field(&mut b, 5, &key_record(k));
    }
    if !c.user.props.is_empty() {
        let props = UserData {
            props: c.user.props.clone(),
            ..UserData::default()
        };
        super::user::put_user(&mut b, 6, &props);
    }
    b.push(0);
    b
}

fn read_center(bytes: &[u8]) -> Result<Rect, CodecError> {
    let mut r = Rect::default();
    for (t, v) in Fields::new(bytes)? {
        match t {
            1 => r.x = unzigzag(uint(v)?)?,
            2 => r.y = unzigzag(uint(v)?)?,
            3 => r.w = u32_of(v, "slice.center.w")?,
            4 => r.h = u32_of(v, "slice.center.h")?,
            _ => {}
        }
    }
    Ok(r)
}

fn read_pivot(bytes: &[u8]) -> Result<(i32, i32), CodecError> {
    let mut p = (0, 0);
    for (t, v) in Fields::new(bytes)? {
        match t {
            1 => p.0 = unzigzag(uint(v)?)?,
            2 => p.1 = unzigzag(uint(v)?)?,
            _ => {}
        }
    }
    Ok(p)
}

fn read_key(bytes: &[u8]) -> Result<SliceKey, CodecError> {
    let mut k = SliceKey {
        frame: 0,
        shape: SliceShape::default(),
    };
    for (t, v) in Fields::new(bytes)? {
        match t {
            1 => k.frame = u32_of(v, "slice.key.frame")?,
            2 => k.shape.bounds.x = unzigzag(uint(v)?)?,
            3 => k.shape.bounds.y = unzigzag(uint(v)?)?,
            4 => k.shape.bounds.w = u32_of(v, "slice.key.w")?,
            5 => k.shape.bounds.h = u32_of(v, "slice.key.h")?,
            6 => k.shape.center = Some(read_center(v)?),
            7 => k.shape.pivot = Some(read_pivot(v)?),
            _ => {}
        }
    }
    Ok(k)
}

pub(super) fn read_slice(bytes: &[u8]) -> Result<Slice, CodecError> {
    let mut c = Slice {
        id: SliceId(0),
        name: String::new(),
        user: UserData::default(),
        keys: Vec::new(),
    };
    let text =
        |v: &[u8], campo| String::from_utf8(v.to_vec()).map_err(|_| CodecError::Invalid(campo));
    for (t, v) in Fields::new(bytes)? {
        match t {
            1 => c.id = SliceId(u32_of(v, "slice.id")?),
            2 => c.name = text(v, "slice.name")?,
            3 => {
                c.user.color =
                    <[u8; 4]>::try_from(v).map_err(|_| CodecError::Invalid("slice.color"))?;
            }
            4 => c.user.text = text(v, "slice.data")?,
            5 => c.keys.push(read_key(v)?),
            6 => c.user.props = super::user::read_user(v)?.props,
            _ => {}
        }
    }
    Ok(c)
}
