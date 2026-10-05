//! **A GEOMETRIA de uma forma, preparada UMA vez** — o preenchimento e o contorno do traço
//! aplanados em segmentos, em [`LEVELS`] níveis de detalhe.
//!
//! ⭐ **Por que níveis, e não um aplanamento só.** O passe desenha cada cópia a um tamanho de ecrã
//! diferente, e o erro de um aplanamento cresce com a escala: uma tolerância que é invisível numa
//! cópia de `20 px` é uma quina visível numa de `2 000 px`. O shader de vértice escolhe, por cópia,
//! o nível mais GROSSO cujo erro no ecrã é `≤ 0,25 px` — **a tolerância do próprio Vello**
//! (`vello_shaders/shader/flatten.wgsl`: `let tol = 0.25`). ⇒ o erro nunca passa do que o
//! desenhador de hoje já aceita, e uma cópia pequena paga poucos segmentos.
//!
//! ⚠️ **As tolerâncias são RELATIVAS à extensão da forma**, porque a geometria vive em unidades
//! LOCAIS (a do `source.shape` tem extensão `1`) e o que decide o erro é o produto
//! `tolerância × escala`. Com [`TOL_BASE`] `= 1/16` e passo `1/4`, o nível mais fino é `1/262 144`
//! da extensão — `0,25 px` numa forma de `65 536 px` de lado.

use crate::eixo::{EixoItem, eixo};
use bytemuck::{Pod, Zeroable};
use ph2d_vector::{BezPath, PathEl, Point, Shape, Stroke, StrokeOpts, expand_stroke, flatten};

/// Quantos níveis de detalhe cada geometria leva.
pub const LEVELS: usize = 8;
/// A tolerância do nível `0`, como fracção da extensão da forma.
pub const TOL_BASE: f64 = 1.0 / 16.0;
/// O factor entre um nível e o seguinte. ⚠️ `1/4` e não `1/2`: o número de segmentos de uma
/// curva aplanada cresce com `1/√tol`, logo cada nível DOBRA os segmentos — o mesmo passo que o
/// shader dá na escala.
pub const TOL_STEP: f64 = 0.25;

/// `flags` bit 0: a regra even-odd.
pub const FLAG_EVEN_ODD: u32 = 1;
/// `flags` bit 1: **o traço só se desenha sob afim CONFORME** — um tracejado que o eixo não exprime
/// (outro que `[traço, vão]` com fase `0`, [`crate::eixo::tracejado_do_eixo`]); sob escala não
/// uniforme ele mede-se no MUNDO e o eixo é quem o faz. Quem chama manda as cópias não conformes
/// dele ao Vello.
pub const FLAG_SO_CONFORME: u32 = 2;

/// A regra de preenchimento — a mesma escolha que o Vello recebe (`Fill`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FillRule {
    NonZero,
    EvenOdd,
}

/// O TRAÇO de uma forma: a linha que ele segue, o estilo (largura · pontas · juntas · tracejado)
/// e a cor.
pub struct StrokeInput<'a> {
    pub path: &'a BezPath,
    pub style: &'a Stroke,
    /// RGBA **não** pré-multiplicado, no espaço de cor em que o Vello a recebe.
    pub color: [f32; 4],
}

/// O que uma forma desenha: o preenchimento (com a cor da CÓPIA) e, por cima, o traço (com a
/// cor dele).
pub struct ShapeInput<'a> {
    pub fill: Option<(&'a BezPath, FillRule)>,
    /// Peças de traço, desenhadas pela ordem. ⚠️ Um só traço é o caso comum; as pontas de um
    /// traço com marcadores são peças a mais, com a MESMA cor.
    pub strokes: Vec<StrokeInput<'a>>,
    /// Preenchimentos que pintam com a cor do TRAÇO (os marcadores cheios de uma ponta).
    pub stroke_fills: Vec<&'a BezPath>,
}

