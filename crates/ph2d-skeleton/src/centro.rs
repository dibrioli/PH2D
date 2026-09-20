//! ⭐⭐⭐ **EM TORNO DE QUE PONTO CADA PEDAÇO DA ARTE RODA** — a cura do entalhe do cotovelo.
//!
//! # ⛔⛔⛔ O defeito, medido
//!
//! A mistura linear (`Σ wᵢ · Mᵢ p`) interpola POSIÇÕES, e a interpolação de duas posições de uma
//! rotação é a **CORDA** do arco, nunca o arco. ⇒ um ponto a meio caminho entre dois ossos que
//! divergem `θ` é puxado para `cos(θ/2)` da distância à junta, e a arte **colapsa** — é o
//! *«candy-wrapper»* de toda a literatura, e o entalhe que o dono fotografou em 2026-09-19.
//!
//! Medido numa barra de `7 × 1` com dois ossos (o PESCOÇO é o sítio mais estreito da forma, e em
//! repouso ele é a espessura dela):
//!
//! | dobra | pescoço, mistura linear | área |
//! |---|---|---|
//! | `60°` | `0,4882` | `91,3 %` |
//! | `90°` | `0,3343` | `82,6 %` |
//! | `120°` | `0,1643` | `73,9 %` |
//!
//! # ⭐⭐ A lei: em 2D o centro de rotação NÃO se estima — ele é SABIDO
//!
//! A literatura estima-o (*Optimized Centers of Rotation*, Le & Hodgins 2016: uma média das
//! posições de repouso pesada pela SEMELHANÇA entre vectores de peso, com um `σ` a afinar). ⛔ Aqui
//! ele não precisa de ser estimado: **um esqueleto 2D é uma árvore de segmentos que PARTILHAM
//! pontas**, e dois ossos que disputam um ponto partilham uma junta. ⇒ o centro é a média das
//! juntas de cada par, pesada pelo produto dos pesos do par.
//!
//! Medido lado a lado, na mesma barra com a junta FORA do centróide:
//!
//! | dobra | mistura linear | núcleo de semelhança (`σ² = 5e-4`) | **a JUNTA** |
//! |---|---|---|---|
//! | `60°` | `0,4882` | `0,9496` | **`0,9432`** |
//! | `90°` | `0,3343` | `0,8615` | **`0,8659`** |
//! | `120°` | `0,1643` | `0,3317` | **`0,3763`** |
//!
//! ⇒ *a junta iguala ou bate o estimador, **sem um parâmetro e sem um byte guardado**.*
//!
//! ⚠️⚠️ **A 1.ª fixtura escondeu o mecanismo**: com a junta no MEIO da barra, o centróide da forma
//! calha nela, e o estimador devolvia `(3,500 · 0,500)` para **todo** peso — `σ²` de `2e-5` a `100`
//! dava o mesmo número. *Uma fixtura simétrica aprova um estimador que não estima nada.*
//!
//! # ⭐ Porque não há descontinuidade onde um osso manda sozinho
//!
//! Com `w = (1, 0, …)` a transformação é RÍGIDA, e uma rotação rígida leva `p` ao mesmo sítio
//! **qualquer que seja o centro**: `R(θ₁)(p − c) + M₁(c) = M₁(p)` para todo `c`. ⇒ o centro deixa de
//! importar exactamente onde ele deixa de existir, e a lei é contínua por construção.
//!
//! ⛔ **E o `dual quaternion` (o `se(2)`) é RECUSA MEDIDA deste repo** — ele piora a dobra de uma
//! imagem (`2,52 % → 4,47 %` dobrada sobre si mesma). Ele resolve o mesmo defeito escolhendo a
//! ROTAÇÃO certa e deixando o centro ao acaso; esta lei escolhe o CENTRO e deixa a rotação ser a
//! média.

use crate::{Skin, SkinBone};

impl SkinBone {
    /// O segmento de repouso **deste sub-osso** — ver [`SkinBone::sub`].
    ///
    /// ⚠️ Os sub-ossos de um osso que dobra partilham `rest_a`/`rest_b`; o que os distingue é a
    /// fatia. *Sem isto, dois sub-ossos consecutivos teriam o eixo inteiro como junta partilhada e
    /// a média cairia no meio do osso em vez de na dobra.*
    #[must_use]
    pub fn eixo_do_sub(&self) -> ([f64; 2], [f64; 2]) {
        let n = f64::from(self.sub.1.max(1));
        let k = f64::from(self.sub.0);
        let lerp = |t: f64| {
            [
                (self.rest_b[0] - self.rest_a[0]).mul_add(t, self.rest_a[0]),
                (self.rest_b[1] - self.rest_a[1]).mul_add(t, self.rest_a[1]),
            ]
        };
        (lerp(k / n), lerp((k + 1.0) / n))
    }

