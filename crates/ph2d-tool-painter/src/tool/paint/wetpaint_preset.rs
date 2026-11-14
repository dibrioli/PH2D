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
use ph2d_wet_paint::tuning::{KNOB_DEFS, Knob};
use std::collections::BTreeMap;

/// A Opacity do Wet Paint sem preset: todo o pigmento do pincel (a claridade de fábrica é o
/// `pigmentPerDab` de cada preset, dono 10/10: «o correto para clarear não seria usar pigment?»).
pub const OPACIDADE_DE_FABRICA: f32 = 1.0;

mod campos; // as listas dos campos do `BrushSpec` (dentro e fora do preset) — filho por LOC
#[cfg(test)]
mod captura_tests;
pub(in crate::tool::paint) mod controlos_e10; // a migração da E10 por função — filho por LOC
#[cfg(test)]
mod controlos_e10_tests;
#[cfg(test)]
mod corpo_da_agua_tests;
mod ferramentas;
#[cfg(test)]
mod glaze_no_preset_tests;
pub(in crate::tool::paint) mod json;
#[cfg(test)]
mod km_no_preset_tests;
#[cfg(test)]
mod material_tests;
#[cfg(test)]
mod medida_oleo_real;
#[cfg(test)]
mod medida_preview;
mod meios;
mod miniatura;
mod pedido;
mod preview;
mod preview_borracha;
#[cfg(test)]
mod receita_e4_tests;
#[cfg(test)]
mod shape_color_tests;
#[cfg(test)]
mod sonda_custo_oleo;
#[cfg(test)]
mod sonda_oleo_derrete;
#[cfg(test)]
mod sonda_oleo_estria;
#[cfg(test)]
mod sonda_oleo_furos;
#[cfg(test)]
mod sonda_oleo_grumos;
#[cfg(test)]
mod sonda_relevo_oleo;
#[cfg(test)]
mod sonda_varredura_oleo;
#[cfg(test)]
mod larguras_do_pool_tests;
#[cfg(test)]
mod tests;
mod texturas;
#[cfg(test)]
mod volumes_tests;

use campos::{AQUARELA, DA_OBRA, DIGITAL, DO_MEIO, DO_SCULPT, DOS_KNOBS, IMPASTO, POR_DAB, VOLUME};
pub use campos::{CAMPOS_DO_PINCEL, CAMPOS_FORA, campos_da_macro};
pub use ferramentas::{Gaveta, ferramenta_do_preset, nome_da_ferramenta};
pub use meios::{MeioDesconhecido, campos_do_meio, meio_do_preset, motor_do_meio, nome_do_meio};
pub use miniatura::{DIAMETRO_DA_MINIATURA, miniatura_do_preset};
pub use pedido::PedidoPreset;
pub use preview::{
    ALTURA_DO_PREVIEW, CenaDoPreview, ChaveDoPreview, LARGURA_DO_PREVIEW, pintar_preview,
};
pub use preview_borracha::COR_DO_FUNDO_DA_BORRACHA;

/// (Testes) O preset de fábrica `nome` — ou, se saiu da fábrica, o ficheiro do seu candidato em
/// `docs/Painter/Pinceis/W2_<grupo>_candidatos/` (o Oil Bristle, smoke do dono 27/10: a base de muitas medidas).
#[cfg(test)]
pub(in crate::tool::paint) fn de_fabrica_ou_candidato(nome: &str) -> Preset {
    let raiz = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../");
    ph2d_brush_presets::Biblioteca::carregar(&raiz.join("assets/brushes"), None)
        .entradas
        .into_iter()
        .find(|e| e.preset.parametros.name == nome)
        .map(|e| e.preset)
        .or_else(|| {
            std::fs::read_dir(raiz.join("docs/Painter/Pinceis"))
                .ok()?
                .flatten()
                .filter(|d| {
                    let n = d.file_name();
                    let n = n.to_string_lossy();
                    n.starts_with("W2_") && n.ends_with("_candidatos")
                })
                .find_map(|d| std::fs::read(d.path().join(format!("{nome}.ph2d-brush.png"))).ok())
                .map(|b| ph2d_brush_presets::ler(&b).expect("o ficheiro do candidato lê-se"))
        })
        .unwrap_or_else(|| panic!("o preset {nome:?}: nem na fábrica nem candidato"))
}

