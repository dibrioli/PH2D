//! **Tween** — o inbetween automático (W3.T3.6), port do `interpolate.cc` +
//! `interpolate_curves.cc` do GP (`02_referencia §3`), clean-room.
//!
//! Dois desenhos-chave A e B; o tween produz o desenho intermediário no fator `t`.
//! Quatro peças, todas com uma razão:
//!
//! 1. **Pareamento POR ÍNDICE** — o i-ésimo traço de A com o i-ésimo de B. Simples
//!    e ESTÁVEL (não pisca entre quadros). Traço de A sem par vira cópia estática;
//!    traço extra de B nunca aparece (ou faz fade-in, se pedido).
//! 2. **Contagem = MAX(A, B) com padding** ([`sample_padded`]) — os pontos da curva
//!    MENOR são preservados EXATAMENTE e os extras se distribuem ∝ comprimento de
//!    arco. É o que garante que em `t=0` e `t=1` os extremos saem **idênticos** ao
//!    original (uma reamostragem uniforme NÃO tem essa propriedade — o desenho do
//!    artista "escorregaria" ao entrar no tween).
//! 3. **Auto-flip** — se B foi desenhado no sentido contrário, o lerp faria o traço
//!    dar um nó. O teste é geométrico (cordas que se cruzam / direções opostas),
//!    com desempate por distância quando as cordas são quase paralelas (< 15°).
//! 4. **Lerp NÃO-clampado** com o fator em `[-1, +2]`: overshoot é ferramenta
//!    (antecipação/rebote), não bug.
//!
//! Os inbetweens nascem [`KeyKind::Breakdown`], e re-tweenar **exclui os
//! breakdowns** do intervalo antes de recomeçar — regenerar é idempotente.

use crate::color::Rgba;
use crate::drawing::FlipDrawing;
use crate::frame::{Hold, KeyKind};
use crate::ids::{Frame, LayerId};
use crate::object::FlipObject;
use crate::stroke::{FlipStroke, Point};
use ph2d_anim::Interp;
use ph2d_core::Vec2;

/// `sin(15°)` — o limiar de "cordas quase paralelas" do auto-flip, em forma
/// polinomial (comparar senos, nunca chamar `acos` — HR-5).
const SIN_15: f32 = 0.258_819_04;

/// Limites do fator de mistura. Fora de `[0,1]` é EXTRAPOLAÇÃO deliberada
/// (overshoot); o GP clampa em `[-1, 2]` e nós também.
const FACTOR_MIN: f32 = -1.0;
const FACTOR_MAX: f32 = 2.0;

/// Opções do tween.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct TweenOptions {
    /// Curva de easing aplicada ao fator (`Interp::Linear` = uniforme).
    pub easing: Interp,
    /// Corrigir automaticamente traços desenhados no sentido contrário.
    pub auto_flip: bool,
    /// Traços que só existem em B aparecem com fade-in (em vez de não aparecer).
    pub fade_orphans: bool,
}

impl Default for TweenOptions {
    fn default() -> Self {
        Self {
            easing: Interp::Linear,
            auto_flip: true,
            fade_orphans: false,
        }
    }
}

/// **O desenho intermediário entre `a` e `b` no fator `t`** (`0` = A, `1` = B).
///
/// `t` é o fator BRUTO (posição no intervalo); o easing é aplicado aqui dentro.
#[must_use]
pub fn tween_drawing(a: &FlipDrawing, b: &FlipDrawing, t: f32, opts: TweenOptions) -> FlipDrawing {
    let u = ease(t, opts.easing);
    let mut out = FlipDrawing::new();
    for (i, sa) in a.strokes.iter().enumerate() {
        match b.strokes.get(i) {
            Some(sb) => out.strokes.push(tween_stroke(sa, sb, u, opts.auto_flip)),
            // Sem par: cópia estática de A (não pisca, não some).
            None => out.strokes.push(sa.clone()),
        }
    }
    if opts.fade_orphans {
        // Traços que só B tem: entram com a opacidade subindo (em vez de "pipocar"
        // inteiros no último quadro).
        for sb in b.strokes.iter().skip(a.strokes.len()) {
            let mut s = sb.clone();
            for o in s.opacities_mut() {
                *o *= u.clamp(0.0, 1.0);
            }
            out.strokes.push(s);
        }
    }
    out
}

