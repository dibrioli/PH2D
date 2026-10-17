//! **As configurações que o estado inicial de uma macro leva** (`crate::macro_track::inicial`): o estado
//! do Wet Paint pela porta do PRESET (knobs, máscaras, texturas), o meio e a ferramenta na mão, e por cima
//! deles cada PINCEL inteiro ([`super::wetpaint_preset::campos_da_macro`]) — o vivo e o guardado de cada
//! modo (`brush_by_mode`: trocar de meio ou de ferramenta a gravar carrega o do destino) —, mais o estado
//! do meio e do traço que o preset não guarda. A Shape Color vai no preset (prompt 45). Tudo por NOME (JSON): um campo novo num leitor antigo
//! salta, um campo ausente fica o de fábrica; um estado de antes da W6c (sem `meio`) é Wet Paint.
//!
//! A completude é gateada pelo censo dos controlos (`ph2d-panel-painter-layers`,
//! `tests/it/macro_estado_inicial.rs`): cada controlo do painel de cada meio, mexido antes de armar ou a
//! gravar, tem de chegar à reprodução.

use super::wetpaint_preset::{campos_da_macro, json};
use crate::tool::PainterTool;
use ph2d_brush_presets::{Miniatura, Origem, Parametros, Preset};
use serde_json::{Value, json};
use std::collections::BTreeMap;

mod tela;
mod trilho;

/// Porque as configurações não se aplicaram.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ErroDoEstado {
    Json(String),
    /// Texturas que o estado pedia e não se leram.
    Texturas(Vec<String>),
}

impl PainterTool {
    /// As configurações de agora: o JSON e as texturas (PNG por id).
    pub(crate) fn macro_estado(&self) -> (String, BTreeMap<String, Vec<u8>>) {
        let (preset, mut texturas) =
            self.wet_preset_capturar(&Parametros::novo("WetPaint", "macro", "macro"));
        let (trilho, crus) = self.macro_trilho();
        texturas.extend(crus);
        let w = &self.paint.wetpaint;
        let p = &self.paint;
        let campos = campos_da_macro();
        let v = json!({
            "meio": self.paint_media().to_u8(),
            "modo": self.active_paint_mode_id(),
            "preset": preset,
            "pincel": json::pincel_para_json(&p.brush, &campos),
            "pinceis": p
                .brush_by_mode
                .iter()
                .map(|b| json::pincel_para_json(b, &campos))
                .collect::<Vec<_>>(),
            "ligados": p.link_shared_settings,
            "falloff_armado": p.falloff_armed,
            "metodo_por_meio": p.method_by_media,
            "tela": self.macro_tela(),
            "trilho": trilho,
            "wet": {
                "grid_ratio": w.grid_ratio,
                "flow_ratio": w.flow_ratio,
                "km_mixing": w.km_mixing,
                "km_glaze": w.km_glaze,
                "smooth_edges": w.smooth_edges,
                "show_wet": w.show_wet,
                "paper_visual": w.paper_visual,
                "opacidade": w.opacidade,
                "relogio_fixo": w.relogio_fixo,
            },
            "traco": {
                "seed": p.seed,
                "tex_rng": p.tex_rng,
                "tiling": p.tiling,
                "eraser": p.eraser,
                "substrate_depth": p.substrate_depth,
                "substrate_rough": p.substrate_rough,
            },
        });
        (v.to_string(), texturas)
    }