/// O registo de uma geometria, como a placa o lê (`shape.wgsl`, `Record`).
#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct GeometryRecord {
    /// A caixa LOCAL que cobre o preenchimento e o contorno do traço: `x0, y0, x1, y1`.
    pub bbox: [f32; 4],
    /// A cor do traço, não pré-multiplicada.
    pub stroke_color: [f32; 4],
    /// A tolerância LOCAL de cada nível.
    pub tol: [f32; LEVELS],
    /// Por nível: `fill_start, fill_count, stroke_start, stroke_count`, em segmentos. Relativos a
    /// esta geometria até ao upload, que os torna absolutos. ⚠️ O traço começa pelas MARCAS (os
    /// preenchimentos com a cor dele) e segue com o contorno expandido.
    pub ranges: [[u32; 4]; LEVELS],
    /// Por nível: `eixo_start, eixo_count, marcas_count, 0` — os itens do [`crate::eixo`] (em
    /// itens, relativos até ao upload) e quantos dos segmentos do traço são MARCAS.
    pub eixo: [[u32; 4]; LEVELS],
    /// A caixa LOCAL dos pontos do eixo — o shader alarga-a no ecrã pela caneta de cada cópia.
    pub eixo_bbox: [f32; 4],
    /// [`FLAG_EVEN_ODD`] · [`FLAG_SO_CONFORME`].
    pub flags: u32,
    /// Quanto o traço vai para FORA do eixo, em unidades locais de LARGURA (`meia × esquadria`), antes
    /// da caneta da cópia (`× √|det|`).
    pub ext_fora: f32,
    pub _pad: [u32; 2],
}

/// Uma geometria pronta: o registo e os segmentos de todos os níveis, em espaço LOCAL.
pub struct ShapeGeometry {
    pub record: GeometryRecord,
    /// `[x0, y0, x1, y1]` por segmento, cada trecho completado até um múltiplo de
    /// [`crate::SEGS_POR_BLOCO`] ([`crate::blocos`]).
    pub segments: Vec<[f32; 4]>,
    /// Um bloco por [`crate::SEGS_POR_BLOCO`] segmentos — o que o shader pergunta primeiro.
    pub blocos: Vec<crate::BlocoDeSegmentos>,
    /// O eixo do traço de todos os níveis ([`crate::eixo`]).
    pub eixo: Vec<EixoItem>,
}

impl ShapeGeometry {
    /// Prepara uma forma. `None` quando ela não desenha nada (sem preenchimento nem traço, ou
    /// uma caixa vazia).
    #[must_use]
    pub fn prepare(input: &ShapeInput<'_>) -> Option<Self> {
        let ext = extensao(input)?;
        let mut segments = Vec::new();
        let mut itens: Vec<EixoItem> = Vec::new();
        let mut record = GeometryRecord::zeroed();
        let mut caixa = Caixa::vazia();
        let mut ext_fora: f32 = 0.0;
        // ⭐ doc 121 §9.9 — o eixo percorre o tracejado `[traço, vão]` da casa; um padrão que ele não
        // exprime fica com a cerca de sempre (só conforme).
        let tracejado = input.strokes.iter().any(|s| {
            !s.style.dash_pattern.is_empty() && crate::eixo::tracejado_do_eixo(s.style).is_none()
        });
        for nivel in 0..LEVELS {
            let tol = tolerancia(ext, nivel);
            #[expect(clippy::cast_possible_truncation, reason = "a placa lê f32")]
            {
                record.tol[nivel] = tol as f32;
            }
            let fill_start = conta(&segments);
            if let Some((bp, _)) = input.fill {
                aplana_fechado(bp.iter(), tol, &mut segments, &mut caixa);
            }
            crate::blocos::completa(&mut segments);
            let fill_count = conta(&segments) - fill_start;
            let stroke_start = conta(&segments);
            // As MARCAS primeiro: sob afim não conforme o shader lê-as sozinhas (o contorno
            // expandido dá lugar ao eixo), e a regra não-nula não depende da ordem.
            for bp in &input.stroke_fills {
                aplana_fechado(bp.iter(), tol, &mut segments, &mut caixa);
            }
            crate::blocos::completa(&mut segments);
            let marcas = conta(&segments) - stroke_start;
            for contorno in contornos_dos_tracos(input, tol) {
                aplana_fechado(contorno.iter(), tol * 0.5, &mut segments, &mut caixa);
            }
            crate::blocos::completa(&mut segments);
            let stroke_count = conta(&segments) - stroke_start;
            record.ranges[nivel] = [fill_start, fill_count, stroke_start, stroke_count];
            let eixo_start = conta_eixo(&itens);
            if !tracejado {
                let (pecas, fora) = eixo_do_nivel(input, tol);
                ext_fora = ext_fora.max(fora);
                itens.extend(crate::eixo::em_blocos(&pecas));
            }
            record.eixo[nivel] = [eixo_start, conta_eixo(&itens) - eixo_start, marcas, 0];
        }
        if segments.is_empty() {
            return None;
        }
        record.bbox = caixa.como_f32();
        record.stroke_color = input.strokes.first().map_or([0.0; 4], |s| s.color);
        let mut ceixo = Caixa::vazia();
        for it in &itens {
            for q in [it.a, it.b] {
                ceixo.inclui(Point::new(f64::from(q[0]), f64::from(q[1])));
            }
        }
        record.eixo_bbox = if itens.is_empty() {
            record.bbox
        } else {
            ceixo.como_f32()
        };
        record.ext_fora = ext_fora;
        record.flags = if matches!(input.fill, Some((_, FillRule::EvenOdd))) {
            FLAG_EVEN_ODD
        } else {
            0
        } | if tracejado { FLAG_SO_CONFORME } else { 0 };
        Some(Self {
            record,
            blocos: crate::blocos::blocos_de(&segments),
            segments,
            eixo: itens,
        })
    }
}

