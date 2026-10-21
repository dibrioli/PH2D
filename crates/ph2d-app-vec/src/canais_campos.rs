//! **Os campos do VETOR, puros** — [`campos`] enumera o que um caminho tem, [`ler`] dá o número de
//! um campo e [`escrever`] aplica um número (quantizando onde o documento quantiza). Uma porta por
//! pergunta: quem publica e quem verifica o eco leem por [`ler`]. O vocabulário (o que `campo`, `i`
//! e `j` significam) é `ph2d_ecs::canal_vetor`; o ECS e a timeline ficam em [`crate::canais`].

use ph2d_ecs::canal::familia::VETOR;
use ph2d_ecs::canal_vetor as c;
use ph2d_ecs::{FieldKey, VecShape};
use ph2d_vec_scene::paint_stack::PaintKind;
use ph2d_vec_scene::{
    BlendMode, DashPattern, FillRule, GradientStop, LineCap, LineJoin, MAX_DASH_PAIRS, Marker,
    Opacity, Paint, PatternMode, Rgba8, StrokeAlign, StrokePaint, StrokeSpec, TileKind, VecPath,
    VecVertex, VertexKind,
};

fn k(campo: u16, i: u16, j: u16) -> FieldKey {
    FieldKey::new(VETOR, campo, i, j)
}

fn n16(n: usize) -> u16 {
    u16::try_from(n).unwrap_or(u16::MAX)
}

#[expect(
    clippy::cast_possible_truncation,
    reason = "o canal é f32; o documento guarda f64"
)]
fn f(x: f64) -> f32 {
    x as f32
}

/// O índice discreto que a curva mandou, arredondado e preso a `0..n`.
#[expect(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    reason = "arredondado e preso a 0..n antes da conversão"
)]
fn indice(v: f32, n: usize) -> usize {
    (v.round().max(0.0) as usize).min(n.saturating_sub(1))
}

fn byte(v: f32) -> u8 {
    #[expect(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "preso a 0..=255"
    )]
    let b = v.round().clamp(0.0, 255.0) as u8;
    b
}

fn rgba(c: Rgba8) -> [f32; 4] {
    [c.r, c.g, c.b, c.a].map(f32::from)
}

fn set_rgba(cor: &mut Rgba8, canal: u16, v: f32) {
    match canal {
        0 => cor.r = byte(v),
        1 => cor.g = byte(v),
        2 => cor.b = byte(v),
        _ => cor.a = byte(v),
    }
}

const VERTICES: [VertexKind; 3] = [
    VertexKind::Corner,
    VertexKind::Smooth,
    VertexKind::Symmetric,
];
const REGRAS: [FillRule; 2] = [FillRule::NonZero, FillRule::EvenOdd];
const PONTAS: [LineCap; 3] = [LineCap::Butt, LineCap::Round, LineCap::Square];
const JUNCOES: [LineJoin; 3] = [LineJoin::Miter, LineJoin::Round, LineJoin::Bevel];
const ALINHAMENTOS: [StrokeAlign; 3] =
    [StrokeAlign::Centre, StrokeAlign::Inner, StrokeAlign::Outer];
const LADRILHOS: [TileKind; 4] = [
    TileKind::Grid,
    TileKind::BrickRow,
    TileKind::BrickCol,
    TileKind::Hex,
];
const MODOS: [PatternMode; 3] = [PatternMode::Tile, PatternMode::Mirror, PatternMode::Clamp];

fn pos<T: PartialEq>(todos: &[T], x: &T) -> f32 {
    #[expect(clippy::cast_precision_loss, reason = "índices pequenos")]
    let i = todos.iter().position(|y| y == x).unwrap_or(0) as f32;
    i
}

fn contornos(p: &VecPath) -> impl Iterator<Item = (&[VecVertex], bool)> {
    std::iter::once((p.verts.as_slice(), p.closed))
        .chain(p.subpaths.iter().map(|s| (s.verts.as_slice(), s.closed)))
}

fn vertice_mut(p: &mut VecPath, i: u16, j: u16) -> Option<&mut VecVertex> {
    let verts = match i {
        0 => &mut p.verts,
        _ => &mut p.subpaths.get_mut(usize::from(i) - 1)?.verts,
    };
    verts.get_mut(usize::from(j))
}