    /// Aplica as configurações de [`Self::macro_estado`] (num tool que já tem o documento).
    ///
    /// # Errors
    /// [`ErroDoEstado`] — o JSON não se lê, ou faltam texturas.
    pub(crate) fn macro_estado_aplica(
        &mut self,
        texto: &str,
        mut texturas: BTreeMap<String, Vec<u8>>,
    ) -> Result<(), ErroDoEstado> {
        let v: Value =
            serde_json::from_str(texto).map_err(|e| ErroDoEstado::Json(e.to_string()))?;
        let crus = trilho::separa(&mut texturas);
        let parametros: Parametros = serde_json::from_value(v["preset"].clone())
            .map_err(|e| ErroDoEstado::Json(e.to_string()))?;
        let preset = Preset {
            parametros,
            origem: Origem::default(),
            miniatura: Miniatura {
                largura: 1,
                altura: 1,
                rgba: vec![0; 4],
            },
            texturas,
        };
        // Uma textura que o estado não trazia é um slot `Image` SEM imagem no tool que gravou (o preset
        // não tem o que guardar): a aplicação deixa-o igual. Só a que vinha e não se leu é erro.
        let trazidas: Vec<String> = preset.texturas.keys().cloned().collect();
        let mut falhas = self.wet_preset_aplicar(&preset);
        falhas.retain(|id| trazidas.contains(id));
        if !falhas.is_empty() {
            return Err(ErroDoEstado::Texturas(falhas));
        }
        // O preset arma o Wet Paint; o meio gravado e a ferramenta na mão vêm depois, pelas portas de
        // sempre. Os pincéis vêm por ÚLTIMO, crus: as trocas acima carregam slots e assentam métodos, e
        // o que a gravação viu é o que fica.
        let meio = v["meio"].as_u64().and_then(|m| u8::try_from(m).ok());
        self.set_paint_media(meio.map_or(super::PaintMedia::WetPaint, super::PaintMedia::from_u8));
        if let Some(modo) = v["modo"].as_str() {
            self.set_paint_tool_mode(modo);
        }
        let campos = campos_da_macro();
        let pincel = |x: &Value, base| {
            x.as_object().map_or(base, |m| {
                let m: BTreeMap<String, Value> = m.clone().into_iter().collect();
                json::pincel_de_json(&m, &campos, base)
            })
        };
        if let Some(a) = v["pinceis"].as_array() {
            for (slot, x) in self.paint.brush_by_mode.iter_mut().zip(a) {
                *slot = pincel(x, *slot);
            }
        }
        self.paint.brush = pincel(&v["pincel"], self.paint.brush);
        let p = &mut self.paint;
        p.link_shared_settings = v["ligados"].as_bool().unwrap_or(p.link_shared_settings);
        p.falloff_armed = v["falloff_armado"].as_bool().unwrap_or(p.falloff_armed);
        if let Some(a) = v["metodo_por_meio"].as_array() {
            for (m, x) in p.method_by_media.iter_mut().zip(a) {
                *m = x.as_u64().and_then(|n| u8::try_from(n).ok()).unwrap_or(*m);
            }
        }
        self.macro_tela_aplica(&v["tela"]);
        self.macro_trilho_aplica(&v["trilho"], crus)?;
        let wv = &v["wet"];
        let w = &mut self.paint.wetpaint;
        let u8_ = |x: &Value, base: u8| {
            x.as_u64()
                .and_then(|n| u8::try_from(n).ok())
                .unwrap_or(base)
        };
        w.grid_ratio = u8_(&wv["grid_ratio"], w.grid_ratio);
        w.flow_ratio = u8_(&wv["flow_ratio"], w.flow_ratio);
        w.km_mixing = wv["km_mixing"].as_bool().unwrap_or(w.km_mixing);
        w.km_glaze = wv["km_glaze"].as_bool().unwrap_or(w.km_glaze);
        w.show_wet = wv["show_wet"].as_bool().unwrap_or(w.show_wet);
        w.paper_visual = wv["paper_visual"].as_bool().unwrap_or(w.paper_visual);
        if let Some(x) = wv["smooth_edges"].as_f64() {
            w.smooth_edges = x as f32;
        }
        if let Some(x) = wv["opacidade"].as_f64() {
            w.opacidade = x as f32;
        }
        w.relogio_fixo = wv["relogio_fixo"].as_f64();
        self.macro_shape_color_aplica(&v["shape_color"]);
        let tv = &v["traco"];
        let p = &mut self.paint;
        p.seed = tv["seed"].as_u64().unwrap_or(p.seed);
        p.tex_rng = tv["tex_rng"].as_u64().unwrap_or(p.tex_rng);
        if let Some(a) = tv["tiling"].as_array() {
            for (i, t) in p.tiling.iter_mut().enumerate() {
                *t = a.get(i).and_then(Value::as_bool).unwrap_or(*t);
            }
        }
        p.eraser = tv["eraser"].as_bool().unwrap_or(p.eraser);
        if let Some(x) = tv["substrate_depth"].as_f64() {
            p.substrate_depth = x as f32;
        }
        if let Some(x) = tv["substrate_rough"].as_f64() {
            p.substrate_rough = x as f32;
        }
        Ok(())
    }

    /// A Shape Color de uma macro gravada ANTES do prompt 45 (o preset não a levava; hoje vai no
    /// `Parametros::shape_color`): por cima da que o preset pôs; um campo ausente fica como está.
    fn macro_shape_color_aplica(&mut self, sc: &Value) {
        if sc.is_object() {
            let mut e = self.shape_color_estado();
            e.ler_json(sc);
            self.set_shape_color_estado(&e);
        }
    }
}
