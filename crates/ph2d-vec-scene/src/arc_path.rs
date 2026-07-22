//! **Um contorno parametrizado por COMPRIMENTO DE ARCO** — [`ArcPath`].
//!
//! O [`crate::arclen`] responde por **uma cúbica**; um contorno tem várias, e a pergunta que todo
//! consumidor faz é a mesma: *"onde fica o arco `s` neste caminho inteiro, e para onde ele aponta
//! ali?"*. Este módulo é a porta única dessa pergunta.
//!
//! # Por que é uma porta e não um helper copiado
//!
//! O cabeçalho do [`crate::arclen`] já nomeava os consumidores previstos — *"Trim, Repeater,
//! Pattern Along Path, **texto em caminho**"* — e o primeiro a chegar
//! ([`crate::fx_zigzag`]) escreveu o seu próprio walker privado. Um segundo consumidor a
//! escrever o dele daria **duas respostas para uma pergunta geométrica**, que é a forma que este
//! repositório já pagou uma dúzia de vezes: elas não divergem no dia em que nascem, divergem no
//! dia em que uma das duas ganha um cuidado (a busca binária, a cúspide, a saturação nas pontas)
//! e a outra não.
//!
//! # O que está aqui, e o que deliberadamente NÃO está
//!
//! Está: construir os segmentos, o prefixo somado de arco, e resolver `s → (ponto, tangente)`.
//!
//! **Não** está: a política de amostragem. Onde amostrar é do EFEITO — o Zig Zag quer a grade de
//! cristas unida às âncoras de entrada, o texto quer uma posição por glifo. Puxar isso para cá
//! faria a porta ter opinião sobre features que ainda não existem.

use crate::VecVertex;
use crate::arclen::{Cubic, arclen, inv_arclen, point_at, tangent_at};
use crate::corner_live::segment;

/// Um contorno pronto a ser percorrido **por comprimento de arco**.
///
/// Construir custa um [`arclen`] por segmento (Gauss-Legendre de 16 nós); consultar custa uma
/// busca binária + uma inversão. Construa **uma vez** e consulte `n` vezes — um `ArcPath` por
/// amostra transformaria um efeito linear em quadrático.
#[derive(Clone, Debug)]
pub struct ArcPath {
    segs: Vec<Cubic>,
    /// `starts[i]` = a posição de arco onde o segmento `i` começa; a ÚLTIMA entrada é o total.
    ///
    /// Prefixo somado, e não uma lista de comprimentos, por duas razões que se somam: localizar
    /// `s` vira busca binária, e as âncoras de entrada **são exatamente estas posições** — que é
    /// o que permite a um efeito amostrar onde o caminho já tem vértice.
    starts: Vec<f64>,
}

impl ArcPath {
    /// Prepara o contorno. `None` se não há segmento nenhum (menos de dois vértices).
    ///
    /// Um contorno fechado tem `n` segmentos (o último volta ao primeiro); um aberto tem `n - 1`.
    #[must_use]
    pub fn from_contour(verts: &[VecVertex], closed: bool) -> Option<Self> {
        let n = verts.len();
        if n < 2 {
            return None;
        }
        let seg_count = if closed { n } else { n - 1 };
        let segs: Vec<Cubic> = (0..seg_count).map(|i| segment(verts, i, n)).collect();
        let mut starts = Vec::with_capacity(seg_count + 1);
        let mut acc = 0.0;
        for c in &segs {
            starts.push(acc);
            acc += arclen(c);
        }
        starts.push(acc);
        Some(Self { segs, starts })
    }

    /// O comprimento total do contorno.
    ///
    /// Pode ser `0` num contorno degenerado (vértices coincidentes) — quem divide por ele tem de
    /// o testar, e é por isso que este método existe em vez de o total ser presumido positivo.
    #[must_use]
    pub fn total(&self) -> f64 {
        // `starts` nunca está vazio: `from_contour` empurra sempre o total.
        self.starts[self.starts.len() - 1]
    }

    /// As posições de arco das **âncoras** de entrada (uma por segmento, sem o total).
    ///
    /// É o conjunto que um efeito une à própria grade para não fazer aliasing sobre um caminho
    /// que já tem onda ([`crate::fx_zigzag`], §"as âncoras de ENTRADA entram no conjunto").
    #[must_use]
    pub fn anchor_arcs(&self) -> &[f64] {
        &self.starts[..self.segs.len()]
    }

    /// Onde o comprimento `s` cai: o **ponto** e a **tangente unitária** ali.
    ///
    /// `s` fora de `[0, total]` satura nas pontas (o [`inv_arclen`] já o faz por segmento), o que
    /// é a resposta útil para quem varre uma grade que pode encostar no fim por arredondamento.
    ///
    /// ⚠️ **Numa cúspide não há tangente** e devolve-se `[0, 0]` — quem chama desloca por zero em
    /// vez de por uma direção inventada. É um vetor NULO, não unitário: quem normaliza a jusante
    /// tem de o testar.
    #[must_use]
    pub fn frame_at(&self, s: f64) -> ([f64; 2], [f64; 2]) {
        let i = self
            .starts
            .partition_point(|&p| p <= s)
            .saturating_sub(1)
            .min(self.segs.len() - 1);
        let t = inv_arclen(&self.segs[i], (s - self.starts[i]).max(0.0));
        (
            point_at(&self.segs[i], t),
            tangent_at(&self.segs[i], t).unwrap_or([0.0, 0.0]),
        )
    }
}

#[cfg(test)]
#[path = "arc_path_tests.rs"]
mod tests;
