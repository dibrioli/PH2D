//! **As ALÇAS de cada forma no canvas** (F9) — os parâmetros saem do painel e passam a
//! agarrar-se SOBRE a forma, como as alças vermelhas do Affinity e os *Live Shapes* do
//! Illustrator. Lei, oráculos e recusas: `docs/UI_New_and_Simple/vetor_nova_ui/13_alcas_das_formas.md`.
//!
//! ⭐ **A alça É um ponto do contorno COZIDO** ([`Spot`]) — nunca uma fórmula re-derivada ao
//! lado da geometria: o que se agarra e o que se desenha saem do MESMO `cook`. O arrasto é uma
//! MÉTRICA ([`Law`]) com que esse ponto persegue a mão, dentro da faixa do descritor — as
//! métricas foram medidas no Inkscape, não escolhidas.
//!
//! Tudo vive no espaço LOCAL da forma (o do `cook`, centrado na origem, `half` = a meia-caixa):
//! quem chama leva o ponteiro para lá pelo inverso do afim do objeto, e as alças seguem rotação
//! e escala de graça.

use ph2d_vec_scene::{ShapeKind, ShapeValues, VecPath};

use crate::shapes::{self, FieldUnit};

/// Onde uma alça fica: um ponto do contorno que o `cook` devolve.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Spot {
    /// O vértice `i` do contorno principal.
    Vertex(usize),
    /// O vértice `i` contado do FIM do contorno principal (`0` = o último).
    FromEnd(usize),
    /// O ponto médio dos vértices `i` e `j` do contorno principal (o meio de uma faixa).
    Mid(usize, usize),
    /// O vértice `i` do sub-contorno `s` (o furo de uma etiqueta, o olho de um osso).
    Sub(usize, usize),
    /// O início do ARREDONDAMENTO da quina `corner` do contorno VIVO (os campos `sharp` a
    /// zero): o vértice cozido que pousa na aresta que sai da quina para a seguinte (`next`) ou
    /// a anterior, o mais perto da quina. Com raio zero é a própria quina — o vértice muda de
    /// índice quando o raio nasce, então não há índice fixo que o descreva.
    Rounding { corner: usize, next: bool },
    /// Um ponto da ELIPSE da caixa ao ângulo `start` (+ `sweep` inteiro, ou metade com `mid`),
    /// à escala do campo `scale` (o furo) — a família do círculo, cujo número de vértices muda
    /// com a abertura. É a definição do arco (`round::arc_verts` põe ali o vértice).
    Ellipse {
        start: usize,
        sweep: Option<usize>,
        mid: bool,
        scale: Option<usize>,
    },
    /// O MEIO (`t = ½`) da aresta que sai do vértice `i` (F10: a envergadura — numa aresta curva o
    /// meio da corda não está no contorno).
    EdgeMid(usize),
    /// Onde o contorno COZIDO cruza o raio do centro à quina `corner` do contorno vivo (os campos
    /// `sharp` a zero): o meio do arredondamento dela — o *Outer/Inner circle* do Affinity, cujo
    /// ponto vermelho desce pelo eixo da ponta à medida que ela arredonda. Sem raio é a quina.
    Apex(usize),
    /// Um ponto FIXO da caixa VISÍVEL — a do contorno cozido, a que o gizmo desenha (`u` da
    /// esquerda, `v` do topo, em OITAVOS dela) — um widget fora do contorno: o ⇕ das pontas do
    /// Illustrator, na borda direita da caixa (na caixa dos campos ele ficava fora da visível, e o
    /// cartão lateral tapava-o — a foto da estrela nova).
    Box { u: u8, v: u8 },
}

