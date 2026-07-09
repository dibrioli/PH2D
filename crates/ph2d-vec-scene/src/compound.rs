//! Compound paths (ADR-0108): um [`VecPath`] pode carregar contornos ADICIONAIS
//! além do primário. É assim que BURACO existe — subtrair um disco de outro
//! devolve um único path com o contorno de fora e o de dentro, e a [`FillRule`]
//! decide que a região entre eles é vazia (rosquinha). Sem isto, cada contorno
//! virava um path solto e a "rosquinha" renderizava como dois discos sólidos.
//!
//! **Extensão append-only** (CLAUDE.md §0.2): `verts`/`closed` continuam sendo o
//! contorno PRIMÁRIO e mantêm o significado que sempre tiveram; `subpaths` só
//! acrescenta. Path simples ⇒ `subpaths` vazio ⇒ todo caminho de código antigo
//! observa exatamente a geometria de antes.
//!
//! # Espaço de índice PLANO
//!
//! Vértices e segmentos são endereçados por um índice **plano** que percorre o
//! contorno primário e depois cada subpath, em ordem. Isso deixa
//! `selected_verts: Vec<usize>` e o `(seg, t)` do hit-test valerem para um
//! compound sem mudar assinatura nenhuma — e, quando `subpaths` está vazio, o
//! índice plano É o índice de `verts`, então nada muda para o caso comum.

use crate::{VecPath, VecPathId, VecScene, VecVertex};
use serde::{Deserialize, Serialize};

/// Regra de preenchimento de um path de múltiplos contornos.
///
/// `NonZero` respeita a orientação (winding) de cada contorno; `EvenOdd` alterna
/// dentro/fora a cada cruzamento, então um contorno aninhado vira buraco
/// **independente de como foi orientado**. Resultados de booleana e compounds
/// montados à mão usam `EvenOdd` justamente por essa robustez. Para um path de
/// contorno único as duas regras coincidem — daí o default `NonZero` preservar o
/// comportamento histórico byte-a-byte.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum FillRule {
    #[default]
    NonZero,
    EvenOdd,
}

/// Um contorno adicional de um compound path (buraco ou ilha).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Contour {
    pub verts: Vec<VecVertex>,
    pub closed: bool,
}

impl Contour {
    /// Contorno fechado a partir dos vértices (o caso de booleana / compound).
    #[must_use]
    pub fn new_closed(verts: Vec<VecVertex>) -> Self {
        Self {
            verts,
            closed: true,
        }
    }
}

/// Nº de segmentos de um contorno (fechado inclui o segmento de fechamento).
#[must_use]
pub(crate) fn contour_segments(verts: &[VecVertex], closed: bool) -> usize {
    if verts.len() < 2 {
        0
    } else if closed {
        verts.len()
    } else {
        verts.len() - 1
    }
}

impl VecPath {
    /// O path tem contornos além do primário?
    #[must_use]
    pub fn is_compound(&self) -> bool {
        !self.subpaths.is_empty()
    }

    /// Quantos contornos (primário + subpaths).
    #[must_use]
    pub fn contour_count(&self) -> usize {
        1 + self.subpaths.len()
    }

    /// Contorno `c` (`0` = primário) como `(vértices, fechado)`.
    #[must_use]
    pub fn contour(&self, c: usize) -> Option<(&[VecVertex], bool)> {
        match c {
            0 => Some((&self.verts, self.closed)),
            _ => self
                .subpaths
                .get(c - 1)
                .map(|s| (s.verts.as_slice(), s.closed)),
        }
    }

    /// Contorno `c` mutável (`0` = primário).
    pub fn contour_mut(&mut self, c: usize) -> Option<(&mut Vec<VecVertex>, &mut bool)> {
        match c {
            0 => Some((&mut self.verts, &mut self.closed)),
            _ => self
                .subpaths
                .get_mut(c - 1)
                .map(|s| (&mut s.verts, &mut s.closed)),
        }
    }

    /// Todos os vértices, primário primeiro — na ordem do índice plano.
    pub fn verts_all(&self) -> impl Iterator<Item = &VecVertex> {
        self.verts
            .iter()
            .chain(self.subpaths.iter().flat_map(|s| s.verts.iter()))
    }

    /// Aplica `f` a todo vértice de todo contorno (base das transformações).
    pub fn for_each_vert_mut(&mut self, mut f: impl FnMut(&mut VecVertex)) {
        for v in &mut self.verts {
            f(v);
        }
        for s in &mut self.subpaths {
            for v in &mut s.verts {
                f(v);
            }
        }
    }

    /// Total de vértices somando todos os contornos.
    #[must_use]
    pub fn total_verts(&self) -> usize {
        self.verts.len() + self.subpaths.iter().map(|s| s.verts.len()).sum::<usize>()
    }

    /// Índice plano de vértice → `(contorno, índice local)`.
    #[must_use]
    pub fn locate_vert(&self, i: usize) -> Option<(usize, usize)> {
        let mut rest = i;
        for c in 0..self.contour_count() {
            let n = self.contour(c)?.0.len();
            if rest < n {
                return Some((c, rest));
            }
            rest -= n;
        }
        None
    }

    /// `(contorno, índice local)` → índice plano de vértice.
    #[must_use]
    pub fn flat_vert(&self, c: usize, local: usize) -> Option<usize> {
        let mut base = 0;
        for k in 0..c {
            base += self.contour(k)?.0.len();
        }
        (local < self.contour(c)?.0.len()).then_some(base + local)
    }

