//! **O preset nos QUATRO meios** (L1, `docs/Painter/Pinceis/03_MOTORES.md` §2): o `media` do ficheiro é LIDO e
//! respeitado — o Wet Paint pelo caminho de sempre, ao bit ([`super`]); Digital, Watercolor e Impasto pelo
//! deste ficheiro; um meio desconhecido é RECUSADO ([`MeioDesconhecido`]), nunca pintado no Digital calado.
//!
//! O que cada meio grava sai do CENSO do [`super::CAMPOS_FORA`] (o porquê de cada campo), sem lista à mão: o
//! carimbo inteiro ([`super::CAMPOS_DO_PINCEL`]), o gesto e o Flow/Accumulate, mais a família do próprio
//! meio (os campos da aguada OU os do impasto). Ficam fora o volume, a obra, a porta do meio e a família do
//! outro. A mão (pressão, inclinação) vai no mapa `engine` pelas chaves dos knobs da caneta do Wet
//! ([`crate::tool::paint::mao`]) — o mesmo sítio e os mesmos nomes nos quatro meios.

use super::super::mao::{self, LINHAS_DA_MAO};
use super::super::{PaintMedia, PaintMode};
use super::{
    AQUARELA, CAMPOS_DO_PINCEL, CAMPOS_FORA, DA_OBRA, DIGITAL, DO_MEIO, DO_SCULPT, DOS_KNOBS,
    IMPASTO, MOTOR_DOS_PRESETS, PLANO, POR_DAB, VOLUME, curto, json,
};
use crate::PainterTool;
use ph2d_brush_presets::{Parametros, Preset, SCHEMA, Volumes};
use std::collections::BTreeMap;

/// O texto do `media` de cada meio (o nome da variante: `"WetPaint"` é o de sempre).
#[must_use]
pub fn nome_do_meio(m: PaintMedia) -> &'static str {
    match m {
        PaintMedia::Digital => "Digital",
        PaintMedia::Watercolor => "Watercolor",
        PaintMedia::Impasto => "Impasto",
        PaintMedia::WetPaint => super::MEIO_DO_PRESET,
    }
}

/// O meio que o `media` de um preset nomeia; `None` = desconhecido (recusado).
#[must_use]
pub fn meio_do_preset(media: &str) -> Option<PaintMedia> {
    (0..PaintMedia::COUNT)
        .map(PaintMedia::from_u8)
        .find(|m| nome_do_meio(*m) == media)
}

/// A pasta-raiz dos presets de cada meio em `assets/brushes/` e na pasta do artista — irmãs de
/// [`MOTOR_DOS_PRESETS`] (`WetPainter`). Nenhum caminho gravado muda.
#[must_use]
pub fn motor_do_meio(m: PaintMedia) -> &'static str {
    match m {
        PaintMedia::WetPaint => MOTOR_DOS_PRESETS,
        outro => nome_do_meio(outro),
    }
}

/// Um preset cujo `media` nenhum meio deste PH2D conhece — o texto, para o aviso ao artista.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MeioDesconhecido(pub String);

/// **Os campos do pincel que o preset de `m` grava**, pelo censo dos porquês (ver o doc do módulo).
#[must_use]
pub fn campos_do_meio(m: PaintMedia) -> Vec<&'static str> {
    if m == PaintMedia::WetPaint {
        return CAMPOS_DO_PINCEL.to_vec();
    }
    let fora: &[&str] = match m {
        PaintMedia::Watercolor => &[
            VOLUME, DA_OBRA, DO_MEIO, DO_SCULPT, POR_DAB, DOS_KNOBS, IMPASTO, DIGITAL, PLANO,
        ],
        PaintMedia::Impasto => &[
            VOLUME, DA_OBRA, DO_MEIO, DO_SCULPT, POR_DAB, DOS_KNOBS, AQUARELA, DIGITAL, PLANO,
        ],
        _ => &[
            VOLUME, DA_OBRA, DO_MEIO, DO_SCULPT, POR_DAB, DOS_KNOBS, AQUARELA, IMPASTO,
        ],
    };
    CAMPOS_DO_PINCEL
        .iter()
        .copied()
        .chain(
            CAMPOS_FORA
                .iter()
                .filter(|(_, porque)| !fora.contains(porque))
                .map(|(c, _)| *c),
        )
        .collect()
}

/// As dez ferramentas do Impasto pelo nome (a ordem de `impasto_tool.rs`: Deposit · Knife · os oito verbos).
const FERRAMENTAS_DO_IMPASTO: [&str; 10] = [
    "Deposit", "Knife", "Smooth", "Sharpen", "Flatten", "Scrape", "Fill", "Chisel", "Layer",
    "Inflate",
];

/// Os modos de pintura que um preset do Digital/Watercolor pode fixar (o resto não é um pincel: Clone, Mask…).
const MODOS: [(PaintMode, &str, &str); 3] = [
    (PaintMode::Paint, "Paint", "brush"),
    (PaintMode::Smear, "Smear", "smear"),
    (PaintMode::Blur, "Blur", "blur"),
];

