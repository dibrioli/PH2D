//! Transformações de path inteiro (ADR-0108): flip / rotate 90° / bbox /
//! translate / scale / containment. Extraído de `lib.rs` para respeitar o teto de
//! LOC de produção (700); reshape (smooth/simplify/…) mora em `reshape.rs`.
//! Blocos `impl VecScene` inerentes podem viver em qualquer módulo da crate — a
//! API pública fica idêntica.
//!
//! Tudo aqui varre **todos os contornos** de um compound path (`VecPath::subpaths`):
//! mover/escalar/rodar leva o buraco junto, e as bboxes enquadram a forma toda.

use crate::compound::contour_segments;
use crate::{FillRule, FlipAxis, Paint, Rotate90, VecPath, VecPathId, VecScene, VecVertex};

/// Amostras por segmento cúbico nas varreduras de geometria (bbox / containment).
/// Apertado o bastante p/ o gizmo e transcendental-free.
const CURVE_SAMPLES: usize = 16;

/// Avalia a cúbica (P0,P1,P2,P3) em `t` (base de Bernstein).
fn cubic(p0: [f64; 2], p1: [f64; 2], p2: [f64; 2], p3: [f64; 2], t: f64) -> [f64; 2] {
    let u = 1.0 - t;
    let (uu, tt) = (u * u, t * t);
    let (b0, b1, b2, b3) = (uu * u, 3.0 * uu * t, 3.0 * u * tt, tt * t);
    [
        b0 * p0[0] + b1 * p1[0] + b2 * p2[0] + b3 * p3[0],
        b0 * p0[1] + b1 * p1[1] + b2 * p2[1] + b3 * p3[1],
    ]
}

impl VecScene {
    /// Espelha o path `id` no eixo `axis`, em torno do centro da bbox dos seus
    /// pontos de controle (âncora + 2 handles de cada vértice, de todos os
    /// contornos). Reflete os três pontos de cada vértice — a forma inverte, a
    /// ordem/topologia dos vértices fica igual. `false` se o id sumiu ou vazio.
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
        for v in path.verts_all() {
            for p in [v.anchor, v.in_handle, v.out_handle] {
                lo = lo.min(p[a]);
                hi = hi.max(p[a]);
            }
        }
        let twice_center = lo + hi;
        path.for_each_vert_mut(|v| {
            v.anchor[a] = twice_center - v.anchor[a];
            v.in_handle[a] = twice_center - v.in_handle[a];
            v.out_handle[a] = twice_center - v.out_handle[a];
        });
        // The gradient geometry mirrors with the shape (only the flipped axis).
        transform_fill_geometry(
            &mut path.fill,
            |mut p| {
                p[a] = twice_center - p[a];
                p
            },
            1.0,
        );
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
        for v in path.verts_all() {
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
        path.for_each_vert_mut(|v| {
            v.anchor = rot(v.anchor);
            v.in_handle = rot(v.in_handle);
            v.out_handle = rot(v.out_handle);
        });
        // Gradient geometry rotates with the shape (about the same pivot).
        transform_fill_geometry(&mut path.fill, rot, 1.0);
        true
    }

    /// **Assa** o afim `x` na geometria do path `id`. `false` se o id sumiu.
    pub fn transform_path(&mut self, id: VecPathId, x: &crate::Xform) -> bool {
        let Some(path) = self.paths.iter_mut().find(|p| p.id == id) else {
            return false;
        };
        bake_xform(path, x);
        true
    }

    /// Inverte a ordem dos vértices do contorno primário do path `id` — o 1º vira o
    /// último. Os handles in/out trocam de papel (o sentido inverteu). `false` se o
    /// id sumiu. Usado para orientar uma junção de paths (weld) na direção certa.
    pub fn reverse_path(&mut self, id: VecPathId) -> bool {
        let Some(path) = self.paths.iter_mut().find(|p| p.id == id) else {
            return false;
        };
        path.verts.reverse();
        for v in &mut path.verts {
            std::mem::swap(&mut v.in_handle, &mut v.out_handle);
        }
        true
    }

