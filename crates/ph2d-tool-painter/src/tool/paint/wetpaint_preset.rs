//! **O preset do Wet Painter** (W1, `docs/Painter/NewFeaturesWetPaint/02 §5` e `05 §2`): o que um
//! ficheiro `*.ph2d-brush.png` leva deste meio, e as duas portas — [`PainterTool::wet_preset_aplicar`]
//! e [`PainterTool::wet_preset_capturar`].
//!
//! Um preset é o PINCEL: os knobs do fluido e a borracha molhada, o sub-conjunto do `BrushSpec` que o
//! traço do Wet Paint lê ([`CAMPOS_DO_PINCEL`]), as máscaras da forma e do grão, o modo (a ferramenta
//! molhada) e os volumes. Fica de fora o que é da OBRA, do GESTO e dos outros meios
//! ([`CAMPOS_FORA`], cada um com o porquê). Todo campo do `BrushSpec` está numa das duas listas, e o
//! gate `o_pincel_guardado_pinta_o_mesmo_traco` prova que os de fora não mudam o traço.

use super::brush_settings::BrushTextureImage;
use super::*;
use ph2d_brush_presets::{Parametros, Preset, SCHEMA, Volumes, escrever_textura, ler_textura};
use ph2d_painter_brush::TextureKind;
use ph2d_wet_paint::tuning::KNOB_DEFS;
use std::collections::BTreeMap;

/// A Opacity do Wet Paint sem preset: todo o pigmento do pincel (a claridade de fábrica é o
/// `pigmentPerDab` de cada preset, dono 10/10: «o correto para clarear não seria usar pigment?»).
pub const OPACIDADE_DE_FABRICA: f32 = 1.0;

mod json;
#[cfg(test)]
mod medida_preview;
mod miniatura;
mod preview;
#[cfg(test)]
mod tests;

pub use miniatura::{DIAMETRO_DA_MINIATURA, miniatura_do_preset};
pub use preview::{
    ALTURA_DO_PREVIEW, CenaDoPreview, ChaveDoPreview, LARGURA_DO_PREVIEW, pintar_preview,
};

/// O meio-alvo de todo preset desta família (o `media` do JSON).
pub const MEIO_DO_PRESET: &str = "WetPaint";
/// A pasta-raiz da família em `assets/brushes/` e em `~/.local/share/ph2d/brushes/`.
pub const MOTOR_DOS_PRESETS: &str = "WetPainter";
/// A chave da borracha molhada no mapa `engine` (não é knob do registo: é o slider `erase` do motor).
const CHAVE_ERASE: &str = "erase";
const TEXTURA_FORMA: &str = "shape";
const TEXTURA_GRAO: &str = "grain";

/// Os campos do `BrushSpec` que o preset leva, pelo nome do campo.
pub const CAMPOS_DO_PINCEL: [&str; 28] = [
    "hardness",
    "falloff",
    "custom_falloff",
    "spacing",
    "jitter",
    "jitter_unit",
    "jitter_absolute_px",
    "jitter_scale",
    "jitter_rotate",
    "jitter_spacing",
    "spray_count",
    "input_samples",
    "stabilizer",
    "taper",
    "texture",
    "grain_depth",
    "shape",
    "dab_flatten",
    "dab_angle_deg",
    "dab_curve",
    "color_jitter_enabled",
    "color_jitter_hue",
    "color_jitter_sat",
    "color_jitter_val",
    "blend",
    "pigment",
    "pigment_mix",
    "paper_depth",
];

const VOLUME: &str = "volume do topo (Size px = 2·raio, Opacity = força)";
const DA_OBRA: &str =
    "da obra, não do pincel (a cor e o papel são do documento; a simetria é da tela)";
const DO_GESTO: &str = "do gesto (o método do traço e o cartão Line), não do pincel";
const DO_MEIO: &str = "a porta do meio (`set_paint_media`) é a dona";
const DO_SCULPT: &str = "canal derivado do Sculpt (`sculpt_panel.rs`)";
const AQUARELA: &str = "lido só pela Watercolor";
const IMPASTO: &str = "lido só pelo Impasto";
const NAO_LIDO: &str = "o Wet Paint não o lê (deposita pela lei do fluido)";

