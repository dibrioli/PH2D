//! Edição geométrica de paths vetoriais (ADR-0108): retype de vértice
//! (auto-smooth Corner→Smooth), split de Bézier (de Casteljau) e projeção do
//! ponto mais próximo do traço. Extraído de `lib.rs` para respeitar o teto de
//! LOC de produção (700).
//!
//! Vértices e segmentos são endereçados pelo **índice plano** do módulo
//! `compound` (primário, depois cada subpath) — então editar o buraco de uma
//! rosquinha usa exatamente as mesmas chamadas de editar a borda de fora. Num
//! path de contorno único o índice plano É o índice de `verts`.

use crate::compound::contour_segments;
use crate::{VecPath, VecVertex, VertexKind};

/// Fração do vão ao vizinho usada como comprimento de handle no auto-smooth
/// (Corner→Smooth com handles degenerados). 1/3 é o default de facto (Inkscape).
const AUTO_SMOOTH_FRAC: f64 = 1.0 / 3.0;

/// Retipa o vértice de índice PLANO `i` de `path` para `kind`, ajustando os
/// handles conforme a restrição do tipo (o núcleo da edição rica de handles,
/// ADR-0108 Fase 1):
///
/// - **Corner**: mantém as posições dos handles (vira cusp de handles
///   independentes; se colineares antes, continuam onde estão até o próximo drag).
/// - **Smooth**: torna os handles **colineares** preservando cada comprimento.
/// - **Symmetric**: colineares **e** comprimento igual (média).
///
/// A tangente vem dos handles atuais (`out_rel − in_rel`); se ambos forem
/// degenerados (cusp reto), é **sintetizada dos vizinhos** (auto-smooth):
/// tangente = direção `prev→next`, comprimento = [`AUTO_SMOOTH_FRAC`] do vão.
/// Retorna `true` se algo mudou. Puro; sem trig além de `sqrt` (normalização).
#[must_use]
pub fn retype_vertex(path: &mut VecPath, i: usize, kind: VertexKind) -> bool {
    let Some((c, local)) = path.locate_vert(i) else {
        return false;
    };
    let Some((verts, closed)) = path.contour_mut(c) else {
        return false;
    };
    retype_in_contour(verts, *closed, local, kind)
}

/// Torna o vértice de índice PLANO `i` uma **quina afiada**: recolhe os dois handles na
/// âncora (zero) e marca `Corner`. É o que o [`crate::corner_live`] precisa para haver
/// ângulo a arredondar — um vértice `Smooth` tem os handles COLINEARES, e `corner_at` não
/// vê quina nele (não há ângulo). Recolher os handles cria a dobra; as tangentes da quina
/// caem na cascata do `tangent_at_start`/`tangent_at_end` (um controle nulo desce para o
/// vizinho), então a quina fica bem-definida a partir das âncoras vizinhas.
///
/// É a metade "primeiro transforma em quina" das ferramentas Fillet/Chamfer: quem clica um
/// ponto suave quer arredondá-lo, e para isso ele precisa virar quina antes. Devolve `true`
/// se algo mudou (handles já recolhidos e já `Corner` ⇒ `false`, sem passo de undo espúrio).
#[must_use]
pub fn make_sharp_corner(path: &mut VecPath, i: usize) -> bool {
    let Some(v) = path.vert_mut(i) else {
        return false;
    };
    let before = *v;
    v.in_handle = v.anchor;
    v.out_handle = v.anchor;
    v.kind = VertexKind::Corner;
    *v != before
}

