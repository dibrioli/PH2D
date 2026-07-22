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

    /// A posição de arco do ponto do contorno **mais próximo** de `p`.
    ///
    /// É o inverso da pergunta que o [`Self::frame_at`] responde — *"onde na curva cai este
    /// ponto de mundo?"* —, e é o que uma alça arrastável precisa: o dedo anda pela tela e a
    /// resposta é onde no caminho ele pousou. Um `NaN` num consumidor a jusante viraria uma alça
    /// que salta para o começo; por isso a busca é fechada e sempre devolve um `s` em `[0, total]`.
    ///
    /// **Amostra grossa + refino local**, sem derivada (HR-5): a curva não é convexa, então uma
    /// busca puramente local (Newton) cairia num mínimo errado numa curva em S. `samples` fixa a
    /// densidade da varredura grossa — o suficiente para não pular um laço — e a bisseção
    /// dourada em torno do vencedor tira a resposta de cima da grade. `O(samples + iters)` por
    /// chamada, e uma alça é arrastada, não varrida em massa: é barato onde é chamado.
    #[must_use]
    pub fn closest_arc(&self, p: [f64; 2]) -> f64 {
        let total = self.total();
        if total <= 0.0 {
            return 0.0;
        }
        let dist2 = |s: f64| {
            let (q, _) = self.frame_at(s);
            let (dx, dy) = (q[0] - p[0], q[1] - p[1]);
            dx.mul_add(dx, dy * dy)
        };
        // Varredura grossa: ~um passo por 2 px de curva, com um piso para contornos curtos.
        let samples = ((total * 0.5).ceil() as usize).clamp(24, 512);
        let step = total / samples as f64;
        let mut best_s = 0.0;
        let mut best = dist2(0.0);
        for i in 1..=samples {
            let s = i as f64 * step;
            let d = dist2(s);
            if d < best {
                best = d;
                best_s = s;
            }
        }
        // Refino: encolhe o intervalo `[best_s − step, best_s + step]` amostrando o meio de cada
        // metade e ficando com a melhor. Converge geometricamente; 32 passos levam um passo de
        // ~centenas de px abaixo do sub-pixel.
        let (mut lo, mut hi) = ((best_s - step).max(0.0), (best_s + step).min(total));
        for _ in 0..32 {
            let m = 0.5 * (lo + hi);
            let (a, b) = (0.5 * (lo + m), 0.5 * (m + hi));
            if dist2(a) < dist2(b) {
                hi = m;
            } else {
                lo = m;
            }
        }
        0.5 * (lo + hi)
    }
}

#[cfg(test)]
#[path = "arc_path_tests.rs"]
mod tests;