/// Como o arrasto escreve nos campos — a MÉTRICA com que o ponto da alça persegue a mão. Todas
/// MEDIDAS no Inkscape 1.4.4 (`oraculo_alcas/`, doc 13 §2), com o Illustrator nas contagens.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Law {
    /// O ponto da alça cai o mais perto possível da mão: num trilho recto é a PROJECÇÃO — o
    /// `rx` do Inkscape é a projecção do ponteiro na aresta, e a componente perpendicular não
    /// conta (±0,4 px em 28 arrastos). Dois campos = um plano (um eixo cada).
    Nearest,
    /// O raio NORMALIZADO da alça (a caixa vira círculo) iguala o do ponteiro: o `r2` da estrela
    /// do Inkscape é a DISTÂNCIA ao centro, não a projecção no raio (70,0 contra 67,6 a 15°).
    Radial,
    /// Contagem pelo ÂNGULO: o vértice que a contagem põe mais perto do ângulo normalizado da
    /// mão (o *side widget* do Illustrator; o Affinity só tem a barra).
    Count,
    /// A ABERTURA de um arco (o fim): o ângulo normalizado da mão — o `atan2` na elipse
    /// normalizada, como o `end` do Inkscape (40,1° contra 31,3° do ponto mais perto) —,
    /// desenrolado desde o press (satura na volta inteira e em 1°, em vez de saltar de 360° a
    /// 1° ao passar pelo início como o Inkscape). Com `pairs` (Ctrl) o início acompanha.
    Sweep { start: usize },
    /// O INÍCIO de um arco: o ângulo da mão, com o FIM parado (a abertura `sweep` compensa) —
    /// o `start` do Inkscape deixa o `end` em 270,00 nos 10 arrastos. Com `pairs` (Ctrl) o fim
    /// acompanha e o arco RODA inteiro (o Shift do Inkscape; aqui Ctrl = «em pares», Affinity).
    ArcStart { sweep: usize },
    /// Um PONTO da caixa (campos `[u, v]`, frações da esquerda e do TOPO): os dois campos andam
    /// o que a mão anda, em frações da caixa, desde o press — o bico de um balão (o *Tail end
    /// position* e o *Tail height* do Affinity, um ponto 2D). Sem busca: o `cook` cola o bico à aresta e troca de
    /// sector num SALTO, e uma métrica sobre o vértice prendia-se na diagonal (8 de 60 arrastos a
    /// ≥ 7e-3; uma grelha conjunta 24×24 também, e regrediu a cabeça da seta). RELATIVO e não a
    /// posição absoluta da mão: com o bico colado, o vértice que se desenha não é o ponto do
    /// parâmetro, e agarrá-lo saltaria.
    BoxPoint,
    /// Contagem por VOLTAS: cada volta inteira do ponteiro à volta do centro soma (ou tira) uma;
    /// a distância não conta — as `revolution` do Inkscape só mudam com o ângulo (150 e 70 px a
    /// 0° deixam 3,000; +90° dá 3,250).
    Winding,
    /// Contagem por DEGRAUS verticais desde o press: cada 1/24 da altura da caixa para CIMA soma
    /// uma, para baixo tira — o ⇕ «Points» do Illustrator (o Affinity só tem a barra).
    Steps,
}

/// O que o Alt (`alone`) faz numa alça partilhada: escreve OUTROS campos e devolve a soma aos
/// `keep` — o canto sozinho do rectângulo arredondado.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Alone {
    pub fields: &'static [usize],
    /// Campos cuja soma com o 1.º de `fields` fica parada (o raio dos outros cantos).
    pub keep: &'static [usize],
}

/// Uma alça do catálogo: que campos ela move, onde fica, e a lei do arrasto.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct HandleDesc {
    pub fields: &'static [usize],
    pub spot: Spot,
    pub law: Law,
    /// Os campos de ARREDONDAMENTO da forma: o [`Spot`] lê-se no contorno VIVO (eles a zero) —
    /// a quina de um polígono arredondado continua a ser «o 2.º vértice». O [`Spot::Rounding`]
    /// usa os dois contornos (o vivo diz onde é a quina, o cozido onde o arco começa).
    pub sharp: &'static [usize],
    pub alone: Option<Alone>,
    /// A alça ESTACIONA a uma distância mínima de onde os campos a põem no mínimo — ver
    /// [`handle_grip`].
    pub park: bool,
    /// A alça desenha-se `park_at` para DENTRO (para o centro da caixa): o ponto dela vive SEMPRE
    /// sobre uma aresta da caixa, onde mora uma alça de escala (F10: o meio do losango).
    pub inset: bool,
}