    /// O endpoint (1º ou último vértice) do path `id` **em coordenadas de MUNDO**,
    /// pelo afim de `xforms`. `None` se o id sumiu ou o path está vazio.
    #[must_use]
    pub fn endpoint_world(
        &self,
        xforms: &crate::VecXforms,
        id: VecPathId,
        first: bool,
    ) -> Option<[f64; 2]> {
        let path = self.paths.iter().find(|p| p.id == id)?;
        let v = if first {
            path.verts.first()?
        } else {
            path.verts.last()?
        };
        Some(crate::xform_of(xforms, id).apply(v.anchor))
    }

    /// Funde `src` no FIM de `dst`: anexa os vértices de `src` (convertidos do afim
    /// dele para o de `dst`) ao contorno primário de `dst`, e **remove** `src`. Os
    /// dois viram um só path. `reverse_src` percorre `src` de trás pra frente
    /// (trocando handles). `false` se um id sumiu ou o afim de `dst` é degenerado.
    ///
    /// É a primitiva do weld de endpoints: geometria local de `src` → mundo → local
    /// de `dst`, a mesma regra de fronteira. `weld_junction` funde o vértice de emenda
    /// (o 1º de `src`, que coincide com o último de `dst`) num só — o último de `dst`
    /// herda a tangente de saída de `src`, e a duplicata some. Sem ele, uma emenda
    /// deixaria dois vértices sobrepostos.
    pub fn merge_path_into(
        &mut self,
        dst: VecPathId,
        src: VecPathId,
        xforms: &crate::VecXforms,
        reverse_src: bool,
        weld_junction: bool,
    ) -> bool {
        let Some(inv_dst) = crate::xform_of(xforms, dst).inverse() else {
            return false;
        };
        let src_xf = crate::xform_of(xforms, src);
        let Some(src_path) = self.paths.iter().find(|p| p.id == src) else {
            return false;
        };
        let mut mapped: Vec<VecVertex> = src_path
            .verts
            .iter()
            .map(|v| VecVertex {
                anchor: inv_dst.apply(src_xf.apply(v.anchor)),
                in_handle: inv_dst.apply(src_xf.apply(v.in_handle)),
                out_handle: inv_dst.apply(src_xf.apply(v.out_handle)),
                kind: v.kind,
            })
            .collect();
        if reverse_src {
            mapped.reverse();
            for v in &mut mapped {
                std::mem::swap(&mut v.in_handle, &mut v.out_handle);
            }
        }
        let Some(d) = self.paths.iter_mut().find(|p| p.id == dst) else {
            return false;
        };
        if weld_junction
            && !mapped.is_empty()
            && let Some(last) = d.verts.last_mut()
        {
            // A emenda coincide (o weld só chega aqui quando coincide): o vértice de
            // junção é um só, com o in do lado de `dst` e o out do lado de `src`.
            last.out_handle = mapped[0].out_handle;
            mapped.remove(0);
        }
        d.verts.extend(mapped);
        self.remove_path(src);
        true
    }

    /// Move o endpoint (1º ou último vértice) do path `id` para que caia EXATAMENTE
    /// em `target_world`. Translada o vértice inteiro (âncora + os dois handles), de
    /// modo que a tangente ali se preserva — só o ponto de emenda anda. Usado no weld
    /// para a forma NOVA ceder ao nó pré-existente sem que este se mexa.
    fn snap_endpoint_world(
        &mut self,
        id: VecPathId,
        xforms: &crate::VecXforms,
        first: bool,
        target_world: [f64; 2],
    ) {
        let Some(inv) = crate::xform_of(xforms, id).inverse() else {
            return;
        };
        let t = inv.apply(target_world);
        if let Some(p) = self.paths.iter_mut().find(|p| p.id == id) {
            let v = if first {
                p.verts.first_mut()
            } else {
                p.verts.last_mut()
            };
            if let Some(v) = v {
                let d = [t[0] - v.anchor[0], t[1] - v.anchor[1]];
                for pt in [&mut v.anchor, &mut v.in_handle, &mut v.out_handle] {
                    pt[0] += d[0];
                    pt[1] += d[1];
                }
            }
        }
    }