impl PainterTool {
    /// ⭐ **Aplica um preset de QUALQUER meio** — a porta que o navegador, a miniatura, a bancada e a macro
    /// usam: arma o meio do ficheiro (clicar num preset de outro meio TROCA o meio) e põe o pincel inteiro.
    /// Devolve o que o ficheiro pedia e não se aplicou (texturas ilegíveis, um modo que o meio não tem) para
    /// quem chama o dizer.
    ///
    /// # Errors
    /// [`MeioDesconhecido`] — nada muda no pincel nem no meio.
    pub fn preset_aplicar(&mut self, p: &Preset) -> Result<Vec<String>, MeioDesconhecido> {
        let Some(m) = meio_do_preset(&p.parametros.media) else {
            return Err(MeioDesconhecido(p.parametros.media.clone()));
        };
        Ok(crate::macro_track::porta!(
            self,
            |_| crate::macro_track::Passo::PresetAplica {
                parametros: serde_json::to_string(&p.parametros).unwrap_or_default(),
                texturas: p
                    .texturas
                    .iter()
                    .map(|(k, v)| (k.clone(), v.clone()))
                    .collect()
            },
            {
                if m == PaintMedia::WetPaint {
                    self.wet_preset_aplicar_corpo(p)
                } else {
                    self.meio_preset_aplicar_corpo(m, p)
                }
            }
        ))
    }

    /// ⭐ **O que o pincel guarda FORA do `BrushSpec` volta ao de fábrica** antes de um preset: a pilha do
    /// Composite, a Shape Color e as Grain Colors. Escolher um preset dá ESSE pincel, nunca ele com a pilha ou as
    /// rampas do anterior (medido 22/10, `o_preset_nao_herda`: o Composite sobrevivia aos presets de ficheiro e ao
    /// «Basic»; a Shape Color ao «Basic»). A porta única das três entradas de preset.
    pub(in crate::tool::paint) fn repoe_o_pincel_fora_do_spec(&mut self) {
        let p = &mut self.paint;
        p.composite_enabled = false;
        p.composite_len = 0;
        p.composite = std::array::from_fn(|_| Default::default());
        p.composite_add_op = super::super::CompositeOp::Brush;
        self.set_shape_color_estado(&Default::default());
        self.grain_color_de_json(None);
    }

    fn meio_preset_aplicar_corpo(&mut self, m: PaintMedia, p: &Preset) -> Vec<String> {
        self.repoe_o_pincel_fora_do_spec();
        if self.paint_media() != m {
            self.set_paint_media(m);
        }
        let prm = &p.parametros;
        let mut falhas = Vec::new();
        // O modo ANTES do pincel: cada modo tem o seu slot de pincel (`brush_by_mode`).
        if let Some(modo) = prm.paint_mode.as_deref()
            && !self.meio_aplicar_modo(m, modo)
        {
            falhas.push(format!("paint_mode:{modo}"));
        }
        let campos = campos_do_meio(m);
        let de_fabrica = json::pincel_de_json(
            &prm.brush,
            &campos,
            ph2d_painter_brush::BrushSpec::default(),
        );
        self.paint.brush = json::pincel_de_json(
            &json::pincel_para_json(&de_fabrica, &campos),
            &campos,
            self.paint.brush,
        );
        self.settle_stroke_method();
        falhas.extend(self.instalar_mascaras(&p.texturas));
        // A mão: a de fábrica e, por cima, a que o ficheiro fixa (só as linhas que o meio lê).
        self.repor_mao();
        for l in LINHAS_DA_MAO.into_iter().filter(|l| l.lida_por(m)) {
            if let Some(&v) = prm.engine.get(l.chave()) {
                mao::escreve(
                    &mut self.paint.dynamics,
                    &mut self.paint.brush.pen_tilt,
                    l,
                    l.limpa(v),
                );
            }
        }
        let mut sc = super::super::shape_ramp::EstadoDaShapeColor::default();
        if let Some(v) = &prm.shape_color {
            sc.ler_json(v);
        }
        self.set_shape_color_estado(&sc);
        self.grain_color_de_json(prm.grain_color.as_ref());
        self.meio_aplicar_volumes(m, &prm.volumes);
        falhas
    }

    /// O modo pelo nome no meio `m` (`false` = o meio não o tem; nada muda).
    fn meio_aplicar_modo(&mut self, m: PaintMedia, modo: &str) -> bool {
        match m {
            PaintMedia::Impasto => match FERRAMENTAS_DO_IMPASTO.iter().position(|f| *f == modo) {
                Some(i) => {
                    self.set_impasto_tool(i as u8);
                    true
                }
                None => false,
            },
            PaintMedia::WetPaint => {
                self.wet_preset_aplicar_modo_corpo(modo);
                true
            }
            _ => match MODOS.iter().find(|(pm, nome, _)| {
                *nome == modo && (m == PaintMedia::Digital || m.works_in(*pm))
            }) {
                Some((_, _, fio)) => {
                    self.set_paint_tool_mode(fio);
                    true
                }
                None => false,
            },
        }
    }