/// Os modificadores do arrasto (teclas do Illustrator; Ctrl = «em pares», Affinity).
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub struct Mods {
    /// Shift: os ângulos em passos de [`ANGLE_SNAP_DEG`].
    pub snap: bool,
    /// Ctrl: o início do arco leva o fim (o arco roda).
    pub pairs: bool,
    /// Alt: o canto sozinho.
    pub alone: bool,
}

/// O passo dos ângulos com Shift — o do Inkscape medido (Ctrl: 37° → 30°, `arc_slice`), que é
/// também o passo de rotação do Illustrator e do nosso gizmo.
pub const ANGLE_SNAP_DEG: f64 = 15.0;

/// As alças de `kind`, na ordem em que se desenham (a ÚLTIMA fica por cima e ganha o clique).
#[must_use]
pub fn handles_of(kind: ShapeKind) -> &'static [HandleDesc] {
    crate::shape_handles_table::of(kind)
}

/// Os campos de `kind` que NENHUMA alça move — ficam só na fila ② (o número exacto vive lá
/// para todos). É uma CONTA sobre a tabela, nunca uma lista à parte.
pub fn fields_without_handle(kind: ShapeKind) -> impl Iterator<Item = usize> {
    let hs = handles_of(kind);
    (0..shapes::desc(kind).fields.len()).filter(move |i| {
        !hs.iter()
            .any(|h| h.fields.contains(i) || h.alone.is_some_and(|a| a.fields.contains(i)))
    })
}

fn ellipse_pt(half: [f64; 2], deg: f64, s: f64) -> [f64; 2] {
    let a = deg.to_radians();
    [half[0] * s * a.cos(), half[1] * s * a.sin()]
}

/// A abertura como o `cook` a lê (`≤ 0` = volta inteira — `round::full_if_unset`).
fn sweep_of(v: f64) -> f64 {
    if v <= 0.0 { 360.0 } else { v.min(360.0) }
}