    /// O deslocamento em MUNDO que encaixa a ponta mais próxima do path aberto `id`
    /// num nó de endpoint de OUTRO path aberto (≤ `tol`), ou `None` se nenhuma ponta
    /// está perto. O chamador soma isso ao `Transform` da forma — ela desliza inteira
    /// e RÍGIDA para o encaixe (nada de geometria distorcida), como o snap de um
    /// editor CAD. Depois um `weld_new_shape` funde de fato (a ponta já coincide).
    #[must_use]
    pub fn rigid_snap_delta(
        &self,
        id: VecPathId,
        xforms: &crate::VecXforms,
        tol: f64,
    ) -> Option<[f64; 2]> {
        let t2 = tol * tol;
        let d2 = |a: [f64; 2], b: [f64; 2]| (a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2);
        if self
            .paths
            .iter()
            .find(|p| p.id == id)
            .is_some_and(|p| p.closed)
        {
            return None;
        }
        let nf = self.endpoint_world(xforms, id, true)?;
        let nl = self.endpoint_world(xforms, id, false)?;
        let mut best: Option<([f64; 2], f64)> = None;
        for p in self
            .paths
            .iter()
            .filter(|p| p.id != id && !p.closed && !p.verts.is_empty())
        {
            let (Some(bf), Some(bl)) = (
                self.endpoint_world(xforms, p.id, true),
                self.endpoint_world(xforms, p.id, false),
            ) else {
                continue;
            };
            for npt in [nf, nl] {
                for bpt in [bf, bl] {
                    let dd = d2(npt, bpt);
                    if dd <= t2 && best.is_none_or(|(_, bd)| dd < bd) {
                        best = Some(([bpt[0] - npt[0], bpt[1] - npt[1]], dd));
                    }
                }
            }
        }
        best.map(|(delta, _)| delta)
    }

