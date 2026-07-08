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
}
