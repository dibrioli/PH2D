//! O formato de ficheiro: `PH2DMACRO` + major/minor + registos de campos etiquetados.
//!
//! Molde `.riv` (Rive): dentro da mesma MAJOR um leitor antigo lê um ficheiro novo — cada
//! campo leva etiqueta e comprimento, e uma etiqueta desconhecida SALTA-SE. MAJOR diferente
//! recusa-se com erro ([`CodecError::MajorMismatch`]), nunca se adivinha.
//!
//! ```text
//! ficheiro = MAGIC · major:u16le · minor:u16le · registo(MacroDoc)
//! registo  = { etiqueta:varint≥1 · comprimento:varint · bytes }* · 0
//! MacroDoc : 1 reference (w:f32le h:f32le) · 2 Track*        (repetido) · 3 exige (utf8, repetido; 1.1)
//! Track    : 1 family (utf8) · 2 target (varint) · 3 initial (bytes) · 4 Event* (repetido)
//!            · 5 EdicaoDeEtapa* (repetido; 1.1)
//! Event    : 1 t_us (varint) · 2 payload (bytes)
//! Edicao   : 1 ancora (varint) · 2 nome (utf8) · 3 espera_us (zigzag) · 4 troca* · 5 insercao*
//!            · 6 sai_da_gravacao (varint) · 7 apagada (varint) · 8 dica (utf8; um leitor sem ela
//!            ignora-a: não muda o que toca) · 9 desloca_us (zigzag; 1.2, `exige` "etapa.desloca")
//!            — troca/insercao: 1 evento (varint) · 2 carga (bytes)
//! ```
//! Etiqueta repetida de campo único: vence a última.
//!
//! **`exige`** nomeia o que o leitor TEM de saber aplicar para tocar o ficheiro como foi feito; um nome
//! que ele não conhece recusa-se ([`CodecError::Exige`]) — saltar edições tocaria outra macro, calado.
//! O escritor só sobe o MINOR e só escreve `exige` quando o documento os usa: uma gravação sem edições
//! continua a ser um ficheiro 1.0, ao byte.

use crate::doc::{Event, MacroDoc, MacroTime, RefRect, Track};
use crate::etapa::EdicaoDeEtapa;
use crate::rec::{Cursor, Record, put_field, varint_bytes};