/// Os campos do `BrushSpec` que o preset NÃO leva, e porquê.
pub const CAMPOS_FORA: [(&str, &str); 66] = [
    ("radius_px", VOLUME),
    ("strength", VOLUME),
    ("color", DA_OBRA),
    ("paper", DA_OBRA),
    ("symmetry", DA_OBRA),
    ("stroke_method", DO_GESTO),
    ("grid_cell_px", DO_GESTO),
    ("grid_offset_px", DO_GESTO),
    ("grid_fit", DO_GESTO),
    ("edge_to_edge", DO_GESTO),
    ("line_kind", DO_GESTO),
    ("style_solid", DO_GESTO),
    ("sketchy_reach", DO_GESTO),
    ("sketchy_density", DO_GESTO),
    ("thread_width_px", DO_GESTO),
    ("thread_opacity", DO_GESTO),
    ("sketchy_magnetify", DO_GESTO),
    ("wire_history", DO_GESTO),
    ("wire_connection_line", DO_GESTO),
    ("ribbon_weight", DO_GESTO),
    ("ribbon_friction", DO_GESTO),
    ("ribbon_gravity", DO_GESTO),
    ("ribbon_rungs", DO_GESTO),
    ("rough_amount", DO_GESTO),
    ("rough_bowing", DO_GESTO),
    ("rough_passes", DO_GESTO),
    ("dash_ratio", DO_GESTO),
    ("dash_samples", DO_GESTO),
    ("needs_heading", DO_SCULPT),
    ("watercolor", DO_MEIO),
    ("impasto", DO_MEIO),
    ("edge_gain", AQUARELA),
    ("edge_spread", AQUARELA),
    ("smooth_edges", AQUARELA),
    ("granulation", AQUARELA),
    ("fill", AQUARELA),
    ("depth", AQUARELA),
    ("opacity", AQUARELA),
    ("warp", AQUARELA),
    ("edge_flow", AQUARELA),
    ("paper_edge", AQUARELA),
    ("wet_smudge", AQUARELA),
    ("wet_rewet", AQUARELA),
    ("wet_charge", AQUARELA),
    ("wet_dilution", AQUARELA),
    ("wet_pull", AQUARELA),
    ("granulation_use_paper", AQUARELA),
    ("watercolor_shape_auto", AQUARELA),
    ("impasto_smooth_edges", IMPASTO),
    ("impasto_depth", IMPASTO),
    ("film_depth", IMPASTO),
    ("impasto_source", IMPASTO),
    ("impasto_draw_to", IMPASTO),
    ("impasto_smoothing", IMPASTO),
    ("impasto_body", IMPASTO),
    ("impasto_plow", IMPASTO),
    ("impasto_push", IMPASTO),
    ("impasto_shine", IMPASTO),
    ("impasto_roughness", IMPASTO),
    ("impasto_metallic", IMPASTO),
    ("impasto_wax", IMPASTO),
    ("impasto_wax_color", IMPASTO),
    ("flow", NAO_LIDO),
    ("accumulate", NAO_LIDO),
    ("airbrush_rate_s", NAO_LIDO),
    ("space_attenuation", NAO_LIDO),
];

/// Um `f32` para o JSON pelo menor texto que volta ao mesmo `f32` (`0.1`, não `0.10000000149011612`).
fn curto(v: f32) -> f64 {
    format!("{v}").parse().unwrap_or(f64::from(v))
}

/// O índice do botão de cada ferramenta molhada no rádio do painel (`pick_wet_tool`).
fn indice_da_ferramenta(t: WetTool) -> usize {
    match t {
        WetTool::Paint => 0,
        WetTool::Smear => 2,
        WetTool::Blend => 3,
        WetTool::Wet => 4,
        WetTool::Dry => 5,
        WetTool::Blow => 6,
    }
}

const FERRAMENTAS: [WetTool; 6] = [
    WetTool::Paint,
    WetTool::Smear,
    WetTool::Blend,
    WetTool::Wet,
    WetTool::Dry,
    WetTool::Blow,
];

/// O que o bloco de presets do painel pediu e só a ponte da app pode fazer (disco, biblioteca, a
/// política dos volumes). O tool regista; `ph2d-app-painter/src/painter_bridge_presets.rs` executa.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PedidoPreset {
    /// Escolher o preset neste caminho da biblioteca.
    Escolher(String),
    AlternarFavorito,
    GuardarComoNovo,
    GuardarComoPadrao,
    Repor,
    /// A política dos volumes, pelo índice em `Politica::TODAS`.
    Politica(u8),
}

