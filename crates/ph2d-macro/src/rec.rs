//! **Registos de campos etiquetados** — a lei do ficheiro ([`crate::codec`]), aberta às famílias para
//! codificarem as suas cargas (o estado inicial, cada passo). Uma porta só: o ficheiro usa-a também.
//!
//! ```text
//! registo = { etiqueta:varint≥1 · comprimento:varint · bytes }* · 0
//! ```
//! Um leitor salta a etiqueta que não conhece; etiqueta repetida de campo único: vence a última.

use crate::codec::CodecError;

/// Escreve um registo. `finish` fecha-o com a etiqueta `0`.
#[derive(Default)]
pub struct Record(Vec<u8>);

impl Record {
    #[must_use]
    pub fn new() -> Self {
        Self(Vec::with_capacity(32))
    }

    pub fn bytes(&mut self, tag: u32, b: &[u8]) -> &mut Self {
        put_field(&mut self.0, tag, b);
        self
    }

    pub fn varint(&mut self, tag: u32, v: u64) -> &mut Self {
        self.bytes(tag, &varint_bytes(v))
    }

    pub fn f32(&mut self, tag: u32, v: f32) -> &mut Self {
        self.bytes(tag, &v.to_le_bytes())
    }

    pub fn f64(&mut self, tag: u32, v: f64) -> &mut Self {
        self.bytes(tag, &v.to_le_bytes())
    }

    pub fn bool(&mut self, tag: u32, v: bool) -> &mut Self {
        self.varint(tag, u64::from(v))
    }

    pub fn str(&mut self, tag: u32, s: &str) -> &mut Self {
        self.bytes(tag, s.as_bytes())
    }

    #[must_use]
    pub fn finish(mut self) -> Vec<u8> {
        self.0.push(0);
        self.0
    }
}

/// Os campos de um registo, pela ordem do ficheiro.
pub fn fields(bytes: &[u8]) -> Result<Vec<(u32, &[u8])>, CodecError> {
    Cursor(bytes).record()
}

pub fn as_varint(b: &[u8]) -> Result<u64, CodecError> {
    Cursor(b).varint()
}

pub fn as_bool(b: &[u8]) -> Result<bool, CodecError> {
    Ok(as_varint(b)? != 0)
}

pub fn as_f32(b: &[u8]) -> Result<f32, CodecError> {
    let a: [u8; 4] = b.try_into().map_err(|_| bad_len())?;
    Ok(f32::from_le_bytes(a))
}

pub fn as_f64(b: &[u8]) -> Result<f64, CodecError> {
    let a: [u8; 8] = b.try_into().map_err(|_| bad_len())?;
    Ok(f64::from_le_bytes(a))
}

pub fn as_str(b: &[u8]) -> Result<&str, CodecError> {
    std::str::from_utf8(b).map_err(|_| CodecError::BadUtf8)
}

fn bad_len() -> CodecError {
    CodecError::BadFieldLength {
        record: "field",
        tag: 0,
    }
}

pub(crate) fn put_field(out: &mut Vec<u8>, tag: u32, bytes: &[u8]) {
    put_varint(out, u64::from(tag));
    put_varint(out, bytes.len() as u64);
    out.extend_from_slice(bytes);
}

fn put_varint(out: &mut Vec<u8>, mut v: u64) {
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

pub(crate) fn varint_bytes(v: u64) -> Vec<u8> {
    let mut b = Vec::with_capacity(10);
    put_varint(&mut b, v);
    b
}

pub(crate) struct Cursor<'a>(pub(crate) &'a [u8]);

impl<'a> Cursor<'a> {
    pub(crate) fn varint(&mut self) -> Result<u64, CodecError> {
        let mut v: u64 = 0;
        for i in 0..10 {
            let (&b, rest) = self.0.split_first().ok_or(CodecError::Truncated)?;
            self.0 = rest;
            let bits = u64::from(b & 0x7f);
            if i == 9 && b > 1 {
                return Err(CodecError::BadVarint);
            }
            v |= bits << (7 * i);
            if b & 0x80 == 0 {
                return Ok(v);
            }
        }
        Err(CodecError::BadVarint)
    }

    /// Os campos de um registo até à etiqueta `0`.
    pub(crate) fn record(&mut self) -> Result<Vec<(u32, &'a [u8])>, CodecError> {
        let mut fields = Vec::new();
        loop {
            let tag = self.varint()?;
            if tag == 0 {
                return Ok(fields);
            }
            let tag = u32::try_from(tag).map_err(|_| CodecError::BadVarint)?;
            let len = usize::try_from(self.varint()?).map_err(|_| CodecError::Truncated)?;
            if len > self.0.len() {
                return Err(CodecError::Truncated);
            }
            let (body, rest) = self.0.split_at(len);
            self.0 = rest;
            fields.push((tag, body));
        }
    }
}