/// A tolerância LOCAL do nível `nivel` de uma forma de extensão `ext` (o `record.tol`, em `f64`).
#[must_use]
pub fn tolerancia(ext: f64, nivel: usize) -> f64 {
    #[expect(
        clippy::cast_possible_wrap,
        clippy::cast_possible_truncation,
        reason = "LEVELS é oito"
    )]
    let k = nivel as i32;
    ext * TOL_BASE * TOL_STEP.powi(k)
}

/// O eixo de TODOS os traços de `input` aplanado à tolerância de nível `tol` (sem os cabeçalhos de bloco)
/// e quanto ele vai para fora (`ext_fora`, em unidades locais).
#[must_use]
pub fn eixo_do_nivel(input: &ShapeInput<'_>, tol: f64) -> (Vec<EixoItem>, f32) {
    let mut pecas = Vec::new();
    let mut fora: f32 = 0.0;
    for s in &input.strokes {
        #[expect(clippy::cast_possible_truncation, reason = "a placa lê f32")]
        let meia = (s.style.width * 0.5) as f32;
        fora = fora.max(meia * eixo(s.path, s.style, tol * 0.5, &mut pecas));
    }
    (pecas, fora)
}

/// O contorno de cada traço de `input` no nível de tolerância `tol` — metade do orçamento para a expansão e
/// metade para o aplanamento: os dois erros SOMAM-se, e o nível promete `tol` no total.
fn contornos_dos_tracos<'a>(
    input: &'a ShapeInput<'_>,
    tol: f64,
) -> impl Iterator<Item = BezPath> + 'a {
    input
        .strokes
        .iter()
        .map(move |s| expand_stroke(s.path.iter(), s.style, &StrokeOpts::default(), tol * 0.5))
}

/// ⭐ doc 121 §9.18 (C) — **o contorno do traço de uma cópia CONFORME no nível `tol`**, aplanado como as
/// marcas da placa (os MESMOS pontos, em `f64`): a rota Vello do Motion preenche-o em vez de traçar.
#[must_use]
pub fn contorno_conforme(input: &ShapeInput<'_>, tol: f64) -> BezPath {
    let mut bp = BezPath::new();
    for c in contornos_dos_tracos(input, tol) {
        flatten(c.iter(), tol * 0.5, |el| bp.push(el));
    }
    bp
}

fn conta_eixo(v: &[EixoItem]) -> u32 {
    u32::try_from(v.len()).expect("um eixo com mais de 4 mil milhoes de itens")
}

fn conta(v: &[[f32; 4]]) -> u32 {
    u32::try_from(v.len()).expect("uma geometria com mais de 4 mil milhoes de segmentos")
}

