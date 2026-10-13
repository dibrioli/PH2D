//! O formato de ficheiro: `PH2DMACRO` + major/minor + registos de campos etiquetados.
//!
//! Molde `.riv` (Rive): dentro da mesma MAJOR um leitor antigo lê um ficheiro novo — cada
//! campo leva etiqueta e comprimento, e uma etiqueta desconhecida SALTA-SE. MAJOR diferente
//! recusa-se com erro ([`CodecError::MajorMismatch`]), nunca se adivinha.
//!
//! ```text
//! ficheiro = MAGIC · major:u16le · minor:u16le · registo(MacroDoc)
//! registo  = { etiqueta:varint≥1 · comprimento:varint · bytes }* · 0
//! MacroDoc : 1 reference (w:f32le h:f32le) · 2 Track*        (repetido)
//! Track    : 1 family (utf8) · 2 target (varint) · 3 initial (bytes) · 4 Event* (repetido)
//! Event    : 1 t_us (varint) · 2 payload (bytes)
//! ```
//! Etiqueta repetida de campo único: vence a última.

use crate::doc::{Event, MacroDoc, MacroTime, RefRect, Track};
use crate::rec::{Cursor, put_field, varint_bytes};

pub const MAGIC: &[u8; 9] = b"PH2DMACRO";
pub const MAJOR: u16 = 1;
pub const MINOR: u16 = 0;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CodecError {
    NotAMacro,
    MajorMismatch { found: u16, supported: u16 },
    Truncated,
    BadVarint,
    BadUtf8,
    MissingField { record: &'static str, tag: u32 },
    BadFieldLength { record: &'static str, tag: u32 },
    NonMonotonicTime { track: usize },
}

impl std::fmt::Display for CodecError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

impl std::error::Error for CodecError {}

// ── escrita ─────────────────────────────────────────────────────────────────

#[must_use]
pub fn write(doc: &MacroDoc) -> Vec<u8> {
    let mut out = Vec::with_capacity(64);
    out.extend_from_slice(MAGIC);
    out.extend_from_slice(&MAJOR.to_le_bytes());
    out.extend_from_slice(&MINOR.to_le_bytes());
    let mut rec = Vec::new();
    let mut r = Vec::with_capacity(8);
    r.extend_from_slice(&doc.reference.w.to_le_bytes());
    r.extend_from_slice(&doc.reference.h.to_le_bytes());
    put_field(&mut rec, 1, &r);
    for tr in &doc.tracks {
        put_field(&mut rec, 2, &write_track(tr));
    }
    rec.push(0);
    out.extend_from_slice(&rec);
    out
}

fn write_track(tr: &Track) -> Vec<u8> {
    let mut rec = Vec::new();
    put_field(&mut rec, 1, tr.family.as_bytes());
    put_field(&mut rec, 2, &varint_bytes(tr.target));
    put_field(&mut rec, 3, &tr.initial);
    for ev in &tr.events {
        let mut e = Vec::new();
        put_field(&mut e, 1, &varint_bytes(ev.t.0));
        put_field(&mut e, 2, &ev.payload);
        e.push(0);
        put_field(&mut rec, 4, &e);
    }
    rec.push(0);
    rec
}

// ── leitura ─────────────────────────────────────────────────────────────────

/// Lê um ficheiro de macro. Etiquetas desconhecidas saltam-se; MAJOR diferente recusa.
pub fn read(bytes: &[u8]) -> Result<MacroDoc, CodecError> {
    let rest = bytes.strip_prefix(MAGIC).ok_or(CodecError::NotAMacro)?;
    if rest.len() < 4 {
        return Err(CodecError::Truncated);
    }
    let major = u16::from_le_bytes([rest[0], rest[1]]);
    if major != MAJOR {
        return Err(CodecError::MajorMismatch {
            found: major,
            supported: MAJOR,
        });
    }
    let mut cur = Cursor(&rest[4..]);
    let mut reference = None;
    let mut tracks = Vec::new();
    for (tag, body) in cur.record()? {
        match tag {
            1 => {
                if body.len() != 8 {
                    return Err(CodecError::BadFieldLength {
                        record: "MacroDoc",
                        tag,
                    });
                }
                reference = Some(RefRect {
                    w: f32::from_le_bytes([body[0], body[1], body[2], body[3]]),
                    h: f32::from_le_bytes([body[4], body[5], body[6], body[7]]),
                });
            }
            2 => tracks.push(read_track(body)?),
            _ => {}
        }
    }
    let doc = MacroDoc {
        reference: reference.ok_or(CodecError::MissingField {
            record: "MacroDoc",
            tag: 1,
        })?,
        tracks,
    };
    if let Some(track) = doc
        .tracks
        .iter()
        .position(|tr| !tr.events.windows(2).all(|w| w[0].t <= w[1].t))
    {
        return Err(CodecError::NonMonotonicTime { track });
    }
    Ok(doc)
}

fn read_track(bytes: &[u8]) -> Result<Track, CodecError> {
    let mut family = None;
    let mut target = 0;
    let mut initial = Vec::new();
    let mut events = Vec::new();
    for (tag, body) in Cursor(bytes).record()? {
        match tag {
            1 => {
                family = Some(
                    std::str::from_utf8(body)
                        .map_err(|_| CodecError::BadUtf8)?
                        .to_owned(),
                );
            }
            2 => target = Cursor(body).varint()?,
            3 => initial = body.to_vec(),
            4 => events.push(read_event(body)?),
            _ => {}
        }
    }
    Ok(Track {
        family: family.ok_or(CodecError::MissingField {
            record: "Track",
            tag: 1,
        })?,
        target,
        initial,
        events,
    })
}

fn read_event(bytes: &[u8]) -> Result<Event, CodecError> {
    let mut t = None;
    let mut payload = Vec::new();
    for (tag, body) in Cursor(bytes).record()? {
        match tag {
            1 => t = Some(MacroTime(Cursor(body).varint()?)),
            2 => payload = body.to_vec(),
            _ => {}
        }
    }
    Ok(Event {
        t: t.ok_or(CodecError::MissingField {
            record: "Event",
            tag: 1,
        })?,
        payload,
    })
}