    /// Vértice pelo índice plano.
    #[must_use]
    pub fn vert(&self, i: usize) -> Option<&VecVertex> {
        let (c, local) = self.locate_vert(i)?;
        self.contour(c)?.0.get(local)
    }

    /// Vértice mutável pelo índice plano.
    pub fn vert_mut(&mut self, i: usize) -> Option<&mut VecVertex> {
        let (c, local) = self.locate_vert(i)?;
        self.contour_mut(c)?.0.get_mut(local)
    }

    /// Total de segmentos somando todos os contornos (fechado conta o fechamento).
    #[must_use]
    pub fn total_segments(&self) -> usize {
        (0..self.contour_count())
            .filter_map(|c| self.contour(c))
            .map(|(v, cl)| contour_segments(v, cl))
            .sum()
    }

    /// Índice plano de segmento → `(contorno, índice local de segmento)`.
    #[must_use]
    pub fn locate_segment(&self, s: usize) -> Option<(usize, usize)> {
        let mut rest = s;
        for c in 0..self.contour_count() {
            let (v, cl) = self.contour(c)?;
            let n = contour_segments(v, cl);
            if rest < n {
                return Some((c, rest));
            }
            rest -= n;
        }
        None
    }

    /// `(contorno, segmento local)` → índice plano de segmento.
    #[must_use]
    pub fn flat_segment(&self, c: usize, local: usize) -> Option<usize> {
        let mut base = 0;
        for k in 0..c {
            let (v, cl) = self.contour(k)?;
            base += contour_segments(v, cl);
        }
        let (v, cl) = self.contour(c)?;
        (local < contour_segments(v, cl)).then_some(base + local)
    }

    /// Remove o contorno `c`. O primário nunca é removido isoladamente: pedir `0`
    /// **promove** o primeiro subpath a primário (o path continua válido).
    /// `false` se `c` não existe ou o path é de contorno único.
    pub fn remove_contour(&mut self, c: usize) -> bool {
        if c >= self.contour_count() || self.subpaths.is_empty() {
            return false;
        }
        if c == 0 {
            let promoted = self.subpaths.remove(0);
            self.verts = promoted.verts;
            self.closed = promoted.closed;
        } else {
            self.subpaths.remove(c - 1);
        }
        true
    }
}

impl VecScene {
    /// Como `push_path`, mas insere na posição `index` da pilha de z (clampado ao
    /// topo) em vez de empilhar no topo. É o que deixa o resultado de uma booleana
    /// ficar onde a base estava, em vez de saltar pra frente de tudo.
    pub fn insert_path(&mut self, index: usize, mut path: VecPath) -> VecPathId {
        let id = self.next_id;
        self.next_id += 1;
        path.id = id;
        let at = index.min(self.paths.len());
        self.paths.insert(at, path);
        id
    }

    /// Funde os paths `ids` (≥ 2) num único **compound path**: o de mais baixo na
    /// pilha de z vira a base (mantém id e estilo) e recebe os contornos de todos
    /// os outros como subpaths; os demais são removidos. A regra de preenchimento
    /// vira [`FillRule::EvenOdd`] — é o que faz um contorno DENTRO de outro virar
    /// buraco na hora, sem depender de orientação. Devolve o id da base, ou `None`
    /// se < 2 ids válidos.
    pub fn make_compound(&mut self, ids: &[VecPathId]) -> Option<VecPathId> {
        // Ordena por z (posição na pilha) para que a base seja a de trás.
        let mut zs: Vec<usize> = ids
            .iter()
            .filter_map(|id| self.paths.iter().position(|p| p.id == *id))
            .collect();
        zs.sort_unstable();
        zs.dedup();
        if zs.len() < 2 {
            return None;
        }
        // Coleta os contornos dos que serão absorvidos (de trás pra frente).
        let mut absorbed: Vec<Contour> = Vec::new();
        for &z in &zs[1..] {
            let p = &self.paths[z];
            absorbed.push(Contour {
                verts: p.verts.clone(),
                closed: p.closed,
            });
            absorbed.extend(p.subpaths.iter().cloned());
        }
        let base_id = self.paths[zs[0]].id;
        {
            let base = &mut self.paths[zs[0]];
            base.subpaths.extend(absorbed);
            base.fill_rule = FillRule::EvenOdd;
        }
        // Remove os absorvidos do maior z pro menor (índices não deslizam).
        for &z in zs[1..].iter().rev() {
            self.paths.remove(z);
        }
        Some(base_id)
    }

    /// Inverso de [`Self::make_compound`]: cada subpath de `id` vira um path
    /// próprio (mesmo estilo, contorno único, [`FillRule::NonZero`]), empilhado
    /// logo acima da base. Devolve os ids novos (vazio se `id` não é compound).
    pub fn release_compound(&mut self, id: VecPathId) -> Vec<VecPathId> {
        let Some(z) = self.paths.iter().position(|p| p.id == id) else {
            return Vec::new();
        };
        let subs: Vec<Contour> = std::mem::take(&mut self.paths[z].subpaths);
        if subs.is_empty() {
            return Vec::new();
        }
        self.paths[z].fill_rule = FillRule::NonZero;
        let template = self.paths[z].clone();
        let mut new_ids = Vec::with_capacity(subs.len());
        for (k, sub) in subs.into_iter().enumerate() {
            new_ids.push(self.insert_path(
                z + 1 + k,
                VecPath {
                    verts: sub.verts,
                    closed: sub.closed,
                    fill: template.fill.clone(),
                    stroke: template.stroke,
                    ..VecPath::default()
                },
            ));
        }
        new_ids
    }
}