    /// O modo de agora como o preset do meio `m` o grava (`None` = um modo que não é de pincel).
    fn meio_modo(&self, m: PaintMedia) -> Option<String> {
        match m {
            PaintMedia::WetPaint => Some(self.wet_preset_modo()),
            PaintMedia::Impasto => FERRAMENTAS_DO_IMPASTO
                .get(usize::from(self.impasto_tool()))
                .map(|s| (*s).to_owned()),
            _ => MODOS
                .iter()
                .find(|(pm, _, _)| *pm == self.paint.paint_mode)
                .map(|(_, nome, _)| (*nome).to_owned()),
        }
    }

    /// Os volumes de um meio antigo: o diâmetro e a Strength (a Watercolor não lê a Strength: dono 12/08).
    fn meio_aplicar_volumes(&mut self, m: PaintMedia, v: &Volumes) {
        if let Some(d) = v.size_px.filter(|d| d.is_finite()) {
            self.set_brush_size_px(d as f32 * 0.5);
        }
        if m != PaintMedia::Watercolor
            && let Some(o) = v.opacity.filter(|o| o.is_finite())
        {
            self.set_brush_strength(o as f32);
        }
    }

    /// Os volumes de agora no meio de agora (o que a política «lembrar» guarda).
    #[must_use]
    pub fn preset_volumes(&self) -> Volumes {
        match self.paint_media() {
            PaintMedia::WetPaint => self.wet_preset_volumes(),
            m => Volumes {
                size_px: Some(curto(self.paint.brush.radius_px * 2.0)),
                opacity: (m != PaintMedia::Watercolor).then(|| curto(self.paint.brush.strength)),
                ..Volumes::default()
            },
        }
    }

    /// Os volumes que `v` fixa, no meio de agora (a política «lembrar volumes» passa por aqui).
    pub fn preset_aplicar_volumes(&mut self, v: &Volumes) {
        crate::macro_track::porta!(
            self,
            |_| crate::macro_track::Passo::PresetVolumes(
                serde_json::to_string(v).unwrap_or_default()
            ),
            {
                match self.paint_media() {
                    PaintMedia::WetPaint => self.wet_preset_aplicar_volumes_corpo(v),
                    m => self.meio_aplicar_volumes(m, v),
                }
            }
        );
    }

    /// O modo de agora pelo nome que o preset do meio de agora guarda.
    #[must_use]
    pub fn preset_modo(&self) -> Option<String> {
        self.meio_modo(self.paint_media())
    }

    /// O modo pelo nome, no meio de agora; um nome que o meio não tem não muda nada.
    pub fn preset_aplicar_modo(&mut self, modo: &str) {
        crate::macro_track::porta!(
            self,
            |_| crate::macro_track::Passo::PresetModo(modo.to_owned()),
            {
                let m = self.paint_media();
                self.meio_aplicar_modo(m, modo);
            }
        );
    }

    /// ⭐ **Captura o pincel de agora no meio de agora** sobre `base` (o preset de onde ele veio): o do Wet
    /// Paint é o [`Self::wet_preset_capturar`] ao bit; nos outros, os campos do meio, a mão, o modo, a Shape
    /// Color, as máscaras e os volumes. Um `base` de OUTRO meio só empresta o nome e o grupo.
    #[must_use]
    pub fn preset_capturar(&self, base: &Parametros) -> (Parametros, BTreeMap<String, Vec<u8>>) {
        let m = self.paint_media();
        if m == PaintMedia::WetPaint && meio_do_preset(&base.media) != Some(m) {
            return self.wet_preset_capturar(&Parametros::novo(
                nome_do_meio(m),
                &base.group,
                &base.name,
            ));
        }
        if m == PaintMedia::WetPaint {
            return self.wet_preset_capturar(base);
        }
        let mut p = if meio_do_preset(&base.media) == Some(m) {
            base.clone()
        } else {
            Parametros::novo(nome_do_meio(m), &base.group, &base.name)
        };
        p.schema = SCHEMA;
        p.brush.extend(json::pincel_para_json(
            &self.paint.brush,
            &campos_do_meio(m),
        ));
        for l in LINHAS_DA_MAO {
            let v = mao::le(&self.paint.dynamics, &self.paint.brush.pen_tilt, l);
            if l.lida_por(m) && (l.grava_no_neutro() || v != l.omissao()) {
                p.engine.insert(l.chave().to_owned(), v);
            } else {
                p.engine.remove(l.chave());
            }
        }
        let v = self.preset_volumes();
        p.volumes.size_px = v.size_px;
        p.volumes.opacity = v.opacity;
        p.paint_mode = self.meio_modo(m);
        p.shape_color = self.shape_color_estado().para_json_se_mexida();
        p.grain_color = self.grain_color_para_json();
        (p, self.texturas_do_pincel())
    }
}
