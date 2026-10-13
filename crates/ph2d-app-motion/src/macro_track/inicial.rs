//! **O estado inicial** da trilha do Motion: o documento ao armar (texto canónico + o `next_id`, que o
//! texto não guarda) e as configurações que as portas gravadas leem — o nível, a ajuda dos nós, a lei
//! da bomba, o retângulo e o tamanho de omissão do cozimento.
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
    pub nivel: Option<u32>,
    pub ajuda: bool,
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
        .bool(4, m.node_help_enabled)
        .bool(5, m.pump.a_lei());
    if let Some(n) = m.level {
        r.varint(3, u64::from(n));
    }
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
        nivel: um(3).map(u32_de).transpose()?,
        ajuda: rec::as_bool(precisa(4)?).map_err(reg)?,
        lei: rec::as_bool(precisa(5)?).map_err(reg)?,
        uv: [f32_de(6)?, f32_de(7)?, f32_de(8)?, f32_de(9)?],
        tamanho: [f32_de(10)?, f32_de(11)?],
    })
}

/// **O palco**: um `MotionState` próprio no estado inicial, a cozer na CPU.
#[must_use]
pub fn palco(ini: &Inicial) -> MotionState {
    let mut m = MotionState::new();
    m.doc = ini.doc.clone();
    m.doc.graph.restore_next_id(ini.base);
    m.sinks.clear();
    m.level = ini.nivel;
    m.node_help_enabled = ini.ajuda;
    m.pump.define_a_lei(ini.lei);
    m.default_uv_rect = ini.uv;
    m.default_size = ini.tamanho;
    m.gpu_enabled = false;
    m
}
