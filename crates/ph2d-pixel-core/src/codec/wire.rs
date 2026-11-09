//! Os registos do formato no fio: varint, campo etiquetado, zigzag e o leitor de campos.

use super::CodecError;

pub(super) fn put_varint(out: &mut Vec<u8>, mut v: u64) {
    loop {
        let b = (v & 0x7f) as u8;
        v >>= 7;
        if v == 0 {
            out.push(b);
            return;
        }
        out.push(b | 0x80);
    }
}

pub(super) fn put_field(out: &mut Vec<u8>, tag: u64, body: &[u8]) {
    put_varint(out, tag);
    put_varint(out, body.len() as u64);
    out.extend_from_slice(body);
}

pub(super) fn put_uint(out: &mut Vec<u8>, tag: u64, v: u64) {
    let mut b = Vec::new();
    put_varint(&mut b, v);
    put_field(out, tag, &b);
}

pub(super) fn zigzag(v: i32) -> u64 {
    u64::from(((v << 1) ^ (v >> 31)) as u32)
}

pub(super) fn unzigzag(v: u64) -> Result<i32, CodecError> {
    let v = u32::try_from(v).map_err(|_| CodecError::Invalid("zigzag"))?;
    Ok(((v >> 1) as i32) ^ -((v & 1) as i32))
}

fn read_varint(b: &[u8], at: &mut usize) -> Result<u64, CodecError> {
    let mut v = 0u64;
    for shift in (0..64).step_by(7) {
        let byte = *b.get(*at).ok_or(CodecError::Truncated)?;
        *at += 1;
        v |= u64::from(byte & 0x7f) << shift;
        if byte & 0x80 == 0 {
            return Ok(v);
        }
    }
    Err(CodecError::Invalid("varint"))
}

pub(super) fn uint(b: &[u8]) -> Result<u64, CodecError> {
    let mut at = 0;
    read_varint(b, &mut at)
}

pub(super) fn u16_of(b: &[u8], what: &'static str) -> Result<u16, CodecError> {
    u16::try_from(uint(b)?).map_err(|_| CodecError::Invalid(what))
}

pub(super) fn u32_of(b: &[u8], what: &'static str) -> Result<u32, CodecError> {
    u32::try_from(uint(b)?).map_err(|_| CodecError::Invalid(what))
}

/// Os campos `(etiqueta, corpo)` de um registo, até ao 0 final.
pub(super) struct Fields<'a> {
    items: std::vec::IntoIter<(u64, &'a [u8])>,
}

impl<'a> Fields<'a> {
    pub(super) fn new(b: &'a [u8]) -> Result<Self, CodecError> {
        let mut items = Vec::new();
        let mut at = 0;
        loop {
            let tag = read_varint(b, &mut at)?;
            if tag == 0 {
                break;
            }
            let len =
                usize::try_from(read_varint(b, &mut at)?).map_err(|_| CodecError::Truncated)?;
            let end = at.checked_add(len).ok_or(CodecError::Truncated)?;
            let body = b.get(at..end).ok_or(CodecError::Truncated)?;
            items.push((tag, body));
            at = end;
        }
        Ok(Self {
            items: items.into_iter(),
        })
    }
}

impl<'a> Iterator for Fields<'a> {
    type Item = (u64, &'a [u8]);
    fn next(&mut self) -> Option<Self::Item> {
        self.items.next()
    }
}