/// O fator com easing, clampado à faixa de overshoot.
fn ease(t: f32, easing: Interp) -> f32 {
    let e = easing.remap(f64::from(t.clamp(0.0, 1.0))) as f32;
    e.clamp(FACTOR_MIN, FACTOR_MAX)
}

/// Interpola um par de traços. `u` é o fator já com easing.
fn tween_stroke(a: &FlipStroke, b: &FlipStroke, u: f32, auto_flip: bool) -> FlipStroke {
    let flip = auto_flip && should_flip(a, b);
    let n = a.len().max(b.len());
    let pa = sample_padded(a, n, false);
    let pb = sample_padded(b, n, flip);

    // Atributos de CURVA: vêm de A (o GP não faz crossfade de material/flags). O
    // FILL é a exceção que corrigimos: se ambos têm, a cor interpola (senão o
    // preenchimento saltaria no meio do tween — o "fill_color salta" do original).
    let mut out = a.clone_attrs();
    for i in 0..n {
        let (x, y) = (pa[i], pb[i]);
        out.push_point(Point {
            pos: Vec2::new(lerp(x.pos.x, y.pos.x, u), lerp(x.pos.y, y.pos.y, u)),
            width: lerp(x.width, y.width, u).max(0.0),
            opacity: lerp(x.opacity, y.opacity, u).clamp(0.0, 1.0),
            color: lerp_rgba(x.color, y.color, u),
        });
    }
    out.fill = match (a.fill, b.fill) {
        (Some(fa), Some(fb)) => Some(crate::stroke::Fill {
            color: lerp_rgba(fa.color, fb.color, u),
            opacity: lerp(fa.opacity, fb.opacity, u).clamp(0.0, 1.0),
        }),
        (fa, _) => fa,
    };
    out
}

/// Lerp NÃO-clampado (o overshoot é a ferramenta).
fn lerp(x: f32, y: f32, u: f32) -> f32 {
    x + (y - x) * u
}

fn lerp_rgba(x: Rgba, y: Rgba, u: f32) -> Rgba {
    Rgba::new(
        lerp(x.r(), y.r(), u).clamp(0.0, 1.0),
        lerp(x.g(), y.g(), u).clamp(0.0, 1.0),
        lerp(x.b(), y.b(), u).clamp(0.0, 1.0),
        lerp(x.a(), y.a(), u).clamp(0.0, 1.0),
    )
}

/// **Amostra o traço em `n` pontos preservando os originais** (`sample_curve_padded`).
///
/// Os `len()` pontos originais saem EXATOS; os `n - len()` extras são repartidos
/// pelos segmentos ∝ comprimento de arco (maior resto — determinístico, HR-5).
/// `reverse` inverte a ordem ANTES de amostrar (o auto-flip).
///
/// Em `n == len()` a saída é o próprio traço: é isso que faz `t=0`/`t=1` reproduzir
/// os extremos ponto a ponto.
#[must_use]
fn sample_padded(s: &FlipStroke, n: usize, reverse: bool) -> Vec<Point> {
    let src: Vec<Point> = {
        let mut v: Vec<Point> = (0..s.len()).filter_map(|i| s.point(i)).collect();
        if reverse {
            v.reverse();
        }
        v
    };
    if src.is_empty() {
        return Vec::new();
    }
    if src.len() == 1 || n <= src.len() {
        let mut v = src;
        v.truncate(n.max(1));
        while v.len() < n {
            let last = *v.last().expect("não-vazio");
            v.push(last);
        }
        return v;
    }
    // Comprimentos de segmento (abertos: len-1 segmentos; fechados: + o de fecho).
    let seg_count = if s.closed { src.len() } else { src.len() - 1 };
    let lens: Vec<f32> = (0..seg_count)
        .map(|j| {
            let p = src[j].pos;
            let q = src[(j + 1) % src.len()].pos;
            let d = q - p;
            (d.x * d.x + d.y * d.y).sqrt()
        })
        .collect();
    let extras = n - src.len();
    let quota = largest_remainder(&lens, extras);

    let mut out: Vec<Point> = Vec::with_capacity(n);
    for j in 0..seg_count {
        let p = src[j];
        let q = src[(j + 1) % src.len()];
        out.push(p);
        for m in 1..=quota[j] {
            let f = m as f32 / (quota[j] + 1) as f32;
            out.push(lerp_point(p, q, f));
        }
    }
    if !s.closed {
        out.push(*src.last().expect("não-vazio"));
    }
    debug_assert_eq!(out.len(), n, "o padding tem de bater exatamente");
    out
}