    /// O ângulo da parte linear da [`SkinBone::pose`] — quanto este osso RODOU do repouso.
    #[must_use]
    pub fn angulo_da_pose(&self) -> f64 {
        let [a, b, ..] = self.pose.0;
        b.atan2(a)
    }
}

/// A junta que dois sub-ossos partilham, em repouso — a **ponta mais próxima** de um ao outro.
///
/// ⭐ Para um par pai→filho e para dois sub-ossos consecutivos ela é **exacta** (as duas pontas
/// coincidem). Para dois ossos de ramos diferentes é o ponto médio das pontas mais próximas, que é
/// a leitura honesta de *«por onde estes dois se encontram»*.
fn junta(x: &SkinBone, y: &SkinBone) -> [f64; 2] {
    let (xa, xb) = x.eixo_do_sub();
    let (ya, yb) = y.eixo_do_sub();
    let d2 = |p: [f64; 2], q: [f64; 2]| (p[0] - q[0]).hypot(p[1] - q[1]);
    let mut melhor = (f64::INFINITY, [0.0, 0.0]);
    for p in [xa, xb] {
        for q in [ya, yb] {
            let d = d2(p, q);
            if d < melhor.0 {
                melhor = (d, [(p[0] + q[0]) * 0.5, (p[1] + q[1]) * 0.5]);
            }
        }
    }
    melhor.1
}

impl Skin {
    /// ⭐⭐⭐ **O CENTRO EM TORNO DO QUAL UM PONTO COM ESTES PESOS RODA.**
    ///
    /// `None` quando **um só** osso manda (não há par, logo não há junta) — e ali o centro é
    /// irrelevante, porque a transformação é rígida. Ver o cabeçalho do módulo.
    #[must_use]
    pub fn centro_de_rotacao(&self, w: &[f64]) -> Option<[f64; 2]> {
        let (mut num, mut den) = ([0.0_f64, 0.0], 0.0_f64);
        for (i, a) in self.bones.iter().enumerate() {
            let wi = w.get(i).copied().unwrap_or(0.0);
            if wi <= 0.0 {
                continue;
            }
            for (j, b) in self.bones.iter().enumerate().skip(i + 1) {
                let wj = w.get(j).copied().unwrap_or(0.0);
                if wj <= 0.0 {
                    continue;
                }
                let q = wi * wj;
                let c = junta(a, b);
                num[0] = q.mul_add(c[0], num[0]);
                num[1] = q.mul_add(c[1], num[1]);
                den += q;
            }
        }
        (den > 0.0 && num[0].is_finite() && num[1].is_finite())
            .then(|| [num[0] / den, num[1] / den])
    }

    /// ⭐⭐⭐ **A MISTURA — rodar em torno da JUNTA, e transladar pela mistura linear DELA.**
    ///
    /// ⚠️ **Ela tem o nome `blend` de propósito:** era esse o nome da lei antiga, e toda a casa o
    /// chama. *Trocar o corpo e deixar o nome é o que faz a cura chegar às quatro mídias sem um
    /// mapa de excepções* — a antiga ficou como [`Skin::blend_linear`], que é o CONTROLO.
    ///
    /// `p' = R(θ̄)·(p − c) + Σ wᵢ Mᵢ(c)`
    ///
    /// ⚠️ **O ângulo é a média em CÍRCULO** (`atan2(Σ w sin, Σ w cos)`), nunca a soma `Σ w θ`: os
    /// ângulos vêm de um `atan2` e vivem em `(−π, π]`, logo somá-los **salta** quando um osso passa
    /// meia volta. ⛔ A média em círculo tem uma degenerescência própria — duas rotações a `180°`
    /// exactas com pesos iguais —, e ali a soma é zero e a lei cai na mistura linear.
    #[must_use]
    pub fn blend(&self, p: [f64; 2], w: &[f64]) -> [f64; 2] {
        let Some(c) = self.centro_de_rotacao(w) else {
            return self.blend_linear(p, w);
        };
        let (mut sx, mut sy, mut soma) = (0.0_f64, 0.0_f64, 0.0_f64);
        for (b, &peso) in self.bones.iter().zip(w.iter()) {
            if peso == 0.0 {
                continue;
            }
            let t = b.angulo_da_pose();
            sx = peso.mul_add(t.cos(), sx);
            sy = peso.mul_add(t.sin(), sy);
            soma += peso;
        }
        if soma == 0.0 || (sx == 0.0 && sy == 0.0) {
            return self.blend_linear(p, w);
        }
        let (co, si) = {
            let n = sx.hypot(sy);
            (sx / n, sy / n)
        };
        let base = self.blend_linear(c, w);
        let d = [p[0] - c[0], p[1] - c[1]];
        [
            si.mul_add(-d[1], co.mul_add(d[0], base[0])),
            si.mul_add(d[0], co.mul_add(d[1], base[1])),
        ]
    }
}

#[cfg(test)]
#[path = "centro_tests.rs"]
mod centro_tests;