/// [`retype_vertex`] dentro de um contorno já resolvido (índice LOCAL).
fn retype_in_contour(verts: &mut [VecVertex], closed: bool, i: usize, kind: VertexKind) -> bool {
    let before = verts[i];
    let a = before.anchor;

    if kind == VertexKind::Corner {
        // Cusp: só marca o tipo; posições preservadas (independentes a partir daqui).
        verts[i].kind = VertexKind::Corner;
        return verts[i] != before;
    }

    // Handles atuais relativos à âncora + comprimentos.
    let in_rel = [before.in_handle[0] - a[0], before.in_handle[1] - a[1]];
    let out_rel = [before.out_handle[0] - a[0], before.out_handle[1] - a[1]];
    let li = (in_rel[0] * in_rel[0] + in_rel[1] * in_rel[1]).sqrt();
    let lo = (out_rel[0] * out_rel[0] + out_rel[1] * out_rel[1]).sqrt();
    let degenerate = li < 1e-9 && lo < 1e-9;

    // Tangente dos vizinhos (para auto-smooth quando degenerado / sem direção).
    let neighbor = neighbor_tangent(verts, closed, i);

    // Direção da tangente unitária.
    let tan = if degenerate {
        match neighbor {
            Some((t, _)) => t,
            None => return false, // nada de que sintetizar (path minúsculo)
        }
    } else {
        // out_rel − in_rel aponta ao longo da tangente (out no +t, in no −t).
        let d = [out_rel[0] - in_rel[0], out_rel[1] - in_rel[1]];
        match normalize(d).or_else(|| neighbor.map(|(t, _)| t)) {
            Some(t) => t,
            None => return false,
        }
    };

    let (len_in, len_out) = if degenerate {
        let base = neighbor.map(|(_, b)| b).unwrap_or(0.0);
        (base, base)
    } else if kind == VertexKind::Symmetric {
        let m = (li + lo) * 0.5;
        (m, m)
    } else {
        (li, lo) // Smooth preserva comprimentos
    };

    verts[i].out_handle = [a[0] + tan[0] * len_out, a[1] + tan[1] * len_out];
    verts[i].in_handle = [a[0] - tan[0] * len_in, a[1] - tan[1] * len_in];
    verts[i].kind = kind;
    verts[i] != before
}

/// Tangente unitária `prev→next` no vértice `i` do contorno (wrap se fechado) +
/// comprimento de handle sugerido ([`AUTO_SMOOTH_FRAC`] do meio-vão). `None` se
/// não houver vizinhos utilizáveis (contorno degenerado) ou eles coincidirem.
fn neighbor_tangent(verts: &[VecVertex], closed: bool, i: usize) -> Option<([f64; 2], f64)> {
    let n = verts.len();
    let a = verts[i].anchor;
    let prev = if i > 0 {
        Some(verts[i - 1].anchor)
    } else if closed {
        Some(verts[n - 1].anchor)
    } else {
        None
    };
    let next = if i + 1 < n {
        Some(verts[i + 1].anchor)
    } else if closed {
        Some(verts[0].anchor)
    } else {
        None
    };
    // Endpoints de contorno aberto usam a própria âncora como o vizinho ausente.
    let (p, q) = match (prev, next) {
        (Some(p), Some(q)) => (p, q),
        (Some(p), None) => (p, a),
        (None, Some(q)) => (a, q),
        (None, None) => return None,
    };
    let d = [q[0] - p[0], q[1] - p[1]];
    let len = (d[0] * d[0] + d[1] * d[1]).sqrt();
    if len < 1e-12 {
        return None;
    }
    Some(([d[0] / len, d[1] / len], len * 0.5 * AUTO_SMOOTH_FRAC))
}

/// Comprimento² mínimo para uma alça contar como "visível" (não-degenerada) — o
/// mesmo limiar que o overlay e o hit-test usam para pular alças coincidentes.
const GHOST_EPS_SQ: f64 = 1e-18;

/// Posição de EXIBIÇÃO de uma alça (in/out) de um vértice, para que um ponto
/// Smooth/Symmetric com alça de comprimento zero ("invisível") ainda mostre um toco
/// agarrável — SEM tocar a geometria (a curva só muda quando o usuário arrasta o
/// toco). Devolve:
/// - a alça REAL quando já está deslocada da âncora (não-degenerada);
/// - para Smooth/Symmetric com alça zero: um deslocamento sintético ao longo da
///   tangente suave — oposto à outra alça (com o comprimento dela) se ela estiver
///   deslocada, senão a tangente dos vizinhos —, para que os DOIS lados de um ponto
///   suave apareçam (a "alça lateral" antes invisível);
/// - `None` para a alça zero de um Corner (quina reta não mostra nada, de propósito)
///   ou quando não há de que sintetizar (contorno degenerado).
///
/// `out` escolhe a alça de saída (`true`) ou de entrada (`false`). Em espaço LOCAL do
/// path (igual às alças do `VecVertex`); o chamador aplica o Transform. Pura (só lê).
#[must_use]
pub fn ghost_handle(path: &VecPath, i: usize, out: bool) -> Option<[f64; 2]> {
    let (c, local) = path.locate_vert(i)?;
    let (verts, closed) = path.contour(c)?;
    ghost_in_contour(verts, closed, local, out)
}

