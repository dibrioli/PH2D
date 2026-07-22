//! **Tween** — o inbetween automático (W3.T3.6 + **Tween v2**), clean-room do
//! `interpolate.cc` + `interpolate_curves.cc` do GP (`02_referencia §3`), com os dois
//! upgrades qualificados de `04 §2`.
//!
//! Dois desenhos-chave A e B; o tween produz o desenho intermediário no fator `t`.
//! As peças, cada uma com uma razão:
//!
//! 1. **Correspondência ESPACIAL** ([`crate::TweenPlan`], `tween_match`) — quem vira quem
//!    sai da geometria, não do índice. A ordem de desenho continua contando **como termo do
//!    custo**, então o par ordinal ganha quando tudo mais empata: o v2 subsume o v1.
//!    Traço sem par vira cópia estática (ou fade, se pedido).
//! 2. **Contagem = MAX(A, B) com padding** ([`sample_padded`]) — os pontos da curva
//!    MENOR são preservados EXATAMENTE e os extras se distribuem ∝ comprimento de
//!    arco. É o que garante que em `t=0` e `t=1` os extremos saem **idênticos** ao
//!    original (uma reamostragem uniforme NÃO tem essa propriedade — o desenho do
//!    artista "escorregaria" ao entrar no tween).
//! 3. **Auto-flip** — se B foi desenhado no sentido contrário, o lerp faria o traço
//!    dar um nó. O teste é geométrico (cordas que se cruzam / direções opostas),
//!    com desempate por distância quando as cordas são quase paralelas (< 15°).
//! 4. **Movimento por ESPIRAL logarítmica** ([`StrokeMotion`], `tween_spiral`) — o traço
//!    percorre o ARCO entre as duas poses em vez da corda, e por isso um braço que gira não
//!    encolhe no meio do caminho. Translação pura cai na variante `Lerp` e o resultado é
//!    **byte-idêntico** ao do v1.
//! 5. **Fator NÃO-clampado** em `[-1, +2]`: overshoot é ferramenta (antecipação/rebote),
//!    não bug — e a espiral o honra CONTINUANDO a girar, em vez de esticar uma reta.
//!
//! Os inbetweens nascem [`KeyKind::Breakdown`], e re-tweenar **exclui os
//! breakdowns** do intervalo antes de recomeçar — regenerar é idempotente.

use crate::color::Rgba;
use crate::drawing::FlipDrawing;
use crate::frame::{Hold, KeyKind};
use crate::ids::{Frame, LayerId};
use crate::object::FlipObject;
use crate::stroke::{FlipStroke, Point};
use crate::tween_flip::should_flip;
use crate::tween_match::TweenPlan;
use crate::tween_spiral::StrokeMotion;
use ph2d_anim::Interp;
use ph2d_core::Vec2;

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
///
/// Constrói a correspondência na hora. Para gerar VÁRIOS inbetweens do mesmo par, use
/// [`tween_drawing_with`] com um [`TweenPlan`] construído uma vez — a correspondência é
/// função do PAR, não do fator, e refazê-la por quadro é repetir o mesmo trabalho.
#[must_use]
pub fn tween_drawing(a: &FlipDrawing, b: &FlipDrawing, t: f32, opts: TweenOptions) -> FlipDrawing {
    tween_drawing_with(a, b, t, opts, &TweenPlan::build(a, b))
}