/// O tipo de uma tinta: Solid 0 · Linear 1 · Radial 2 · MultiPoint 3 · Pattern 4 · Brush 5.
fn tipo_tinta(p: &Paint) -> f32 {
    match p {
        Paint::Solid(_) => 0.0,
        Paint::Linear { .. } => 1.0,
        Paint::Radial { .. } => 2.0,
        Paint::MultiPoint { .. } => 3.0,
        Paint::Pattern(_) => 4.0,
    }
}

/// O pedaço de tinta que um `i` nomeia: a de base (`0`) ou a da entrada `i-1` da pilha.
enum Tinta<'a> {
    Fill(Option<&'a Paint>),
    Traco(Option<&'a StrokePaint>),
}

fn tinta(p: &VecPath, base: u16, i: u16) -> Option<Tinta<'_>> {
    let preench = base == c::TINTA_PREENCH;
    match i {
        0 if preench => Some(Tinta::Fill(p.fill.as_ref())),
        0 => Some(Tinta::Traco(p.stroke.as_ref().map(|s| &s.paint))),
        _ => match &p.paints.get(usize::from(i) - 1)?.kind {
            PaintKind::Fill(t) if preench => Some(Tinta::Fill(Some(t))),
            PaintKind::Stroke(s) if !preench => Some(Tinta::Traco(Some(&s.paint))),
            _ => None,
        },
    }
}

fn traco(p: &VecPath, i: u16) -> Option<&StrokeSpec> {
    match i {
        0 => p.stroke.as_ref(),
        _ => match &p.paints.get(usize::from(i) - 1)?.kind {
            PaintKind::Stroke(s) => Some(s),
            PaintKind::Fill(_) => None,
        },
    }
}

fn traco_mut(p: &mut VecPath, i: u16) -> Option<&mut StrokeSpec> {
    match i {
        0 => p.stroke.as_mut(),
        _ => match &mut p.paints.get_mut(usize::from(i) - 1)?.kind {
            PaintKind::Stroke(s) => Some(s),
            PaintKind::Fill(_) => None,
        },
    }
}

/// ⭐ **Os campos que este caminho TEM agora** — o inventário vivo que a família publica.
#[must_use]
pub fn campos(p: &VecPath, shape: Option<&VecShape>) -> Vec<FieldKey> {
    let mut out = Vec::new();
    // A geometria de uma forma paramétrica ou de um texto é DERIVADA dos parâmetros: o canal são
    // os parâmetros, nunca os vértices (dois donos para o mesmo desenho divergiriam).
    let derivada = matches!(shape, Some(VecShape::Param { .. } | VecShape::Text(_)));
    if !derivada {
        for (ci, (verts, _)) in contornos(p).enumerate() {
            let i = n16(ci);
            out.push(k(c::FECHADO, i, 0));
            for j in 0..n16(verts.len()) {
                out.extend((c::ANCORA_X..=c::TIPO_VERTICE).map(|campo| k(campo, i, j)));
            }
        }
        out.push(k(c::REGRA, 0, 0));
    }
    for i in 0..=n16(p.paints.len()) {
        campos_da_tinta(p, c::TINTA_PREENCH, i, &mut out);
        campos_da_tinta(p, c::TINTA_TRACO, i, &mut out);
        campos_do_traco(p, i, &mut out);
    }
    for (e, _) in p.paints.iter().enumerate() {
        let i = n16(e + 1);
        out.extend((c::PILHA_LIGADA..=c::PILHA_DILATA_JUNCAO).map(|campo| k(campo, i, 0)));
    }
    out.push(k(c::MISTURA, 0, 0));
    for (e, fx) in p.effects.iter().enumerate() {
        let i = n16(e);
        out.push(k(c::EFEITO_LIGADO, i, 0));
        for j in 0..n16(fx.effect.params().len()) {
            out.push(k(c::EFEITO_PARAM, i, j));
        }
    }
    match shape {
        Some(VecShape::Param { values, .. }) => {
            out.push(k(c::PARAM_LARGURA, 0, 0));
            out.push(k(c::PARAM_ALTURA, 0, 0));
            out.extend((0..n16(values.len())).map(|j| k(c::PARAM_VALOR, 0, j)));
        }
        Some(VecShape::Text(t)) => {
            out.extend((c::TEXTO_TAMANHO..=c::TEXTO_QUEBRA).map(|campo| k(campo, 0, 0)));
            out.push(k(c::TEXTO_QUEBRA_FIXA, 0, 0));
            out.extend((0..n16(t.axes.len())).map(|j| k(c::TEXTO_EIXO, 0, j)));
        }
        _ => {}
    }
    out.retain(|&kk| ler(p, shape, kk).is_some());
    out
}