/// A extensão da forma — o maior lado da caixa do preenchimento e do traço (com a metade da
/// largura, que é o que o contorno acrescenta). `None` para uma forma vazia.
#[must_use]
pub fn extensao(input: &ShapeInput<'_>) -> Option<f64> {
    let mut ext: f64 = 0.0;
    let mut algo = false;
    let mut conta = |bp: &BezPath, folga: f64| {
        if bp.elements().is_empty() {
            return;
        }
        let r = bp.bounding_box();
        ext = ext
            .max(r.width() + 2.0 * folga)
            .max(r.height() + 2.0 * folga);
        algo = true;
    };
    if let Some((bp, _)) = input.fill {
        conta(bp, 0.0);
    }
    for s in &input.strokes {
        conta(s.path, s.style.width * 0.5);
    }
    for bp in &input.stroke_fills {
        conta(bp, 0.0);
    }
    (algo && ext > 0.0 && ext.is_finite()).then_some(ext)
}

/// Aplana `path` e acrescenta os segmentos, com cada sub-caminho FECHADO — a regra do Vello para
/// um preenchimento (um sub-caminho aberto fecha-se implicitamente).
///
/// ⛔⛔ **Um segmento HORIZONTAL no espaço LOCAL entra, e é MEDIDO:** a 1.ª redacção saltava-os
/// (*«a contribuição deles é `dy = 0`, exacta»*) — verdade só no referencial em que o shader os
/// avalia, que é o ECRÃ. Uma cópia RODADA põe a aresta de cima de um rectângulo arredondado a
/// atravessar linhas de pixel, e sem ela o contorno fica ABERTO: riscos horizontais na caixa da
/// forma (gate de paridade do PRODUTO, `ph2d-app-motion`, 2026-09-29 — o gate desta crate não os via
/// porque nenhuma das suas formas tinha uma aresta horizontal). Só sai o segmento de comprimento
/// ZERO, que não tem direcção em referencial nenhum.
fn aplana_fechado(
    path: impl IntoIterator<Item = PathEl>,
    tol: f64,
    out: &mut Vec<[f32; 4]>,
    caixa: &mut Caixa,
) {
    let mut inicio: Option<Point> = None;
    let mut ultimo: Option<Point> = None;
    let mut empurra = |a: Point, b: Point, caixa: &mut Caixa| {
        caixa.inclui(a);
        caixa.inclui(b);
        if a != b {
            #[expect(clippy::cast_possible_truncation, reason = "a placa lê f32")]
            out.push([a.x as f32, a.y as f32, b.x as f32, b.y as f32]);
        }
    };
    flatten(path, tol, |el| match el {
        PathEl::MoveTo(p) => {
            if let (Some(s), Some(u)) = (inicio, ultimo)
                && s != u
            {
                empurra(u, s, caixa);
            }
            inicio = Some(p);
            ultimo = Some(p);
        }
        PathEl::LineTo(p) => {
            if let Some(u) = ultimo {
                empurra(u, p, caixa);
            }
            ultimo = Some(p);
        }
        PathEl::ClosePath => {
            if let (Some(s), Some(u)) = (inicio, ultimo)
                && s != u
            {
                empurra(u, s, caixa);
            }
            ultimo = inicio;
        }
        // O `flatten` só emite os três acima.
        PathEl::QuadTo(..) | PathEl::CurveTo(..) => {}
    });
    if let (Some(s), Some(u)) = (inicio, ultimo)
        && s != u
    {
        empurra(u, s, caixa);
    }
}

struct Caixa {
    x0: f64,
    y0: f64,
    x1: f64,
    y1: f64,
}

impl Caixa {
    fn vazia() -> Self {
        Self {
            x0: f64::INFINITY,
            y0: f64::INFINITY,
            x1: f64::NEG_INFINITY,
            y1: f64::NEG_INFINITY,
        }
    }
    fn inclui(&mut self, p: Point) {
        self.x0 = self.x0.min(p.x);
        self.y0 = self.y0.min(p.y);
        self.x1 = self.x1.max(p.x);
        self.y1 = self.y1.max(p.y);
    }
    #[expect(clippy::cast_possible_truncation, reason = "a placa lê f32")]
    fn como_f32(&self) -> [f32; 4] {
        [
            self.x0 as f32,
            self.y0 as f32,
            self.x1 as f32,
            self.y1 as f32,
        ]
    }
}

#[cfg(test)]
#[path = "geometry_tests.rs"]
mod tests;