/// O ponto de `spot` (espaço local). `None` = o índice não existe nesta geometria (um save
/// corrompido): a alça simplesmente não aparece.
fn spot_at(
    spot: Spot,
    p: &VecPath,
    sharp: Option<&VecPath>,
    values: &ShapeValues,
    half: [f64; 2],
) -> Option<[f64; 2]> {
    let v = |i: usize| p.verts.get(i).map(|v| v.anchor);
    match spot {
        Spot::Vertex(i) => v(i),
        Spot::FromEnd(i) => p.verts.len().checked_sub(1 + i).and_then(v),
        Spot::Mid(i, j) => {
            let (a, b) = (v(i)?, v(j)?);
            Some([(a[0] + b[0]) * 0.5, (a[1] + b[1]) * 0.5])
        }
        Spot::Sub(c, i) => p.subpaths.get(c)?.verts.get(i).map(|v| v.anchor),
        Spot::Box { u, v } => {
            let (lo, hi) = curve_bbox(p)?;
            let (u, v) = (f64::from(u) / 8.0, f64::from(v) / 8.0);
            Some([lo[0] + (hi[0] - lo[0]) * u, hi[1] - (hi[1] - lo[1]) * v])
        }
        Spot::Apex(corner) => {
            let q = sharp?.verts.get(corner)?.anchor;
            spoke_hit(p, q)
        }
        Spot::EdgeMid(i) => {
            let n = p.verts.len();
            let (a, b) = (p.verts.get(i)?, p.verts.get((i + 1) % n.max(1))?);
            Some(cubic_at(
                &[a.anchor, a.out_handle, b.in_handle, b.anchor],
                0.5,
            ))
        }
        Spot::Ellipse {
            start,
            sweep,
            mid,
            scale,
        } => {
            let s0 = *values.get(start)?;
            let sw = sweep.map_or(0.0, |i| sweep_of(values[i]));
            let deg = if mid { s0 + sw * 0.5 } else { s0 + sw };
            let k = scale.map_or(1.0, |i| values[i].clamp(0.0, 0.95));
            Some(ellipse_pt(half, deg, k))
        }
        Spot::Rounding { corner, next } => {
            let sharp = sharp?;
            let n = sharp.verts.len();
            if n < 2 {
                return None;
            }
            let q = sharp.verts.get(corner)?.anchor;
            let o = sharp.verts[if next {
                (corner + 1) % n
            } else {
                (corner + n - 1) % n
            }]
            .anchor;
            let (ex, ey) = (o[0] - q[0], o[1] - q[1]);
            let len = ex.hypot(ey);
            if len <= 0.0 {
                return None;
            }
            let tol = len * 1e-7;
            p.verts
                .iter()
                .map(|v| v.anchor)
                .filter_map(|a| {
                    let t = ((a[0] - q[0]) * ex + (a[1] - q[1]) * ey) / (len * len);
                    let off = ((a[0] - q[0]) * ey - (a[1] - q[1]) * ex).abs() / len;
                    (off <= tol && (-1e-9..=0.5 + 1e-9).contains(&t)).then_some((t, a))
                })
                .min_by(|x, y| x.0.total_cmp(&y.0))
                .map(|(_, a)| a)
        }
    }
}

/// A posição LOCAL VERDADEIRA da alça `h` (o ponto do contorno). `cook` é a porta única de
/// geometria do chamador (`shape_live::recook_shape` com a caixa da forma).
#[must_use]
pub fn handle_point(
    h: &HandleDesc,
    values: &ShapeValues,
    half: [f64; 2],
    cook: &dyn Fn(&ShapeValues) -> VecPath,
) -> Option<[f64; 2]> {
    if h.sharp.is_empty() {
        return spot_at(h.spot, &cook(values), None, values, half);
    }
    let sharp = cook(&zeroed(values, h.sharp));
    if matches!(h.spot, Spot::Rounding { .. } | Spot::Apex(_)) {
        spot_at(h.spot, &cook(values), Some(&sharp), values, half)
    } else {
        spot_at(h.spot, &sharp, None, values, half)
    }
}

fn zeroed(values: &ShapeValues, fields: &[usize]) -> ShapeValues {
    let mut z = *values;
    for &i in fields {
        z[i] = 0.0;
    }
    z
}

