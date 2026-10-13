//! **O estado inicial da trilha** (Corel «Initial State», MyPaint): o documento pela porta do disco
//! ([`crate::PaintedDocument`], postcard — o MESMO recorte que o projeto guarda) e as configurações
//! ([`crate::tool::paint`] `macro_estado`: o preset + a obra + o gesto + o meio).
//!
//! ```text
//! inicial = 1 versão (varint) · 2 configurações (JSON) · 3 textura* {1 id · 2 png}
//!         · 4 documento (postcard) · 5 versão do documento (varint)
//! ```
//! ⚠️ O postcard é POSICIONAL: o documento lê-se só na [`VERSAO_DO_DOCUMENTO`] em que foi gravado, e o
//! gate `o_documento_da_macro_tem_a_versao_do_seu_formato` prende o número ao formato.

use crate::PaintedDocument;
use crate::tool::PainterTool;
use ph2d_macro::rec::{self, Record};
use std::collections::BTreeMap;

/// A versão do formato do estado inicial.
pub const VERSAO: u64 = 1;

/// A versão do formato posicional do [`PaintedDocument`] dentro do estado inicial. Sobe junto com o
/// degrau do `PROJECT_SCHEMA` que mudar o documento (o gate acusa).
pub const VERSAO_DO_DOCUMENTO: u64 = 1;

/// A entidade com que o tocador liga o documento no seu tool próprio.
pub(crate) const ENTIDADE_DO_TOCADOR: u64 = 1;

/// Porque o estado inicial não se aplica.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ErroDoInicial {
    Registo(ph2d_macro::CodecError),
    /// Gravado por uma versão mais nova.
    VersaoFutura(u64),
    /// O documento foi gravado noutro formato.
    DocumentoNoutroFormato(u64),
    Documento(String),
    Configuracoes(String),
    Texturas(Vec<String>),
}

pub(super) fn captura(t: &PainterTool) -> Vec<u8> {
    let (cfg, texturas) = t.macro_estado();
    let doc = t.documento_vivo(0);
    let mut r = Record::new();
    r.varint(1, VERSAO).str(2, &cfg);
    for (id, png) in &texturas {
        let mut s = Record::new();
        s.str(1, id).bytes(2, png);
        r.bytes(3, &s.finish());
    }
    r.bytes(
        4,
        &postcard::to_allocvec(&doc).expect("o documento é serde e serializa sempre"),
    );
    r.varint(5, VERSAO_DO_DOCUMENTO);
    r.finish()
}

/// Um tool novo no estado inicial `bytes`.
///
/// # Errors
/// [`ErroDoInicial`].
pub fn tool_no_inicial(bytes: &[u8]) -> Result<PainterTool, ErroDoInicial> {
    let campos = rec::fields(bytes).map_err(ErroDoInicial::Registo)?;
    let campo = |k: u32| campos.iter().rev().find(|(t, _)| *t == k).map(|(_, v)| *v);
    let v = campo(1)
        .map_or(Ok(0), rec::as_varint)
        .map_err(ErroDoInicial::Registo)?;
    if v > VERSAO {
        return Err(ErroDoInicial::VersaoFutura(v));
    }
    let vd = campo(5)
        .map_or(Ok(0), rec::as_varint)
        .map_err(ErroDoInicial::Registo)?;
    if vd != VERSAO_DO_DOCUMENTO {
        return Err(ErroDoInicial::DocumentoNoutroFormato(vd));
    }
    let doc: PaintedDocument = postcard::from_bytes(campo(4).unwrap_or_default())
        .map_err(|e| ErroDoInicial::Documento(e.to_string()))?;
    let mut texturas = BTreeMap::new();
    for (_, sub) in campos.iter().filter(|(t, _)| *t == 3) {
        let c = rec::fields(sub).map_err(ErroDoInicial::Registo)?;
        let get = |k: u32| c.iter().rev().find(|(t, _)| *t == k).map(|(_, v)| *v);
        let id = get(1)
            .map_or(Ok(""), rec::as_str)
            .map_err(ErroDoInicial::Registo)?;
        texturas.insert(id.to_owned(), get(2).unwrap_or_default().to_vec());
    }
    let cfg = campo(2)
        .map_or(Ok(""), rec::as_str)
        .map_err(ErroDoInicial::Registo)?;
    let (w, h) = doc.size;
    let mut t = PainterTool::default();
    t.install_document(ENTIDADE_DO_TOCADOR, doc);
    t.bind_document(ENTIDADE_DO_TOCADOR, Vec::new(), w, h);
    t.macro_estado_aplica(cfg, texturas).map_err(|e| match e {
        crate::tool::paint::ErroDoEstado::Json(s) => ErroDoInicial::Configuracoes(s),
        crate::tool::paint::ErroDoEstado::Texturas(v) => ErroDoInicial::Texturas(v),
    })?;
    Ok(t)
}
