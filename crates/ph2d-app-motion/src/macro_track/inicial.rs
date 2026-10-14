//! **O estado inicial** da trilha do Motion: o documento ao armar (texto canónico + o `next_id`, que o
//! texto não guarda) e as configurações que o cozimento lê — a lei da bomba, o retângulo e o tamanho de
//! omissão. O nível e a ajuda dos nós viajam em CADA lote do grafo, o único que os lê (uma porta só); a
//! etiqueta `3` fica reservada.
//!
//! O palco ([`palco`]) é um `MotionState` PRÓPRIO nesse estado: o documento autorado nunca é tocado.

use crate::motion_state::MotionState;
use ph2d_macro::CodecError;
use ph2d_macro::rec::{self, Record};
use ph2d_motion_doc::MotionDoc;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ErroDoInicial {
    Registo(CodecError),
    /// O texto do documento não se lê neste build.
    Documento(String),
}

/// As configurações lidas de volta.
#[derive(Clone, Debug)]
pub struct Inicial {
    pub doc: MotionDoc,
    /// O `next_id` do grafo ao armar — a base dos nós criados a gravar ([`super::passo::Alvo`]).
    pub base: u32,
    pub lei: bool,
    pub uv: [f32; 4],
    pub tamanho: [f32; 2],
}

/// O estado inicial de `m`, agora.
#[must_use]
pub fn captura(m: &MotionState) -> Vec<u8> {
    let mut r = Record::new();
    r.str(1, &m.doc.to_text())
        .varint(2, u64::from(m.doc.graph.next_id()))
        .bool(5, m.pump.a_lei());
    for (i, v) in m.default_uv_rect.iter().enumerate() {
        r.f32(6 + i as u32, *v);
    }
    for (i, v) in m.default_size.iter().enumerate() {
        r.f32(10 + i as u32, *v);
    }
    r.finish()
}

/// # Errors
/// [`ErroDoInicial`].
pub fn le(bytes: &[u8]) -> Result<Inicial, ErroDoInicial> {
    let f = rec::fields(bytes).map_err(ErroDoInicial::Registo)?;
    let um = |tag: u32| f.iter().rev().find(|(t, _)| *t == tag).map(|(_, b)| *b);
    let precisa = |tag: u32| {
        um(tag).ok_or(ErroDoInicial::Registo(CodecError::MissingField {
            record: "motion.inicial",
            tag,
        }))
    };
    let reg = ErroDoInicial::Registo;
    let texto = rec::as_str(precisa(1)?).map_err(reg)?;
    let doc =
        MotionDoc::from_text(texto).map_err(|e| ErroDoInicial::Documento(format!("{e:?}")))?;
    let u32_de = |b: &[u8]| -> Result<u32, ErroDoInicial> {
        u32::try_from(rec::as_varint(b).map_err(reg)?)
            .map_err(|_| ErroDoInicial::Registo(CodecError::BadVarint))
    };
    let f32_de =
        |tag: u32| -> Result<f32, ErroDoInicial> { rec::as_f32(precisa(tag)?).map_err(reg) };
    Ok(Inicial {
        doc,
        base: u32_de(precisa(2)?)?,
        lei: rec::as_bool(precisa(5)?).map_err(reg)?,
        uv: [f32_de(6)?, f32_de(7)?, f32_de(8)?, f32_de(9)?],
        tamanho: [f32_de(10)?, f32_de(11)?],
    })
}

/// **O palco**: um `MotionState` próprio no estado inicial (coze na placa quando o tocador lha dá).
#[must_use]
pub fn palco(ini: &Inicial) -> MotionState {
    let mut m = MotionState::new();
    m.doc = ini.doc.clone();
    m.doc.graph.restore_next_id(ini.base);
    m.sinks.clear();
    m.pump.define_a_lei(ini.lei);
    m.default_uv_rect = ini.uv;
    m.default_size = ini.tamanho;
    m
}