/// ⭐ **Onde a alça se DESENHA e se agarra** — o ponto verdadeiro, salvo uma alça `park` a menos
/// de `park_at` (local) do ponto onde os campos dela no MÍNIMO a poriam: essa ESTACIONA a
/// `park_at` dele, na direcção em que os campos crescem. Com raio zero a alça de canto de um
/// rectângulo cai na alça de escala da caixa, e o furo de uma pizza no pivô — duas alças no mesmo
/// ponto deixam uma inalcançável; o widget de canto do Illustrator nasce recolhido para dentro
/// pela mesma razão. `park_at` = o raio de acerto das alças da caixa.
#[must_use]
pub fn handle_grip(
    kind: ShapeKind,
    h: &HandleDesc,
    values: &ShapeValues,
    half: [f64; 2],
    px_to_world: f64,
    park_at: f64,
    cook: &dyn Fn(&ShapeValues) -> VecPath,
) -> Option<[f64; 2]> {
    let at = handle_point(h, values, half, cook)?;
    // ⭐ As alças de ÂNGULO vivem `park_at` para DENTRO do aro, sobre o raio delas: no aro da
    // elipse os pontos cardeais SÃO as alças do meio das arestas da caixa (medido na foto da cena
    // 15: o início da pizza às 3h sumia debaixo da de escala). A lei delas só lê o ÂNGULO da mão,
    // então recolhê-las não muda nada do que fazem; até metade do raio, para não cair no pivô.
    if reads_only_the_angle(h) {
        let r = at[0].hypot(at[1]);
        if r <= 0.0 {
            return Some(at);
        }
        let k = (r - park_at.min(r * 0.5)) / r;
        return Some([at[0] * k, at[1] * k]);
    }
    if h.inset {
        let r = at[0].hypot(at[1]);
        let k = if r > 0.0 {
            (r - park_at.min(r * 0.5)) / r
        } else {
            1.0
        };
        return Some([at[0] * k, at[1] * k]);
    }
    if !h.park {
        return Some(at);
    }
    let mut lo = *values;
    let mut up = *values;
    for &i in h.fields {
        let (min, max, _) = range_world(kind, i, px_to_world)?;
        lo[i] = min;
        up[i] = min + (max - min) * 1e-3;
    }
    let zero = handle_point(h, &lo, half, cook)?;
    let next = handle_point(h, &up, half, cook)?;
    let (dx, dy) = (next[0] - zero[0], next[1] - zero[1]);
    let l = dx.hypot(dy);
    if (at[0] - zero[0]).hypot(at[1] - zero[1]) >= park_at || l <= 0.0 {
        return Some(at);
    }
    Some([zero[0] + dx / l * park_at, zero[1] + dy / l * park_at])
}

/// `true` = a lei da alça só lê o ÂNGULO da mão — ela pode viver recolhida para dentro do aro, e a
/// pega dela mede-se a partir do ponto DESENHADO (o mesmo ângulo do verdadeiro): segurar parado
/// não muda nada e a translação não torce o ângulo.
#[must_use]
pub fn reads_only_the_angle(h: &HandleDesc) -> bool {
    matches!(
        h.law,
        Law::Sweep { .. } | Law::ArcStart { .. } | Law::Count | Law::Winding
    )
}

/// O estado de UM arrasto vivo de alça: os valores do press (o `Esc` repõe-nos), a pega (o
/// ponteiro nunca salta para o centro da alça) e o ângulo desenrolado.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct HandleDrag {
    pub handle: usize,
    pub at_press: ShapeValues,
    grip: [f64; 2],
    /// Onde a alça estava no press (local) — o zero do [`Law::BoxPoint`].
    origin: [f64; 2],
    last_angle: f64,
    wound: f64,
}

/// O ângulo NORMALIZADO de `p` (graus, anti-horário a partir das 3h, Y-para-cima).
fn norm_angle(p: [f64; 2], half: [f64; 2]) -> f64 {
    let (hx, hy) = (half[0].max(1e-12), half[1].max(1e-12));
    (p[1] / hy).atan2(p[0] / hx).to_degrees()
}

fn wrap180(d: f64) -> f64 {
    (d + 180.0).rem_euclid(360.0) - 180.0
}

impl HandleDrag {
    /// Começa o arrasto da alça `handle`: `p` é o ponteiro e `grip` onde a alça estava (os dois
    /// locais) — a diferença viaja com a mão, então agarrar a alça pela borda não a faz saltar.
    #[must_use]
    pub fn start(
        handle: usize,
        at_press: ShapeValues,
        p: [f64; 2],
        grip: [f64; 2],
        half: [f64; 2],
    ) -> Self {
        Self {
            handle,
            at_press,
            grip: [grip[0] - p[0], grip[1] - p[1]],
            origin: grip,
            last_angle: norm_angle(grip, half),
            wound: 0.0,
        }
    }
}

/// Amostras por campo antes do refinamento (o trilho inteiro, para o mínimo ser o GLOBAL).
const SAMPLES: usize = 96;
/// Rodadas de refinamento (a janela encolhe ×4 por rodada à volta do melhor).
const REFINE: usize = 14;