/// [`tween_drawing`] com a correspondência já resolvida (a forma que o documento usa).
#[must_use]
pub fn tween_drawing_with(
    a: &FlipDrawing,
    b: &FlipDrawing,
    t: f32,
    opts: TweenOptions,
    plan: &TweenPlan,
) -> FlipDrawing {
    let u = ease(t, opts.easing);
    let mut out = FlipDrawing::new();
    // O movimento rígido de cada par, guardado com o centróide de A: os ÓRFÃOS o consultam
    // para viajar junto com o vizinho (um braço que some tem de acompanhar o corpo enquanto
    // desaparece, não ficar pregado no ar enquanto a figura anda embora).
    let mut motions: Vec<(Vec2, StrokeMotion)> = Vec::new();
    for (i, sa) in a.strokes.iter().enumerate() {
        match plan.pair_of_a(i).and_then(|j| b.strokes.get(j)) {
            Some(sb) => {
                let (m, s) = tween_stroke(sa, sb, u, opts.auto_flip);
                motions.push((mean_pos(sa), m));
                out.strokes.push(s);
            }
            // Sem par: cópia estática de A (não pisca, não some) — ou fade-out, se pedido.
            None => out.strokes.push(sa.clone()),
        }
    }
    if opts.fade_orphans {
        // O fade dos órfãos é SIMÉTRICO. Antes só o lado de B tinha fade (e por índice, o
        // que só enxergava "B tem mais traços que A"): o nome dizia "orphans" e a metade de
        // A ficava de fora — um traço que SOME saltava para fora da tela de um quadro para
        // o outro.
        for (i, sa) in a.strokes.iter().enumerate() {
            if plan.pair_of_a(i).is_some() {
                continue;
            }
            let m = nearest_motion(&motions, mean_pos(sa));
            out.strokes[i] = fade_orphan(sa, 1.0 - u, |p| m.advance(p, u));
        }
        for (j, sb) in b.strokes.iter().enumerate() {
            if plan.pair_of_b(j).is_some() {
                continue;
            }
            // O órfão de B chega vindo de onde o vizinho estava: a mesma espiral, andada
            // PARA TRÁS (de `u` até 1) — em `u=1` ele está exatamente onde B o desenhou.
            let m = nearest_motion(&motions, mean_pos(sb));
            out.strokes
                .push(fade_orphan(sb, u, |p| m.advance(p, u - 1.0)));
        }
    }
    out
}

/// Média das posições — a régua de proximidade dos órfãos (não é a feature de
/// correspondência: aqui a pergunta é só *"quem está perto?"*, e a média basta).
fn mean_pos(s: &FlipStroke) -> Vec2 {
    let n = s.len();
    if n == 0 {
        return Vec2::ZERO;
    }
    s.positions().iter().fold(Vec2::ZERO, |a, &p| a + p) / n as f32
}

/// O movimento do par mais próximo de `p` — parado, se não houver par nenhum (um
/// desenho em que NADA casou não tem vizinho para dizer para onde as coisas foram).
fn nearest_motion(motions: &[(Vec2, StrokeMotion)], p: Vec2) -> StrokeMotion {
    motions
        .iter()
        .min_by(|(a, _), (b, _)| {
            (*a - p)
                .length_squared()
                .total_cmp(&(*b - p).length_squared())
        })
        .map_or(StrokeMotion::Translate(Vec2::ZERO), |&(_, m)| m)
}