/// Reparte `extras` unidades pelos pesos `w` — método do maior resto (Hare), com
/// desempate por índice: mesma entrada ⇒ mesma saída, sempre.
fn largest_remainder(w: &[f32], extras: usize) -> Vec<usize> {
    let mut out = vec![0usize; w.len()];
    if w.is_empty() || extras == 0 {
        return out;
    }
    let total: f32 = w.iter().sum();
    if total <= 0.0 {
        // Comprimento zero (todos os pontos coincidem): reparte igual.
        for (i, o) in out.iter_mut().enumerate() {
            *o = extras / w.len() + usize::from(i < extras % w.len());
        }
        return out;
    }
    let mut rest: Vec<(f32, usize)> = Vec::with_capacity(w.len());
    let mut used = 0usize;
    for (i, &wi) in w.iter().enumerate() {
        let exact = wi / total * extras as f32;
        let floor = exact.floor();
        out[i] = floor as usize;
        used += out[i];
        rest.push((exact - floor, i));
    }
    // Ordena por resto DECRESCENTE, desempatando pelo índice (determinismo).
    rest.sort_by(|a, b| b.0.total_cmp(&a.0).then(a.1.cmp(&b.1)));
    for &(_, i) in rest.iter().take(extras.saturating_sub(used)) {
        out[i] += 1;
    }
    out
}

fn lerp_point(p: Point, q: Point, f: f32) -> Point {
    Point {
        pos: Vec2::new(lerp(p.pos.x, q.pos.x, f), lerp(p.pos.y, q.pos.y, f)),
        width: lerp(p.width, q.width, f),
        opacity: lerp(p.opacity, q.opacity, f),
        color: lerp_rgba(p.color, q.color, f),
    }
}

/// **O traço `b` está invertido em relação a `a`?** (o auto-flip do GP).
///
/// Se as cordas ponta-a-ponta (`a.first→b.first` e `a.last→b.last`) se CRUZAM, os
/// traços estão trocados de ponta. Cordas quase paralelas (< 15°) não decidem nada
/// — aí vale a distância: emparelhar as pontas mais próximas. Sem cruzamento,
/// inverte se as direções apontam para lados opostos.
#[must_use]
fn should_flip(a: &FlipStroke, b: &FlipStroke) -> bool {
    let (Some(a0), Some(a1)) = (a.point(0), a.point(a.len().saturating_sub(1))) else {
        return false;
    };
    let (Some(b0), Some(b1)) = (b.point(0), b.point(b.len().saturating_sub(1))) else {
        return false;
    };
    let (a0, a1, b0, b1) = (a0.pos, a1.pos, b0.pos, b1.pos);
    let (c1, c2) = (b0 - a0, b1 - a1); // as duas cordas, como vetores

    if segments_cross(a0, b0, a1, b1) {
        // Quase paralelas: o cruzamento é ruído numérico — decide a distância.
        if nearly_parallel(c1, c2) {
            return dist2(a0, b1) + dist2(a1, b0) < dist2(a0, b0) + dist2(a1, b1);
        }
        return true;
    }
    let (da, db) = (a1 - a0, b1 - b0);
    da.x * db.x + da.y * db.y < 0.0
}

