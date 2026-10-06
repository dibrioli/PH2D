//! ⭐⭐ (W9) **A DOMINÂNCIA ENTRE FRENTES** — o que torna a procura ponderada barata com muita lama
//! (plano 30 §17; a medição: `examples/medir_custo.rs` §4 da `ph2d-navmesh`).
//!
//! # O defeito que cura
//!
//! Um intervalo que atravessa uma fronteira de custo nasce em VÁRIAS raízes (a grelha da aresta,
//! `polyanya_custo.rs`), e cada uma vê o polígono do outro lado inteiro ⇒ `k` frentes paralelas que
//! cobrem o mesmo chão, e só se podavam nos cantos (`root_g`) e na fronteira seguinte (`steiner_g`).
//! Medido: `64 500` nós por consulta contra `5 650` da uniforme na cena grande.
//!
//! # A lei
//!
//! Dois nós na MESMA aresta, a entrar no MESMO polígono, vêm de raízes na mesma região de custo `w`:
//! o custo de chegar a um ponto `y` da aresta é `g + w·|ρ − y|` para cada um. Onde um nó JÁ EXPANDIDO
//! `A` cobre `y` e chega lá por menos ou igual, tudo o que o nó novo `B` alcança através de `y` também
//! se alcança por `A` (o troço de `y` em diante é o mesmo, e de `ρ_A` a procura acha o caminho até lá
//! por menos — é o argumento da poda por raiz do artigo, ponto a ponto). ⇒ corta-se a parte de `B`
//! dominada; se não sobra nada, `B` não expande.
//!
//! `D(y) = g_A + w·|ρ_A − y| − g_B − w·|ρ_B − y|` ao longo de uma recta tem NO MÁXIMO um extremo (a
//! curva de nível de `|ρ_A − y| − |ρ_B − y|` é uma hipérbole, que a recta corta em dois pontos no
//! máximo) ⇒ a parte dominada a partir de uma ponta acha-se exacta: o sinal de `D'` nas pontas diz se
//! há um máximo no meio, e uma bissecção acha o zero. Corta-se só a partir das PONTAS (o meio
//! dominado fica — é a escolha conservadora: cortar a menos nunca perde um caminho).
//!
//! ⚠️ Só corre na procura PONDERADA (`Polyanya::dominancia`): sem áreas a procura é a de sempre, ao
//! bit (gate). Só contra nós EXPANDIDOS (o que se guarda já gerou os filhos). E só onde o custo NÃO
//! muda na aresta: onde muda, a refracção já se poda ponto a ponto na grelha (`steiner_g`), e cortar
//! o intervalo perdia as PONTAS dele — o ponto exacto onde um caminho que roça um canto atravessa,
//! que a frente dominante não gera.

use super::Polyanya;
use crate::geom::{EPS, V2, dist, dot, lerp, sub};
use crate::mesh::NavMesh;

/// Um nó expandido numa aresta: a raiz, o custo dela, e o intervalo que propagou (depois de cortado).
#[derive(Clone, Copy, Debug)]
struct Frente {
    rho: V2,
    g: f64,
    w: f64,
    left: V2,
    right: V2,
    entry: u32,
    /// A frente seguinte do mesmo polígono (`NONE` = a última).
    next: u32,
}

/// As frentes expandidas, numa lista por polígono (a cabeça por polígono, o resto num só vector) —
/// sem árvore nem tabela de dispersão no laço quente.
#[derive(Debug, Default)]
pub(super) struct Frentes {
    head: Vec<u32>,
    pool: Vec<Frente>,
    touched: Vec<u32>,
}

impl Frentes {
    pub(super) fn clear(&mut self, mesh: &NavMesh) {
        if self.head.len() == mesh.poly_count() {
            for &p in &self.touched {
                self.head[p as usize] = NONE;
            }
        } else {
            self.head = vec![NONE; mesh.poly_count()];
        }
        self.touched.clear();
        self.pool.clear();
    }
}

const NONE: u32 = u32::MAX;

/// Os passos da bissecção que acha onde a dominância acaba. O corte é CONSERVADOR (fica o lado
/// dominado), logo menos passos só cortam um pouco menos — o recurso é o tempo por nó (medido, a cena
/// grande da `medir_custo`, 40 consultas: `60` passos `347 ms`, `24` `291 ms`, `12` `275 ms`, os
/// mesmos nós e o mesmo custo).
const BISSECCOES: usize = 12;