    /// **Solda** os endpoints da forma RECÉM-CRIADA `new_id` aos nós de endpoint de
    /// outros paths abertos que estejam a ≤ `tol` (mundo). É o que permite fechar uma
    /// forma com várias linhas/arcos/traços da pen: **basta os nós ficarem próximos**
    /// (Enio 2026-07-09). Devolve quantas soldas/fechamentos houve.
    ///
    /// A geometria PRÉ-EXISTENTE é sacrossanta — nunca se mexe. Quem cede é a forma
    /// nova: seu endpoint snapa EXATO no nó vizinho antes de fundir, então a curva já
    /// desenhada fica idêntica (sem o deslocamento de ~`tol` que uma emenda ingênua
    /// causaria). Se os dois extremos da forma nova acabam no mesmo ponto, ela fecha e
    /// (sendo agora uma região) recebe `fill_on_close`. `tol` é world-units.
    pub fn weld_new_shape(
        &mut self,
        new_id: VecPathId,
        xforms: &crate::VecXforms,
        tol: f64,
        fill_on_close: Option<Paint>,
    ) -> usize {
        let t2 = tol * tol;
        let d2 = |a: [f64; 2], b: [f64; 2]| (a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2);
        let mut welds = 0usize;

        // No máximo 2 fusões — um alvo por endpoint da forma nova.
        for _ in 0..2 {
            let (Some(nf), Some(nl)) = (
                self.endpoint_world(xforms, new_id, true),
                self.endpoint_world(xforms, new_id, false),
            ) else {
                break;
            };
            // Melhor alvo: (B, endpoint-da-nova-é-first, endpoint-de-B-é-first, ponto).
            let others: Vec<VecPathId> = self
                .paths
                .iter()
                .filter(|p| p.id != new_id && !p.closed && !p.verts.is_empty())
                .map(|p| p.id)
                .collect();
            let mut best: Option<(VecPathId, bool, bool, [f64; 2], f64)> = None;
            for b in others {
                let (Some(bf), Some(bl)) = (
                    self.endpoint_world(xforms, b, true),
                    self.endpoint_world(xforms, b, false),
                ) else {
                    continue;
                };
                for (n_first, npt) in [(true, nf), (false, nl)] {
                    for (b_first, bpt) in [(true, bf), (false, bl)] {
                        let dd = d2(npt, bpt);
                        if dd <= t2 && best.is_none_or(|(.., bd)| dd < bd) {
                            best = Some((b, n_first, b_first, bpt, dd));
                        }
                    }
                }
            }
            let Some((b, n_first, b_first, bpt, _)) = best else {
                break;
            };
            // A nova cede: seu endpoint snapa EXATO no nó pré-existente `bpt`.
            self.snap_endpoint_world(new_id, xforms, n_first, bpt);
            // Orienta para o FIM da nova ligar ao INÍCIO de B, e funde B na nova.
            if n_first {
                self.reverse_path(new_id);
            }
            self.merge_path_into(new_id, b, xforms, !b_first, true);
            welds += 1;
        }

        // Fecha se a forma nova virou um laço (extremos ≤ tol em mundo). Depois das
        // fusões acima o array é `[verts-da-nova…, verts-do-existente…]` — o 1º vértice
        // é o lado NOVO, o último é o lado EXISTENTE (o arco/curva pré-desenhado).
        if welds > 0
            && let Some(p) = self.paths.iter().find(|p| p.id == new_id)
            && p.verts.len() >= 3
            && let (Some(first_w), Some(last_w)) = (
                self.endpoint_world(xforms, new_id, true),
                self.endpoint_world(xforms, new_id, false),
            )
            && d2(first_w, last_w) <= t2
        {
            // O fecho é uma EMENDA, não um pop: o 1º vértice (novo) snapa EXATO no
            // último (existente, sacrossanto), o vértice de fecho herda a tangente de
            // SAÍDA do lado novo — a aresta de fecho fica com o handle certo (reta se a
            // nova era uma linha) — e a duplicata do 1º some. Sem isso, um `pop()` cego
            // apagaria a ponta do arco e o deformaria.
            self.snap_endpoint_world(new_id, xforms, true, last_w);
            if let Some(p) = self.paths.iter_mut().find(|p| p.id == new_id) {
                let first_out = p.verts.first().map(|v| v.out_handle);
                if let (Some(out), Some(last)) = (first_out, p.verts.last_mut()) {
                    last.out_handle = out;
                }
                p.verts.remove(0);
                p.closed = true;
                if let Some(fp) = fill_on_close {
                    p.fill = Some(fp);
                }
            }
            welds += 1;
        }
        welds
    }

    /// Bounding box da curva do path `id` **em coordenadas de MUNDO** (ADR-0111): cada
    /// ponto amostrado sobe pelo afim da entidade antes do min/max.
    ///
    /// É o que qualquer número de mundo precisa — o readout de posição/tamanho do
    /// painel, e sobretudo `align`/`distribute`, que comparam formas **entre si**. A
    /// bbox local de toda forma assentada está centrada na origem (ADR-0112), então
    /// compará-las localmente alinharia tudo no mesmo ponto.
    ///
    /// Para uma forma girada isto é o AABB do quadrilátero — o que o usuário vê ao
    /// encostar as caixas, e não a caixa orientada.
    pub fn path_world_curve_bbox(
        &self,
        xforms: &crate::VecXforms,
        id: VecPathId,
    ) -> Option<([f64; 2], [f64; 2])> {
        let path = self.paths.iter().find(|p| p.id == id)?;
        let x = crate::xform_of(xforms, id);
        let f0 = x.apply(path.verts.first()?.anchor);
        let (mut lo, mut hi) = (f0, f0);
        for_each_curve_point(path, |pt| {
            let w = x.apply(pt);
            lo[0] = lo[0].min(w[0]);
            lo[1] = lo[1].min(w[1]);
            hi[0] = hi[0].max(w[0]);
            hi[1] = hi[1].max(w[1]);
        });
        Some((lo, hi))
    }