/// Um órfão no fator dado: opacidade `k` e cada ponto levado por `warp`.
fn fade_orphan(s: &FlipStroke, k: f32, warp: impl Fn(Vec2) -> Vec2) -> FlipStroke {
    let mut out = s.clone();
    let k = k.clamp(0.0, 1.0);
    for p in out.positions_mut() {
        *p = warp(*p);
    }
    for o in out.opacities_mut() {
        *o *= k;
    }
    // Um PREENCHIMENTO não é visível pela opacidade dos pontos (eles nem são
    // rasterizados) — quem manda é o `fill.opacity`. Sem isto, uma região colorida nova
    // "pipocava" inteira no 1º inbetween, que é exatamente o que o fade existe para evitar.
    if let Some(f) = out.fill.as_mut() {
        f.opacity *= k;
    }
    for h in &mut out.holes {
        for p in h.iter_mut() {
            *p = warp(*p);
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
///
/// Devolve TAMBÉM o movimento rígido ajustado — quem chama precisa dele para levar os
/// órfãos vizinhos, e ajustá-lo uma segunda vez lá seria a segunda porta para a mesma
/// pergunta (*"para onde este traço foi?"*), que é como duas respostas divergem.
fn tween_stroke(
    a: &FlipStroke,
    b: &FlipStroke,
    u: f32,
    auto_flip: bool,
) -> (StrokeMotion, FlipStroke) {
    let flip = auto_flip && should_flip(a, b);
    let n = a.len().max(b.len());
    let pa = sample_padded(a, n, false);
    let pb = sample_padded(b, n, flip);

    // **A espiral é ajustada sobre a MESMA correspondência que a interpolação usa** (os
    // arrays já padded e já flipados) — não sobre os traços crus, que têm contagens
    // diferentes e ponto 0 possivelmente trocado.
    let motion = StrokeMotion::fit(
        &pa.iter().map(|p| p.pos).collect::<Vec<_>>(),
        &pb.iter().map(|p| p.pos).collect::<Vec<_>>(),
    );

    // Atributos de CURVA: vêm de A (o GP não faz crossfade de material/flags). O
    // FILL é a exceção que corrigimos: se ambos têm, a cor interpola (senão o
    // preenchimento saltaria no meio do tween — o "fill_color salta" do original).
    let mut out = a.clone_attrs();
    for i in 0..n {
        let (x, y) = (pa[i], pb[i]);
        out.push_point(Point {
            // A porta única do ponto pareado: rígido + resíduo, e na translação pura
            // ela é `x + (y − x)·u` — a expressão do v1, ao bit.
            pos: motion.point_at(x.pos, y.pos, u),
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

    // **Os BURACOS também têm de andar.** O `clone_attrs` traz os furos de A verbatim —
    // e um furo parado enquanto o contorno externo se move sai de dentro da forma: o "O"
    // fica sólido no meio do tween e uma mancha de cor solta viaja pelo caminho (o
    // even-odd conta o anel órfão como região preenchida).
    //
    // Os furos são pareados por índice (são poucos e nascem juntos com o contorno, então
    // a ordem é a informação que existe). Quem não tem par **viaja com o contorno**, pela
    // espiral do próprio traço: deixá-lo parado o faria sair de dentro da forma, que é
    // exatamente o defeito que este bloco existe para impedir.
    out.holes = a
        .holes
        .iter()
        .enumerate()
        .map(|(i, ha)| match b.holes.get(i) {
            Some(hb) => tween_ring(ha, hb, u, motion),
            None => ha.iter().map(|&p| motion.advance(p, u)).collect(),
        })
        .collect();
    (motion, out)
}

/// Interpola um anel de buraco (pareamento por índice + padding ao maior, exatamente
/// como o contorno — um furo é uma polilinha fechada como qualquer outra), pelo MESMO
/// movimento rígido do contorno que o carrega: se o furo tivesse espiral própria, ele
/// giraria por conta e sairia da forma.
fn tween_ring(a: &[Vec2], b: &[Vec2], u: f32, motion: StrokeMotion) -> Vec<Vec2> {
    if a.is_empty() || b.is_empty() {
        return a.to_vec();
    }
    let n = a.len().max(b.len());
    let at = |ring: &[Vec2], i: usize| -> Vec2 {
        // Amostragem por proporção: um anel de m pontos "estica" para n reamostrando os
        // índices (o mesmo espírito do `sample_padded`, que é o do contorno).
        let m = ring.len();
        let j = if n <= 1 { 0 } else { i * (m - 1) / (n - 1) };
        ring[j.min(m - 1)]
    };
    (0..n)
        .map(|i| {
            let (p, q) = (at(a, i), at(b, i));
            motion.point_at(p, q, u)
        })
        .collect()
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
        // As POSES dos extremos: o inbetween interpola o LUGAR junto com a forma. Dois
        // extremos com a mesma arte em poses diferentes (uma instância deslocada) tweenam
        // num deslizamento — sem isto, os inbetweens nasceriam na pose neutra e a arte
        // pularia para a origem no meio do movimento.
        let (pose_a, pose_b) = (layer.frame_pose(from), layer.frame_pose(to));
        // Os quadros livres do intervalo (o que sobrou de chaves reais fica).
        let count = req.count.min((gap - 1) as u32);
        let a = self.drawing(da).expect("chave A tem desenho").clone();
        let b = self.drawing(db).expect("chave B tem desenho").clone();
        // **A correspondência é função do PAR, não do fator** — uma busca por inbetween
        // refaria o mesmo trabalho N vezes e, pior, poderia dar respostas DIFERENTES entre
        // quadros vizinhos, o que na tela é o traço piscando de identidade.
        let plan = TweenPlan::build(&a, &b);

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
            let art = tween_drawing_with(&a, &b, t, req.options, &plan);
            let Some(new_id) = self.insert_frame(req.layer, f, Hold::Implicit, KeyKind::Breakdown)
            else {
                continue;
            };
            if let Some(d) = self.drawing_mut(new_id) {
                d.strokes = art.strokes;
                made += 1;
            }
            if let Some(l) = self.layer_mut(req.layer) {
                l.set_frame_pose(f, pose_a.lerp(&pose_b, t));
            }
        }
        made
    }
}
#[cfg(test)]
#[path = "tween_tests.rs"]
mod tests;
