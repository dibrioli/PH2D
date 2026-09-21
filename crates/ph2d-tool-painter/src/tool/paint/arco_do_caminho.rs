//! **A curva osculadora do caminho — CONSTRUÍDA, MEDIDA e RECUSADA.**
//!
//! ⛔⛔⛔ **Ela NÃO shipa.** O produto anda em LINHA RECTA no retro-traçado, como sempre andou;
//! isto fica porque é o **instrumento da recusa** — sem ele, a medição que a rejeitou não é
//! repetível.
//!
//! ## A hipótese
//!
//! O esfregão acumula um mapa de volta por composição, `D(p) = v + D(p − v)`, e o passo `p − v`
//! é uma **CORDA**. Numa recta a corda é o caminho; numa curva ela sai do arco por `|v|²/2r` em
//! cada elo, e a corrente tem **centenas** de elos ⇒ *o caminho de volta afasta-se do traço, o
//! re-amostrar aterra onde não há tinta, e a figura perde `15,6 %` dela*. A cura óbvia é o passo
//! de volta ser uma **ROTAÇÃO** em torno do centro de curvatura, que segue o arco por construção.
//!
//! ## A medição que a matou
//!
//! Ela ARMA (`99,6 %` dos dabs de uma elipse recebem arco, medido) e **não cura**:
//!
//! | | anel guardado | `\|disp\|` | radial | raio de `p − D` (anel `185..245`) |
//! |---|---|---|---|---|
//! | passo recto (shipa) | `84,4 %` | `79,00` | `43,72` | `205,9` |
//! | **passo pelo arco** | **`84,8 %`** | `80,11` | `43,50` | `207,2` |
//!
//! ⚠️⚠️ **E a refutação já estava numa medição ANTERIOR que eu não reli:** a sonda da curvatura
//! (`diag_a_perda_e_a_curvatura`) mostra que a perda **CRESCE com o raio** (`98,6 %` a `r = 40`,
//! `84,0 %` a `r = 300`) — *se a perda não é função da curvatura, uma cura que só corrige
//! curvatura não a pode tocar*. **Construí um remédio para uma causa que a minha própria tabela
//! tinha refutado duas medições antes.**
//!
//! ⇒ O que sobra medido é que **a perda segue o MÓDULO do deslocamento**, e o único mecanismo
//! que a cura é limitá-lo — que mata o transporte longo que o dono exigiu
//! ([`ph2d_painter_brush::TECTO_MEDIDO_E_RECUSADO_EM_RAIOS`]). As duas saídas continuam em
//! tensão, e a escolha é do dono.

use ph2d_painter_brush::Arco;

/// ⚠️ **O centro é DERIVADO, não construído com um sinal adivinhado.** Ele é a solução de
/// *«rodar o centro deste dab para trás por `dθ` aterra no centro do dab anterior»* —
/// `(R(−dθ) − I)·u = −passo` com `u = c − C` —, que fecha em forma fechada e não deixa escolha
/// de mão esquerda ou direita por fazer.
///
/// Devolve `None` — e aí o passo de volta é o recto de sempre, **byte-idêntico** — quando:
///
/// * não há passo anterior (o 1.º dab de um lote, ou logo depois de uma fronteira de sub-figura);
/// * um dos passos é nulo (não há direcção de que tirar um ângulo);
/// * o caminho mal virou (`|dθ|` abaixo do ruído: ali a corda **é** o arco);
/// * o caminho virou DEMAIS (`|dθ| ≥ π/2`) — isso é uma QUINA, não uma curva, e um círculo
///   osculador ali é um modelo do que não está lá;
/// * o centro de curvatura cai DENTRO da pegada (`|r| < raio`) — rodar em torno de um centro que
///   a pegada contém dobra-a sobre si mesma.
pub(super) fn osculador(
    anterior: Option<[f32; 2]>,
    passo: [f32; 2],
    centro_do_dab: [f32; 2],
    raio_do_dab: f32,
) -> Option<Arco> {
    let a = anterior?;
    let (na, np) = (a[0].hypot(a[1]), passo[0].hypot(passo[1]));
    if !(na > 0.0 && np > 0.0) {
        return None;
    }
    let cruz = a[0] * passo[1] - a[1] * passo[0];
    let escalar = a[0] * passo[0] + a[1] * passo[1];
    let dtheta = cruz.atan2(escalar);
    if !dtheta.is_finite() || dtheta.abs() < ANGULO_MINIMO || dtheta.abs() >= FRAC_PI_2 {
        return None;
    }
    // `(R(−dθ) − I)·u = −passo`, com `u = c − C`. A solução crua divide por `2 − 2·cos dθ`, que
    // para `dθ` pequeno é cancelamento catastrófico — e foi o que a fixtura apanhou (o centro
    // saía `0,08 px` fora num raio de `215`). A identidade `2 − 2cos θ = 4·sin²(θ/2)` cancela-a
    // à mão e deixa a fórmula clássica do circunraio:
    //
    //     u = ½·(passo.x + k·passo.y,  passo.y − k·passo.x)   com  k = cot(dθ/2)
    //
    // cujo módulo é `|passo| / (2·sin(dθ/2))` — exactamente o circunraio.
    let k = (dtheta * 0.5).tan();
    if k.abs() <= f32::EPSILON {
        return None;
    }
    let k = 1.0 / k;
    let ux = 0.5 * (passo[0] + k * passo[1]);
    let uy = 0.5 * (passo[1] - k * passo[0]);
    if !(ux.is_finite() && uy.is_finite()) {
        return None;
    }
    if ux.hypot(uy) < raio_do_dab {
        return None;
    }
    Some(Arco {
        centro: [centro_do_dab[0] - ux, centro_do_dab[1] - uy],
        dtheta,
    })
}