/// Cordas com ângulo < 15° entre si (mesmo sentido): `|sin| < sin15` e `cos > 0`.
fn nearly_parallel(u: Vec2, v: Vec2) -> bool {
    let cross = (u.x * v.y - u.y * v.x).abs();
    let dot = u.x * v.x + u.y * v.y;
    let mag = (u.x * u.x + u.y * u.y).sqrt() * (v.x * v.x + v.y * v.y).sqrt();
    dot > 0.0 && cross < SIN_15 * mag
}

fn dist2(p: Vec2, q: Vec2) -> f32 {
    let d = q - p;
    d.x * d.x + d.y * d.y
}

/// Os segmentos `p→q` e `r→s` se cruzam? (teste de orientação 2D, sem divisão.)
fn segments_cross(p: Vec2, q: Vec2, r: Vec2, s: Vec2) -> bool {
    fn orient(a: Vec2, b: Vec2, c: Vec2) -> f32 {
        (b.x - a.x) * (c.y - a.y) - (b.y - a.y) * (c.x - a.x)
    }
    let (d1, d2) = (orient(p, q, r), orient(p, q, s));
    let (d3, d4) = (orient(r, s, p), orient(r, s, q));
    (d1 * d2 < 0.0) && (d3 * d4 < 0.0)
}

// ── a operação de documento ──────────────────────────────────────────────────

/// Um pedido de tween: `count` inbetweens entre as chaves `from` e `to`.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct TweenRequest {
    pub layer: LayerId,
    /// Chave de origem (A).
    pub from: Frame,
    /// Chave de destino (B).
    pub to: Frame,
    /// Quantos inbetweens gerar (limitado pelos quadros livres do intervalo).
    pub count: u32,
    pub options: TweenOptions,
}