    /// Translada o path `id` por um delta de **MUNDO**, convertendo-o para o espaço
    /// local dele. `false` se o id sumiu ou o afim é degenerado.
    pub fn translate_path_world(
        &mut self,
        xforms: &crate::VecXforms,
        id: VecPathId,
        dx: f64,
        dy: f64,
    ) -> bool {
        let Some(inv) = crate::xform_of(xforms, id).inverse() else {
            return false;
        };
        let d = inv.apply_vec([dx, dy]);
        self.translate_path(id, d[0], d[1])
    }

    /// Bounding box das ÂNCORAS do path `id` (`(min, max)` em world-units, todos os
    /// contornos) — a extensão usada pelo readout de transform (posição/tamanho).
    /// `None` se o id sumiu ou o path está vazio.
    pub fn path_bbox(&self, id: VecPathId) -> Option<([f64; 2], [f64; 2])> {
        let path = self.paths.iter().find(|p| p.id == id)?;
        let first = path.verts.first()?;
        let (mut lo, mut hi) = (first.anchor, first.anchor);
        for v in path.verts_all() {
            lo[0] = lo[0].min(v.anchor[0]);
            lo[1] = lo[1].min(v.anchor[1]);
            hi[0] = hi[0].max(v.anchor[0]);
            hi[1] = hi[1].max(v.anchor[1]);
        }
        Some((lo, hi))
    }

    /// Bounding box da CURVA renderizada do path `id` (`(min, max)` world-units) —
    /// amostra cada segmento cúbico de cada contorno, então cobre a forma inteira
    /// incluindo o que abaula PARA FORA das âncoras (ao contrário de
    /// [`Self::path_bbox`], que só enquadra as âncoras). É o que o gizmo de
    /// transform usa. `None` se vazio.
    pub fn path_curve_bbox(&self, id: VecPathId) -> Option<([f64; 2], [f64; 2])> {
        self.path_curve_bbox_in_frame(id, 1.0, 0.0)
    }

    /// Como [`Self::path_curve_bbox`], mas o min/max é medido num **frame
    /// rotacionado por −θ** (`c = cos θ`, `s = sin θ`) — cada ponto amostrado vira
    /// `[x·c + y·s, −x·s + y·c]` antes do min/max. É o bbox ORIENTADO (alinhado a θ)
    /// que o gizmo usa pra girar junto com a forma. `θ = 0` (`c=1,s=0`) = axis-aligned.
    pub fn path_curve_bbox_in_frame(
        &self,
        id: VecPathId,
        c: f64,
        s: f64,
    ) -> Option<([f64; 2], [f64; 2])> {
        let path = self.paths.iter().find(|p| p.id == id)?;
        let rot = |p: [f64; 2]| [p[0] * c + p[1] * s, -p[0] * s + p[1] * c];
        let f0 = rot(path.verts.first()?.anchor);
        let mut lo = f0;
        let mut hi = f0;
        for_each_curve_point(path, |pt| {
            let r = rot(pt);
            lo[0] = lo[0].min(r[0]);
            lo[1] = lo[1].min(r[1]);
            hi[0] = hi[0].max(r[0]);
            hi[1] = hi[1].max(r[1]);
        });
        Some((lo, hi))
    }