impl PainterTool {
    /// Os controlos do bloco de presets: os botões e as escolhas viram um [`PedidoPreset`]; os volumes
    /// e os seus valores rápidos aplicam-se aqui mesmo (não tocam no disco).
    pub(crate) fn route_wet_preset_event(
        &mut self,
        event: &ph2d_editor_core::tool::PanelEvent,
    ) -> bool {
        use crate::ids;
        use ph2d_editor_core::tool::PanelEvent;
        let pedido = match event {
            PanelEvent::Click(id) if *id == ids::PAINTER_WETPAINT_PRESET_FAVORITE => {
                PedidoPreset::AlternarFavorito
            }
            PanelEvent::Click(id) if *id == ids::PAINTER_WETPAINT_PRESET_SAVE_NEW => {
                PedidoPreset::GuardarComoNovo
            }
            PanelEvent::Click(id) if *id == ids::PAINTER_WETPAINT_PRESET_SAVE_DEFAULT => {
                PedidoPreset::GuardarComoPadrao
            }
            PanelEvent::Click(id) if *id == ids::PAINTER_WETPAINT_PRESET_RESET => {
                PedidoPreset::Repor
            }
            PanelEvent::SelectOption(id, rel) if *id == ids::PAINTER_WETPAINT_PRESET => {
                PedidoPreset::Escolher(rel.clone())
            }
            PanelEvent::SelectOption(id, v) if *id == ids::PAINTER_WETPAINT_PRESET_POLICY => {
                match v.parse::<u8>() {
                    Ok(i) if i < ids::PAINTER_WETPAINT_PRESET_POLICY_COUNT => {
                        PedidoPreset::Politica(i)
                    }
                    _ => return false,
                }
            }
            PanelEvent::SelectOption(id, v) if ids::PAINTER_WETPAINT_VOL_QUICK.contains(id) => {
                let vol = ids::PAINTER_WETPAINT_VOL_QUICK
                    .iter()
                    .position(|q| q == id)
                    .unwrap_or(0);
                let Some(&x) = v
                    .parse::<usize>()
                    .ok()
                    .and_then(|i| ids::vol_quick_values(vol).get(i))
                else {
                    return false;
                };
                self.aplicar_volume(vol, f64::from(x));
                return true;
            }
            PanelEvent::SetValue(id, v) if *id == ids::PAINTER_WETPAINT_VOL_SIZE => {
                self.aplicar_volume(0, *v);
                return true;
            }
            PanelEvent::SetValue(id, v) if *id == ids::PAINTER_WETPAINT_VOL_OPACITY => {
                self.aplicar_volume(2, *v);
                return true;
            }
            _ => return false,
        };
        self.paint.wetpaint.preset_pedido = Some(pedido);
        true
    }

    /// Um volume VIVO pelo índice do painel (Size px · Water · Opacity).
    fn aplicar_volume(&mut self, vol: usize, x: f64) {
        let v = match vol {
            0 => Volumes {
                size_px: Some(x),
                ..Volumes::default()
            },
            1 => Volumes {
                water: Some(x),
                ..Volumes::default()
            },
            _ => Volumes {
                opacity: Some(x),
                ..Volumes::default()
            },
        };
        self.wet_preset_aplicar_volumes(&v);
    }

    /// O Wet Paint está armado (a ponte abre a biblioteca na primeira vez que estiver).
    #[must_use]
    pub fn wetpaint_armed(&self) -> bool {
        self.paint.wetpaint.armed
    }

    /// Tira (e limpa) o pedido do bloco de presets — a ponte da app chama-o a cada quadro.
    pub fn take_wet_preset_request(&mut self) -> Option<PedidoPreset> {
        self.paint.wetpaint.preset_pedido.take()
    }

