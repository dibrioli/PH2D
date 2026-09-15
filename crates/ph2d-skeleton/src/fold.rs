//! ⭐⭐⭐ **A RÉGUA DA DOBRA — quanto da arte a pele vira do avesso** (pedido do dono, 2026-09-14:
//! seguir o artefacto que sobra depois do `Smooth`).
//!
//! ⛔⛔ **Esta régua NÃO EXISTIA no repo, e os números que a fila citava eram de uma medição
//! avulsa** (`−0,129` a `60°`, `−1,017` a `150°`). *Uma lei sem instrumento é uma nota que
//! envelhece* — e sem ela nenhuma cura pode ser comparada com a doença.
//!
//! # O que se mede, e porquê ESTA grandeza
//!
//! O colapso do *linear blend skinning* não é «a silhueta ficou feia»: é o mapa deixar de ser
//! **injectivo**. Num ponto onde ele se dobra, o **determinante do jacobiano** passa por zero e fica
//! negativo — a vizinhança daquele ponto é desenhada **do avesso**, e a arte passa por cima de si
//! mesma. ⇒ as duas colunas são o **menor determinante** (quão invertido fica o pior ponto) e a
//! **fracção da área** em que ele é negativo (quanto da imagem está dobrada).
//!
//! ⚠️ **Nenhuma das réguas que este módulo já tinha vê isto**: o desvio em píxeis da silhueta
//! (a régua do `Smooth`) mede a APROXIMAÇÃO do campo, e o campo dobrado é aproximado com fidelidade
//! — quanto melhor o `Smooth`, mais nítida a dobra. *Uma régua que mede o erro da amostragem é cega
//! ao erro do que está a ser amostrado.*
//!
//! ⛔⛔ **E o PONTO ÓRFÃO contamina esta régua se não for excluído.** Fora do raio de todo osso a
//! pele salta para o osso mais próximo — um mapa **descontínuo** —, e uma diferença finita que
//! atravesse esse salto devolve um determinante enorme e falso (medi **`−57`** num mapa de escala
//! `1`). ⇒ as amostras que tocam num órfão saem da conta, e a fracção delas sai no relatório: sem
//! essa coluna, uma «cura» que deixasse a arte inteira órfã leria-se **perfeita**.
//!
//! ⚠️ **Uma grelha nunca cai num conjunto de medida nula**, e aqui isso não é armadilha: a região
//! dobrada é uma ÁREA, não uma curva. ⛔ O que seria armadilha é medir a LINHA onde o determinante
//! passa por zero — essa é medida nula, e uma grelha reporta-a sempre limpa.

use super::Skin;

/// O que uma pele faz à área da arte que ela carrega.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FoldReport {
    /// O menor determinante do jacobiano sobre as amostras. **Negativo ⇒ há arte do avesso.**
    pub det_min: f64,
    /// Que fracção da área amostrada tem determinante `≤ 0`.
    pub inverted: f64,
    /// Quantas amostras entraram na conta — o **piso de população** da régua: uma caixa fora da
    /// pele devolveria `0` amostras e um relatório limpo que não mediu nada.
    pub samples: usize,
    /// A fracção das amostras que ficou **de fora** por tocar num ponto ÓRFÃO (fora do raio de todo
    /// osso, onde a pele salta para o osso mais próximo).
    ///
    /// ⛔⛔ **Ela não é diagnóstico, é a ANTI-VACUIDADE desta régua.** O salto do órfão é
    /// **descontínuo**, e uma diferença finita que o atravesse devolve um determinante enorme e
    /// falso — medi `−57` num mapa de escala `1`. Excluí-los é obrigatório para comparar curas; mas
    /// sem esta coluna, uma «cura» que deixasse a arte inteira órfã leria-se **perfeita**, tendo
    /// deitado fora todas as amostras.
    pub orphan: f64,
}

/// ⭐⭐⭐ **Mede a dobra de uma pele sobre uma caixa de repouso.**
///
/// `caixa` é `[x0, y0, x1, y1]` no espaço de REPOUSO (é lá que a arte vive); `n` é a resolução da
/// grelha em cada eixo.
///
/// ⚠️ **O jacobiano sai por diferenças centrais**, e o passo é uma fracção da célula da grelha —
/// não um epsilon absoluto: a mesma lei medida numa arte dez vezes maior tem de dar o mesmo número,
/// e um passo em unidades do documento tornaria a régua função do tamanho do desenho.
#[must_use]
pub fn measure(skin: &Skin, caixa: [f64; 4], n: usize) -> FoldReport {
    let [x0, y0, x1, y1] = caixa;
    let n = n.max(2);
    let (dx, dy) = ((x1 - x0) / (n - 1) as f64, (y1 - y0) / (n - 1) as f64);
    // ⚠️ Um quarto da célula: grande o bastante para o `f64` não comer a diferença, pequeno o
    // bastante para não alisar a dobra que se está a medir.
    let h = 0.25 * dx.abs().min(dy.abs()).max(f64::MIN_POSITIVE);
    let mut w = skin.scratch();
    let mut ponto = |p: [f64; 2]| -> Option<[f64; 2]> {
        // ⚠️ O teste do órfão vem da MESMA porta que dá o ponto: perguntar duas vezes com duas
        // regras seria a segunda resposta à mesma pergunta.
        skin.weights_at(p, &mut w).then(|| {
            let mut out = [0.0, 0.0];
            for (b, &peso) in skin.bones().iter().zip(w.iter()) {
                if peso != 0.0 {
                    let q = b.pose.apply(p);
                    out[0] += peso * q[0];
                    out[1] += peso * q[1];
                }
            }
            out
        })
    };
    let (mut det_min, mut invertidas, mut amostras, mut orfas) =
        (f64::INFINITY, 0usize, 0usize, 0usize);
    for i in 0..n {
        for j in 0..n {
            let p = [x0 + dx * i as f64, y0 + dy * j as f64];
            let quatro = [
                ponto([p[0] + h, p[1]]),
                ponto([p[0] - h, p[1]]),
                ponto([p[0], p[1] + h]),
                ponto([p[0], p[1] - h]),
            ];
            let (Some(px), Some(mx), Some(py), Some(my)) =
                (quatro[0], quatro[1], quatro[2], quatro[3])
            else {
                orfas += 1;
                continue;
            };
            let (ax, ay) = ((px[0] - mx[0]) / (2.0 * h), (px[1] - mx[1]) / (2.0 * h));
            let (bx, by) = ((py[0] - my[0]) / (2.0 * h), (py[1] - my[1]) / (2.0 * h));
            let det = ax * by - ay * bx;
            amostras += 1;
            det_min = det_min.min(det);
            if det <= 0.0 {
                invertidas += 1;
            }
        }
    }
    let total = amostras + orfas;
    FoldReport {
        det_min: if amostras == 0 { 0.0 } else { det_min },
        inverted: if amostras == 0 {
            0.0
        } else {
            invertidas as f64 / amostras as f64
        },
        samples: amostras,
        orphan: if total == 0 {
            0.0
        } else {
            orfas as f64 / total as f64
        },
    }
}