impl Polyanya {
    /// ⭐ Corta o intervalo `[left, right]` (na aresta `entry` de `poly`, visto de `rho` a `g`, região de
    /// custo `w`) pelas frentes já expandidas na mesma aresta, e guarda-o. `None` = dominado inteiro.
    #[allow(clippy::too_many_arguments)]
    pub(super) fn domina(
        &mut self,
        mesh: &NavMesh,
        poly: u32,
        entry: u32,
        rho: V2,
        g: f64,
        w: f64,
        left: V2,
        right: V2,
    ) -> Option<(V2, V2)> {
        let corte = if self.corte_fechado {
            corte_fechado
        } else {
            corte
        };
        let f = &mut self.frentes;
        if f.head.len() != mesh.poly_count() {
            f.clear(mesh);
        }
        let (mut l, mut r) = (left, right);
        let mut i = f.head[poly as usize];
        while i != NONE {
            let a = f.pool[i as usize];
            i = a.next;
            self.stats.compared += 1;
            if a.entry != entry || a.w != w {
                continue;
            }
            // A ponta esquerda, depois a direita (as duas pela mesma porta, de pontas trocadas).
            if let Some(s) = corte(&a, rho, g, w, l, r) {
                if s >= 1.0 {
                    self.stats.dominated += 1;
                    return None;
                }
                l = lerp(l, r, s);
            }
            if let Some(s) = corte(&a, rho, g, w, r, l) {
                if s >= 1.0 {
                    self.stats.dominated += 1;
                    return None;
                }
                r = lerp(r, l, s);
            }
        }
        let cortou = dist(l, left) > EPS || dist(r, right) > EPS;
        if cortou && dist(l, r) <= EPS {
            self.stats.dominated += 1;
            return None;
        }
        if cortou {
            self.stats.trimmed += 1;
        }
        let h = &mut f.head[poly as usize];
        if *h == NONE {
            f.touched.push(poly);
        }
        f.pool.push(Frente {
            rho,
            g,
            w,
            left: l,
            right: r,
            entry,
            next: *h,
        });
        *h = (f.pool.len() - 1) as u32;
        Some((l, r))
    }
}

/// Quanto de `[de, para]`, a partir de `de`, a frente `a` domina: `Some(s)` = o prefixo `[0, s]` (no
/// parâmetro de `de` para `para`; `1` = tudo), `None` = nem a ponta `de`.
fn corte(a: &Frente, rho: V2, g: f64, w: f64, de: V2, para: V2) -> Option<f64> {
    // Os limites da desigualdade triangular: `|D(y) − (g_A − g)| ≤ w·|ρ_A − ρ|` em todo `y`.
    let (delta, k) = (a.g - g, w * dist(a.rho, rho));
    if delta > k {
        return None;
    }
    let toda = delta + k <= 0.0;
    let d = sub(para, de);
    let l2 = dot(d, d);
    let param = |p: V2| {
        if l2 <= EPS * EPS {
            0.0
        } else {
            dot(sub(p, de), d) / l2
        }
    };
    // O que `a` cobre, no parâmetro de `de` para `para` (a mesma aresta, logo a mesma recta).
    let (sa, sb) = (param(a.left), param(a.right));
    let (cob0, cob1) = (sa.min(sb), sa.max(sb));
    let tol = if l2 > 0.0 { EPS / l2.sqrt() } else { 0.0 };
    if cob0 > tol || cob1 < -tol {
        return None;
    }
    let fim = cob1.min(1.0);
    if toda {
        return Some(if fim >= 1.0 { 1.0 } else { fim.max(0.0) });
    }
    let y = |s: f64| lerp(de, para, s);
    let dd = |s: f64| a.g + w * dist(a.rho, y(s)) - g - w * dist(rho, y(s));
    if dd(0.0) > 0.0 {
        return None;
    }
    if fim <= 0.0 || l2 <= EPS * EPS {
        return Some(if fim >= 1.0 { 1.0 } else { 0.0 });
    }
    // A derivada de `D` ao longo de `d` (o sinal basta).
    let der = |s: f64| {
        let p = y(s);
        let (u, v) = (sub(p, a.rho), sub(p, rho));
        dot(u, d) / dot(u, u).sqrt().max(EPS) - dot(v, d) / dot(v, v).sqrt().max(EPS)
    };
    // Há um MÁXIMO no meio? Então o primeiro zero está antes dele (se o máximo passa de zero).
    let mut topo = fim;
    if der(0.0) > 0.0 && der(fim) < 0.0 {
        let (mut lo, mut hi) = (0.0, fim);
        for _ in 0..BISSECCOES {
            let m = 0.5 * (lo + hi);
            if der(m) > 0.0 {
                lo = m;
            } else {
                hi = m;
            }
        }
        topo = lo;
    }
    if dd(topo) <= 0.0 && dd(fim) <= 0.0 {
        return Some(if fim >= 1.0 { 1.0 } else { fim });
    }
    // `D ≤ 0` em `0` e `> 0` mais à frente, com um só cruzamento no caminho: bissecção, e fica o
    // lado DOMINADO (corta-se a menos, nunca a mais).
    let alvo = if dd(topo) > 0.0 { topo } else { fim };
    let (mut lo, mut hi) = (0.0, alvo);
    for _ in 0..BISSECCOES {
        let m = 0.5 * (lo + hi);
        if dd(m) <= 0.0 {
            lo = m;
        } else {
            hi = m;
        }
    }
    Some(lo)
}