fn ghost_in_contour(verts: &[VecVertex], closed: bool, i: usize, out: bool) -> Option<[f64; 2]> {
    let v = verts.get(i)?;
    let a = v.anchor;
    let (this, other) = if out {
        (v.out_handle, v.in_handle)
    } else {
        (v.in_handle, v.out_handle)
    };
    if sq_dist(this, a) > GHOST_EPS_SQ {
        return Some(this); // alça real já visível
    }
    if v.kind == VertexKind::Corner {
        return None; // a quina de um cusp fica oculta
    }
    if sq_dist(other, a) > GHOST_EPS_SQ {
        // Oposto à outra alça (continuação suave), mesmo comprimento.
        let d = normalize([a[0] - other[0], a[1] - other[1]])?;
        let len = sq_dist(other, a).sqrt();
        Some([a[0] + d[0] * len, a[1] + d[1] * len])
    } else {
        // Ambas zero: tangente dos vizinhos (out no +t, in no −t).
        let (t, base) = neighbor_tangent(verts, closed, i)?;
        let dir = if out { t } else { [-t[0], -t[1]] };
        Some([a[0] + dir[0] * base, a[1] + dir[1] * base])
    }
}

fn sq_dist(p: [f64; 2], a: [f64; 2]) -> f64 {
    let (dx, dy) = (p[0] - a[0], p[1] - a[1]);
    dx * dx + dy * dy
}

/// Normaliza `v`; `None` se ~zero.
fn normalize(v: [f64; 2]) -> Option<[f64; 2]> {
    let l = (v[0] * v[0] + v[1] * v[1]).sqrt();
    if l < 1e-9 {
        None
    } else {
        Some([v[0] / l, v[1] / l])
    }
}

/// Interpolação linear.
fn lerp(a: [f64; 2], b: [f64; 2], t: f64) -> [f64; 2] {
    [a[0] + (b[0] - a[0]) * t, a[1] + (b[1] - a[1]) * t]
}

/// Avalia a cúbica de Bézier (P0,P1,P2,P3) em `t` (base de Bernstein).
pub(crate) fn cubic_at(p0: [f64; 2], p1: [f64; 2], p2: [f64; 2], p3: [f64; 2], t: f64) -> [f64; 2] {
    let u = 1.0 - t;
    let (w0, w1, w2, w3) = (u * u * u, 3.0 * u * u * t, 3.0 * u * t * t, t * t * t);
    [
        w0 * p0[0] + w1 * p1[0] + w2 * p2[0] + w3 * p3[0],
        w0 * p0[1] + w1 * p1[1] + w2 * p2[1] + w3 * p3[1],
    ]
}

/// Divide o segmento cúbico de índice PLANO `seg` no parâmetro `t ∈ [0,1]` via
/// **de Casteljau**, inserindo um vértice **Smooth** novo — a FORMA é preservada
/// exatamente (as duas cúbicas resultantes somam a original). Ajusta o out-handle
/// do vértice anterior e o in-handle do seguinte. Devolve o índice PLANO do
/// vértice novo, ou `None` se o segmento não existe. É o núcleo do "inserir
/// vértice num segmento" (ADR-0108 Fase 1).
pub fn split_segment(path: &mut VecPath, seg: usize, t: f64) -> Option<usize> {
    let (c, local) = path.locate_segment(seg)?;
    {
        let (verts, _) = path.contour_mut(c)?;
        let n = verts.len();
        let a = local;
        let b = (local + 1) % n;
        let t = t.clamp(0.0, 1.0);
        let (p0, p1) = (verts[a].anchor, verts[a].out_handle);
        let (p2, p3) = (verts[b].in_handle, verts[b].anchor);
        let q0 = lerp(p0, p1, t);
        let q1 = lerp(p1, p2, t);
        let q2 = lerp(p2, p3, t);
        let r0 = lerp(q0, q1, t);
        let r1 = lerp(q1, q2, t);
        let s = lerp(r0, r1, t);
        // Ajusta os vizinhos ANTES de inserir (índices ainda válidos).
        verts[a].out_handle = q0;
        verts[b].in_handle = q2;
        verts.insert(
            a + 1,
            VecVertex {
                anchor: s,
                in_handle: r0,
                out_handle: r1,
                kind: VertexKind::Smooth,
                corner_radius: 0.0,
            },
        );
    }
    path.flat_vert(c, local + 1)
}