impl FlipObject {
    /// **Gera os inbetweens** entre duas chaves e devolve quantos criou.
    ///
    /// Os quadros são distribuídos uniformemente no intervalo aberto `(from, to)`;
    /// cada um nasce [`KeyKind::Breakdown`]. **Breakdowns pré-existentes no
    /// intervalo são removidos antes** — re-tweenar sobre o próprio resultado
    /// re-interpola entre os EXTREMOS originais (idempotente), em vez de tweenar
    /// tween.
    ///
    /// `0` se as chaves não existem, não têm desenho, ou não há quadro livre.
    pub fn tween(&mut self, req: TweenRequest) -> u32 {
        let (from, to) = (req.from.min(req.to), req.from.max(req.to));
        let gap = to - from;
        if gap < 2 || req.count == 0 {
            return 0;
        }
        let Some(layer) = self.layer(req.layer) else {
            return 0;
        };
        // Limpa os breakdowns antigos do intervalo (regeneração idempotente) e
        // RECLAMA os desenhos deles. A compactação REMAPEIA os `DrawingId`, então os
        // extremos só podem ser resolvidos DEPOIS (resolvê-los antes deixaria `da`/
        // `db` apontando para o desenho errado — o bug silencioso do índice
        // posicional).
        let stale: Vec<Frame> = layer
            .frames()
            .range((from + 1)..to)
            .filter(|(_, f)| f.kind == KeyKind::Breakdown)
            .map(|(&k, _)| k)
            .collect();
        for k in stale {
            self.remove_frame(req.layer, k);
        }
        self.remove_unused_drawings();

        let Some(layer) = self.layer(req.layer) else {
            return 0;
        };
        let (Some(da), Some(db)) = (
            layer.frames().get(&from).and_then(|f| f.drawing),
            layer.frames().get(&to).and_then(|f| f.drawing),
        ) else {
            return 0;
        };
        // Os quadros livres do intervalo (o que sobrou de chaves reais fica).
        let count = req.count.min((gap - 1) as u32);
        let a = self.drawing(da).expect("chave A tem desenho").clone();
        let b = self.drawing(db).expect("chave B tem desenho").clone();

        let mut made = 0;
        for i in 1..=count {
            // Posição ABSOLUTA no intervalo (o denominador é `to - from`): é o que
            // faz o scrub por posição e o easing baterem com o que se vê.
            let f = from + (gap * i as i32) / (count as i32 + 1);
            if f <= from || f >= to {
                continue;
            }
            if self
                .layer(req.layer)
                .and_then(|l| l.frames().get(&f))
                .is_some_and(|fr| fr.drawing.is_some())
            {
                continue; // chave real do usuário no caminho: respeita
            }
            let t = (f - from) as f32 / gap as f32;
            let art = tween_drawing(&a, &b, t, req.options);
            let Some(new_id) = self.insert_frame(req.layer, f, Hold::Implicit, KeyKind::Breakdown)
            else {
                continue;
            };
            if let Some(d) = self.drawing_mut(new_id) {
                d.strokes = art.strokes;
                made += 1;
            }
        }
        made
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ids::FlipObjectId;

    fn stroke(pts: &[(f32, f32)]) -> FlipStroke {
        let mut s = FlipStroke::new();
        for &(x, y) in pts {
            s.push_default(Vec2::new(x, y));
        }
        s
    }

    fn drawing(strokes: Vec<FlipStroke>) -> FlipDrawing {
        let mut d = FlipDrawing::new();
        d.strokes = strokes;
        d
    }

    fn pts(s: &FlipStroke) -> Vec<(f32, f32)> {
        s.positions().iter().map(|p| (p.x, p.y)).collect()
    }

    /// **A propriedade que justifica o padding:** nos extremos (`t=0`/`t=1`) o
    /// tween reproduz os desenhos originais PONTO A PONTO, mesmo com contagens
    /// diferentes. (Uma reamostragem uniforme falharia aqui.)
    #[test]
    fn the_endpoints_are_reproduced_exactly() {
        let a = drawing(vec![stroke(&[(0.0, 0.0), (10.0, 0.0)])]); // 2 pontos
        let b = drawing(vec![stroke(&[(0.0, 5.0), (5.0, 5.0), (10.0, 5.0)])]); // 3
        let o = TweenOptions::default();

        let at0 = tween_drawing(&a, &b, 0.0, o);
        assert_eq!(at0.strokes[0].len(), 3, "conta pelo MAIOR");
        // Os 2 pontos de A saem exatos; o extra caiu no meio do segmento.
        assert_eq!(
            pts(&at0.strokes[0]),
            vec![(0.0, 0.0), (5.0, 0.0), (10.0, 0.0)]
        );

        let at1 = tween_drawing(&a, &b, 1.0, o);
        assert_eq!(pts(&at1.strokes[0]), pts(&b.strokes[0]), "B, ponto a ponto");
    }

    /// O meio é o meio (com easing linear).
    #[test]
    fn the_middle_is_the_average() {
        let a = drawing(vec![stroke(&[(0.0, 0.0), (10.0, 0.0)])]);
        let b = drawing(vec![stroke(&[(0.0, 10.0), (10.0, 10.0)])]);
        let mid = tween_drawing(&a, &b, 0.5, TweenOptions::default());
        assert_eq!(pts(&mid.strokes[0]), vec![(0.0, 5.0), (10.0, 5.0)]);
    }

    /// **Auto-flip:** B desenhado ao contrário (mesma forma, ordem invertida). Sem
    /// o flip, o meio colapsaria num X; com ele, o traço só se translada.
    #[test]
    fn auto_flip_untangles_a_reversed_stroke() {
        let a = drawing(vec![stroke(&[(0.0, 0.0), (10.0, 0.0)])]);
        let b = drawing(vec![stroke(&[(10.0, 10.0), (0.0, 10.0)])]); // invertido
        let mid = tween_drawing(&a, &b, 0.5, TweenOptions::default());
        assert_eq!(
            pts(&mid.strokes[0]),
            vec![(0.0, 5.0), (10.0, 5.0)],
            "com flip: translação limpa"
        );
        // Sem auto-flip, o mesmo par vira o nó (as pontas cruzam no meio).
        let no_flip = TweenOptions {
            auto_flip: false,
            ..Default::default()
        };
        let bad = tween_drawing(&a, &b, 0.5, no_flip);
        assert_eq!(
            pts(&bad.strokes[0]),
            vec![(5.0, 5.0), (5.0, 5.0)],
            "colapsa"
        );
    }

    /// Cordas quase paralelas (< 15°) não decidem por cruzamento — decide a
    /// distância. Aqui as pontas próximas já estão pareadas: NÃO inverte.
    #[test]
    fn nearly_parallel_chords_are_decided_by_distance() {
        let a = stroke(&[(0.0, 0.0), (10.0, 0.0)]);
        let b = stroke(&[(0.2, 1.0), (10.1, 1.0)]);
        assert!(!should_flip(&a, &b));
    }

    /// Traço de A sem par vira cópia estática (não some, não pisca).
    #[test]
    fn unpaired_strokes_of_a_stay_static() {
        let a = drawing(vec![
            stroke(&[(0.0, 0.0), (1.0, 0.0)]),
            stroke(&[(5.0, 5.0), (6.0, 5.0)]),
        ]);
        let b = drawing(vec![stroke(&[(0.0, 2.0), (1.0, 2.0)])]);
        let mid = tween_drawing(&a, &b, 0.5, TweenOptions::default());
        assert_eq!(mid.strokes.len(), 2);
        assert_eq!(pts(&mid.strokes[1]), pts(&a.strokes[1]), "cópia estática");
        // Órfão de B não aparece (default); com fade_orphans, aparece esmaecido.
        let o = TweenOptions {
            fade_orphans: true,
            ..Default::default()
        };
        let with = tween_drawing(&b, &a, 0.5, o); // agora B é o maior
        assert_eq!(with.strokes.len(), 2);
        assert!(
            (with.strokes[1].opacities()[0] - 0.5).abs() < 1e-6,
            "fade-in"
        );
    }

    /// O padding reparte os extras ∝ comprimento: o segmento longo leva mais.
    #[test]
    fn padding_favours_the_longer_segment() {
        let s = stroke(&[(0.0, 0.0), (1.0, 0.0), (11.0, 0.0)]); // segs 1 e 10
        let out = sample_padded(&s, 6, false);
        assert_eq!(out.len(), 6);
        // Os 3 originais estão lá.
        for p in [(0.0, 0.0), (1.0, 0.0), (11.0, 0.0)] {
            assert!(
                out.iter().any(|q| (q.pos.x - p.0).abs() < 1e-6),
                "original {p:?} preservado"
            );
        }
        // O segundo segmento (10× mais longo) ficou com quase todos os extras.
        let in_second = out
            .iter()
            .filter(|p| p.pos.x > 1.0 && p.pos.x < 11.0)
            .count();
        assert!(
            in_second >= 2,
            "extras foram pro segmento longo: {in_second}"
        );
    }

    /// A op de documento: N inbetweens BREAKDOWN entre duas chaves, e re-tweenar
    /// regenera (não empilha).
    #[test]
    fn the_document_op_creates_breakdowns_and_is_idempotent() {
        let mut o = FlipObject::new(FlipObjectId(1), "O");
        let l = o.add_layer("L");
        let a = o
            .insert_frame(l, 0, Hold::Implicit, KeyKind::Keyframe)
            .unwrap();
        let b = o
            .insert_frame(l, 8, Hold::Implicit, KeyKind::Keyframe)
            .unwrap();
        o.drawing_mut(a)
            .unwrap()
            .strokes
            .push(stroke(&[(0.0, 0.0), (10.0, 0.0)]));
        o.drawing_mut(b)
            .unwrap()
            .strokes
            .push(stroke(&[(0.0, 10.0), (10.0, 10.0)]));

        let req = TweenRequest {
            layer: l,
            from: 0,
            to: 8,
            count: 3,
            options: TweenOptions::default(),
        };
        assert_eq!(o.tween(req), 3);
        let cells = o.layer(l).unwrap().cells();
        let frames: Vec<Frame> = cells.iter().map(|(k, _, _)| *k).collect();
        assert_eq!(frames, vec![0, 2, 4, 6, 8], "espaçados no intervalo");
        // Tipos: os do meio são BREAKDOWN.
        let kinds: Vec<KeyKind> = frames
            .iter()
            .map(|f| o.layer(l).unwrap().frames()[f].kind)
            .collect();
        assert_eq!(kinds[2], KeyKind::Breakdown);
        assert_eq!(kinds[0], KeyKind::Keyframe);
        // O do meio (t=0.5) está no meio do caminho.
        let mid_id = o.drawing_at(l, 4).unwrap();
        assert_eq!(
            pts(&o.drawing(mid_id).unwrap().strokes[0]),
            vec![(0.0, 5.0), (10.0, 5.0)]
        );

        // Re-tweenar com outra contagem REGENERA (não tweena os breakdowns).
        let again = TweenRequest { count: 1, ..req };
        assert_eq!(o.tween(again), 1);
        let frames: Vec<Frame> = o
            .layer(l)
            .unwrap()
            .cells()
            .iter()
            .map(|(k, _, _)| *k)
            .collect();
        assert_eq!(frames, vec![0, 4, 8], "os antigos sumiram");
        // E o inbetween novo é o do MEIO do caminho — prova de que a regeneração
        // interpolou entre os EXTREMOS (0 e 8), não entre a chave e um breakdown
        // sobrevivente. Com os ids remapeados pela compactação, um `da`/`db` resolvido
        // ANTES apontaria para o desenho errado e este valor sairia torto.
        let mid = o.drawing_at(l, 4).unwrap();
        assert_eq!(
            pts(&o.drawing(mid).unwrap().strokes[0]),
            vec![(0.0, 5.0), (10.0, 5.0)],
            "o inbetween regenerado é a média dos extremos"
        );
    }

    /// **O bug do smoke:** depois de gerar inbetweens, a chave "seguinte" para um novo
    /// tween tem de ser o próximo **KEYFRAME** — não o breakdown vizinho. Sem isso, o
    /// 2º Add interpolaria entre a chave 0 e o inbetween 2 (lixo entre 0 e 2), em vez
    /// de regenerar o intervalo 0→8.
    #[test]
    fn the_next_tween_target_skips_the_breakdowns_it_just_made() {
        let mut o = FlipObject::new(FlipObjectId(1), "O");
        let l = o.add_layer("L");
        let a = o
            .insert_frame(l, 0, Hold::Implicit, KeyKind::Keyframe)
            .unwrap();
        let b = o
            .insert_frame(l, 8, Hold::Implicit, KeyKind::Keyframe)
            .unwrap();
        o.drawing_mut(a)
            .unwrap()
            .strokes
            .push(stroke(&[(0.0, 0.0), (10.0, 0.0)]));
        o.drawing_mut(b)
            .unwrap()
            .strokes
            .push(stroke(&[(0.0, 10.0), (10.0, 10.0)]));
        o.tween(TweenRequest {
            layer: l,
            from: 0,
            to: 8,
            count: 3,
            options: TweenOptions::default(),
        });
        let layer = o.layer(l).unwrap();
        // O próximo DESENHO depois de 0 é o breakdown em 2 …
        assert_eq!(layer.next_drawing_key(0), Some(2));
        // … mas o próximo KEYFRAME (o extremo do tween) continua sendo o 8.
        assert_eq!(layer.next_keyframe_key(0), Some(8));
        // E parado EM CIMA de um inbetween, o extremo A é a chave 0 (não o breakdown).
        assert_eq!(layer.keyframe_at_or_before(4), Some(0));
        assert_eq!(layer.next_keyframe_key(4), Some(8));
    }
}
