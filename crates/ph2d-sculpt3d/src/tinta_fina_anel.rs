//! ⭐⭐ **Os dois pincéis que LÊEM O ANEL sobre a retícula** — o `Blur` e o
//! `Smear Color` da tinta fina. Filho do [`super`] (a janela do empréstimo e o
//! dab por amostra), partido por RESPONSABILIDADE: lá o plano e a sua janela,
//! aqui a lei de vizinhança que dois verbos lêem.

use crate::{Brush, Dab, Verb};

use super::Apanhada;

impl crate::SculptStroke {
    /// Os ALVOS das três leis de cor, sobre amostras.
    ///
    /// ⭐⭐ O [`Verb::Paint`] deposita a cor do pincel; os dois que leem o ANEL
    /// puxam-na da vizinhança — e a vizinhança aqui é a **retícula**, que
    /// atravessa a aresta da malha porque a fronteira é PARTILHADA. *É esta
    /// linha que o atlas não consegue escrever: lá, o vizinho de um texel pode
    /// estar noutra ponta da peça.*
    pub(super) fn alvos_de_cor_fino(
        &self,
        brush: &Brush,
        dab: &Dab,
        amostras: &[Apanhada],
    ) -> Vec<[f32; 3]> {
        if brush.verb == Verb::Paint {
            return vec![brush.color; amostras.len()];
        }
        let fina = self.tinta_fina.as_ref().expect("chamado de dentro do dab");
        let cor = |i: usize| fina.tinta.amostras()[amostras[i].idx as usize];
        // ⚠️⚠️ **A PRÓPRIA amostra entra com peso `1`, nas DUAS leis** — é uma
        // relaxação *para* a vizinhança e não uma substituição por ela. Sem
        // ela, um dab a peso cheio apaga a cor de uma vez e o pincel deixa de
        // ter gradação. *Esquecê-la foi o meu segundo defeito nesta wave, e o
        // gate contra o caminho por-vértice mediu-o em `3,8e-2`.*
        let mut soma: Vec<[f32; 3]> = (0..amostras.len()).map(cor).collect();
        let mut peso = vec![1.0f32; amostras.len()];
        let smear = brush.verb == Verb::SmearColor;
        for &(a, b) in &fina.pares {
            let (ia, ib) = (a as usize, b as usize);
            let (ca, cb) = (cor(ia), cor(ib));
            let (pa, pb) = (amostras[ia].pos, amostras[ib].pos);
            let (wa, wb) = if smear {
                // ⚠️ **A direcção é do MODO e não do caminho** — os três modos
                // do esfregão (arrastar · apertar · espalhar) e a
                // degenerescência de cada um vivem na
                // [`crate::SmearMode::direction`], que é a porta que o caminho
                // por-vértice também lê.
                let da = unit_ou_nada(brush.smear_mode.direction(dab.path, dab.center, pa));
                let db = unit_ou_nada(brush.smear_mode.direction(dab.path, dab.center, pb));
                let e = unit_ou_nada([pb[0] - pa[0], pb[1] - pa[1], pb[2] - pa[2]]);
                match (da, db, e) {
                    (_, _, None) => (0.0, 0.0),
                    (da, db, Some(e)) => {
                        // `a` recebe de `b` quando `b` está a MONTANTE de `a`.
                        let ga = da.map_or(0.0, |d| {
                            (-(d[0] * e[0] + d[1] * e[1] + d[2] * e[2])).max(0.0)
                        });
                        let gb =
                            db.map_or(0.0, |d| (d[0] * e[0] + d[1] * e[1] + d[2] * e[2]).max(0.0));
                        (ga, gb)
                    }
                }
            } else {
                (1.0, 1.0)
            };
            if wa > 0.0 {
                for k in 0..3 {
                    soma[ia][k] += cb[k] * wa;
                }
                peso[ia] += wa;
            }
            if wb > 0.0 {
                for k in 0..3 {
                    soma[ib][k] += ca[k] * wb;
                }
                peso[ib] += wb;
            }
        }
        // ⛔⛔ **DIVIDIR, e não multiplicar pelo recíproco** — é a única coisa
        // que torna a lei um NO-OP AO BIT numa peça de cor uniforme, e é uma
        // lei escrita no [`crate::stroke_cor`] com a medição ao lado. *Um
        // pincel que muda a peça onde não há nada a mudar é um passo de undo,
        // um upload de GPU e um ficheiro diferente por nada.*
        (0..amostras.len())
            .map(|i| {
                [
                    soma[i][0] / peso[i],
                    soma[i][1] / peso[i],
                    soma[i][2] / peso[i],
                ]
            })
            .collect()
    }
}

/// O unitário, ou `None` quando o vector **não tem direcção**. Irmão do da
/// [`crate::stroke_cor`], e pela mesma razão: o limiar é o zero EXACTO.
fn unit_ou_nada(v: [f32; 3]) -> Option<[f32; 3]> {
    let q = v[0] * v[0] + v[1] * v[1] + v[2] * v[2];
    if q == 0.0 {
        return None;
    }
    let inv = 1.0 / q.sqrt();
    Some([v[0] * inv, v[1] * inv, v[2] * inv])
}
