//! Os registos da P13 (MINOR 9): rampas, ciclos, guias, som e efeitos de camada.
//!
//! ```text
//! Sprite 17 Ramp*   : 1 id · 2 nome · 3 índices (n bytes)
//! Sprite 18 Cycle*  : 1 rampa · 2 período ms · 3 direcção (0 Forward, 1 Reverse, 2 PingPong)
//! Sprite 19 Guias   : 1|2|3 ponto de fuga (Ponto) · 4 horizonte (bits f32) · 5 iso (Ponto da origem)
//! Sprite 20 Audio*  : 1 asset (utf8) · 2 início ms (zigzag) · 3 ganho (bits f32) · 4 mudo
//! Layer  9  Efeito* : 1 espécie (1 = Outline) · 2 cor (4 bytes) · 3 cor do fundo (4 bytes; sem =
//!                     o transparente) · 4 matriz (9 bits, linha a linha) · 5 Inside · 6 Tiled
//! Ponto             : 1 x (bits f32) · 2 y (bits f32)
//! ```
//! Sem campo = o de omissão; o documento sem nada disto grava como o MINOR 8.

use super::{CodecError, Fields, put_field, put_uint, u32_of, uint, unzigzag, zigzag};
use crate::audio::FaixaDeAudio;
use crate::cycle::{Cycle, CycleDirection};
use crate::efeito::EfeitoDeCamada;
use crate::filter::outline::{Background, Outline, Place};
use crate::guia::{Guias, Iso2x1};
use crate::ramp::{Ramp, RampId};
use crate::sprite::Sprite;

fn utf8(v: &[u8], what: &'static str) -> Result<String, CodecError> {
    String::from_utf8(v.to_vec()).map_err(|_| CodecError::Invalid(what))
}

fn f32_of(v: &[u8], what: &'static str) -> Result<f32, CodecError> {
    Ok(f32::from_bits(u32_of(v, what)?))
}

fn ponto(p: [f32; 2]) -> Vec<u8> {
    let mut b = Vec::new();
    put_uint(&mut b, 1, u64::from(p[0].to_bits()));
    put_uint(&mut b, 2, u64::from(p[1].to_bits()));
    b.push(0);
    b
}

fn read_ponto(v: &[u8]) -> Result<[f32; 2], CodecError> {
    let mut p = [0.0; 2];
    for (t, b) in Fields::new(v)? {
        match t {
            1 => p[0] = f32_of(b, "guide.x")?,
            2 => p[1] = f32_of(b, "guide.y")?,
            _ => {}
        }
    }
    Ok(p)
}

/// Os campos 17–20 do registo do sprite.
pub(super) fn put_sprite(r: &mut Vec<u8>, s: &Sprite) {
    for ramp in &s.ramps.ramps {
        let mut b = Vec::new();
        put_uint(&mut b, 1, u64::from(ramp.id.0));
        put_field(&mut b, 2, ramp.name.as_bytes());
        put_field(&mut b, 3, &ramp.indices);
        b.push(0);
        put_field(r, 17, &b);
    }
    for c in &s.ramps.cycles {
        let mut b = Vec::new();
        put_uint(&mut b, 1, u64::from(c.ramp.0));
        put_uint(&mut b, 2, u64::from(c.periodo_ms));
        put_uint(&mut b, 3, u64::from(c.direcao.code()));
        b.push(0);
        put_field(r, 18, &b);
    }
    if s.guias != Guias::default() {
        let g = &s.guias;
        let mut b = Vec::new();
        for (tag, p) in (1..).zip(g.pontos_de_fuga) {
            if let Some(p) = p {
                put_field(&mut b, tag, &ponto(p));
            }
        }
        if let Some(h) = g.horizonte {
            put_uint(&mut b, 4, u64::from(h.to_bits()));
        }
        if let Some(iso) = g.iso {
            put_field(&mut b, 5, &ponto(iso.origem));
        }
        b.push(0);
        put_field(r, 19, &b);
    }
    for a in &s.audio {
        let mut b = Vec::new();
        put_field(&mut b, 1, a.asset.as_bytes());
        put_uint(&mut b, 2, zigzag(a.inicio_ms));
        put_uint(&mut b, 3, u64::from(a.ganho.to_bits()));
        put_uint(&mut b, 4, u64::from(a.mudo));
        b.push(0);
        put_field(r, 20, &b);
    }
}

