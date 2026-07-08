//! Transformações de path inteiro (ADR-0108): flip / rotate 90° / bbox /
//! translate / scale / smooth / sharpen. Extraído de `lib.rs` para respeitar o
//! teto de LOC de produção (700). Blocos `impl VecScene` inerentes podem viver
//! em qualquer módulo da crate — a API pública fica idêntica.

use crate::{FlipAxis, Rotate90, VecPathId, VecScene, VecVertex, VertexKind};

impl VecScene {
    /// Espelha o path `id` no eixo `axis`, em torno do centro da bbox dos seus
    /// pontos de controle (âncora + 2 handles de cada vértice). Reflete os três
    /// pontos de cada vértice — a forma inverte, a ordem/topologia dos vértices
    /// fica igual. `false` se o id sumiu ou o path está vazio.
    pub fn flip_path(&mut self, id: VecPathId, axis: FlipAxis) -> bool {
        let Some(path) = self.paths.iter_mut().find(|p| p.id == id) else {
            return false;
        };
        if path.verts.is_empty() {
            return false;
        }
        let a = match axis {
            FlipAxis::Horizontal => 0,
            FlipAxis::Vertical => 1,
        };
        let (mut lo, mut hi) = (f64::INFINITY, f64::NEG_INFINITY);
        for v in &path.verts {
            for p in [v.anchor, v.in_handle, v.out_handle] {
                lo = lo.min(p[a]);
                hi = hi.max(p[a]);
            }
        }
        let twice_center = lo + hi;
        for v in &mut path.verts {
            v.anchor[a] = twice_center - v.anchor[a];
            v.in_handle[a] = twice_center - v.in_handle[a];
            v.out_handle[a] = twice_center - v.out_handle[a];
        }
        true
    }

    /// Rotaciona o path `id` em 90° ([`Rotate90`]) em torno do centro da bbox dos
    /// seus pontos de controle. Quarto-de-volta = troca de eixo + sinal (sem
    /// transcendentais, HR-5); handles rotacionam junto, `VertexKind` preservado
    /// (colinearidade é invariante a rotação). `false` se o id sumiu ou vazio.
    pub fn rotate_path(&mut self, id: VecPathId, dir: Rotate90) -> bool {
        let Some(path) = self.paths.iter_mut().find(|p| p.id == id) else {
            return false;
        };
        if path.verts.is_empty() {
            return false;
        }
        let (mut lo, mut hi) = ([f64::INFINITY; 2], [f64::NEG_INFINITY; 2]);
        for v in &path.verts {
            for p in [v.anchor, v.in_handle, v.out_handle] {
                lo[0] = lo[0].min(p[0]);
                lo[1] = lo[1].min(p[1]);
                hi[0] = hi[0].max(p[0]);
                hi[1] = hi[1].max(p[1]);
            }
        }
        let (cx, cy) = ((lo[0] + hi[0]) * 0.5, (lo[1] + hi[1]) * 0.5);
        // Screen convention (Y down): CW maps (dx,dy)→(−dy, dx); CCW →(dy, −dx).
        let rot = |p: [f64; 2]| {
            let (dx, dy) = (p[0] - cx, p[1] - cy);
            match dir {
                Rotate90::Cw => [cx - dy, cy + dx],
                Rotate90::Ccw => [cx + dy, cy - dx],
            }
        };
        for v in &mut path.verts {
            v.anchor = rot(v.anchor);
            v.in_handle = rot(v.in_handle);
            v.out_handle = rot(v.out_handle);
        }
        true
    }

    /// Bounding box das ÂNCORAS do path `id` (`(min, max)` em world-units) — a
    /// extensão usada pelo readout de transform (posição/tamanho). `None` se o id
    /// sumiu ou o path está vazio.
    pub fn path_bbox(&self, id: VecPathId) -> Option<([f64; 2], [f64; 2])> {
        let path = self.paths.iter().find(|p| p.id == id)?;
        let first = path.verts.first()?;
        let (mut lo, mut hi) = (first.anchor, first.anchor);
        for v in &path.verts {
            lo[0] = lo[0].min(v.anchor[0]);
            lo[1] = lo[1].min(v.anchor[1]);
            hi[0] = hi[0].max(v.anchor[0]);
            hi[1] = hi[1].max(v.anchor[1]);
        }
        Some((lo, hi))
    }

    /// Translada o path `id` por `(dx, dy)` world-units (âncora + handles de todos
    /// os vértices). `false` se o id sumiu.
    pub fn translate_path(&mut self, id: VecPathId, dx: f64, dy: f64) -> bool {
        let Some(path) = self.paths.iter_mut().find(|p| p.id == id) else {
            return false;
        };
        for v in &mut path.verts {
            v.anchor = [v.anchor[0] + dx, v.anchor[1] + dy];
            v.in_handle = [v.in_handle[0] + dx, v.in_handle[1] + dy];
            v.out_handle = [v.out_handle[0] + dx, v.out_handle[1] + dy];
        }
        true
    }