/// ⭐ (plano 30 §26, C1) [`corte`] sem as bissecções: as rejeições sem `sqrt` primeiro (a cobertura no
/// parâmetro), e os zeros de `D` sobre a recta em FORMA FECHADA — `D = 0` é `|ρ_A − y| − |ρ − y| = c`, com
/// `c = (g − g_A)/w`, uma hipérbole; elevar ao quadrado dá uma quadrática em `s` (os ramos trocados saem
/// pelo sinal de `L = |ρ_A − y|² − |ρ − y|² − c² = 2c·|ρ − y|`). Entre dois zeros o sinal de `D` é um só:
/// o do ponto do meio decide. O corte fica do lado dominado (verificado); se a verificação falha, as
/// bissecções.
fn corte_fechado(a: &Frente, rho: V2, g: f64, w: f64, de: V2, para: V2) -> Option<f64> {
    let d = sub(para, de);
    let l2 = dot(d, d);
    let param = |p: V2| {
        if l2 <= EPS * EPS {
            0.0
        } else {
            dot(sub(p, de), d) / l2
        }
    };
    let (sa, sb) = (param(a.left), param(a.right));
    let (cob0, cob1) = (sa.min(sb), sa.max(sb));
    let tol = if l2 > 0.0 { EPS / l2.sqrt() } else { 0.0 };
    if cob0 > tol || cob1 < -tol {
        return None;
    }
    let (delta, k) = (a.g - g, w * dist(a.rho, rho));
    if delta > k {
        return None;
    }
    let fim = cob1.min(1.0);
    if delta + k <= 0.0 {
        return Some(if fim >= 1.0 { 1.0 } else { fim.max(0.0) });
    }
    let y = |s: f64| lerp(de, para, s);
    let dd = |s: f64| a.g + w * dist(a.rho, y(s)) - g - w * dist(rho, y(s));
    if dd(0.0) > 0.0 {
        return None;
    }
    if fim <= 0.0 || l2 <= EPS * EPS {
        return Some(if fim >= 1.0 { 1.0 } else { 0.0 });
    }
    if w <= 0.0 {
        return corte(a, rho, g, w, de, para);
    }
    let c = (g - a.g) / w;
    let (pa, pb) = (sub(de, a.rho), sub(de, rho));
    let alfa = dot(pa, pa) - dot(pb, pb) - c * c;
    let beta = 2.0 * (dot(pa, d) - dot(pb, d));
    let (c2, pbd) = (c * c, dot(pb, d));
    let (qa, qb, qc) = (
        beta * beta - 4.0 * c2 * l2,
        2.0 * alfa * beta - 8.0 * c2 * pbd,
        alfa * alfa - 4.0 * c2 * dot(pb, pb),
    );
    let mut raizes = [f64::NAN; 2];
    if qa.abs() <= 1e-12 * qb.abs().max(qc.abs()) {
        if qb != 0.0 {
            raizes[0] = -qc / qb;
        }
    } else {
        let disc = qb * qb - 4.0 * qa * qc;
        if disc >= 0.0 {
            let q = -0.5 * (qb + qb.signum() * disc.sqrt());
            raizes[0] = q / qa;
            if q != 0.0 {
                raizes[1] = qc / q;
            }
        }
    }
    // Só os zeros de `D` (o ramo certo) dentro de `(0, fim)`, por ordem.
    let mut zeros: [f64; 2] = [f64::NAN; 2];
    let mut nz = 0;
    for r in raizes {
        if r > 0.0 && r < fim && (alfa + beta * r) * c >= 0.0 {
            zeros[nz] = r;
            nz += 1;
        }
    }
    if nz == 2 && zeros[1] < zeros[0] {
        zeros.swap(0, 1);
    }
    let mut ini = 0.0;
    for &z in zeros[..nz].iter().chain(std::iter::once(&fim)) {
        if dd(0.5 * (ini + z)) > 0.0 {
            let s = (ini - tol).max(0.0);
            return if dd(s) <= 0.0 {
                Some(s)
            } else {
                corte(a, rho, g, w, de, para)
            };
        }
        ini = z;
    }
    if dd(fim) > 0.0 {
        return corte(a, rho, g, w, de, para);
    }
    Some(if fim >= 1.0 { 1.0 } else { fim })
}