fn campos_da_tinta(p: &VecPath, base: u16, i: u16, out: &mut Vec<FieldKey>) {
    let Some(t) = tinta(p, base, i) else { return };
    if i == 0 {
        out.push(k(base + c::OFF_EXISTE, 0, 0));
    }
    let blocos: &[(u16, u16)] = match t {
        Tinta::Fill(None) | Tinta::Traco(None) => &[],
        Tinta::Fill(Some(Paint::Solid(_))) | Tinta::Traco(Some(StrokePaint::Solid(_))) => {
            &[(c::OFF_TIPO, 1), (c::OFF_SOLIDA, 4)]
        }
        Tinta::Fill(Some(Paint::Linear { .. })) => {
            &[(c::OFF_TIPO, 1), (c::OFF_N_PARAGENS, 1), (c::OFF_LINEAR, 4)]
        }
        Tinta::Fill(Some(Paint::Radial { .. })) => {
            &[(c::OFF_TIPO, 1), (c::OFF_N_PARAGENS, 1), (c::OFF_RADIAL, 3)]
        }
        Tinta::Fill(Some(Paint::MultiPoint { .. })) => &[(c::OFF_TIPO, 1), (c::OFF_N_PONTOS, 1)],
        Tinta::Fill(Some(Paint::Pattern(_))) | Tinta::Traco(Some(StrokePaint::Pattern(_))) => &[
            (c::OFF_TIPO, 1),
            (c::OFF_PADRAO, 8),
            (c::OFF_PADRAO_MODOS, 2),
            (c::OFF_PADRAO_DESFASE, 1),
        ],
        Tinta::Traco(Some(StrokePaint::Brush(_))) => &[(c::OFF_TIPO, 1)],
    };
    for &(off, n) in blocos {
        out.extend((off..off + n).map(|o| k(base + o, i, 0)));
    }
    let (por_j, n) = match t {
        Tinta::Fill(Some(Paint::Linear { stops, .. } | Paint::Radial { stops, .. })) => {
            (c::OFF_PARAGEM..c::OFF_PARAGEM + 5, stops.len())
        }
        Tinta::Fill(Some(Paint::MultiPoint { points })) => {
            (c::OFF_PONTO..c::OFF_PONTO + 8, points.len())
        }
        _ => (0..0, 0),
    };
    for j in 0..n16(n) {
        out.extend(por_j.clone().map(|o| k(base + o, i, j)));
    }
}

fn campos_do_traco(p: &VecPath, i: u16, out: &mut Vec<FieldKey>) {
    let Some(s) = traco(p, i) else { return };
    for campo in c::TRACO_LARGURA..=c::TRACO_MARCA_REDONDA {
        if campo == c::TRACO_TRACO || campo == c::TRACO_VAO {
            if s.dash.is_some() {
                out.extend((0..n16(MAX_DASH_PAIRS)).map(|j| k(campo, i, j)));
            }
        } else {
            out.push(k(campo, i, 0));
        }
    }
    if s.brush().is_some() {
        out.extend((c::PINCEL_ESPACO..=c::PINCEL_ESPELHO).map(|campo| k(campo, i, 0)));
    }
}