    /// Escala o path `id` por `(sx, sy)` em torno de `pivot` world-units (âncora +
    /// handles). `false` se o id sumiu.
    pub fn scale_path(&mut self, id: VecPathId, sx: f64, sy: f64, pivot: [f64; 2]) -> bool {
        let Some(path) = self.paths.iter_mut().find(|p| p.id == id) else {
            return false;
        };
        let s = |p: [f64; 2]| {
            [
                pivot[0] + (p[0] - pivot[0]) * sx,
                pivot[1] + (p[1] - pivot[1]) * sy,
            ]
        };
        for v in &mut path.verts {
            v.anchor = s(v.anchor);
            v.in_handle = s(v.in_handle);
            v.out_handle = s(v.out_handle);
        }
        true
    }

    /// Suaviza TODOS os vértices do path `id` de forma **consistente e incremental**.
    ///
    /// Cada vértice vira `Smooth` com handles ao longo da tangente de Catmull-Rom
    /// (direção `prev→next`, calculada SEMPRE a partir das âncoras — não dos handles
    /// atuais — para que todo ponto suavize pela MESMA regra, independente de edições
    /// anteriores). O comprimento é uma FRAÇÃO do vão à âncora vizinha; a fração
    /// **cresce a cada clique** ([`SMOOTH_GROWTH`]) a partir de [`SMOOTH_BASE_FRAC`]
    /// (Catmull-Rom uniforme) até saturar em [`SMOOTH_MAX_FRAC`] — a forma fica
    /// redonda e clicar de novo não muda mais nada (retorna `false`). O "nível" de
    /// suavização é lido do próprio comprimento atual do handle, sem estado externo.
    ///
    /// `false` se o id sumiu, o path tem < 3 vértices, ou já saturou. Só `sqrt`
    /// (normalização) — sem transcendentais (HR-5).
    pub fn smooth_path(&mut self, id: VecPathId) -> bool {
        /// Fração inicial do vão (Catmull-Rom uniforme) no 1º clique.
        const SMOOTH_BASE_FRAC: f64 = 1.0 / 3.0;
        /// Multiplicador da fração por clique (cresce até saturar).
        const SMOOTH_GROWTH: f64 = 1.2;
        /// Fração máxima: acima disso a curva passaria a "estufar"/laçar; ~aqui um
        /// polígono regular já parece um círculo.
        const SMOOTH_MAX_FRAC: f64 = 0.45;

        let Some(path) = self.paths.iter_mut().find(|p| p.id == id) else {
            return false;
        };
        let n = path.verts.len();
        if n < 3 {
            return false;
        }
        // Snapshot das âncoras: a tangente de cada vértice depende dos vizinhos, que
        // não podem ser lidos com o path já emprestado mutável vértice-a-vértice.
        let anchors: Vec<[f64; 2]> = path.verts.iter().map(|v| v.anchor).collect();
        let closed = path.closed;
        let mut changed = false;
        for i in 0..n {
            let a = anchors[i];
            let prev = if i > 0 {
                anchors[i - 1]
            } else if closed {
                anchors[n - 1]
            } else {
                a
            };
            let next = if i + 1 < n {
                anchors[i + 1]
            } else if closed {
                anchors[0]
            } else {
                a
            };
            // Tangente de Catmull-Rom (prev→next) — a MESMA regra para todo ponto.
            let dir = [next[0] - prev[0], next[1] - prev[1]];
            let dl = (dir[0] * dir[0] + dir[1] * dir[1]).sqrt();
            if dl < 1e-12 {
                continue; // vizinhos coincidentes — sem direção definível.
            }
            let u = [dir[0] / dl, dir[1] / dl];
            let d_out = [next[0] - a[0], next[1] - a[1]];
            let d_in = [a[0] - prev[0], a[1] - prev[1]];
            let chord_out = (d_out[0] * d_out[0] + d_out[1] * d_out[1]).sqrt();
            let chord_in = (d_in[0] * d_in[0] + d_in[1] * d_in[1]).sqrt();

            let v = &mut path.verts[i];
            // Nível atual = fração do handle out em relação ao vão (0 se degenerado).
            let out_rel = [v.out_handle[0] - a[0], v.out_handle[1] - a[1]];
            let cur_len = (out_rel[0] * out_rel[0] + out_rel[1] * out_rel[1]).sqrt();
            let cur_frac = if chord_out > 1e-12 {
                cur_len / chord_out
            } else {
                0.0
            };
            // Cresce a fração e satura: o piso (base) cobre o 1º clique degenerado,
            // o teto (max) garante convergência (base < max).
            let frac = (cur_frac * SMOOTH_GROWTH).clamp(SMOOTH_BASE_FRAC, SMOOTH_MAX_FRAC);
            let updated = VecVertex {
                anchor: a,
                in_handle: [a[0] - u[0] * frac * chord_in, a[1] - u[1] * frac * chord_in],
                out_handle: [
                    a[0] + u[0] * frac * chord_out,
                    a[1] + u[1] * frac * chord_out,
                ],
                kind: VertexKind::Smooth,
            };
            if *v != updated {
                *v = updated;
                changed = true;
            }
        }
        changed
    }

