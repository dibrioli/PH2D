//! **O ficheiro `.ph2dmacro`** (W18, ADR-0190): o MESMO formato que o componente `Macro` guarda
//! ([`crate::codec`]), com dois campos de topo que só o ficheiro tem — o modo de uso e o nome.
//!
//! ```text
//! .ph2dmacro = codec(MacroDoc) com, no topo: 4 modo (varint; o índice de `ModoDaMacro`) · 5 nome (utf8)
//! ```
//! O componente nunca os escreve: dentro do projecto o modo é um campo do componente e o nome é o do
//! objecto. Um leitor do componente que receba um ficheiro salta-os (etiquetas desconhecidas).

use crate::codec::{self, CodecError};
use crate::doc::MacroDoc;
use crate::rec::{as_str, as_varint, varint_bytes};

/// A extensão, com o ponto.
pub const EXTENSAO: &str = ".ph2dmacro";

const MODO: u32 = 4;
const NOME: u32 = 5;

/// Um ficheiro lido.
#[derive(Clone, Debug, PartialEq)]
pub struct Ficheiro {
    pub doc: MacroDoc,
    /// O índice de `ph2d_ecs::ModoDaMacro` (0 = Fiel).
    pub modo: u8,
    pub nome: String,
}

#[must_use]
pub fn escreve(f: &Ficheiro) -> Vec<u8> {
    codec::write_com_extras(
        &f.doc,
        &[
            (MODO, &varint_bytes(u64::from(f.modo))),
            (NOME, f.nome.as_bytes()),
        ],
    )
}

/// Lê um ficheiro. Sem modo = 0; sem nome = vazio (um `Macro.doc` lê-se assim também).
///
/// # Errors
/// [`CodecError`].
pub fn le(bytes: &[u8]) -> Result<Ficheiro, CodecError> {
    let (doc, extras) = codec::read_com_extras(bytes)?;
    let mut f = Ficheiro {
        doc,
        modo: 0,
        nome: String::new(),
    };
    for (tag, b) in &extras {
        match *tag {
            MODO => f.modo = u8::try_from(as_varint(b)?).map_err(|_| CodecError::BadVarint)?,
            NOME => as_str(b)?.clone_into(&mut f.nome),
            _ => {}
        }
    }
    Ok(f)
}