pub(super) use controlos_e10::migra_a_receita;

/// O meio-alvo de todo preset desta família (o `media` do JSON).
pub const MEIO_DO_PRESET: &str = "WetPaint";
/// A pasta-raiz da família em `assets/brushes/` e em `~/.local/share/ph2d/brushes/`.
pub const MOTOR_DOS_PRESETS: &str = "WetPainter";
/// A marca da lei da E4 no mapa `engine` (`1`): `pigmentPerDab` é a parte de pigmento da receita e `intensity` a
/// carga (a quantidade). Ausente = um preset da lei velha, que [`migra_a_receita`] traduz ao carregar.
pub(super) const CHAVE_RECEITA: &str = "receitaE4";
/// A chave da borracha molhada no mapa `engine` (não é knob do registo: é o slider `erase` do motor). B35.9: a
/// escala nova (`erase' = 0,4·erase`, o Eraser fundido) mora noutra chave; a velha `erase` (0,4 nos 57 de
/// fábrica) fica sem leitor e o preset cai na fábrica 0,16 — a mesma força, ao bit. Limpa-a o B35.20.
pub(super) const CHAVE_ERASE: &str = "eraseStrength";
/// O **Glaze layering (K-M)** no mapa `engine` (não é knob do registo: é do traço, `km_glaze`). Gravado só LIGADO
/// (`1`): ausente = desligado, o de fábrica — nenhuma receita de fábrica o liga, e os ficheiros dela ficam ao byte.
pub(super) const CHAVE_GLAZE: &str = "glazeLayering";
/// O **Pigment mixing (K-M)** no mapa `engine` (do pincel, `km_mixing`, dono 30/10): ligado de fábrica, por isso
/// gravado só DESLIGADO (`0`) — ausente = ligado; os ficheiros de fábrica ficam ao byte.
pub(super) const CHAVE_KM: &str = "pigmentMixing";
/// A **Paint Texture** no mapa `engine` (do traço, `paper_visual`, dono 28/10): gravada só LIGADA, como o Glaze —
/// ausente = desligada, a de fábrica; os ficheiros de fábrica ficam ao byte.
pub(super) const CHAVE_TEXTURA: &str = "paintTexture";
/// A caixa **Real Oil** no mapa `engine` (do traço, `oleo_real`, dono 28/10): gravada só LIGADA, como o Glaze.
pub(super) const CHAVE_OLEO: &str = "realOil";
const TEXTURA_FORMA: &str = "shape";
const TEXTURA_GRAO: &str = "grain";
/// A imagem do Flow da Ragged Edge («Use as Flow»): só a Aquarela a lê. O Paper NÃO viaja — é da obra
/// (`CAMPOS_FORA`, `DA_OBRA`): o mesmo papel sob todos os pincéis.
const TEXTURA_FLUXO: &str = "flow";

/// Um `f32` para o JSON pelo menor texto que volta ao mesmo `f32` (`0.1`, não `0.10000000149011612`).
fn curto(v: f32) -> f64 {
    format!("{v}").parse().unwrap_or(f64::from(v))
}