/// ⭐ **Quanto vale o campo `kk` agora** — `None` se o caminho não o tem.
#[must_use]
pub fn ler(p: &VecPath, shape: Option<&VecShape>, kk: FieldKey) -> Option<f32> {
    let (campo, i, j) = (kk.campo, kk.i, kk.j);
    let ju = usize::from(j);
    match campo {
        c::FECHADO => contornos(p)
            .nth(usize::from(i))
            .map(|(_, fechado)| f32::from(u8::from(fechado))),
        c::ANCORA_X..=c::TIPO_VERTICE => {
            let (verts, _) = contornos(p).nth(usize::from(i))?;
            let v = verts.get(ju)?;
            Some(match campo {
                c::ANCORA_X => f(v.anchor[0]),
                c::ANCORA_Y => f(v.anchor[1]),
                c::ENTRADA_X => f(v.in_handle[0]),
                c::ENTRADA_Y => f(v.in_handle[1]),
                c::SAIDA_X => f(v.out_handle[0]),
                c::SAIDA_Y => f(v.out_handle[1]),
                c::CANTO => f(v.corner_radius),
                _ => pos(&VERTICES, &v.kind),
            })
        }
        c::REGRA => Some(pos(&REGRAS, &p.fill_rule)),
        c::TINTA_PREENCH..c::TRACO_LARGURA => {
            let base = if campo < c::TINTA_TRACO {
                c::TINTA_PREENCH
            } else {
                c::TINTA_TRACO
            };
            ler_tinta(tinta(p, base, i)?, campo - base, i, ju)
        }
        c::TRACO_LARGURA..=c::TRACO_MARCA_REDONDA => {
            let s = traco(p, i)?;
            Some(match campo {
                c::TRACO_LARGURA => f(s.width),
                c::TRACO_PONTA => pos(&PONTAS, &s.cap),
                c::TRACO_JUNCAO => pos(&JUNCOES, &s.join),
                c::TRACO_ALINHAMENTO => pos(&ALINHAMENTOS, &s.align),
                c::TRACO_TRACEJADO => f32::from(u8::from(s.dash.is_some())),
                c::TRACO_TRACO => f(s.dash.as_ref()?.pairs.get(ju)?[0]),
                c::TRACO_VAO => f(s.dash.as_ref()?.pairs.get(ju)?[1]),
                c::TRACO_MARCA_INICIO => f32::from(s.marker_start as u8),
                c::TRACO_MARCA_FIM => f32::from(s.marker_end as u8),
                c::TRACO_ESCALA_INICIO => f(s.marker_scale_start),
                c::TRACO_ESCALA_FIM => f(s.marker_scale_end),
                _ => f(s.marker_round),
            })
        }
        c::PINCEL_ESPACO..=c::PINCEL_ESPELHO => ler_pincel(traco(p, i)?.brush()?, campo),
        c::MISTURA => Some(f32::from(p.blend.to_u8())),
        c::PILHA_LIGADA..=c::PILHA_DILATA_JUNCAO => {
            let e = p.paints.get(usize::from(i).checked_sub(1)?)?;
            Some(match campo {
                c::PILHA_LIGADA => f32::from(u8::from(e.enabled)),
                c::PILHA_OPACIDADE => e.opacity.get(),
                c::PILHA_MISTURA => f32::from(e.blend.to_u8()),
                c::PILHA_DESLOCA_X => f(e.offset[0]),
                c::PILHA_DESLOCA_Y => f(e.offset[1]),
                c::PILHA_DILATA => f(e.dilate),
                _ => f32::from(e.dilate_join),
            })
        }
        c::EFEITO_LIGADO => p
            .effects
            .get(usize::from(i))
            .map(|e| f32::from(u8::from(e.enabled))),
        c::EFEITO_PARAM => {
            let e = p.effects.get(usize::from(i))?;
            (ju < e.effect.params().len()).then(|| f(e.effect.get(ju)))
        }
        c::PARAM_LARGURA..=c::PARAM_VALOR => match shape? {
            VecShape::Param { w, h, values, .. } => Some(match campo {
                c::PARAM_LARGURA => f(*w),
                c::PARAM_ALTURA => f(*h),
                _ => f(*values.get(ju)?),
            }),
            _ => None,
        },
        c::TEXTO_TAMANHO..=c::TEXTO_QUEBRA_FIXA => match shape? {
            VecShape::Text(t) => Some(match campo {
                c::TEXTO_QUEBRA_FIXA => f32::from(u8::from(t.wrap_width.is_some())),
                c::TEXTO_TAMANHO => f(t.size),
                c::TEXTO_PESO => t.weight,
                c::TEXTO_ENTRELINHA => f(t.line_height),
                c::TEXTO_TRACKING => f(t.tracking),
                c::TEXTO_ALINHAMENTO => f32::from(t.align),
                c::TEXTO_QUEBRA => f(t.wrap_width?),
                _ => t.axes.get(ju)?.1,
            }),
            _ => None,
        },
        _ => None,
    }
}