/// A faixa de AUTORIA do campo `i` em unidades de documento (os `Px` atravessam a fronteira
/// pela mesma conta do painel) e se ele é inteiro.
fn range_world(kind: ShapeKind, i: usize, px_to_world: f64) -> Option<(f64, f64, bool)> {
    let f = shapes::desc(kind).fields.get(i)?;
    let k = if f.unit == FieldUnit::Px {
        px_to_world
    } else {
        1.0
    };
    let int = matches!(f.unit, FieldUnit::Count | FieldUnit::Choice(_));
    Some((f.min * k, f.max * k, int))
}

/// ⭐ **O arrasto** — os valores novos da forma com o ponteiro em `p` (local). Parte sempre dos
/// valores do PRESS: o arrasto é uma função da mão, não um acumulador de erro.
#[must_use]
pub fn drag(
    kind: ShapeKind,
    d: &mut HandleDrag,
    p: [f64; 2],
    mods: Mods,
    half: [f64; 2],
    px_to_world: f64,
    cook: &dyn Fn(&ShapeValues) -> VecPath,
) -> ShapeValues {
    let Some(h) = handles_of(kind).get(d.handle) else {
        return d.at_press;
    };
    // A pega viaja com a mão como TRANSLAÇÃO (o Inkscape: 20/20 a 0,000°; a rotação erraria até
    // 0,6°); o ângulo dela desenrola-se desde o press.
    let p = [p[0] + d.grip[0], p[1] + d.grip[1]];
    let a = norm_angle(p, half);
    d.wound += wrap180(a - d.last_angle);
    d.last_angle = a;
    let mut v = d.at_press;
    let snap = |x: f64| {
        if mods.snap {
            (x / ANGLE_SNAP_DEG).round() * ANGLE_SNAP_DEG
        } else {
            x
        }
    };
    let (fields, keep) = match (mods.alone, h.alone) {
        (true, Some(al)) => (al.fields, al.keep),
        _ => (h.fields, &[][..]),
    };
    match h.law {
        Law::Winding => {
            if let Some(&i) = fields.first()
                && let Some((lo, hi, _)) = range_world(kind, i, px_to_world)
            {
                v[i] = (d.at_press[i] + (d.wound / 360.0).round()).clamp(lo, hi);
            }
        }
        Law::Sweep { start } => {
            if let Some(&i) = fields.first()
                && let Some((lo, hi, _)) = range_world(kind, i, px_to_world)
            {
                let s0 = d.at_press[start];
                let end = snap(s0 + sweep_of(d.at_press[i]) + d.wound);
                if mods.pairs {
                    if let Some((slo, shi, _)) = range_world(kind, start, px_to_world) {
                        let s = end - sweep_of(d.at_press[i]);
                        v[start] = (s - slo).rem_euclid(shi - slo + 1.0) + slo;
                    }
                } else {
                    v[i] = (end - s0).clamp(lo, hi);
                }
            }
        }
        Law::ArcStart { sweep } => {
            if let Some(&i) = fields.first()
                && let Some((lo, hi, _)) = range_world(kind, i, px_to_world)
                && let Some((slo, shi, _)) = range_world(kind, sweep, px_to_world)
            {
                let s0 = d.at_press[i];
                let end = s0 + sweep_of(d.at_press[sweep]);
                let s = snap(s0 + d.wound);
                if !mods.pairs {
                    v[sweep] = (end - s).clamp(slo, shi);
                }
                // O início é CÍCLICO (0..359): dá a volta em vez de saturar.
                v[i] = (s - lo).rem_euclid(hi - lo + 1.0) + lo;
            }
        }
        Law::BoxPoint => {
            if let [iu, iv] = fields
                && let Some((ulo, uhi, _)) = range_world(kind, *iu, px_to_world)
                && let Some((vlo, vhi, _)) = range_world(kind, *iv, px_to_world)
            {
                let (hx, hy) = (half[0].max(1e-12), half[1].max(1e-12));
                let (dx, dy) = (p[0] - d.origin[0], p[1] - d.origin[1]);
                v[*iu] = (d.at_press[*iu] + dx / (2.0 * hx)).clamp(ulo, uhi);
                v[*iv] = (d.at_press[*iv] - dy / (2.0 * hy)).clamp(vlo, vhi);
            }
        }
        Law::Steps => {
            if let Some(&i) = fields.first()
                && let Some((lo, hi, _)) = range_world(kind, i, px_to_world)
            {
                let step = 2.0 * half[1].max(1e-12) / STEPS_PER_BOX;
                let n = ((p[1] - d.origin[1]) / step).round();
                v[i] = (d.at_press[i] + n).clamp(lo, hi);
            }
        }
        Law::Nearest | Law::Radial | Law::Count => {
            let metric = |vals: &ShapeValues| {
                let Some(q) = handle_point(h, vals, half, cook) else {
                    return f64::INFINITY;
                };
                match h.law {
                    Law::Radial => {
                        let r = |x: [f64; 2]| {
                            (x[0] / half[0].max(1e-12)).hypot(x[1] / half[1].max(1e-12))
                        };
                        (r(q) - r(p)).abs()
                    }
                    Law::Count => wrap180(norm_angle(q, half) - a).abs(),
                    _ => (q[0] - p[0]).hypot(q[1] - p[1]),
                }
            };
            let rounds = if fields.len() > 1 { 3 } else { 1 };
            let before = fields.first().map(|&i| v[i]);
            for _ in 0..rounds {
                for &i in fields {
                    if let Some((lo, hi, int)) = range_world(kind, i, px_to_world) {
                        v[i] = best_on_track(&mut v, i, lo, hi, int, &metric);
                    }
                }
            }
            // O canto sozinho a partir do raio BASE: os outros cantos guardam o raio deles.
            if let (Some(b), Some(&i)) = (before, fields.first()) {
                for &k in keep {
                    v[k] += b - v[i];
                }
            }
        }
    }
    v
}