/// A largura prometida (W2) de `prm`: a lei MEDIDA do preset; um ficheiro sem ela (de antes da W2) usa a do Wet
/// Round, o pincel de fábrica.
fn largura_do_preset(prm: &Parametros) -> super::wetpaint::largura::LeiDaLargura {
    super::wetpaint::largura::LeiDaLargura::de_tabela(&prm.width_table)
        .unwrap_or_else(super::wetpaint::largura::LeiDaLargura::de_omissao)
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
            PanelEvent::Click(id) if *id == ids::PAINTER_WETPAINT_PRESET_IMPORT => {
                PedidoPreset::Importar
            }
            PanelEvent::Click(id) if *id == ids::PAINTER_WETPAINT_PRESET_EXPORT => {
                PedidoPreset::Exportar
            }
            PanelEvent::Click(id) if *id == ids::PAINTER_WETPAINT_PRESET_DELETE => {
                PedidoPreset::Apagar
            }
            PanelEvent::Click(id) if *id == ids::PAINTER_WETPAINT_PRESET_RESET_ORDER => {
                PedidoPreset::ReporPosicoes
            }
            PanelEvent::SelectOption(id, v) if *id == ids::PAINTER_WETPAINT_PRESET_MOVE => {
                let mut it = v.split(ids::PRESET_MOVE_SEP);
                match (it.next(), it.next(), it.next(), it.next()) {
                    (Some(a), Some(b), Some(lado @ ("before" | "after")), None) => {
                        PedidoPreset::Mover {
                            arrastado: a.to_owned(),
                            alvo: b.to_owned(),
                            depois: lado == "after",
                        }
                    }
                    _ => return false,
                }
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

    /// Um volume VIVO pelo índice do painel (Size px · Water · Volume µm — o `Knob::Volume`, E10).
    fn aplicar_volume(&mut self, vol: usize, x: f64) {
        if vol >= 2 {
            self.paint.wetpaint.knobs.set(Knob::Volume, x);
            return;
        }
        let v = match vol {
            0 => Volumes {
                size_px: Some(x),
                ..Volumes::default()
            },
            _ => Volumes {
                water: Some(x),
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
        crate::macro_track::porta!(self, |_| crate::macro_track::Passo::TiraPedidoPreset, {
            self.take_wet_preset_request_corpo()
        })
    }

    fn take_wet_preset_request_corpo(&mut self) -> Option<PedidoPreset> {
        self.paint.wetpaint.preset_pedido.take()
    }

    /// **Aplica um preset do Wet Painter**: arma o meio, põe o pincel (os campos que o preset não fixa
    /// voltam ao de fábrica — um preset é o pincel inteiro, não um delta), os knobs, as máscaras, o modo
    /// e os volumes que ele fixa. Volume ausente = o de agora (o tamanho do artista sobrevive à troca).
    ///
    /// Devolve as texturas que o ficheiro pedia e não se leram (o slot fica sem imagem): quem chama
    /// di-lo ao artista — nunca em silêncio.
    pub fn wet_preset_aplicar(&mut self, p: &Preset) -> Vec<String> {
        crate::macro_track::porta!(
            self,
            |_| crate::macro_track::Passo::PresetAplica {
                parametros: serde_json::to_string(&p.parametros).unwrap_or_default(),
                texturas: p
                    .texturas
                    .iter()
                    .map(|(k, v)| (k.clone(), v.clone()))
                    .collect()
            },
            { self.wet_preset_aplicar_corpo(p) }
        )
    }

    fn wet_preset_aplicar_corpo(&mut self, p: &Preset) -> Vec<String> {
        self.repoe_o_pincel_fora_do_spec();
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
        // As chaves do material de antes da W3 (`roughnessMul` → Roughness `0,5·m`, `translucency` → Subsurface…);
        // a chave nova, se também vier, manda.
        for (chave, &v) in &prm.engine {
            if let Some((knob, v)) = ph2d_wet_paint::tuning::chave_antiga(chave, v) {
                k.set(knob, v);
            }
            for (knob, v) in ph2d_wet_paint::tuning::chave_antiga_atraves(chave, v)
                .into_iter()
                .flatten()
            {
                k.set(knob, v);
            }
        }
        // A Dry speed é da TELA (dono 01/11 Q3): o preset não a lê — fica a de agora.
        for def in KNOB_DEFS.iter().filter(|d| d.knob != Knob::Evaporation) {
            if let Some(&v) = prm.engine.get(def.key) {
                k.set(def.knob, v);
            }
        }
        if let Some(v) = self.paint.wetpaint.suave_da_macro_antiga
            && !prm
                .engine
                .contains_key(KNOB_DEFS[Knob::SmoothEdges as usize].key)
        {
            k.set(Knob::SmoothEdges, v);
        }
        k.set(
            Knob::Evaporation,
            self.paint.wetpaint.knobs.get(Knob::Evaporation),
        );
        if let Some(&v) = prm.engine.get(CHAVE_ERASE) {
            k.erase = if v.is_nan() {
                k.erase
            } else {
                v.clamp(0.0, 1.0)
            };
        }
        k.water = water;
        // A Opacity saiu (E10): a de um preset que não a fixa é a de fábrica.
        let opacidade = f64::from(OPACIDADE_DE_FABRICA);
        if !prm.engine.contains_key(CHAVE_RECEITA) {
            migra_a_receita(&mut k, prm, water, opacidade);
        }
        if !prm.engine.contains_key(controlos_e10::CHAVE_CONTROLOS) {
            controlos_e10::migra_para_controlos(&mut k, prm, water, opacidade);
        }
        // Um preset de antes da W2 do plano 27 traz as cerdas no mapa do MOTOR e o Shape vazio: elas passam
        // ao Shape, com os mesmos valores (o traço é o mesmo, ao bit — a miniatura de fábrica é o portão).
        // Um preset gravado depois já não as traz no mapa ([`Self::wet_preset_capturar`]).
        let traz_cerdas = ph2d_wet_paint::brush::BRISTLE_UNIT_KNOBS
            .iter()
            .any(|kn| prm.engine.contains_key(KNOB_DEFS[*kn as usize].key));
        if traz_cerdas && self.paint.brush.shape.kind == TextureKind::None {
            // Esse mapa antigo sempre leu o tecto de 4: o Fan Brush e o Express Oil trazem 7 e pintam 4.
            let (teto_antigo, _) = ph2d_wet_paint::brush::unidade_do_knob(Knob::BristleSize);
            k.set(Knob::BristleSize, k.get(Knob::BristleSize).min(teto_antigo));
            self.paint.brush.shape = k.cerdas_no_shape(self.paint.brush.shape);
        }
        // A velocidade (L12), a curva em S, o jitter e o atraso não são knobs do motor: moram na `Dynamics` do pincel,
        // que a mão do Wet herda. Ausente = a de fábrica, nunca a do pincel anterior.
        for l in super::mao::linhas_da_dinamica_do_wet() {
            let v = prm
                .engine
                .get(l.chave())
                .copied()
                .unwrap_or_else(|| l.omissao());
            super::mao::escreve(
                &mut self.paint.dynamics,
                &mut self.paint.brush.pen_tilt,
                l,
                l.limpa(v),
            );
        }
        // Uma ferramenta da água (dono 25/10) escreve só o CORPO dela (os knobs que o leitor dela lê) e os da mão: a
        // tela (a física, o papel, os efeitos da água) é partilhada. O corpo (pincel, máscaras, mão, knobs, tamanho,
        // largura) vai ao slot dela.
        // A Borracha também não pousa tinta (Bug #65 A5): a Hardness e a força dela; a receita do Pincel fica.
        let ferramenta = ferramenta_do_preset(prm);
        let borracha = ferramenta == crate::FerramentaDoPincel::Borracha;
        let corpo: Option<Vec<Knob>> = ferramenta
            .ferramenta_da_agua()
            .map(|t| WetKnobs::do_corpo(t).collect())
            .or_else(|| borracha.then(|| vec![Knob::PaperGate, Knob::Eraser]));
        if let Some(corpo) = corpo {
            for knob in corpo.into_iter().chain(super::mao::knobs_da_mao()) {
                self.paint.wetpaint.knobs.set(knob, k.get(knob));
            }
            if borracha {
                self.paint.wetpaint.knobs.erase = k.erase;
            }
            // A largura prometida é do corpo (cada ferramenta tem a sua, `switch_brush_slot`).
            self.paint.wetpaint.largura = Some(largura_do_preset(prm));
            let s = self.brush_settings();
            let v = Volumes {
                opacity: prm
                    .volumes
                    .opacity
                    .filter(|_| s.le(crate::LinhaDaFerramenta::DepositoMolhado)),
                water: prm
                    .volumes
                    .water
                    .filter(|_| s.le(crate::LinhaDaFerramenta::AguaDoVolume)),
                ..prm.volumes.clone()
            };
            self.wet_preset_aplicar_volumes(&v);
            self.set_mao_do_preset(Some(prm));
            self.paint.wetpaint.preset_base = self.paint.wetpaint.knobs;
            return falhas;
        }
        self.paint.wetpaint.knobs = k;
        // O Glaze é do traço (U1, auditoria W3b): um preset é o pincel inteiro — ausente = o de fábrica (desligado).
        self.paint.wetpaint.km_glaze = prm.engine.get(CHAVE_GLAZE).is_some_and(|v| *v >= 0.5);
        // O Pigment mixing é do pincel: ausente = o de fábrica (ligado).
        self.paint.wetpaint.km_mixing = prm.engine.get(CHAVE_KM).is_none_or(|v| *v >= 0.5);
        self.paint.wetpaint.paper_visual = prm.engine.get(CHAVE_TEXTURA).is_some_and(|v| *v >= 0.5);
        let w = &mut self.paint.wetpaint;
        w.oleo_real = prm
            .engine
            .get(CHAVE_OLEO)
            .is_some_and(|v| *v >= 0.5)
            .then_some(w.oleo_real_valores);

        // A ferramenta do fluido é do Pincel; a Borracha já está na mão (`pegar_a_ferramenta_do_preset`).
        if let Some(m) = prm.paint_mode.as_deref()
            && ferramenta_do_preset(prm) == crate::FerramentaDoPincel::Pincel
        {
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
        // A Shape Color (prompt 45): a do pincel, ou NENHUMA — ausente tira a do pincel anterior.
        let mut sc = super::shape_ramp::EstadoDaShapeColor::default();
        if let Some(v) = &prm.shape_color {
            sc.ler_json(v);
        }
        self.set_shape_color_estado(&sc);
        self.grain_color_de_json(prm.grain_color.as_ref());
        self.paint.wetpaint.largura = Some(largura_do_preset(prm));
        self.wet_preset_aplicar_volumes(&prm.volumes);
        self.set_mao_do_preset(Some(prm));
        // B35.8: o ↺ do Tuning volta a ESTE pincel.
        self.paint.wetpaint.preset_base = self.paint.wetpaint.knobs;
        falhas
    }

    /// Os volumes que `v` fixa (a política «lembrar volumes» passa por aqui também). Pressure e Length
    /// ainda não têm leitor (W6/W5): viajam no ficheiro e não mexem em nada.
    pub fn wet_preset_aplicar_volumes(&mut self, v: &Volumes) {
        crate::macro_track::porta!(
            self,
            |_| crate::macro_track::Passo::PresetVolumes(
                serde_json::to_string(v).unwrap_or_default()
            ),
            { self.wet_preset_aplicar_volumes_corpo(v) }
        )
    }

    fn wet_preset_aplicar_volumes_corpo(&mut self, v: &Volumes) {
        if let Some(d) = v.size_px.filter(|d| d.is_finite()) {
            self.set_brush_size_px(d as f32 * 0.5);
        }
        // A `opacity` de antes da E10 já entrou no Volume pela migração; os volumes vivos são Dilution e Volume.
        if let Some(w) = v.water.filter(|w| w.is_finite()) {
            self.paint.wetpaint.knobs.water = w.clamp(0.0, 1.0);
        }
        for knob in [Knob::Dilution, Knob::Volume] {
            if let Some(x) = v
                .extra
                .get(KNOB_DEFS[knob as usize].key)
                .and_then(serde_json::Value::as_f64)
            {
                self.paint.wetpaint.knobs.set(knob, x);
            }
        }
    }

    /// O modo (a ferramenta molhada) pelo nome que o preset guarda; um nome desconhecido não muda nada.
    pub fn wet_preset_aplicar_modo(&mut self, modo: &str) {
        crate::macro_track::porta!(
            self,
            |_| crate::macro_track::Passo::PresetModo(modo.to_owned()),
            { self.wet_preset_aplicar_modo_corpo(modo) }
        )
    }

    fn wet_preset_aplicar_modo_corpo(&mut self, modo: &str) {
        if let Some(t) = FERRAMENTAS.iter().find(|t| format!("{t:?}") == modo) {
            self.pick_wet_tool(indice_da_ferramenta(*t));
        }
    }

    /// Os volumes de agora, como o painel os mostra (Size px = o DIÂMETRO do anel); a Dilution e o Volume (E10)
    /// vão no `extra`, pela chave do knob — o que a política «lembrar volumes» guarda.
    #[must_use]
    pub fn wet_preset_volumes(&self) -> Volumes {
        let k = &self.paint.wetpaint.knobs;
        Volumes {
            size_px: Some(curto(self.paint.brush.radius_px * 2.0)),
            water: Some(k.water),
            extra: [Knob::Dilution, Knob::Volume]
                .map(|kn| (KNOB_DEFS[kn as usize].key.to_owned(), k.get(kn).into()))
                .into_iter()
                .collect(),
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
        p.brush.remove("grao_aplicado");
        // O campo «só quando ligado» que o pincel de agora desligou não fica da base (B35.A1).
        p.brush
            .retain(|k, _| !CAMPOS_DO_PINCEL.contains(&k.as_str()));
        p.brush.extend(json::pincel_para_json(
            &self.pincel_a_gravar(),
            &CAMPOS_DO_PINCEL,
        ));
        // O dente do papel saiu do pincel (W3b): o de um preset antigo não sobrevive à gravação.
        p.brush.remove("paper_depth");
        let k = &self.paint.wetpaint.knobs;
        // As cerdas moram no Shape (plano 27 W2): o mapa do motor não as grava — e a ausência delas é o
        // que diz ao [`Self::wet_preset_aplicar`] que o Shape do ficheiro já é a verdade.
        for def in KNOB_DEFS
            .iter()
            .filter(|d| !ph2d_wet_paint::brush::BRISTLE_UNIT_KNOBS.contains(&d.knob))
        {
            p.engine.insert(def.key.to_owned(), k.get(def.knob));
        }
        p.engine.remove(KNOB_DEFS[Knob::Evaporation as usize].key); // da tela (Q3)
        p.engine
            .insert(controlos_e10::CHAVE_CONTROLOS.to_owned(), 1.0);
        for kn in ph2d_wet_paint::brush::BRISTLE_UNIT_KNOBS {
            p.engine.remove(KNOB_DEFS[kn as usize].key);
        }
        p.engine.insert(CHAVE_ERASE.to_owned(), k.erase);
        p.engine.insert(CHAVE_RECEITA.to_owned(), 1.0);
        if self.paint.wetpaint.km_glaze {
            p.engine.insert(CHAVE_GLAZE.to_owned(), 1.0);
        } else {
            p.engine.remove(CHAVE_GLAZE);
        }
        if self.paint.wetpaint.km_mixing {
            p.engine.remove(CHAVE_KM);
        } else {
            p.engine.insert(CHAVE_KM.to_owned(), 0.0);
        }
        if self.paint.wetpaint.paper_visual {
            p.engine.insert(CHAVE_TEXTURA.to_owned(), 1.0);
        } else {
            p.engine.remove(CHAVE_TEXTURA);
        }
        if self.paint.wetpaint.oleo_real.is_some() {
            p.engine.insert(CHAVE_OLEO.to_owned(), 1.0);
        } else {
            p.engine.remove(CHAVE_OLEO);
        }
        // As linhas da `Dynamics` sem knob só quando o pincel as mexe: o preset sem elas é o de antes, ao byte.
        for l in super::mao::linhas_da_dinamica_do_wet() {
            let v = super::mao::le(&self.paint.dynamics, &self.paint.brush.pen_tilt, l);
            if v == l.omissao() {
                p.engine.remove(l.chave());
            } else {
                p.engine.insert(l.chave().to_owned(), v);
            }
        }
        let v = self.wet_preset_volumes();
        p.volumes.size_px = v.size_px;
        p.volumes.opacity = None; // a Opacity saiu: o Volume vai no `engine` (E10)
        p.volumes.water = v.water;
        let ferramenta = self.ferramenta_dos_presets().and_then(nome_da_ferramenta);
        p.paint_mode = ferramenta.is_none().then(|| self.wet_preset_modo());
        p.ferramenta = ferramenta.map(str::to_owned);
        let w = &self.paint.wetpaint;
        p.tilt = Some(ph2d_brush_presets::Tilt {
            on: w.tilt_on,
            ring: w.tilt_ring,
            spoke: w.tilt_spoke,
        });
        p.live_water = Some(w.agua_viva);
        p.heavy_edge = Some(w.borda_que_pesa);
        p.shape_color = self.shape_color_estado().para_json_se_mexida();
        p.grain_color = self.grain_color_para_json();
        // A lei da largura do pincel de onde ele veio (W2): um pincel mexido guarda a do preset-base.
        if let Some(lei) = &w.largura {
            p.width_table = lei.tabela();
        }
        (p, self.texturas_do_pincel())
    }
}