    /// O ponto `p` (world) está DENTRO da região do path `id`? Amostra cada contorno
    /// FECHADO num polígono e aplica a [`FillRule`] do path (even-odd = paridade de
    /// cruzamentos; non-zero = winding number). Como o buraco de um compound é um
    /// contorno a mais, ele sai `false` de graça. Sempre `false` p/ path sem contorno
    /// fechado ou id inexistente. Usado pelo gizmo: o arrasto do interior só move a
    /// forma quando o clique cai NELA — espaço vazio da bbox segue livre pro Pen
    /// desenhar. Transcendental-free.
    pub fn path_contains_point(&self, id: VecPathId, p: [f64; 2]) -> bool {
        let Some(path) = self.paths.iter().find(|pp| pp.id == id) else {
            return false;
        };
        let even_odd = path.fill_rule == FillRule::EvenOdd;
        let mut crossings = 0i32;
        let mut winding = 0i32;
        let mut any = false;
        for k in 0..path.contour_count() {
            let Some((verts, closed)) = path.contour(k) else {
                continue;
            };
            if !closed || verts.len() < 2 {
                continue;
            }
            any = true;
            let mut poly: Vec<[f64; 2]> = Vec::with_capacity(verts.len() * CURVE_SAMPLES);
            for i in 0..verts.len() {
                let a = &verts[i];
                let b = &verts[(i + 1) % verts.len()];
                for j in 0..CURVE_SAMPLES {
                    let t = j as f64 / CURVE_SAMPLES as f64;
                    poly.push(cubic(a.anchor, a.out_handle, b.in_handle, b.anchor, t));
                }
            }
            let n = poly.len();
            if n < 3 {
                continue;
            }
            let mut j = n - 1;
            for i in 0..n {
                let (a, b) = (poly[j], poly[i]);
                // O guard garante a[1] != b[1] (sem divisão por zero).
                if (a[1] > p[1]) != (b[1] > p[1]) {
                    let t = (p[1] - a[1]) / (b[1] - a[1]);
                    let x = a[0] + t * (b[0] - a[0]);
                    if p[0] < x {
                        crossings += 1;
                        winding += if b[1] > a[1] { 1 } else { -1 };
                    }
                }
                j = i;
            }
        }
        if !any {
            return false;
        }
        if even_odd {
            crossings % 2 != 0
        } else {
            winding != 0
        }
    }

    /// Translada o path `id` por `(dx, dy)` world-units (âncora + handles de todos
    /// os vértices de todos os contornos). `false` se o id sumiu.
    pub fn translate_path(&mut self, id: VecPathId, dx: f64, dy: f64) -> bool {
        let Some(path) = self.paths.iter_mut().find(|p| p.id == id) else {
            return false;
        };
        path.for_each_vert_mut(|v| {
            v.anchor = [v.anchor[0] + dx, v.anchor[1] + dy];
            v.in_handle = [v.in_handle[0] + dx, v.in_handle[1] + dy];
            v.out_handle = [v.out_handle[0] + dx, v.out_handle[1] + dy];
        });
        transform_fill_geometry(&mut path.fill, |p| [p[0] + dx, p[1] + dy], 1.0);
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
        path.for_each_vert_mut(|v| {
            v.anchor = s(v.anchor);
            v.in_handle = s(v.in_handle);
            v.out_handle = s(v.out_handle);
        });
        // Gradient geometry scales with the shape; the radial radius by the mean
        // axis factor (peniko radials are circular, so non-uniform scale is
        // approximated rather than made elliptical).
        transform_fill_geometry(&mut path.fill, s, (sx.abs() + sy.abs()) * 0.5);
        true
    }

    /// Rotaciona o path `id` por um ângulo ARBITRÁRIO (`radians`) em torno de
    /// `pivot` world-units (âncora + 2 handles de cada vértice). Para múltiplos de
    /// 90° prefira [`Self::rotate_path`] (exato, sem trig). `false` se o id sumiu.
    pub fn rotate_path_by(&mut self, id: VecPathId, radians: f64, pivot: [f64; 2]) -> bool {
        let Some(path) = self.paths.iter_mut().find(|p| p.id == id) else {
            return false;
        };
        let (s, c) = radians.sin_cos();
        let rot = |p: [f64; 2]| {
            let (dx, dy) = (p[0] - pivot[0], p[1] - pivot[1]);
            [pivot[0] + dx * c - dy * s, pivot[1] + dx * s + dy * c]
        };
        path.for_each_vert_mut(|v| {
            v.anchor = rot(v.anchor);
            v.in_handle = rot(v.in_handle);
            v.out_handle = rot(v.out_handle);
        });
        // Gradient geometry rotates with the shape about the SAME pivot — so the
        // fill stays locked to the shape and never "breathes" (the Transform R
        // field case the user hit).
        transform_fill_geometry(&mut path.fill, rot, 1.0);
        true
    }