    /// Aguça TODOS os vértices do path `id`: colapsa os handles sobre a âncora
    /// (segmentos retos) e marca cada vértice como `Corner`. Inverso de
    /// [`Self::smooth_path`]. `true` se algo mudou; `false` se o id sumiu.
    pub fn sharpen_path(&mut self, id: VecPathId) -> bool {
        let Some(path) = self.paths.iter_mut().find(|p| p.id == id) else {
            return false;
        };
        let mut changed = false;
        for v in &mut path.verts {
            let flat = VecVertex {
                anchor: v.anchor,
                in_handle: v.anchor,
                out_handle: v.anchor,
                kind: VertexKind::Corner,
            };
            if *v != flat {
                *v = flat;
                changed = true;
            }
        }
        changed
    }

    /// Simplifica o path `id` de forma **progressiva e fiel à curva**.
    ///
    /// Cada clique remove ~[`SIMPLIFY_REMOVE_FRAC`] dos vértices menos significativos
    /// (guloso: sempre o de MENOR distorção primeiro), até o piso de [`SIMPLIFY_MIN`]
    /// pontos — então vira no-op (`false`). Ao remover um vértice, os handles dos
    /// vizinhos são **re-ajustados** (fit cúbico que passa pela âncora removida com as
    /// mesmas tangentes) para que a cúbica resultante siga a curva original em vez de
    /// virar uma corda reta — a forma permanece fiel. "Distorção" de um vértice = a
    /// maior distância da curva original (os 2 segmentos que o cercam) à cúbica
    /// re-ajustada. Endpoints de path aberto são fixos. Só `sqrt` (HR-5).
    pub fn simplify_path(&mut self, id: VecPathId) -> bool {
        /// Fração de vértices removida por clique (progressivo).
        const SIMPLIFY_REMOVE_FRAC: f64 = 0.2;
        /// Piso de vértices — a forma nunca desce abaixo disso.
        const SIMPLIFY_MIN: usize = 3;

        let Some(path) = self.paths.iter_mut().find(|p| p.id == id) else {
            return false;
        };
        let closed = path.closed;
        let n0 = path.verts.len();
        if n0 <= SIMPLIFY_MIN {
            return false;
        }
        let drop = ((n0 as f64) * SIMPLIFY_REMOVE_FRAC).ceil() as usize;
        let target = SIMPLIFY_MIN.max(n0 - drop);

        let mut changed = false;
        while path.verts.len() > target {
            let m = path.verts.len();
            if m <= SIMPLIFY_MIN {
                break;
            }
            // Vértice removível cuja remoção MENOS distorce a curva (com refit).
            let mut best: Option<(usize, f64, [f64; 2], [f64; 2])> = None;
            for i in 0..m {
                if !closed && (i == 0 || i == m - 1) {
                    continue; // endpoints de path aberto são fixos
                }
                let prev = &path.verts[(i + m - 1) % m];
                let mid = &path.verts[i];
                let next = &path.verts[(i + 1) % m];
                let (out, inn, dist) = merged_segment_fit(prev, mid, next);
                if best.is_none_or(|(_, bd, _, _)| dist < bd) {
                    best = Some((i, dist, out, inn));
                }
            }
            let Some((i, _dist, out, inn)) = best else {
                break;
            };
            let pi = (i + m - 1) % m;
            let ni = (i + 1) % m;
            path.verts[pi].out_handle = out;
            path.verts[ni].in_handle = inn;
            path.verts.remove(i);
            changed = true;
        }
        changed
    }
}

