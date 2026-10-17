//! **O estado das ferramentas do TRILHO** no estado inicial de uma macro (W7, `docs/Macros/04_plano_W7.md`
//! item 4): a fonte do Clone, a Máscara (o rascunho de proteção) e a Seleção (a lista de formas, as
//! coberturas e os modos) — estado do tool fora do documento e fora dos pincéis, que muda o que um traço
//! gravado pinta. Os valores vão por NOME no JSON; as coberturas, cruas, ao lado das texturas, com o
//! prefixo [`PREFIXO`], só quando existem. Um estado de antes da W7 não tem `trilho`: fica o de fábrica
//! (nenhuma fonte, seleção ou máscara) — o que ele tocava.
//!
//! Fora, de propósito: o gesto EM VOO (o arrasto, a caneta, a pega de um gizmo — armar a meio de um gesto
//! não é um estado); o anel do Offset (transiente: o undo também o larga, `restore_selection`); a área de
//! transferência (copiar, cortar e colar são portas RECUSADAS, `macro_track/portas.rs`).

use super::ErroDoEstado;
use crate::layers::LayerId;
use crate::tool::PainterTool;
use crate::tool::paint::selection_shapes::{SelectionEntry, SelectionShape};
use serde_json::{Value, json};
use std::collections::BTreeMap;
use std::sync::Arc;

/// O prefixo das coberturas cruas no mapa das texturas do estado inicial.
pub(super) const PREFIXO: &str = "trilho/";
const MASCARA: &str = "trilho/mascara";
const SEL_MASCARA: &str = "trilho/selecao/mascara";
const SEL_BASE: &str = "trilho/selecao/base";
const SEL_CRISP: &str = "trilho/selecao/crisp";

fn raster(i: usize) -> String {
    format!("trilho/selecao/raster/{i}")
}

/// Tira do mapa das texturas as coberturas do trilho (o preset não as conhece).
pub(super) fn separa(texturas: &mut BTreeMap<String, Vec<u8>>) -> BTreeMap<String, Vec<u8>> {
    let (crus, resto) = std::mem::take(texturas)
        .into_iter()
        .partition(|(k, _)| k.starts_with(PREFIXO));
    *texturas = resto;
    crus
}

fn par(v: &Value) -> Option<[f32; 2]> {
    let a = v.as_array()?;
    Some([a.first()?.as_f64()? as f32, a.get(1)?.as_f64()? as f32])
}

fn u8_(v: &Value, base: u8) -> u8 {
    v.as_u64()
        .and_then(|n| u8::try_from(n).ok())
        .unwrap_or(base)
}

fn f32_(v: &Value, base: f32) -> f32 {
    v.as_f64().map_or(base, |n| n as f32)
}

impl PainterTool {
    /// O JSON do trilho e as coberturas cruas (por chave).
    pub(super) fn macro_trilho(&self) -> (Value, BTreeMap<String, Vec<u8>>) {
        let p = &self.paint;
        let mut crus = BTreeMap::new();
        let mut cru = |k: String, b: &Arc<Vec<u8>>| {
            if !b.is_empty() {
                crus.insert(k, b.as_ref().clone());
            }
        };
        cru(MASCARA.to_owned(), &p.mask_scratch_rgba);
        cru(SEL_MASCARA.to_owned(), &p.selection_mask);
        cru(SEL_BASE.to_owned(), &p.selection_base);
        cru(SEL_CRISP.to_owned(), &p.selection_crisp);
        for (i, e) in p.selection_shapes.iter().enumerate() {
            if let SelectionShape::Raster { crisp } = &e.shape {
                cru(raster(i), crisp);
            }
        }
        let v = json!({
            "clone": {
                "fonte": p.clone_source,
                "offset": p.clone_offset,
                "alinhado": p.clone_aligned,
                "a_amostrar": p.clone_sample_armed,
            },
            "mascara": {
                "pincel": p.mask_brush,
                "cor": p.mask_overlay_color,
                "alvo": p.mask_scratch_target.map(|l| l.0),
            },
            "selecao": {
                "ativa": p.selection_active,
                "modo": p.selection_mode,
                "op": p.selection_bool_op,
                "limiar": p.selection_threshold,
                "estabilizador": p.selection_stabilizer,
                "feather": p.selection_feather,
                "edicao": p.selection_edit_mode,
                "opacidade": p.selection_overlay_opacity,
                "aneis": p.selection_ring_stack,
                "formas": serde_json::to_value(&p.selection_shapes).unwrap_or(Value::Null),
            },
        });
        (v, crus)
    }