    /// **Aplica um preset do Wet Painter**: arma o meio, põe o pincel (os campos que o preset não fixa
    /// voltam ao de fábrica — um preset é o pincel inteiro, não um delta), os knobs, as máscaras, o modo
    /// e os volumes que ele fixa. Volume ausente = o de agora (o tamanho do artista sobrevive à troca).
    ///
    /// Devolve as texturas que o ficheiro pedia e não se leram (o slot fica sem imagem): quem chama
    /// di-lo ao artista — nunca em silêncio.
    pub fn wet_preset_aplicar(&mut self, p: &Preset) -> Vec<String> {
        if !self.paint.wetpaint.armed {
            self.set_paint_media(PaintMedia::WetPaint);
        }
        let prm = &p.parametros;
        let de_fabrica = json::pincel_de_json(
            &prm.brush,
            &CAMPOS_DO_PINCEL,
            ph2d_painter_brush::BrushSpec::default(),
        );
        self.paint.brush = json::pincel_de_json(
            &json::pincel_para_json(&de_fabrica, &CAMPOS_DO_PINCEL),
            &CAMPOS_DO_PINCEL,
            self.paint.brush,
        );
        let falhas = self.instalar_mascaras(&p.texturas);

        let water = self.paint.wetpaint.knobs.water;
        let mut k = WetKnobs::DEFAULT;
        for def in &KNOB_DEFS {
            if let Some(&v) = prm.engine.get(def.key) {
                k.set(def.knob, v);
            }
        }
        if let Some(&v) = prm.engine.get(CHAVE_ERASE) {
            k.erase = if v.is_nan() {
                k.erase
            } else {
                v.clamp(0.0, 1.0)
            };
        }
        k.water = water;
        self.paint.wetpaint.knobs = k;

        if let Some(m) = prm.paint_mode.as_deref() {
            self.wet_preset_aplicar_modo(m);
        }
        // O Tilt do pincel (W1e, F4); um preset de antes dele (`None`) não mexe na tela.
        if let Some(tl) = prm.tilt {
            let w = &mut self.paint.wetpaint;
            w.tilt_on = tl.on;
            w.tilt_ring = tl.ring.min(8);
            w.tilt_spoke = tl.spoke % 12;
        }
        // As opções da W1g, pela mesma regra: ausentes num preset de antes delas, não mexem.
        if let Some(v) = prm.live_water {
            self.set_wet_agua_viva(v);
        }
        if let Some(v) = prm.heavy_edge {
            self.set_wet_borda_que_pesa(v);
        }
        // A largura prometida (W2): a lei MEDIDA do preset; um ficheiro sem ela (de antes da W2) usa a
        // do Wet Round, o pincel de fábrica.
        self.paint.wetpaint.largura = Some(
            super::wetpaint::largura::LeiDaLargura::de_tabela(&prm.width_table)
                .unwrap_or_else(super::wetpaint::largura::LeiDaLargura::de_omissao),
        );
        self.wet_preset_aplicar_volumes(&prm.volumes);
        falhas
    }

    /// Os volumes que `v` fixa (a política «lembrar volumes» passa por aqui também). Pressure e Length
    /// ainda não têm leitor (W6/W5): viajam no ficheiro e não mexem em nada.
    pub fn wet_preset_aplicar_volumes(&mut self, v: &Volumes) {
        if let Some(d) = v.size_px.filter(|d| d.is_finite()) {
            self.set_brush_size_px(d as f32 * 0.5);
        }
        if let Some(o) = v.opacity.filter(|o| o.is_finite()) {
            self.paint.wetpaint.opacidade = (o as f32).clamp(0.0, 1.0);
        }
        if let Some(w) = v.water.filter(|w| w.is_finite()) {
            self.paint.wetpaint.knobs.water = w.clamp(0.0, 1.0);
        }
    }

    /// O modo (a ferramenta molhada) pelo nome que o preset guarda; um nome desconhecido não muda nada.
    pub fn wet_preset_aplicar_modo(&mut self, modo: &str) {
        if let Some(t) = FERRAMENTAS.iter().find(|t| format!("{t:?}") == modo) {
            self.pick_wet_tool(indice_da_ferramenta(*t));
        }
    }

    /// Os volumes de agora, como o painel os mostra (Size px = o DIÂMETRO do anel).
    #[must_use]
    pub fn wet_preset_volumes(&self) -> Volumes {
        Volumes {
            size_px: Some(curto(self.paint.brush.radius_px * 2.0)),
            opacity: Some(curto(self.paint.wetpaint.opacidade)),
            water: Some(self.paint.wetpaint.knobs.water),
            ..Volumes::default()
        }
    }