pub const MAGIC: &[u8; 9] = b"PH2DMACRO";
pub const MAJOR: u16 = 1;
pub const MINOR: u16 = 2;
/// O que este leitor sabe aplicar (`exige`).
pub const SABE: &[&str] = &[EXIGE_ETAPAS, EXIGE_DESLOCA];
/// As edições das etapas (1.1).
pub const EXIGE_ETAPAS: &str = "etapas";
/// A etapa que anda sozinha no tempo (`EdicaoDeEtapa::desloca_us`, 1.2): um leitor sem ela tocá-la-ia
/// no instante gravado, calado.
pub const EXIGE_DESLOCA: &str = "etapa.desloca";

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CodecError {
    NotAMacro,
    MajorMismatch {
        found: u16,
        supported: u16,
    },
    Truncated,
    BadVarint,
    BadUtf8,
    MissingField {
        record: &'static str,
        tag: u32,
    },
    BadFieldLength {
        record: &'static str,
        tag: u32,
    },
    NonMonotonicTime {
        track: usize,
    },
    /// O ficheiro exige uma funcionalidade que este leitor não sabe aplicar.
    Exige(String),
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
    let editado = doc.tracks.iter().any(|t| !t.edicoes.is_empty());
    let desloca = doc
        .tracks
        .iter()
        .any(|t| t.edicoes.iter().any(|e| e.desloca_us != 0));
    let minor: u16 = if desloca { MINOR } else { u16::from(editado) };
    out.extend_from_slice(MAGIC);
    out.extend_from_slice(&MAJOR.to_le_bytes());
    out.extend_from_slice(&minor.to_le_bytes());
    let mut rec = Vec::new();
    let mut r = Vec::with_capacity(8);
    r.extend_from_slice(&doc.reference.w.to_le_bytes());
    r.extend_from_slice(&doc.reference.h.to_le_bytes());
    put_field(&mut rec, 1, &r);
    for tr in &doc.tracks {
        put_field(&mut rec, 2, &write_track(tr));
    }
    if editado {
        put_field(&mut rec, 3, EXIGE_ETAPAS.as_bytes());
    }
    if desloca {
        put_field(&mut rec, 3, EXIGE_DESLOCA.as_bytes());
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
    for ed in &tr.edicoes {
        put_field(&mut rec, 5, &write_edicao(ed));
    }
    rec.push(0);
    rec
}

fn write_edicao(ed: &EdicaoDeEtapa) -> Vec<u8> {
    let mut r = Record::new();
    r.varint(1, ed.ancora as u64);
    if let Some(n) = &ed.nome {
        r.str(2, n);
    }
    if ed.espera_us != 0 {
        r.varint(3, zigzag(ed.espera_us));
    }
    for (tag, lista) in [(4, &ed.trocas), (5, &ed.insercoes)] {
        for (i, c) in lista {
            let mut p = Record::new();
            p.varint(1, *i as u64).bytes(2, c);
            r.bytes(tag, &p.finish());
        }
    }
    if ed.sai_da_gravacao {
        r.bool(6, true);
    }
    if ed.apagada {
        r.bool(7, true);
    }
    if let Some(d) = &ed.dica {
        r.str(8, d);
    }
    if ed.desloca_us != 0 {
        r.varint(9, zigzag(ed.desloca_us));
    }
    r.finish()
}

fn zigzag(v: i64) -> u64 {
    ((v << 1) ^ (v >> 63)).cast_unsigned()
}

fn unzigzag(v: u64) -> i64 {
    (v >> 1).cast_signed() ^ -((v & 1).cast_signed())
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
            3 => {
                let nome = std::str::from_utf8(body).map_err(|_| CodecError::BadUtf8)?;
                if !SABE.contains(&nome) {
                    return Err(CodecError::Exige(nome.to_owned()));
                }
            }
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
    let mut edicoes = Vec::new();
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
            5 => edicoes.push(read_edicao(body)?),
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
        edicoes,
    })
}

fn read_edicao(bytes: &[u8]) -> Result<EdicaoDeEtapa, CodecError> {
    let mut ancora = None;
    let mut ed = EdicaoDeEtapa::default();
    let indice = |b: &[u8]| usize::try_from(Cursor(b).varint()?).map_err(|_| CodecError::BadVarint);
    for (tag, body) in Cursor(bytes).record()? {
        match tag {
            1 => ancora = Some(indice(body)?),
            2 => {
                ed.nome = Some(
                    std::str::from_utf8(body)
                        .map_err(|_| CodecError::BadUtf8)?
                        .to_owned(),
                );
            }
            3 => ed.espera_us = unzigzag(Cursor(body).varint()?),
            4 | 5 => {
                let mut i = None;
                let mut carga = Vec::new();
                for (t, b) in Cursor(body).record()? {
                    match t {
                        1 => i = Some(indice(b)?),
                        2 => carga = b.to_vec(),
                        _ => {}
                    }
                }
                let i = i.ok_or(CodecError::MissingField {
                    record: "Troca",
                    tag: 1,
                })?;
                if tag == 4 {
                    &mut ed.trocas
                } else {
                    &mut ed.insercoes
                }
                .push((i, carga));
            }
            6 => ed.sai_da_gravacao = Cursor(body).varint()? != 0,
            7 => ed.apagada = Cursor(body).varint()? != 0,
            8 => {
                ed.dica = Some(
                    std::str::from_utf8(body)
                        .map_err(|_| CodecError::BadUtf8)?
                        .to_owned(),
                );
            }
            9 => ed.desloca_us = unzigzag(Cursor(body).varint()?),
            _ => {}
        }
    }
    ed.ancora = ancora.ok_or(CodecError::MissingField {
        record: "EdicaoDeEtapa",
        tag: 1,
    })?;
    Ok(ed)
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