/// O valor do campo `i` em `[lo, hi]` com a menor métrica: varredura do trilho inteiro e
/// refinamento à volta do melhor. Empates (o trilho satura — um raio maior que meia aresta não
/// anda mais) ficam com o valor MAIS PERTO do atual: a alça não salta para o fim da faixa.
fn best_on_track(
    v: &mut ShapeValues,
    i: usize,
    lo: f64,
    hi: f64,
    int: bool,
    metric: &dyn Fn(&ShapeValues) -> f64,
) -> f64 {
    let cur = v[i];
    let mut eval = |x: f64| {
        v[i] = x;
        metric(v)
    };
    // O empate é da MÉTRICA (um trilho saturado devolve a mesma distância), nunca da faixa: com a
    // mão 25 px fora de uma aresta a distância é quase plana no mínimo (`dx²/50`), e uma
    // tolerância à escala da faixa puxava o raio 0,005 para o valor antigo (medido no `rx`).
    let better = |(d, x): (f64, f64), (bd, bx): (f64, f64)| {
        if !bd.is_finite() {
            return d.is_finite();
        }
        let tie = 1e-12 * (1.0 + bd);
        d < bd - tie || ((d - bd).abs() <= tie && (x - cur).abs() < (bx - cur).abs())
    };
    let mut best = (f64::INFINITY, cur);
    if int {
        #[expect(
            clippy::cast_possible_truncation,
            reason = "as faixas de contagem do catálogo são pequenas (≤ 60)"
        )]
        let (a, b) = (lo.ceil() as i64, hi.floor() as i64);
        for n in a..=b {
            #[expect(clippy::cast_precision_loss, reason = "n ≤ 60")]
            let x = n as f64;
            let c = (eval(x), x);
            if better(c, best) {
                best = c;
            }
        }
        v[i] = cur;
        return best.1;
    }
    for k in 0..=SAMPLES {
        #[expect(clippy::cast_precision_loss, reason = "k ≤ SAMPLES")]
        let x = lo + (hi - lo) * k as f64 / SAMPLES as f64;
        let c = (eval(x), x);
        if better(c, best) {
            best = c;
        }
    }
    #[expect(clippy::cast_precision_loss, reason = "SAMPLES é pequeno")]
    let mut half = (hi - lo) / SAMPLES as f64;
    for _ in 0..REFINE {
        for k in -2..=2 {
            let x = (best.1 + half * f64::from(k) * 0.5).clamp(lo, hi);
            let c = (eval(x), x);
            if better(c, best) {
                best = c;
            }
        }
        half *= 0.25;
    }
    v[i] = cur;
    best.1
}