    /// O modo de agora (a ferramenta molhada), pelo nome que o preset guarda.
    #[must_use]
    pub fn wet_preset_modo(&self) -> String {
        format!("{:?}", self.paint.wetpaint.tool)
    }

    /// **Captura o pincel de agora** sobre `base` (o preset de onde ele veio): os campos que esta versão
    /// conhece são reescritos; os de ondas futuras que `base` trazia sobrevivem.
    #[must_use]
    pub fn wet_preset_capturar(
        &self,
        base: &Parametros,
    ) -> (Parametros, BTreeMap<String, Vec<u8>>) {
        let mut p = base.clone();
        p.schema = SCHEMA;
        MEIO_DO_PRESET.clone_into(&mut p.media);
        p.brush
            .extend(json::pincel_para_json(&self.paint.brush, &CAMPOS_DO_PINCEL));
        let k = &self.paint.wetpaint.knobs;
        for def in &KNOB_DEFS {
            p.engine.insert(def.key.to_owned(), k.get(def.knob));
        }
        p.engine.insert(CHAVE_ERASE.to_owned(), k.erase);
        let v = self.wet_preset_volumes();
        p.volumes.size_px = v.size_px;
        p.volumes.opacity = v.opacity;
        p.volumes.water = v.water;
        p.paint_mode = Some(self.wet_preset_modo());
        let w = &self.paint.wetpaint;
        p.tilt = Some(ph2d_brush_presets::Tilt {
            on: w.tilt_on,
            ring: w.tilt_ring,
            spoke: w.tilt_spoke,
        });
        p.live_water = Some(w.agua_viva);
        p.heavy_edge = Some(w.borda_que_pesa);
        // A lei da largura do pincel de onde ele veio (W2): um pincel mexido guarda a do preset-base.
        if let Some(lei) = &w.largura {
            p.width_table = lei.tabela();
        }

        let mut texturas = BTreeMap::new();
        if self.paint.brush.shape.kind == TextureKind::Image
            && let Some((lum, w, h)) = self.brush_shape_image()
        {
            texturas.insert(TEXTURA_FORMA.to_owned(), escrever_textura(lum, w, h));
        }
        if self.paint.brush.texture.kind == TextureKind::Image
            && let Some((lum, w, h)) = self.brush_texture_image()
        {
            texturas.insert(TEXTURA_GRAO.to_owned(), escrever_textura(lum, w, h));
        }
        (p, texturas)
    }

    /// As máscaras da forma e do grão: instaladas DIRECTAMENTE (as portas da UI re-ajustam o aspecto e
    /// a rampa do grão, e o preset já traz os valores que essas portas deram). Um slot `Image` sem a
    /// textura no ficheiro volta a `None` — uma imagem que não está lá não pinta — e o id vai na
    /// lista devolvida.
    fn instalar_mascaras(&mut self, texturas: &BTreeMap<String, Vec<u8>>) -> Vec<String> {
        let mut falhas = Vec::new();
        let mut ler = |id: &str| {
            let t = texturas.get(id).and_then(|b| ler_textura(b).ok());
            if t.is_none() {
                falhas.push(id.to_owned());
            }
            t
        };
        self.paint.shape_layers.clear();
        self.paint.shape_image = None;
        if self.paint.brush.shape.kind == TextureKind::Image {
            match ler(TEXTURA_FORMA) {
                Some((lum, w, h)) => {
                    self.paint.shape_image = Some(BrushTextureImage::new(lum, w, h));
                }
                None => self.paint.brush.shape.kind = TextureKind::None,
            }
        }
        self.paint.shape_image_pending = false;
        self.paint.shape_image_version = self.paint.shape_image_version.wrapping_add(1);

        self.paint.texture_image = None;
        if self.paint.brush.texture.kind == TextureKind::Image {
            match ler(TEXTURA_GRAO) {
                Some((lum, w, h)) => {
                    self.paint.texture_image = Some(BrushTextureImage::new(lum, w, h));
                }
                None => self.paint.brush.texture.kind = TextureKind::None,
            }
        }
        self.paint.texture_image_pending = false;
        self.paint.texture_image_version = self.paint.texture_image_version.wrapping_add(1);
        falhas
    }
}