    /// Aplica [`Self::macro_trilho`] (num tool que já tem o documento: o alvo da máscara é uma camada
    /// dele). Uma cobertura de outro tamanho que a tela é erro: o estado não é desta tela.
    pub(super) fn macro_trilho_aplica(
        &mut self,
        v: &Value,
        mut crus: BTreeMap<String, Vec<u8>>,
    ) -> Result<(), ErroDoEstado> {
        if v.is_null() {
            return Ok(());
        }
        let (w, h) = self.source_size;
        let n = w as usize * h as usize;
        let mut toma = |k: &str, canais: usize| -> Result<Arc<Vec<u8>>, ErroDoEstado> {
            match crus.remove(k) {
                None => Ok(Arc::new(Vec::new())),
                Some(b) if b.len() == n * canais => Ok(Arc::new(b)),
                Some(b) => Err(ErroDoEstado::Json(format!(
                    "{k}: {} bytes numa tela {w}×{h}",
                    b.len()
                ))),
            }
        };
        let (cv, mv, sv) = (&v["clone"], &v["mascara"], &v["selecao"]);
        let mascara = toma(MASCARA, 4)?;
        let (sel, base, crisp) = (
            toma(SEL_MASCARA, 1)?,
            toma(SEL_BASE, 1)?,
            toma(SEL_CRISP, 1)?,
        );
        let mut formas: Vec<SelectionEntry> = match &sv["formas"] {
            Value::Null => Vec::new(),
            f => {
                serde_json::from_value(f.clone()).map_err(|e| ErroDoEstado::Json(e.to_string()))?
            }
        };
        for (i, e) in formas.iter_mut().enumerate() {
            if let SelectionShape::Raster { crisp } = &mut e.shape {
                *crisp = toma(&raster(i), 1)?;
            }
        }
        let p = &mut self.paint;
        p.clone_source = par(&cv["fonte"]);
        p.clone_offset = par(&cv["offset"]);
        p.clone_aligned = cv["alinhado"].as_bool().unwrap_or(p.clone_aligned);
        p.clone_sample_armed = cv["a_amostrar"].as_bool().unwrap_or(false);
        p.mask_brush = u8_(&mv["pincel"], p.mask_brush);
        p.mask_overlay_color = u8_(&mv["cor"], p.mask_overlay_color);
        p.selection_mode = u8_(&sv["modo"], p.selection_mode);
        p.selection_bool_op = u8_(&sv["op"], p.selection_bool_op);
        p.selection_threshold = f32_(&sv["limiar"], p.selection_threshold);
        p.selection_stabilizer = f32_(&sv["estabilizador"], p.selection_stabilizer);
        p.selection_edit_mode = sv["edicao"].as_bool().unwrap_or(p.selection_edit_mode);
        p.selection_overlay_opacity = f32_(&sv["opacidade"], p.selection_overlay_opacity);
        p.selection_ring_stack = sv["aneis"].as_bool().unwrap_or(false);
        p.selection_base = base;
        p.selection_shapes = formas;
        p.selection_raster_cache = Vec::new();
        let alvo = mv["alvo"].as_u64().map(LayerId);
        self.restore_mask_scratch(mascara, alvo);
        let feather = f32_(&sv["feather"], self.paint.selection_feather);
        let ativa = sv["ativa"].as_bool().unwrap_or(false);
        self.restore_selection(sel, ativa, crisp, feather);
        self.invalidate_composite();
        Ok(())
    }
}