/// O TRILHO de uma alça de um campo (as métricas de perto e de raio): os pontos por onde ela
/// passa ao varrer a faixa inteira — a guia vermelha que o Affinity mostra sobre a alça antes do
/// arrasto, para a mão saber para onde ir. Vazio nas outras (o trilho de um ângulo é o próprio
/// contorno; o de uma contagem são saltos).
#[must_use]
pub fn track(
    kind: ShapeKind,
    h: &HandleDesc,
    values: &ShapeValues,
    half: [f64; 2],
    px_to_world: f64,
    cook: &dyn Fn(&ShapeValues) -> VecPath,
) -> Vec<[f64; 2]> {
    let ([i], Law::Nearest | Law::Radial) = (h.fields, h.law) else {
        return Vec::new();
    };
    let Some((lo, hi, _)) = range_world(kind, *i, px_to_world) else {
        return Vec::new();
    };
    let mut v = *values;
    let mut out: Vec<[f64; 2]> = Vec::with_capacity(TRACK_POINTS + 1);
    for k in 0..=TRACK_POINTS {
        #[expect(clippy::cast_precision_loss, reason = "TRACK_POINTS é pequeno")]
        let x = lo + (hi - lo) * k as f64 / TRACK_POINTS as f64;
        v[*i] = x;
        if let Some(q) = handle_point(h, &v, half, cook)
            && out.last() != Some(&q)
        {
            out.push(q);
        }
    }
    out
}

/// Pontos do trilho desenhado (um trilho recto ou um raio precisam de 2; os curvos do catálogo
/// — o furo de uma pizza segue uma recta, um vale de estrela também — ficam lisos com estes).
const TRACK_POINTS: usize = 48;

/// O duplo-clique numa alça REPÕE os campos dela no valor de fábrica (o Affinity: *«double-click
/// the red handle that has been previously moved»* — só essa alça; as outras ficam).
#[must_use]
pub fn reset(kind: ShapeKind, handle: usize, values: &ShapeValues) -> ShapeValues {
    let mut v = *values;
    let def = kind.defaults();
    if let Some(h) = handles_of(kind).get(handle) {
        for &i in h.fields {
            v[i] = def[i];
        }
    }
    v
}

/// Degraus do [`Law::Steps`] na altura da caixa (uma estrela de 400 px: ~17 px por ponta).
const STEPS_PER_BOX: f64 = 24.0;

#[path = "shape_handles_spots.rs"]
mod spots;
use spots::{cubic_at, curve_bbox, spoke_hit};

#[cfg(test)]
#[path = "shape_handles_tests.rs"]
mod tests;

/// O resto dos gates da F9 — irmão pelo tecto de LOC.
#[cfg(test)]
#[path = "shape_handles_f9_oracle_tests.rs"]
mod f9_oracle_tests;

/// O oráculo dos parâmetros da F10 — irmão pelo tecto de LOC.
#[cfg(test)]
#[path = "shape_handles_f10_tests.rs"]
mod f10_tests;