fn ler_tinta(t: Tinta<'_>, off: u16, i: u16, j: usize) -> Option<f32> {
    if off == c::OFF_EXISTE {
        // Só a de base pode faltar; a da pilha existe enquanto a entrada existe.
        let tem = match t {
            Tinta::Fill(x) => x.is_some(),
            Tinta::Traco(x) => x.is_some(),
        };
        return (i == 0).then_some(f32::from(u8::from(tem)));
    }
    match t {
        Tinta::Fill(p) => ler_paint(p?, off, j),
        Tinta::Traco(s) => match (s?, off) {
            (StrokePaint::Solid(_), o) if o == c::OFF_TIPO => Some(0.0),
            (StrokePaint::Pattern(_), o) if o == c::OFF_TIPO => Some(4.0),
            (StrokePaint::Brush(_), o) if o == c::OFF_TIPO => Some(5.0),
            (StrokePaint::Solid(cor), o) if (c::OFF_SOLIDA..c::OFF_SOLIDA + 4).contains(&o) => {
                Some(rgba(*cor)[usize::from(o - c::OFF_SOLIDA)])
            }
            (StrokePaint::Pattern(pt), o) => ler_padrao(pt, o),
            _ => None,
        },
    }
}

/// A ordem em que as escritas de um quadro se aplicam: o que muda a ESTRUTURA (existe, tipo, nº de
/// paragens, tracejado ligado, a forma paramétrica) antes do que preenche os números dela.
#[must_use]
pub fn prioridade(kk: FieldKey) -> u8 {
    let off_tinta = match kk.campo {
        cc @ c::TINTA_PREENCH..c::TINTA_TRACO => Some(cc - c::TINTA_PREENCH),
        cc @ c::TINTA_TRACO..c::TRACO_LARGURA => Some(cc - c::TINTA_TRACO),
        _ => None,
    };
    match (kk.campo, off_tinta) {
        (_, Some(c::OFF_EXISTE)) | (c::TEXTO_QUEBRA_FIXA, _) => 0,
        (_, Some(c::OFF_TIPO)) => 1,
        (_, Some(c::OFF_N_PARAGENS | c::OFF_N_PONTOS)) | (c::TRACO_TRACEJADO, _) => 2,
        _ => 3,
    }
}