use core::f32::consts::FRAC_PI_2;

/// **O ângulo abaixo do qual a corda É o arco.**
///
/// ⛔ Ele não é escolhido por conforto: a `dθ` o desvio de um elo à corda vale `|v|·dθ/2`, logo
/// **meio TEXEL** — a resolução em que este esfregão escreve — pede `dθ ≥ 1/|v|`. Com o passo
/// mais longo que um traço à mão produz (`~24 px` a um pincel de `120`), isso dá `0,04`; com o
/// mais curto (`1 px`), dá `1,0`. Tomar o menor dos dois é o único valor que nunca DESLIGA a
/// correcção onde ela se vê, e `0,001` fica **uma ordem de grandeza** abaixo dele — o que ele
/// corta é só o ruído de um caminho que não virou.
const ANGULO_MINIMO: f32 = 0.001;

#[cfg(test)]
mod tests {
    use super::osculador;

    /// **A lei em forma fechada: rodar para trás aterra no dab anterior.** É isto que prova que
    /// o centro está certo — e não a inspecção de um sinal.
    #[test]
    fn rodar_para_tras_aterra_no_dab_anterior() {
        // Um círculo de raio 215 percorrido em passos de 6 px, nos dois sentidos.
        for sentido in [1.0f32, -1.0] {
            for k in 1..40 {
                let r = 215.0f32;
                let dt = sentido * 6.0 / r;
                let ang = |i: i32| (k as f32 + i as f32) * dt;
                let ponto = |i: i32| [350.0 + r * ang(i).cos(), 350.0 + r * ang(i).sin()];
                let (p0, p1, p2) = (ponto(-2), ponto(-1), ponto(0));
                let passo_a = [p1[0] - p0[0], p1[1] - p0[1]];
                let passo = [p2[0] - p1[0], p2[1] - p1[1]];
                let arco = osculador(Some(passo_a), passo, p2, 30.0).expect("um arco");

                // ⚠️ **A barra do centro é a precisão da ENTRADA e não um número escolhido.**
                // Os três pontos são `f32` de magnitude `~565`, logo cada coordenada carrega
                // até `ulp(565)/2 ≈ 3,0e-5`; um passo é a DIFERENÇA de dois pontos (`6,1e-5`
                // por componente, `8,6e-5` em módulo) e a direcção dele herda `1,4e-5` rad —
                // e `dθ` é a diferença de DUAS direcções, logo **`2,9e-5`**. O circunraio
                // `r = |passo|/dθ` herda o erro RELATIVO de `dθ` (`2,9e-5/0,0279 = 1,0e-3`)
                // ⇒ o centro pode sair por `215 × 1,0e-3 = 0,22 px`, e mede-se `0,09`.
                //
                // ⚠️⚠️ A 1.ª redacção desta conta esqueceu que `dθ` é uma DIFERENÇA e escreveu
                // `0,08`; o gate reprovou **sobre uma implementação correcta**, e a cura foi
                // refazer a derivação e não afrouxar o número.
                //
                // ⭐ **E a LEI é o passo de volta, não o centro:** um erro `e` no centro só
                // produz `e·dθ` de erro na posição rodada (`0,09 × 0,028 = 0,003 px`), logo
                // ele é duas ordens de grandeza mais preciso do que o centro de que sai.
                let d = (arco.centro[0] - 350.0).hypot(arco.centro[1] - 350.0);
                assert!(d < 0.25, "o centro saiu por {d:.4} px");
                let (sn, cs) = (-arco.dtheta).sin_cos();
                let (dx, dy) = (p2[0] - arco.centro[0], p2[1] - arco.centro[1]);
                let q = [
                    arco.centro[0] + cs * dx - sn * dy,
                    arco.centro[1] + sn * dx + cs * dy,
                ];
                let e = (q[0] - p1[0]).hypot(q[1] - p1[1]);
                assert!(
                    e < 0.01,
                    "rodar para trás falhou o dab anterior por {e:.4} px"
                );
            }
        }
    }

    /// **As cinco recusas, e o que cada uma impede.**
    ///
    /// ⚠️ A da RECTA é a que preserva o transporte longo byte-a-byte: sem ela, todo traço a
    /// direito passaria pelo caminho do arco e deixaria de ser o que o dono aprovou.
    #[test]
    fn as_recusas() {
        let passo = [6.0f32, 0.0];
        assert!(
            osculador(None, passo, [0.0, 0.0], 30.0).is_none(),
            "sem anterior"
        );
        assert!(
            osculador(Some([0.0, 0.0]), passo, [0.0, 0.0], 30.0).is_none(),
            "passo anterior nulo"
        );
        assert!(
            osculador(Some(passo), [0.0, 0.0], [0.0, 0.0], 30.0).is_none(),
            "passo nulo"
        );
        assert!(
            osculador(Some([6.0, 0.0]), [6.0, 0.0], [0.0, 0.0], 30.0).is_none(),
            "a RECTA: o caminho não virou"
        );
        assert!(
            osculador(Some([6.0, 0.0]), [0.0, 6.0], [0.0, 0.0], 30.0).is_none(),
            "a QUINA: virou 90°"
        );
        // O centro dentro da pegada: um círculo de raio 10 com um pincel de 30.
        let r = 10.0f32;
        let pt = |i: f32| [r * (i * 0.3).cos(), r * (i * 0.3).sin()];
        let (p0, p1, p2) = (pt(0.0), pt(1.0), pt(2.0));
        assert!(
            osculador(
                Some([p1[0] - p0[0], p1[1] - p0[1]]),
                [p2[0] - p1[0], p2[1] - p1[1]],
                p2,
                30.0
            )
            .is_none(),
            "o centro de curvatura cai DENTRO da pegada"
        );
    }
}