    /// Define se o contorno PRIMÁRIO do path `id` é fechado (loop) ou aberto (fita).
    /// Fechar exige ≥ 2 vértices. `false` (no-op) se o id sumiu, já estava nesse
    /// estado, ou fechar é impossível (< 2 vértices).
    pub fn set_path_closed(&mut self, id: VecPathId, closed: bool) -> bool {
        let Some(path) = self.paths.iter_mut().find(|p| p.id == id) else {
            return false;
        };
        if path.closed == closed || (closed && path.verts.len() < 2) {
            return false;
        }
        path.closed = closed;
        true
    }
}

/// Chama `f` em cada ponto amostrado da curva de TODOS os contornos de `path`
/// (`CURVE_SAMPLES` por segmento, fechamento incluído). Base das bboxes de curva.
fn for_each_curve_point(path: &crate::VecPath, mut f: impl FnMut([f64; 2])) {
    for k in 0..path.contour_count() {
        let Some((verts, closed)) = path.contour(k) else {
            continue;
        };
        for seg in 0..contour_segments(verts, closed) {
            let a = &verts[seg];
            let b = &verts[(seg + 1) % verts.len()];
            for j in 0..=CURVE_SAMPLES {
                let t = j as f64 / CURVE_SAMPLES as f64;
                f(cubic(a.anchor, a.out_handle, b.in_handle, b.anchor, t));
            }
        }
    }
}

/// **Assa** o afim `x` na geometria de `path`: âncoras, handles e a geometria do
/// gradiente passam a estar no frame de destino.
///
/// É o que reconcilia frames diferentes antes de uma operação de geometria
/// (booleana, merge, offset): os operandos vêm de entidades com `Transform`
/// distintos, e um resultado só pode viver num frame. Assando os operandos no
/// MUNDO, o resultado nasce em world-space — e a entidade nova dele, na identidade,
/// o desenha exatamente onde as formas de origem estavam.
///
/// Identidade é no-op.
pub fn bake_xform(path: &mut VecPath, x: &crate::Xform) {
    if x.is_identity() {
        return;
    }
    let f = |p: [f64; 2]| x.apply(p);
    path.for_each_vert_mut(|v| {
        v.anchor = f(v.anchor);
        v.in_handle = f(v.in_handle);
        v.out_handle = f(v.out_handle);
    });
    transform_fill_geometry(&mut path.fill, f, x.mean_scale());
}

/// Aplica a transformação de ponto `f` (a MESMA das âncoras) à geometria world-space
/// do gradiente do fill, e escala o raio radial por `radius_scale`. Assim o
/// gradiente transforma rigidamente com a shape (não "respira" ao rotacionar).
/// No-op para `Solid` / sem fill.
fn transform_fill_geometry(
    fill: &mut Option<Paint>,
    f: impl Fn([f64; 2]) -> [f64; 2],
    radius_scale: f64,
) {
    match fill {
        Some(Paint::Linear { start, end, .. }) => {
            *start = f(*start);
            *end = f(*end);
        }
        Some(Paint::Radial { center, radius, .. }) => {
            *center = f(*center);
            *radius *= radius_scale;
        }
        Some(Paint::MultiPoint { points }) => {
            for p in points {
                p.pos = f(p.pos);
            }
        }
        Some(Paint::Solid(_)) | None => {}
    }
}