/// ⭐ **Aplica `v` ao campo `kk`.** Um campo que o caminho não tem fica sem efeito de propósito: a
/// curva de uma paragem que a chave seguinte já não tem (o gradiente perdeu uma paragem) não se
/// aplica a nada. O que o documento FICOU lê-se depois por [`ler`] — a mesma porta do eco.
///
/// ⚠️ **Só escreve se o campo, VISTO PELO CANAL, difere de `v`.** O documento guarda `f64` e o
/// canal `f32`: reescrever o `0,4` que se leu gravaria `0,4000000059604645` — o documento mudava
/// sem ninguém mexer, e uma curva parada sujava-o a cada quadro.
pub fn escrever(p: &mut VecPath, shape: Option<&mut VecShape>, kk: FieldKey, v: f32) {
    if ler(p, shape.as_deref(), kk) == Some(v) {
        return;
    }
    let (campo, i, j) = (kk.campo, kk.i, kk.j);
    let ju = usize::from(j);
    let d = f64::from(v);
    let liga = v >= 0.5;
    match campo {
        c::FECHADO => match i {
            0 => p.closed = liga,
            _ => {
                if let Some(s) = p.subpaths.get_mut(usize::from(i) - 1) {
                    s.closed = liga;
                }
            }
        },
        c::ANCORA_X..=c::TIPO_VERTICE => {
            let Some(vx) = vertice_mut(p, i, j) else {
                return;
            };
            match campo {
                c::ANCORA_X => vx.anchor[0] = d,
                c::ANCORA_Y => vx.anchor[1] = d,
                c::ENTRADA_X => vx.in_handle[0] = d,
                c::ENTRADA_Y => vx.in_handle[1] = d,
                c::SAIDA_X => vx.out_handle[0] = d,
                c::SAIDA_Y => vx.out_handle[1] = d,
                c::CANTO => vx.corner_radius = d,
                _ => vx.kind = VERTICES[indice(v, VERTICES.len())],
            }
        }
        c::REGRA => p.fill_rule = REGRAS[indice(v, REGRAS.len())],
        c::TINTA_PREENCH..c::TINTA_TRACO => escrever_preench(p, campo - c::TINTA_PREENCH, i, ju, v),
        c::TINTA_TRACO..c::TRACO_LARGURA => escrever_tinta_traco(p, campo - c::TINTA_TRACO, i, v),
        c::TRACO_LARGURA..=c::TRACO_MARCA_REDONDA => {
            let Some(s) = traco_mut(p, i) else { return };
            match campo {
                c::TRACO_LARGURA => s.width = d.max(0.0),
                c::TRACO_PONTA => s.cap = PONTAS[indice(v, PONTAS.len())],
                c::TRACO_JUNCAO => s.join = JUNCOES[indice(v, JUNCOES.len())],
                c::TRACO_ALINHAMENTO => s.align = ALINHAMENTOS[indice(v, ALINHAMENTOS.len())],
                c::TRACO_TRACEJADO => match (liga, s.dash.is_some()) {
                    (true, false) => s.dash = Some(DashPattern::single(d.max(1.0), d.max(1.0))),
                    (false, true) => s.dash = None,
                    _ => {}
                },
                c::TRACO_TRACO | c::TRACO_VAO => {
                    if let Some(par) = s.dash.as_mut().and_then(|x| x.pairs.get_mut(ju)) {
                        par[usize::from(campo == c::TRACO_VAO)] = d.max(0.0);
                    }
                }
                c::TRACO_MARCA_INICIO => {
                    s.marker_start = Marker::from_u8(byte(v)).unwrap_or(Marker::None)
                }
                c::TRACO_MARCA_FIM => {
                    s.marker_end = Marker::from_u8(byte(v)).unwrap_or(Marker::None)
                }
                c::TRACO_ESCALA_INICIO => s.marker_scale_start = d,
                c::TRACO_ESCALA_FIM => s.marker_scale_end = d,
                _ => s.marker_round = d,
            }
        }
        c::PINCEL_ESPACO..=c::PINCEL_ESPELHO => {
            if let Some(StrokePaint::Brush(b)) = traco_mut(p, i).map(|s| &mut s.paint) {
                escrever_pincel(b, campo, v);
            }
        }
        c::MISTURA => p.blend = BlendMode::from_u8(byte(v)),
        c::PILHA_LIGADA..=c::PILHA_DILATA_JUNCAO => {
            let Some(e) = usize::from(i)
                .checked_sub(1)
                .and_then(|x| p.paints.get_mut(x))
            else {
                return;
            };
            match campo {
                c::PILHA_LIGADA => e.enabled = liga,
                c::PILHA_OPACIDADE => e.opacity = Opacity::new(v),
                c::PILHA_MISTURA => e.blend = BlendMode::from_u8(byte(v)),
                c::PILHA_DESLOCA_X => e.offset[0] = d,
                c::PILHA_DESLOCA_Y => e.offset[1] = d,
                c::PILHA_DILATA => e.dilate = d,
                _ => e.dilate_join = byte(v),
            }
        }
        c::EFEITO_LIGADO => {
            if let Some(e) = p.effects.get_mut(usize::from(i)) {
                e.enabled = liga;
            }
        }
        c::EFEITO_PARAM => {
            if let Some(e) = p.effects.get_mut(usize::from(i))
                && ju < e.effect.params().len()
            {
                e.effect.set(ju, d);
            }
        }
        c::PARAM_LARGURA..=c::TEXTO_QUEBRA_FIXA => escrever_shape(shape, campo, ju, v),
        _ => {}
    }
}