/// Lê um dos campos 17–20; `false` = a etiqueta não é destas.
pub(super) fn read_sprite_field(s: &mut Sprite, tag: u64, v: &[u8]) -> Result<bool, CodecError> {
    match tag {
        17 => {
            let mut ramp = Ramp {
                id: RampId(0),
                name: String::new(),
                indices: Vec::new(),
            };
            for (t, b) in Fields::new(v)? {
                match t {
                    1 => ramp.id = RampId(u32_of(b, "ramp.id")?),
                    2 => ramp.name = utf8(b, "ramp.name")?,
                    3 => ramp.indices = b.to_vec(),
                    _ => {}
                }
            }
            s.ramps.ramps.push(ramp);
        }
        18 => {
            let mut c = Cycle {
                ramp: RampId(0),
                periodo_ms: 0,
                direcao: CycleDirection::Forward,
            };
            for (t, b) in Fields::new(v)? {
                match t {
                    1 => c.ramp = RampId(u32_of(b, "cycle.ramp")?),
                    2 => c.periodo_ms = u32_of(b, "cycle.period")?,
                    3 => {
                        c.direcao = u8::try_from(uint(b)?)
                            .ok()
                            .and_then(CycleDirection::from_code)
                            .ok_or(CodecError::Invalid("cycle.direction"))?;
                    }
                    _ => {}
                }
            }
            s.ramps.cycles.push(c);
        }
        19 => {
            for (t, b) in Fields::new(v)? {
                match t {
                    1..=3 => s.guias.pontos_de_fuga[t as usize - 1] = Some(read_ponto(b)?),
                    4 => s.guias.horizonte = Some(f32_of(b, "guide.horizon")?),
                    5 => {
                        s.guias.iso = Some(Iso2x1 {
                            origem: read_ponto(b)?,
                        })
                    }
                    _ => {}
                }
            }
        }
        20 => {
            let mut a = FaixaDeAudio {
                asset: String::new(),
                inicio_ms: 0,
                ganho: 1.0,
                mudo: false,
            };
            for (t, b) in Fields::new(v)? {
                match t {
                    1 => a.asset = utf8(b, "audio.asset")?,
                    2 => a.inicio_ms = unzigzag(uint(b)?)?,
                    3 => a.ganho = f32_of(b, "audio.gain")?,
                    4 => a.mudo = uint(b)? != 0,
                    _ => {}
                }
            }
            s.audio.push(a);
        }
        _ => return Ok(false),
    }
    Ok(true)
}

/// O campo 9 do registo de uma camada, um por efeito.
pub(super) fn put_effects(b: &mut Vec<u8>, efeitos: &[EfeitoDeCamada]) {
    for EfeitoDeCamada::Outline(o) in efeitos {
        let mut e = Vec::new();
        put_uint(&mut e, 1, 1);
        put_field(&mut e, 2, &o.color);
        if let Background::Color(c) = o.background {
            put_field(&mut e, 3, &c);
        }
        let m = o.matrix.as_flattened().iter().enumerate();
        let bits = m.fold(0u64, |acc, (i, &on)| acc | (u64::from(on) << i));
        put_uint(&mut e, 4, bits);
        put_uint(&mut e, 5, u64::from(o.place == Place::Inside));
        put_uint(&mut e, 6, u64::from(o.tiled));
        e.push(0);
        put_field(b, 9, &e);
    }
}

pub(super) fn read_effect(v: &[u8]) -> Result<EfeitoDeCamada, CodecError> {
    let mut kind = 0;
    let mut o = Outline {
        color: [0; 4],
        background: Background::Mask,
        matrix: [[false; 3]; 3],
        place: Place::Outside,
        tiled: false,
    };
    let rgba = |b: &[u8]| <[u8; 4]>::try_from(b).map_err(|_| CodecError::Invalid("effect.color"));
    for (t, b) in Fields::new(v)? {
        match t {
            1 => kind = uint(b)?,
            2 => o.color = rgba(b)?,
            3 => o.background = Background::Color(rgba(b)?),
            4 => {
                let bits = uint(b)?;
                for (i, m) in o.matrix.as_flattened_mut().iter_mut().enumerate() {
                    *m = bits >> i & 1 != 0;
                }
            }
            5 => {
                if uint(b)? != 0 {
                    o.place = Place::Inside;
                }
            }
            6 => o.tiled = uint(b)? != 0,
            _ => {}
        }
    }
    match kind {
        1 => Ok(EfeitoDeCamada::Outline(o)),
        _ => Err(CodecError::Invalid("effect.kind")),
    }
}