/// Ajusta uma ÚNICA cúbica `prev→next` que substitui os dois segmentos originais
/// `prev→mid→next` ao remover `mid`, e mede a distorção. A cúbica sai de `prev` e
/// chega em `next` pelas MESMAS direções de tangente de antes e é forçada a passar
/// pela âncora de `mid` (na fração de comprimento de corda) — resolvendo os dois
/// comprimentos de handle por um sistema 2×2. Devolve `(out_handle_prev,
/// in_handle_next, distorção)`, onde distorção é a maior distância da curva original
/// à nova (amostrada). Só `sqrt`.
fn merged_segment_fit(
    prev: &VecVertex,
    mid: &VecVertex,
    next: &VecVertex,
) -> ([f64; 2], [f64; 2], f64) {
    let p0 = prev.anchor;
    let p3 = next.anchor;
    // Direções de tangente atuais (fallback: ao longo da corda prev→next).
    let chord = unit([p3[0] - p0[0], p3[1] - p0[1]]).unwrap_or([1.0, 0.0]);
    let t1 = unit([prev.out_handle[0] - p0[0], prev.out_handle[1] - p0[1]]).unwrap_or(chord);
    let t2 = unit([next.in_handle[0] - p3[0], next.in_handle[1] - p3[1]])
        .unwrap_or([-chord[0], -chord[1]]);
    let through = mid.anchor;
    let d1 = dist(p0, through);
    let d2 = dist(through, p3);
    let t = if d1 + d2 > 1e-12 {
        (d1 / (d1 + d2)).clamp(0.05, 0.95)
    } else {
        0.5
    };
    // Base de Bernstein em t: through = c0·P0 + k1·P1 + k2·P2 + c3·P3, com
    // P1 = P0 + a·t1, P2 = P3 + b·t2 → resolve (a, b).
    let mt = 1.0 - t;
    let c0 = mt * mt * mt + 3.0 * mt * mt * t;
    let c3 = 3.0 * mt * t * t + t * t * t;
    let k1 = 3.0 * mt * mt * t;
    let k2 = 3.0 * mt * t * t;
    let r = [
        through[0] - c0 * p0[0] - c3 * p3[0],
        through[1] - c0 * p0[1] - c3 * p3[1],
    ];
    let cross = t1[0] * t2[1] - t1[1] * t2[0];
    let det = k1 * k2 * cross;
    let chord_len = dist(p0, p3);
    let (a, b) = if det.abs() < 1e-9 {
        (chord_len / 3.0, chord_len / 3.0) // tangentes ~paralelas → default
    } else {
        let a = (r[0] * (k2 * t2[1]) - (k2 * t2[0]) * r[1]) / det;
        let b = ((k1 * t1[0]) * r[1] - r[0] * (k1 * t1[1])) / det;
        (a, b)
    };
    // Handles não-negativos e limitados (evita laços/estouros no fit degenerado).
    let a = a.clamp(0.0, 3.0 * chord_len);
    let b = b.clamp(0.0, 3.0 * chord_len);
    let out = [p0[0] + a * t1[0], p0[1] + a * t1[1]];
    let inn = [p3[0] + b * t2[0], p3[1] + b * t2[1]];

    // Distorção: maior distância da curva original (2 segmentos) à nova cúbica.
    let seg1 = [p0, prev.out_handle, mid.in_handle, through];
    let seg2 = [through, mid.out_handle, next.in_handle, p3];
    let newc = [p0, out, inn, p3];
    let mut maxd: f64 = 0.0;
    const OUTER: usize = 8;
    for seg in [seg1, seg2] {
        for k in 0..=OUTER {
            let s = k as f64 / OUTER as f64;
            let pt = cubic_pt(&seg, s);
            maxd = maxd.max(min_dist_to_cubic(pt, &newc));
        }
    }
    (out, inn, maxd)
}

/// Menor distância de `p` a uma cúbica `seg` (amostragem uniforme).
fn min_dist_to_cubic(p: [f64; 2], seg: &[[f64; 2]; 4]) -> f64 {
    const SAMPLES: usize = 12;
    let mut best = f64::INFINITY;
    for k in 0..=SAMPLES {
        let t = k as f64 / SAMPLES as f64;
        best = best.min(dist(p, cubic_pt(seg, t)));
    }
    best
}

/// Cúbica de Bézier (P0,P1,P2,P3) avaliada em `t` (Bernstein).
fn cubic_pt(seg: &[[f64; 2]; 4], t: f64) -> [f64; 2] {
    let u = 1.0 - t;
    let (w0, w1, w2, w3) = (u * u * u, 3.0 * u * u * t, 3.0 * u * t * t, t * t * t);
    [
        w0 * seg[0][0] + w1 * seg[1][0] + w2 * seg[2][0] + w3 * seg[3][0],
        w0 * seg[0][1] + w1 * seg[1][1] + w2 * seg[2][1] + w3 * seg[3][1],
    ]
}

/// Distância euclidiana.
fn dist(a: [f64; 2], b: [f64; 2]) -> f64 {
    (a[0] - b[0]).hypot(a[1] - b[1])
}

/// Normaliza `v`; `None` se ~zero.
fn unit(v: [f64; 2]) -> Option<[f64; 2]> {
    let l = (v[0] * v[0] + v[1] * v[1]).sqrt();
    if l < 1e-12 {
        None
    } else {
        Some([v[0] / l, v[1] / l])
    }
}