fn escrever_shape(shape: Option<&mut VecShape>, campo: u16, j: usize, v: f32) {
    let d = f64::from(v);
    match shape {
        Some(VecShape::Param { w, h, values, .. }) => match campo {
            c::PARAM_LARGURA => *w = d,
            c::PARAM_ALTURA => *h = d,
            c::PARAM_VALOR => {
                if let Some(x) = values.get_mut(j) {
                    *x = d;
                }
            }
            _ => {}
        },
        Some(VecShape::Text(t)) => match campo {
            c::TEXTO_TAMANHO => t.size = d,
            c::TEXTO_PESO => t.weight = v,
            c::TEXTO_ENTRELINHA => t.line_height = d,
            c::TEXTO_TRACKING => t.tracking = d,
            c::TEXTO_ALINHAMENTO => t.align = byte(v),
            c::TEXTO_QUEBRA => {
                if t.wrap_width.is_some() {
                    t.wrap_width = Some(d);
                }
            }
            c::TEXTO_EIXO => {
                if let Some(a) = t.axes.get_mut(j) {
                    a.1 = v;
                }
            }
            // Ligar a largura fixa semeia o número de fábrica do «Fixed» do painel; a curva da
            // largura (o mesmo grupo) aplica-se a seguir (`prioridade`).
            c::TEXTO_QUEBRA_FIXA => match (v >= 0.5, t.wrap_width.is_some()) {
                (true, false) => t.wrap_width = Some(ph2d_tool_vector::params::DEFAULT_TEXT_WRAP),
                (false, true) => t.wrap_width = None,
                _ => {}
            },
            _ => {}
        },
        _ => {}
    }
}

fn escrever_preench(p: &mut VecPath, off: u16, i: u16, j: usize, v: f32) {
    if i == 0 && off == c::OFF_EXISTE {
        match (v >= 0.5, p.fill.is_some()) {
            (true, false) => p.fill = Some(Paint::Solid(Rgba8::new(0, 0, 0, 255))),
            (false, true) => p.fill = None,
            _ => {}
        }
        return;
    }
    let alvo = match i {
        0 => p.fill.as_mut(),
        _ => match p.paints.get_mut(usize::from(i) - 1).map(|e| &mut e.kind) {
            Some(PaintKind::Fill(t)) => Some(t),
            _ => None,
        },
    };
    if let Some(t) = alvo {
        escrever_paint(t, off, j, v);
    }
}

fn escrever_tinta_traco(p: &mut VecPath, off: u16, i: u16, v: f32) {
    if i == 0 && off == c::OFF_EXISTE {
        match (v >= 0.5, p.stroke.is_some()) {
            (true, false) => p.stroke = Some(StrokeSpec::new(Rgba8::new(0, 0, 0, 255), 1.0)),
            (false, true) => p.stroke = None,
            _ => {}
        }
        return;
    }
    let Some(s) = traco_mut(p, i) else { return };
    match (&mut s.paint, off) {
        (paint, o) if o == c::OFF_TIPO => {
            let cor = match paint {
                StrokePaint::Solid(cor) => *cor,
                StrokePaint::Pattern(pt) => pt.fallback,
                StrokePaint::Brush(_) => Rgba8::new(0, 0, 0, 255),
            };
            // Só sólido ↔ o resto: o padrão e o pincel trazem um asset que a curva não tem como criar.
            if indice(v, 6) == 0 && !matches!(paint, StrokePaint::Solid(_)) {
                *paint = StrokePaint::Solid(cor);
            }
        }
        (StrokePaint::Solid(cor), o) if (c::OFF_SOLIDA..c::OFF_SOLIDA + 4).contains(&o) => {
            set_rgba(cor, o - c::OFF_SOLIDA, v);
        }
        (StrokePaint::Pattern(pt), o) => escrever_padrao(pt, o, v),
        _ => {}
    }
}

#[path = "canais_campos_tinta.rs"]
mod tinta;
use tinta::{escrever_padrao, escrever_paint, escrever_pincel, ler_padrao, ler_paint, ler_pincel};

#[cfg(test)]
#[path = "canais_campos_tests.rs"]
mod tests;