/// **Reformar um SEGMENTO** — arrastar a curva entre duas âncoras sem mexer na topologia dela.
///
/// O ponto do segmento no parâmetro `t` anda exatamente `delta`; as duas âncoras ficam onde
/// estão, e nenhum vértice nasce nem morre. É o gesto normal do Illustrator (Direct Selection) e
/// do Inkscape: *"não se pode reformar uma curva sem alterar a topologia dela"* era a ausência.
///
/// # A distribuição é EXATA, não uma aproximação
///
/// Uma cúbica é **linear nos seus pontos de controle**: `C(t) = B₀P₀ + B₁P₁ + B₂P₂ + B₃P₃`. Mover
/// só `P₁` e `P₂` dá `ΔC(t) = B₁ΔP₁ + B₂ΔP₂`, e a solução de **norma mínima** de
/// `B₁ΔP₁ + B₂ΔP₂ = delta` é `ΔPₖ = delta · Bₖ / (B₁² + B₂²)`. Substituindo:
/// `ΔC(t) = delta·(B₁² + B₂²)/(B₁² + B₂²) = delta`, **ao bit**. Não há iteração nem ajuste — o
/// ponto agarrado segue o dedo por identidade algébrica. (É o que o Inkscape faz.)
///
/// ⚠️ **O `t` é clampado a `[0.05, 0.95]`**: nas pontas `B₁` e `B₂` vão a zero juntos e o sistema
/// fica sem solução — nenhum movimento de handle move a curva NA âncora, que é justamente o que
/// uma âncora é. Perto delas o hit-test da âncora já vence, então o clamp só protege o degenerado.
///
/// `seg` é o índice **PLANO** (a mesma convenção do [`nearest_point_on_path`] e do
/// [`split_segment`], e é dali que ele vem). `false` se o segmento não existe.
pub fn reshape_segment(path: &mut VecPath, seg: usize, t: f64, delta: [f64; 2]) -> bool {
    let Some((c, local)) = path.locate_segment(seg) else {
        return false;
    };
    let Some((verts, _)) = path.contour_mut(c) else {
        return false;
    };
    let n = verts.len();
    if n < 2 || local >= n {
        return false;
    }
    let t = t.clamp(0.05, 0.95);
    let mt = 1.0 - t;
    let b1 = 3.0 * mt * mt * t;
    let b2 = 3.0 * mt * t * t;
    let denom = b1 * b1 + b2 * b2;
    if denom <= 0.0 {
        return false;
    }
    let (k1, k2) = (b1 / denom, b2 / denom);
    let b = (local + 1) % n;
    verts[local].out_handle[0] += delta[0] * k1;
    verts[local].out_handle[1] += delta[1] * k1;
    verts[b].in_handle[0] += delta[0] * k2;
    verts[b].in_handle[1] += delta[1] * k2;
    true
}

/// **O ponto do segmento `seg` no parâmetro `t`** — o mesmo índice PLANO do
/// [`nearest_point_on_path`]. É o que um arrasto de segmento precisa para saber quanto o dedo se
/// afastou do ponto que agarrou; sem ele, quem arrasta teria de re-derivar a cúbica por conta
/// própria, e essa segunda leitura discordaria do [`reshape_segment`] no dia em que a convenção
/// de índice mudasse.
#[must_use]
pub fn point_on_segment(path: &VecPath, seg: usize, t: f64) -> Option<[f64; 2]> {
    let (c, local) = path.locate_segment(seg)?;
    let (verts, _) = path.contour(c)?;
    let n = verts.len();
    if n < 2 || local >= n {
        return None;
    }
    let b = (local + 1) % n;
    Some(cubic_at(
        verts[local].anchor,
        verts[local].out_handle,
        verts[b].in_handle,
        verts[b].anchor,
        t.clamp(0.0, 1.0),
    ))
}

/// Ponto mais próximo de `p` sobre os segmentos de `path` (todos os contornos;
/// amostragem uniforme, `samples` por segmento). Devolve `(seg PLANO, t, dist²)` —
/// o alvo do "clicar perto do traço pra inserir um vértice" — ou `None` se não há
/// segmentos.
#[must_use]
pub fn nearest_point_on_path(
    path: &VecPath,
    p: [f64; 2],
    samples: u32,
) -> Option<(usize, f64, f64)> {
    let s = samples.max(2);
    let mut best: Option<(usize, f64, f64)> = None;
    let mut flat = 0usize;
    for c in 0..path.contour_count() {
        let (verts, closed) = path.contour(c)?;
        let n = verts.len();
        for seg in 0..contour_segments(verts, closed) {
            let (a, b) = (seg, (seg + 1) % n);
            let (p0, p1) = (verts[a].anchor, verts[a].out_handle);
            let (p2, p3) = (verts[b].in_handle, verts[b].anchor);
            for k in 0..=s {
                let t = f64::from(k) / f64::from(s);
                let cpt = cubic_at(p0, p1, p2, p3, t);
                let (dx, dy) = (cpt[0] - p[0], cpt[1] - p[1]);
                let d2 = dx * dx + dy * dy;
                let better = match best {
                    None => true,
                    Some((_, _, bd)) => d2 < bd,
                };
                if better {
                    best = Some((flat + seg, t, d2));
                }
            }
        }
        flat += contour_segments(verts, closed);
    }
    best
}
