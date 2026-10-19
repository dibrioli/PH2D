//! W23 B5(1) — o segmento do MEIO de um trecho do cotovelo ([doc 13]): a lei do tldraw 5.5.2
//! (`elbowMidPoint`, medido): uma fracção do vão entre os RECUOS das duas pontas. Segue as caixas
//! quando elas mexem, nunca entra nelas (o vão fica fora das duas), e quando o trecho deixa de sair
//! em Z a fracção fica guardada sem uso — volta a valer quando o Z volta. O fixo absoluto do
//! Excalidraw atravessava a caixa (foto `excalidraw_C3_*`): recusado.
//!
//! [doc 13]: ../../../docs/MiroClone/13_trecho_do_cotovelo.md

use ph2d_vec_connect::Dir;

/// O segmento do meio de um trecho em Z, como a pega o mostra e o arrasto o lê.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ElbowMid {
    /// O meio do segmento (mundo): onde vive a pega.
    pub at: [f64; 2],
    /// O segmento é vertical (arrasta-se em x); senão é horizontal (arrasta-se em y).
    pub vertical: bool,
    /// O vão `[s0, s1]`, no eixo do arrasto, entre o recuo da ponta de trás e o da da frente.
    pub span: [f64; 2],
}

impl ElbowMid {
    /// A fracção do vão em que fica o segmento se for largado sob `p` — a porta ÚNICA do inverso
    /// (o arrasto e a importação do Excalidraw usam-na).
    #[must_use]
    pub fn frac(&self, p: [f64; 2]) -> f64 {
        let c = if self.vertical { p[0] } else { p[1] };
        ((c - self.span[0]) / (self.span[1] - self.span[0])).clamp(0.0, 1.0)
    }
}

/// O trecho `leg` (do roteador, de estação a estação) saiu em Z entre duas saídas opostas `da`/`db`
/// (`db` aponta para FORA da estação de chegada)? Então põe o segmento do meio na fracção `frac`
/// (`None` = onde o roteador o pôs) e devolve-o; outra forma (L, recta, U) = `None`.
pub(crate) fn place(
    leg: &mut [[f64; 2]],
    da: Dir,
    db: Dir,
    jetty: f64,
    frac: Option<f64>,
) -> Option<ElbowMid> {
    const EPS: f64 = 1e-6;
    let &[a, v1, v2, b] = &*leg else {
        return None;
    };
    if db != da.opposite() {
        return None;
    }
    let d = da.vec();
    // `ax`: o eixo das hastes e do arrasto; o segmento do meio corre no outro.
    let ax = usize::from(d[0] == 0.0);
    let other = 1 - ax;
    let z = (v1[ax] - v2[ax]).abs() < EPS
        && (a[other] - v1[other]).abs() < EPS
        && (b[other] - v2[other]).abs() < EPS
        && (v1[ax] - a[ax]) * d[ax] > -EPS
        && (b[ax] - v2[ax]) * d[ax] > -EPS;
    let span = [a[ax] + jetty * d[ax], b[ax] - jetty * d[ax]];
    if !z || (span[1] - span[0]) * d[ax] <= 0.0 {
        return None;
    }
    if let Some(f) = frac {
        let c = span[0] + f.clamp(0.0, 1.0) * (span[1] - span[0]);
        leg[1][ax] = c;
        leg[2][ax] = c;
    }
    let (p, q) = (leg[1], leg[2]);
    Some(ElbowMid {
        at: [(p[0] + q[0]) / 2.0, (p[1] + q[1]) / 2.0],
        vertical: ax == 0,
        span,
    })
}

#[cfg(test)]
#[path = "elbow_mid_tests.rs"]
mod tests;
